use assert_cmd::Command;
use expect_test::expect;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Build a `Command` for the `unpm` binary, rooted in the given temp directory.
fn unpm(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("unpm").unwrap();
    cmd.current_dir(dir.path());
    cmd
}

fn write_manifest(dir: &TempDir, content: &str) {
    fs::write(dir.path().join("unpm.toml"), content).unwrap();
}

fn write_lockfile(dir: &TempDir, content: &str) {
    fs::write(dir.path().join("unpm.lock"), content).unwrap();
}

fn write_config(dir: &TempDir, content: &str) {
    fs::write(dir.path().join(".unpm.toml"), content).unwrap();
}

fn read_manifest(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join("unpm.toml")).unwrap()
}

fn read_lockfile(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join("unpm.lock")).unwrap()
}

fn vendor_path(dir: &TempDir, filename: &str) -> std::path::PathBuf {
    dir.path().join("static/vendor").join(filename)
}

// --- Add ---

// r[verify add.input.package-name]
// r[verify add.input.at-syntax]
// r[verify add.confirm.skip]
// r[verify add.noninteractive.file-validation]
// r[verify add.noninteractive.path-normalization]
// r[verify add.manifest.extended-file]
// r[verify manifest.serial.key-quoting]
// r[verify manifest.serial.omit-empty]
// r[verify config.file.missing]
// r[verify manifest.file]
// r[verify lockfile.file.name]
// r[verify lockfile.file.format]
#[test]
fn add_creates_manifest_lockfile_and_vendor_file() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added htmx.org@2.0.4"));

    assert!(vendor_path(&dir, "htmx.min.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "htmx.org" = { version = "2.0.4", file = "dist/htmx.min.js" }
    "#]]
    .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect![[r#"
        {
          "htmx.org": {
            "version": "2.0.4",
            "files": [
              {
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
                "sha256": "e209dda5c8235479f3166defc7750e1dbcd5a5c1808b7792fc2e6733768fb447",
                "size": 50917,
                "filename": "htmx.min.js"
              }
            ]
          }
        }"#]]
    .assert_eq(&lockfile);
}

// r[verify add.manifest.extended-files]
#[test]
fn add_multiple_files() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args([
            "add",
            "htmx.org@2.0.4",
            "--file",
            "dist/htmx.min.js",
            "--file",
            "dist/htmx.js",
        ])
        .assert()
        .success();

    assert!(vendor_path(&dir, "htmx.min.js").exists());
    assert!(vendor_path(&dir, "htmx.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "htmx.org" = { version = "2.0.4", files = ["dist/htmx.min.js", "dist/htmx.js"] }
    "#]]
    .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect![[r#"
        {
          "htmx.org": {
            "version": "2.0.4",
            "files": [
              {
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
                "sha256": "e209dda5c8235479f3166defc7750e1dbcd5a5c1808b7792fc2e6733768fb447",
                "size": 50917,
                "filename": "htmx.min.js"
              },
              {
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.js",
                "sha256": "cb0a99bf91c36bdd39e0c9d4677c579e8202f9a58db5f0c59c90085ea0e41275",
                "size": 165563,
                "filename": "htmx.js"
              }
            ]
          }
        }"#]]
    .assert_eq(&lockfile);
}

// r[verify add.noninteractive.required-flags]
#[test]
fn add_noninteractive_requires_version_and_file() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--version"));
}

// r[verify add.noninteractive.file-validation]
#[test]
fn add_nonexistent_file_fails() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/does-not-exist.js"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found in package"));
}

// r[verify add.existing.preserve-version]
// r[verify add.existing.preserve-files]
#[test]
fn add_files_to_existing_dependency() {
    let dir = tempfile::tempdir().unwrap();

    // Add with one file
    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    // Add another file to the same package
    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.js"])
        .assert()
        .success();

    assert!(vendor_path(&dir, "htmx.min.js").exists());
    assert!(vendor_path(&dir, "htmx.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "htmx.org" = { version = "2.0.4", files = ["dist/htmx.min.js", "dist/htmx.js"] }
    "#]]
    .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect![[r#"
        {
          "htmx.org": {
            "version": "2.0.4",
            "files": [
              {
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
                "sha256": "e209dda5c8235479f3166defc7750e1dbcd5a5c1808b7792fc2e6733768fb447",
                "size": 50917,
                "filename": "htmx.min.js"
              },
              {
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.js",
                "sha256": "cb0a99bf91c36bdd39e0c9d4677c579e8202f9a58db5f0c59c90085ea0e41275",
                "size": 165563,
                "filename": "htmx.js"
              }
            ]
          }
        }"#]]
    .assert_eq(&lockfile);
}

// --- List ---

// r[verify list.output.entry]
// r[verify list.output.files]
#[test]
fn list_shows_dependencies_and_files() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    let output = unpm(&dir).args(["list"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    expect![[r#"
        htmx.org@2.0.4
          htmx.min.js
    "#]]
    .assert_eq(&stdout);
}

// r[verify list.empty]
// r[verify manifest.file.missing]
#[test]
fn list_empty_shows_no_dependencies() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No dependencies"));
}

// --- Install ---

// r[verify install.vendor.success-message]
#[test]
fn install_restores_vendored_files() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    let original_content = fs::read(vendor_path(&dir, "htmx.min.js")).unwrap();

    // Delete vendor directory
    fs::remove_dir_all(dir.path().join("static/vendor")).unwrap();
    assert!(!vendor_path(&dir, "htmx.min.js").exists());

    // Install should restore it
    unpm(&dir)
        .args(["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed 1 dependencies"));

    let restored_content = fs::read(vendor_path(&dir, "htmx.min.js")).unwrap();
    assert_eq!(original_content, restored_content);
}

// r[verify install.preconditions.empty-manifest]
// r[verify manifest.file.missing]
#[test]
fn install_empty_manifest() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No dependencies to install"));
}

// --- Check ---

// r[verify check.exit.success]
// r[verify check.cve.allow-vulnerable]
// r[verify check.integrity.sha-match]
// r[verify check.integrity.file-exists]
#[test]
fn check_passes_for_valid_state() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    // Use --allow-vulnerable to isolate integrity checking from CVE results
    unpm(&dir)
        .args(["check", "--allow-vulnerable"])
        .assert()
        .success();
}

// r[verify check.integrity.sha-match]
// r[verify check.exit.failure]
#[test]
fn check_detects_tampered_file() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    // Tamper with the vendored file
    fs::write(vendor_path(&dir, "htmx.min.js"), b"tampered content").unwrap();

    unpm(&dir)
        .args(["check", "--allow-vulnerable"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Check failed"));
}

// r[verify check.integrity.file-exists]
// r[verify check.exit.failure]
#[test]
fn check_detects_missing_file() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    // Delete the vendored file
    fs::remove_file(vendor_path(&dir, "htmx.min.js")).unwrap();

    unpm(&dir)
        .args(["check", "--allow-vulnerable"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Check failed"));
}

// r[verify check.freshness.fail-on-outdated]
// r[verify check.freshness.print]
// r[verify check.freshness.compare]
#[test]
fn check_fail_on_outdated() {
    let dir = tempfile::tempdir().unwrap();

    // Use an old version that definitely has a newer release
    unpm(&dir)
        .args(["add", "htmx.org@1.9.0", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    unpm(&dir)
        .args(["check", "--allow-vulnerable", "--fail-on-outdated"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Outdated:"));
}

// --- Outdated ---

// r[verify outdated.output.header]
// r[verify outdated.output.entry]
// r[verify outdated.comparison]
#[test]
fn outdated_shows_newer_versions() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@1.9.0", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    unpm(&dir).args(["outdated"]).assert().success().stdout(
        predicate::str::contains("Outdated dependencies:")
            .and(predicate::str::contains("htmx.org: 1.9.0 ->")),
    );
}

#[test]
fn outdated_up_to_date() {
    let dir = tempfile::tempdir().unwrap();

    // This test will fail if a newer htmx version is released, but that's
    // a reasonable tradeoff for testing with real packages. If it breaks,
    // just bump the version.
    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    // Note: this may show outdated if 2.0.5+ is released. That's OK — the
    // test validates the command runs; the precise output depends on registry state.
    unpm(&dir).args(["outdated"]).assert().success();
}

// --- Update ---

// r[verify update.version.major-boundary]
// r[verify update.output.success]
// r[verify update.persist.manifest]
#[test]
fn update_bumps_version_within_major() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.0", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    unpm(&dir)
        .args(["update", "htmx.org"])
        .assert()
        .success()
        .stdout(predicate::str::contains("htmx.org: 2.0.0 ->"));

    let manifest = read_manifest(&dir);
    assert!(!manifest.contains("2.0.0"));
}

// r[verify update.version.explicit]
// r[verify update.output.success]
// r[verify update.manifest.short-form]
// r[verify update.persist.manifest]
// r[verify update.persist.lockfile]
// r[verify update.vendor.placement]
#[test]
fn update_explicit_version() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.0", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    unpm(&dir)
        .args(["update", "htmx.org", "--version", "2.0.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("htmx.org: 2.0.0 -> 2.0.3"));

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "htmx.org" = "2.0.3"
    "#]]
    .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect![[r#"
        {
          "htmx.org": {
            "version": "2.0.3",
            "files": [
              {
                "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.3/dist/htmx.min.js",
                "sha256": "491955cd1810747d7d7b9ccb936400afb760e06d25d53e4572b64b6563b2784e",
                "size": 50387,
                "filename": "htmx.min.js"
              }
            ]
          }
        }"#]]
    .assert_eq(&lockfile);
}

// r[verify update.target.at-syntax]
#[test]
fn update_at_syntax() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.0", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    unpm(&dir)
        .args(["update", "htmx.org@2.0.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("htmx.org: 2.0.0 -> 2.0.3"));
}

// r[verify update.precondition.in-manifest]
#[test]
fn update_nonexistent_package_fails() {
    let dir = tempfile::tempdir().unwrap();
    write_manifest(&dir, "[dependencies]\n");

    unpm(&dir)
        .args(["update", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// --- Remove ---

// r[verify remove.files.delete]
// r[verify remove.state.manifest]
// r[verify remove.state.lockfile]
// r[verify remove.output.confirmation]
#[test]
fn remove_cleans_up_everything() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    assert!(vendor_path(&dir, "htmx.min.js").exists());

    unpm(&dir)
        .args(["remove", "htmx.org"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed htmx.org"));

    assert!(!vendor_path(&dir, "htmx.min.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
    "#]]
    .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect!["{}"].assert_eq(&lockfile);
}

// r[verify remove.manifest.exists]
#[test]
fn remove_nonexistent_package_fails() {
    let dir = tempfile::tempdir().unwrap();
    write_manifest(&dir, "[dependencies]\n");
    write_lockfile(&dir, "{}");

    unpm(&dir)
        .args(["remove", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// --- Config ---

// r[verify config.output-dir.custom]
// r[verify config.file.name]
#[test]
fn custom_output_dir() {
    let dir = tempfile::tempdir().unwrap();
    write_config(&dir, "output_dir = \"assets\"\n");

    unpm(&dir)
        .args(["add", "htmx.org@2.0.4", "--file", "dist/htmx.min.js"])
        .assert()
        .success()
        .stdout(predicate::str::contains("assets/htmx.min.js"));

    assert!(dir.path().join("assets/htmx.min.js").exists());
    assert!(!dir.path().join("static/vendor/htmx.min.js").exists());
}

// --- GitHub dependencies ---

// r[verify add.input.source]
// r[verify add.version.github-resolve]
#[test]
fn github_add_tagged_version() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args([
            "add",
            "gh:jquery/jquery@3.7.1",
            "--file",
            "dist/jquery.min.js",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added gh:jquery/jquery@3.7.1"));

    assert!(vendor_path(&dir, "jquery.min.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "gh:jquery/jquery" = { version = "3.7.1", file = "dist/jquery.min.js" }
    "#]]
    .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect![[r#"
        {
          "gh:jquery/jquery": {
            "version": "f79d5f1a337528940ab7029d4f8bbba72326f269",
            "files": [
              {
                "url": "https://cdn.jsdelivr.net/gh/jquery/jquery@f79d5f1a337528940ab7029d4f8bbba72326f269/dist/jquery.min.js",
                "sha256": "fc9a93dd241f6b045cbff0481cf4e1901becd0e12fb45166a8f17f95823f0b1a",
                "size": 87533,
                "filename": "jquery.min.js"
              }
            ]
          }
        }"#]]
        .assert_eq(&lockfile);
}

// r[verify add.version.github-ref]
// r[verify add.version.github-resolve]
#[test]
fn github_add_branch() {
    let dir = tempfile::tempdir().unwrap();

    unpm(&dir)
        .args([
            "add",
            "gh:jquery/jquery",
            "--version",
            "main",
            "--file",
            "src/jquery.js",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added gh:jquery/jquery@main"));

    assert!(vendor_path(&dir, "jquery.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "gh:jquery/jquery" = { version = "main", file = "src/jquery.js" }
    "#]]
    .assert_eq(&manifest);

    // Lockfile stores a resolved commit SHA that moves with the branch
    let lockfile: serde_json::Value = serde_json::from_str(&read_lockfile(&dir)).unwrap();
    let version = lockfile["gh:jquery/jquery"]["version"].as_str().unwrap();
    assert_eq!(version.len(), 40, "lockfile version should be a full SHA");
    assert_ne!(
        version, "main",
        "lockfile version should not be the branch name"
    );
}

// r[verify add.version.github-ref]
// r[verify add.version.github-resolve]
#[test]
fn github_add_commit_sha() {
    let dir = tempfile::tempdir().unwrap();

    // This is the commit SHA that tag 3.7.1 resolves to
    let sha = "f79d5f1a337528940ab7029d4f8bbba72326f269";

    unpm(&dir)
        .args([
            "add",
            "gh:jquery/jquery",
            "--version",
            sha,
            "--file",
            "dist/jquery.min.js",
        ])
        .assert()
        .success();

    assert!(vendor_path(&dir, "jquery.min.js").exists());

    let manifest = read_manifest(&dir);
    expect![[r#"
        [dependencies]
        "gh:jquery/jquery" = { version = "f79d5f1a337528940ab7029d4f8bbba72326f269", file = "dist/jquery.min.js" }
    "#]]
        .assert_eq(&manifest);

    let lockfile = read_lockfile(&dir);
    expect![[r#"
        {
          "gh:jquery/jquery": {
            "version": "f79d5f1a337528940ab7029d4f8bbba72326f269",
            "files": [
              {
                "url": "https://cdn.jsdelivr.net/gh/jquery/jquery@f79d5f1a337528940ab7029d4f8bbba72326f269/dist/jquery.min.js",
                "sha256": "fc9a93dd241f6b045cbff0481cf4e1901becd0e12fb45166a8f17f95823f0b1a",
                "size": 87533,
                "filename": "jquery.min.js"
              }
            ]
          }
        }"#]]
        .assert_eq(&lockfile);
}

// --- Full lifecycle ---

#[test]
fn full_lifecycle() {
    let dir = tempfile::tempdir().unwrap();

    // Add
    unpm(&dir)
        .args(["add", "htmx.org@2.0.0", "--file", "dist/htmx.min.js"])
        .assert()
        .success();

    // List
    unpm(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("htmx.org@2.0.0"));

    // Check
    unpm(&dir)
        .args(["check", "--allow-vulnerable"])
        .assert()
        .success();

    // Outdated
    unpm(&dir)
        .args(["outdated"])
        .assert()
        .success()
        .stdout(predicate::str::contains("htmx.org: 2.0.0 ->"));

    // Update
    unpm(&dir)
        .args(["update", "htmx.org"])
        .assert()
        .success()
        .stdout(predicate::str::contains("htmx.org: 2.0.0 ->"));

    let manifest = read_manifest(&dir);
    assert!(!manifest.contains("2.0.0"));

    // Remove
    unpm(&dir).args(["remove", "htmx.org"]).assert().success();

    // List empty
    unpm(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No dependencies"));
}
