use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use flate2::write::GzEncoder;
use flate2::Compression;
use lazylog::log::detect::detect_from_path;
use lazylog::log::format::LogFormat;
use lazylog::log::source::FileSource;

fn testdata(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(name)
}

/// Gzip a source file into a tempdir and return the path to the `.gz` copy.
fn gz_in_tempdir(src: &PathBuf, tmp: &tempfile::TempDir) -> PathBuf {
    let mut raw = Vec::new();
    File::open(src).unwrap().read_to_end(&mut raw).unwrap();
    let name = src.file_name().unwrap().to_string_lossy();
    let out = tmp.path().join(format!("{name}.gz"));
    let f = File::create(&out).unwrap();
    let mut enc = GzEncoder::new(f, Compression::default());
    enc.write_all(&raw).unwrap();
    enc.finish().unwrap();
    out
}

#[test]
fn gzip_source_matches_plain_bytes() {
    let plain = testdata("nginx-access.log");
    let plain_bytes = std::fs::read(&plain).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let gz = gz_in_tempdir(&plain, &tmp);
    let src = FileSource::open(&gz).unwrap();
    assert!(src.is_compressed());
    assert_eq!(src.bytes(), plain_bytes.as_slice());
}

#[test]
fn gzip_format_detected_from_inner_name() {
    let plain = testdata("nginx-access.log");
    let tmp = tempfile::tempdir().unwrap();
    let gz = gz_in_tempdir(&plain, &tmp);
    assert_eq!(detect_from_path(&gz), LogFormat::NginxAccess);
}
