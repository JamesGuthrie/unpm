use unpm::lockfile::{LockedDependency, Lockfile};

#[test]
fn empty_lockfile() {
    let lockfile = Lockfile::default();
    assert!(lockfile.dependencies.is_empty());
    let json = lockfile.to_json().unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn roundtrip_json() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "htmx.org".to_string(),
        LockedDependency {
            version: "2.0.4".to_string(),
            url: "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js".to_string(),
            sha256: "abc123".to_string(),
            size: 12345,
            filename: "htmx.org_htmx.min.js".to_string(),
        },
    );

    let json = lockfile.to_json().unwrap();
    let reparsed = Lockfile::from_json(&json).unwrap();

    assert_eq!(reparsed.dependencies.len(), 1);
    let dep = &reparsed.dependencies["htmx.org"];
    assert_eq!(dep.version, "2.0.4");
    assert_eq!(dep.sha256, "abc123");
    assert_eq!(dep.size, 12345);
    assert_eq!(dep.filename, "htmx.org_htmx.min.js");
}

#[test]
fn from_json_string() {
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
    assert_eq!(lockfile.dependencies.len(), 1);
    assert_eq!(lockfile.dependencies["htmx.org"].version, "2.0.4");
    assert_eq!(lockfile.dependencies["htmx.org"].filename, "htmx.org_htmx.min.js");
}
