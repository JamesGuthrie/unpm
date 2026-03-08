use unpm::registry::Registry;

#[tokio::test]
async fn test_get_package_versions() {
    let registry = Registry::new();
    let pkg = registry.get_package("htmx.org").await.unwrap();
    assert_eq!(pkg.name, "htmx.org");
    assert!(!pkg.versions.is_empty());
}

#[tokio::test]
async fn test_get_package_not_found() {
    let registry = Registry::new();
    let result = registry
        .get_package("this-package-definitely-does-not-exist-xyz-123")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_package_files() {
    let registry = Registry::new();
    let files = registry
        .get_package_files("htmx.org", "2.0.4")
        .await
        .unwrap();
    assert!(files.default.is_some());
    assert!(!files.files.is_empty());
    let has_htmx = files.files.iter().any(|f| f.path.contains("htmx.min.js"));
    assert!(has_htmx);
}

#[test]
fn test_file_url() {
    let url = Registry::file_url("htmx.org", "2.0.4", "dist/htmx.min.js");
    assert_eq!(
        url,
        "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js"
    );
}
