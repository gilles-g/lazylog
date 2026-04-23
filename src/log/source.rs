use std::path::Path;
use std::sync::Arc;

use memmap2::Mmap;

/// Zero-copy, kernel-backed view on the log file.
/// LogEvent references lines by (offset, len) into this mmap; the raw text
/// is never duplicated on the heap.
pub struct FileSource {
    mmap: Mmap,
}

impl FileSource {
    pub fn open(path: &Path) -> std::io::Result<Arc<Self>> {
        let file = std::fs::File::open(path)?;
        // SAFETY: the file is not mutated while the mmap is held; we expose
        // only &[u8] / &str slices.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Arc::new(Self { mmap }))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.mmap
    }

    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    pub fn slice(&self, offset: u64, len: u32) -> &str {
        let start = offset as usize;
        let end = start.saturating_add(len as usize).min(self.mmap.len());
        std::str::from_utf8(&self.mmap[start..end]).unwrap_or("")
    }
}
