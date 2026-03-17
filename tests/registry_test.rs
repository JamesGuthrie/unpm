use unpm::registry::{PackageSource, Registry};

#[tokio::test]
async fn test_get_npm_package_versions() {
    let registry = Registry::new();
    let source = PackageSource::parse("htmx.org").unwrap();
    let pkg = registry.get_package(&source).await.unwrap();
    assert_eq!(pkg.name, "htmx.org");
    assert!(!pkg.versions.is_empty());
    assert!(pkg.tags.latest.is_some());
}

// r[verify add.resolve.not-found]
#[tokio::test]
async fn test_get_package_not_found() {
    let registry = Registry::new();
    let source = PackageSource::parse("this-package-definitely-does-not-exist-xyz-123").unwrap();
    let result = registry.get_package(&source).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_npm_package_files() {
    let registry = Registry::new();
    let source = PackageSource::parse("htmx.org").unwrap();
    let files = registry.get_package_files(&source, "2.0.4").await.unwrap();
    assert!(files.default.is_some());
    assert!(!files.files.is_empty());
    let has_htmx = files.files.iter().any(|f| f.path.contains("htmx.min.js"));
    assert!(has_htmx);
}

#[tokio::test]
async fn test_get_github_package_versions() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:alpinejs/alpine").unwrap();
    let pkg = registry.get_package(&source).await.unwrap();
    assert_eq!(pkg.name, "alpinejs/alpine");
    assert!(!pkg.versions.is_empty());
}

#[tokio::test]
async fn test_get_github_package_files() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:alpinejs/alpine").unwrap();
    // Use a known version
    let version = &registry
        .get_package(&source)
        .await
        .unwrap()
        .versions
        .last()
        .unwrap()
        .version
        .clone();
    let files = registry.get_package_files(&source, version).await.unwrap();
    assert!(!files.files.is_empty());
}

#[test]
fn test_npm_file_url() {
    let source = PackageSource::parse("htmx.org").unwrap();
    let url = Registry::file_url(&source, "2.0.4", "dist/htmx.min.js");
    assert_eq!(
        url,
        "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js"
    );
}

#[test]
fn test_github_file_url() {
    let source = PackageSource::parse("gh:alpinejs/alpine").unwrap();
    let url = Registry::file_url(&source, "3.14.8", "packages/alpine/dist/cdn.min.js");
    assert_eq!(
        url,
        "https://cdn.jsdelivr.net/gh/alpinejs/alpine@3.14.8/packages/alpine/dist/cdn.min.js"
    );
}

// r[verify manifest.source.default]
// r[verify manifest.source.github-prefix]
// r[verify add.input.github-validation]
#[test]
fn test_parse_package_source() {
    let npm = PackageSource::parse("htmx.org").unwrap();
    assert_eq!(npm, PackageSource::Npm("htmx.org".to_string()));

    let gh = PackageSource::parse("gh:user/repo").unwrap();
    assert_eq!(
        gh,
        PackageSource::GitHub {
            user: "user".to_string(),
            repo: "repo".to_string()
        }
    );

    assert!(PackageSource::parse("gh:noslash").is_err());
    assert!(PackageSource::parse("gh:/repo").is_err());
    assert!(PackageSource::parse("gh:user/").is_err());
}

// r[verify add.version.github-ref]
#[tokio::test]
async fn test_resolve_github_ref_commit_sha() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:jquery/jquery").unwrap();
    // Known commit SHA from jquery repo
    let sha = "32b00373b3f42e5cdcb709df53f3b08b7184a944";
    let result = registry.resolve_github_ref(&source, sha).await.unwrap();
    // SHA resolves directly via GitHub API — manifest and lockfile both get the SHA
    assert_eq!(result.manifest_version, sha);
    assert_eq!(result.lockfile_version, sha);
}

// r[verify add.version.github-resolve]
// r[verify update.version.github-resolve]
#[tokio::test]
async fn test_resolve_github_ref_branch_name() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:jquery/jquery").unwrap();
    let result = registry.resolve_github_ref(&source, "main").await.unwrap();
    // Branch name goes in manifest, resolved SHA goes in lockfile
    assert_eq!(result.manifest_version, "main");
    assert_ne!(result.lockfile_version, "main");
    assert_eq!(result.lockfile_version.len(), 40); // full SHA
}

// r[verify add.version.github-resolve]
// r[verify update.version.github-resolve]
#[tokio::test]
async fn test_resolve_github_ref_semver_tag() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:jquery/jquery").unwrap();
    let result = registry.resolve_github_ref(&source, "3.7.1").await.unwrap();
    // Semver tag: manifest stores the tag, lockfile stores the resolved SHA
    assert_eq!(result.manifest_version, "3.7.1");
    assert_ne!(result.lockfile_version, "3.7.1");
    assert_eq!(result.lockfile_version.len(), 40);
}

#[tokio::test]
async fn test_resolve_github_ref_not_found() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:jquery/jquery").unwrap();
    let result = registry
        .resolve_github_ref(&source, "this-ref-does-not-exist-xyz")
        .await;
    assert!(result.is_err());
}

// r[verify add.input.source]
#[test]
fn test_package_source_display() {
    let npm = PackageSource::parse("htmx.org").unwrap();
    assert_eq!(npm.display_name(), "htmx.org");

    let gh = PackageSource::parse("gh:user/repo").unwrap();
    assert_eq!(gh.display_name(), "gh:user/repo");
}
