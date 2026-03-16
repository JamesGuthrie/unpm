use unpm::lockfile::{LockedDependency, LockedFile, Lockfile};

// r[verify lockfile.file.missing]
#[test]
fn empty_lockfile() {
    let lockfile = Lockfile::default();
    assert!(lockfile.dependencies.is_empty());
    let json = lockfile.to_json().unwrap();
    assert_eq!(json, "{}");
}

// r[verify lockfile.serialization.roundtrip]
// r[verify lockfile.structure.file-entry]
#[test]
fn roundtrip_json() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "htmx.org".to_string(),
        LockedDependency {
            version: "2.0.4".to_string(),
            files: vec![LockedFile {
                url: "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js".to_string(),
                sha256: "abc123".to_string(),
                size: 12345,
                filename: "htmx.org_htmx.min.js".to_string(),
            }],
        },
    );

    let json = lockfile.to_json().unwrap();
    let reparsed = Lockfile::from_json(&json).unwrap();

    assert_eq!(reparsed.dependencies.len(), 1);
    let dep = &reparsed.dependencies["htmx.org"];
    assert_eq!(dep.version, "2.0.4");
    assert_eq!(dep.files.len(), 1);
    assert_eq!(dep.files[0].sha256, "abc123");
    assert_eq!(dep.files[0].size, 12345);
    assert_eq!(dep.files[0].filename, "htmx.org_htmx.min.js");
}

// r[verify lockfile.structure.top-level]
// r[verify lockfile.structure.dependency]
#[test]
fn from_json_string() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4",
            "files": [{
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
                "sha256": "abc123",
                "size": 12345,
                "filename": "htmx.org_htmx.min.js"
            }]
        }
    }"#;
    let lockfile = Lockfile::from_json(json).unwrap();
    assert_eq!(lockfile.dependencies.len(), 1);
    assert_eq!(lockfile.dependencies["htmx.org"].version, "2.0.4");
    assert_eq!(
        lockfile.dependencies["htmx.org"].files[0].filename,
        "htmx.org_htmx.min.js"
    );
}

// r[verify lockfile.serialization.canonical]
#[test]
fn new_format_roundtrip() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "alpine".to_string(),
        LockedDependency {
            version: "3.14.0".to_string(),
            files: vec![LockedFile {
                url: "https://cdn.jsdelivr.net/npm/alpinejs@3.14.0/dist/cdn.min.js".to_string(),
                sha256: "def456".to_string(),
                size: 54321,
                filename: "alpine_cdn.min.js".to_string(),
            }],
        },
    );

    let json = lockfile.to_json().unwrap();
    let reparsed = Lockfile::from_json(&json).unwrap();
    assert_eq!(reparsed.dependencies["alpine"].files.len(), 1);
    assert_eq!(reparsed.dependencies["alpine"].files[0].sha256, "def456");
}

// r[verify lockfile.structure.multi-file]
#[test]
fn new_format_multi_file() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "bootstrap".to_string(),
        LockedDependency {
            version: "5.3.0".to_string(),
            files: vec![
                LockedFile {
                    url: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.min.js"
                        .to_string(),
                    sha256: "aaa111".to_string(),
                    size: 10000,
                    filename: "bootstrap_bootstrap.min.js".to_string(),
                },
                LockedFile {
                    url: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"
                        .to_string(),
                    sha256: "bbb222".to_string(),
                    size: 20000,
                    filename: "bootstrap_bootstrap.min.css".to_string(),
                },
            ],
        },
    );

    let json = lockfile.to_json().unwrap();
    let reparsed = Lockfile::from_json(&json).unwrap();
    let dep = &reparsed.dependencies["bootstrap"];
    assert_eq!(dep.files.len(), 2);
    assert_eq!(dep.files[0].filename, "bootstrap_bootstrap.min.js");
    assert_eq!(dep.files[1].filename, "bootstrap_bootstrap.min.css");
}

// r[verify lockfile.migration.old-format]
#[test]
fn migrate_old_format() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4",
            "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
            "sha256": "abc123",
            "size": 12345,
            "filename": "htmx.org_htmx.min.js"
        }
    }"#;
    let lockfile = Lockfile::from_json(json).unwrap();
    let dep = &lockfile.dependencies["htmx.org"];
    assert_eq!(dep.version, "2.0.4");
    assert_eq!(dep.files.len(), 1);
    assert_eq!(
        dep.files[0].url,
        "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js"
    );
    assert_eq!(dep.files[0].sha256, "abc123");
    assert_eq!(dep.files[0].size, 12345);
    assert_eq!(dep.files[0].filename, "htmx.org_htmx.min.js");
}

// r[verify lockfile.migration.conflict]
#[test]
fn reject_corrupt_lockfile_both_formats() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4",
            "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
            "sha256": "abc123",
            "size": 12345,
            "filename": "htmx.org_htmx.min.js",
            "files": [{
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
                "sha256": "abc123",
                "size": 12345,
                "filename": "htmx.org_htmx.min.js"
            }]
        }
    }"#;
    let result = Lockfile::from_json(json);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("corrupt"),
        "Expected error about corrupt lockfile, got: {err}"
    );
}

// r[verify lockfile.migration.no-file-data]
#[test]
fn reject_lockfile_entry_with_no_file_data() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4"
        }
    }"#;
    assert!(Lockfile::from_json(json).is_err());
}
