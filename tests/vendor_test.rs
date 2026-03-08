use std::fs;
use tempfile::TempDir;
use unpm::vendor;

#[test]
fn test_place_file() {
    let dir = TempDir::new().unwrap();
    let output_dir = dir.path().join("static/vendor");
    vendor::place_file(&output_dir, "htmx.min.js", b"fake content").unwrap();
    let written = fs::read(output_dir.join("htmx.min.js")).unwrap();
    assert_eq!(written, b"fake content");
}

#[test]
fn test_place_file_creates_dirs() {
    let dir = TempDir::new().unwrap();
    let output_dir = dir.path().join("deep/nested/vendor");
    vendor::place_file(&output_dir, "lib.js", b"content").unwrap();
    assert!(output_dir.join("lib.js").exists());
}

#[test]
fn test_remove_file() {
    let dir = TempDir::new().unwrap();
    let output_dir = dir.path().join("vendor");
    vendor::place_file(&output_dir, "lib.js", b"content").unwrap();
    assert!(output_dir.join("lib.js").exists());
    vendor::remove_file(&output_dir, "lib.js").unwrap();
    assert!(!output_dir.join("lib.js").exists());
}

#[test]
fn test_remove_nonexistent_file_ok() {
    let dir = TempDir::new().unwrap();
    vendor::remove_file(dir.path(), "nope.js").unwrap();
}
