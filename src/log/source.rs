use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use memmap2::Mmap;

/// Backing storage for a log source. Either a zero-copy, kernel-backed mmap
/// (plain files) or a heap buffer holding decompressed bytes (gzip archives).
/// Both expose an identical `&[u8]` view so the rest of the pipeline is
/// oblivious to the distinction.
enum Backing {
    Mmap(Mmap),
    Owned(Vec<u8>),
}

impl Backing {
    fn as_slice(&self) -> &[u8] {
        match self {
            Backing::Mmap(m) => m,
            Backing::Owned(v) => v.as_slice(),
        }
    }
}

pub struct FileSource {
    backing: Backing,
    /// True when the source was decompressed in-memory; follow-mode cannot
    /// work in that case (the on-disk bytes have no direct mapping to ours).
    compressed: bool,
}

impl FileSource {
    pub fn open(path: &Path) -> std::io::Result<Arc<Self>> {
        if is_gzip(path) {
            return Self::open_gzip(path);
        }
        let file = std::fs::File::open(path)?;
        // SAFETY: memory-mapping a file is inherently unsound when another
        // process mutates the file concurrently (log rotation, truncation, or
        // a non-atomic write can surface as SIGBUS or torn reads). `lazylog`
        // targets log files which *can* be actively written to: callers must
        // be prepared for the process to receive SIGBUS on mutation. We only
        // expose &[u8] / &str slices (never &mut), so no aliasing violation
        // is introduced by us — the residual risk is OS-level file mutation.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Arc::new(Self {
            backing: Backing::Mmap(mmap),
            compressed: false,
        }))
    }

    fn open_gzip(path: &Path) -> std::io::Result<Arc<Self>> {
        let file = std::fs::File::open(path)?;
        let mut decoder = flate2::read::MultiGzDecoder::new(file);
        // Pre-size the output buffer: rule of thumb ~4× compressed size for
        // text logs. Capped to avoid pathological allocations on tiny files.
        let compressed_size = path.metadata().map(|m| m.len()).unwrap_or(0);
        let hint = compressed_size.saturating_mul(4).min(u64::from(u32::MAX)) as usize;
        let mut buf = Vec::with_capacity(hint.max(64 * 1024));
        decoder.read_to_end(&mut buf)?;
        Ok(Arc::new(Self {
            backing: Backing::Owned(buf),
            compressed: true,
        }))
    }

    pub fn bytes(&self) -> &[u8] {
        self.backing.as_slice()
    }

    pub fn len(&self) -> usize {
        self.backing.as_slice().len()
    }

    pub fn is_empty(&self) -> bool {
        self.backing.as_slice().is_empty()
    }

    pub fn is_compressed(&self) -> bool {
        self.compressed
    }

    pub fn slice(&self, offset: u64, len: u32) -> &str {
        let bytes = self.backing.as_slice();
        let start = (offset as usize).min(bytes.len());
        let end = start.saturating_add(len as usize).min(bytes.len());
        std::str::from_utf8(&bytes[start..end]).unwrap_or("")
    }
}

fn is_gzip(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase().ends_with(".gz"))
        .unwrap_or(false)
}
