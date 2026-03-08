# unpm Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI tool that vendors static JS/CSS/SVG assets into a repository with SHA verification and CVE checking.

**Architecture:** CLI tool with commands (add, install, check, remove). Uses jsdelivr API for package resolution and file fetching, OSV.dev for vulnerability checking. Three config files: `.unpm.toml` (tool config), `unpm.toml` (manifest), `unpm.lock` (lockfile).

**Tech Stack:** Rust, clap, reqwest (rustls), serde, sha2, dialoguer, indicatif, toml, tokio.

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

**Step 1: Initialize the Rust project**

Run: `cargo init --name unpm`

**Step 2: Add dependencies to Cargo.toml**

```toml
[package]
name = "unpm"
version = "0.1.0"
edition = "2024"
description = "Lightweight vendoring of static assets. No node_modules, no runtime fetching."

[dependencies]
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.12", features = ["rustls-tls", "json"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
sha2 = "0.10"
hex = "0.4"
dialoguer = "0.11"
indicatif = "0.17"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

**Step 3: Set up .gitignore**

```
/target
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully.

**Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore
git commit -m "Initialize Rust project with dependencies"
```

---

### Task 2: Data Types & Config Parsing

**Files:**
- Create: `src/config.rs`
- Create: `src/manifest.rs`
- Create: `src/lockfile.rs`
- Modify: `src/main.rs`

**Step 1: Write tests for config parsing**

Create `tests/config_test.rs`:

```rust
use unpm::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.output_dir, "static/vendor");
}

#[test]
fn test_parse_config() {
    let toml_str = r#"output_dir = "assets/vendor""#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.output_dir, "assets/vendor");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test config_test`
Expected: FAIL — module not found.

**Step 3: Implement Config**

`src/config.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_output_dir() -> String {
    "static/vendor".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new(".unpm.toml");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }
}
```

**Step 4: Write tests for manifest parsing**

Create `tests/manifest_test.rs`:

```rust
use unpm::manifest::{Manifest, Dependency};

#[test]
fn test_parse_short_form() {
    let toml_str = r#"
[dependencies]
"htmx.org" = "2.0.4"
"#;
    let manifest: Manifest = toml::from_str(toml_str).unwrap();
    let dep = manifest.dependencies.get("htmx.org").unwrap();
    assert_eq!(dep.version, "2.0.4");
    assert!(dep.file.is_none());
    assert!(dep.url.is_none());
}

#[test]
fn test_parse_extended_form() {
    let toml_str = r#"
[dependencies]
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml_str).unwrap();
    let dep = manifest.dependencies.get("d3").unwrap();
    assert_eq!(dep.version, "7.9.0");
    assert_eq!(dep.file.as_deref(), Some("dist/d3.min.js"));
}

#[test]
fn test_parse_url_form() {
    let toml_str = r#"
[dependencies]
some-lib = { version = "1.0.0", url = "https://example.com/lib.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml_str).unwrap();
    let dep = manifest.dependencies.get("some-lib").unwrap();
    assert_eq!(dep.url.as_deref(), Some("https://example.com/lib.min.js"));
}

#[test]
fn test_parse_ignore_cves() {
    let toml_str = r#"
[dependencies]
d3 = { version = "7.9.0", file = "dist/d3.min.js", ignore-cves = ["CVE-2024-1234"] }
"#;
    let manifest: Manifest = toml::from_str(toml_str).unwrap();
    let dep = manifest.dependencies.get("d3").unwrap();
    assert_eq!(dep.ignore_cves, vec!["CVE-2024-1234"]);
}

#[test]
fn test_roundtrip() {
    let toml_str = r#"
[dependencies]
"htmx.org" = "2.0.4"
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
"#;
    let manifest: Manifest = toml::from_str(toml_str).unwrap();
    let serialized = toml::to_string_pretty(&manifest).unwrap();
    let reparsed: Manifest = toml::from_str(&serialized).unwrap();
    assert_eq!(manifest.dependencies.len(), reparsed.dependencies.len());
}
```

**Step 5: Implement Manifest**

`src/manifest.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Short(String),
    Extended(DependencySpec),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencySpec {
    pub version: String,
    pub file: Option<String>,
    pub url: Option<String>,
    #[serde(default, rename = "ignore-cves")]
    pub ignore_cves: Vec<String>,
}

impl Dependency {
    pub fn version(&self) -> &str {
        match self {
            Dependency::Short(v) => v,
            Dependency::Extended(spec) => &spec.version,
        }
    }

    pub fn file(&self) -> Option<&str> {
        match self {
            Dependency::Short(_) => None,
            Dependency::Extended(spec) => spec.file.as_deref(),
        }
    }

    pub fn url(&self) -> Option<&str> {
        match self {
            Dependency::Short(_) => None,
            Dependency::Extended(spec) => spec.url.as_deref(),
        }
    }

    pub fn ignore_cves(&self) -> &[String] {
        match self {
            Dependency::Short(_) => &[],
            Dependency::Extended(spec) => &spec.ignore_cves,
        }
    }
}

impl Manifest {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new("unpm.toml");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self {
                dependencies: BTreeMap::new(),
            })
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write("unpm.toml", contents)?;
        Ok(())
    }
}
```

Note: The test file uses a flattened `Dependency` struct with direct field access. The actual implementation uses an enum (`Short`/`Extended`) to handle TOML's untagged union. The tests should be adjusted to use the accessor methods (`dep.version()`, `dep.file()`, etc.) rather than direct field access. The implementer should reconcile the test API with the implementation during step 6.

**Step 6: Write tests for lockfile**

Create `tests/lockfile_test.rs`:

```rust
use unpm::lockfile::{Lockfile, LockedDependency};

#[test]
fn test_empty_lockfile() {
    let lockfile = Lockfile::default();
    assert!(lockfile.dependencies.is_empty());
}

#[test]
fn test_lockfile_roundtrip() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "htmx.org".to_string(),
        LockedDependency {
            version: "2.0.4".to_string(),
            url: "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js".to_string(),
            sha256: "abc123".to_string(),
            size: 49012,
        },
    );
    let json = lockfile.to_json().unwrap();
    let reparsed = Lockfile::from_json(&json).unwrap();
    let dep = reparsed.dependencies.get("htmx.org").unwrap();
    assert_eq!(dep.version, "2.0.4");
    assert_eq!(dep.sha256, "abc123");
    assert_eq!(dep.size, 49012);
}
```

**Step 7: Implement Lockfile**

`src/lockfile.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(flatten)]
    pub dependencies: BTreeMap<String, LockedDependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockedDependency {
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

impl Lockfile {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new("unpm.lock");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Self::from_json(&contents)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let contents = self.to_json()?;
        std::fs::write("unpm.lock", contents)?;
        Ok(())
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
```

**Step 8: Wire up modules in main.rs and lib.rs**

Create `src/lib.rs`:

```rust
pub mod config;
pub mod lockfile;
pub mod manifest;
```

Update `src/main.rs`:

```rust
fn main() {
    println!("unpm");
}
```

**Step 9: Run all tests**

Run: `cargo test`
Expected: All tests pass.

**Step 10: Commit**

```bash
git add src/ tests/
git commit -m "Add config, manifest, and lockfile data types with tests"
```

---

### Task 3: jsdelivr Registry Client

**Files:**
- Create: `src/registry.rs`
- Create: `tests/registry_test.rs`
- Modify: `src/lib.rs`

**Step 1: Write tests**

`tests/registry_test.rs` — these are integration tests that hit the real API:

```rust
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
    let result = registry.get_package("this-package-definitely-does-not-exist-xyz-123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_package_files() {
    let registry = Registry::new();
    let files = registry.get_package_files("htmx.org", "2.0.4").await.unwrap();
    assert!(files.default.is_some());
    assert!(!files.files.is_empty());
    // Files should have hashes
    let first_file = files.files.iter().find(|f| f.file_type == "file").unwrap();
    assert!(!first_file.hash.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test registry_test`
Expected: FAIL — module not found.

**Step 3: Implement Registry**

`src/registry.rs`:

```rust
use anyhow::{bail, Context};
use serde::Deserialize;

const JSDELIVR_API: &str = "https://data.jsdelivr.com/v1/packages/npm";

pub struct Registry {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub versions: Vec<VersionInfo>,
    pub tags: Tags,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
    pub latest: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct PackageFiles {
    pub default: Option<String>,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub hash: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub file_type: String,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_package(&self, name: &str) -> anyhow::Result<PackageInfo> {
        let url = format!("{JSDELIVR_API}/{name}");
        let resp = self.client.get(&url).send().await?;
        if resp.status() == 404 {
            bail!("Package '{name}' not found on npm");
        }
        resp.json().await.context("Failed to parse package info")
    }

    pub async fn get_package_files(&self, name: &str, version: &str) -> anyhow::Result<PackageFiles> {
        let url = format!("{JSDELIVR_API}/{name}@{version}");
        let resp = self.client.get(&url).send().await?;
        if resp.status() == 404 {
            bail!("Package '{name}@{version}' not found");
        }
        resp.json().await.context("Failed to parse package files")
    }

    pub fn file_url(name: &str, version: &str, file_path: &str) -> String {
        format!("https://cdn.jsdelivr.net/npm/{name}@{version}/{file_path}")
    }
}
```

Note: The `FileEntry` struct uses a flat structure. The actual jsdelivr API returns a recursive directory tree (directories contain nested `files` arrays). The implementer should check the actual API response shape and adjust — likely `FileEntry` needs a `files: Option<Vec<FileEntry>>` field for directories, and a helper to flatten the tree into a list of file paths.

**Step 4: Add module to lib.rs**

Add `pub mod registry;` to `src/lib.rs`.

**Step 5: Run tests**

Run: `cargo test --test registry_test`
Expected: All tests pass.

**Step 6: Commit**

```bash
git add src/registry.rs src/lib.rs tests/registry_test.rs
git commit -m "Add jsdelivr registry client with API integration tests"
```

---

### Task 4: HTTP Fetching & SHA Verification

**Files:**
- Create: `src/fetch.rs`
- Create: `tests/fetch_test.rs`
- Modify: `src/lib.rs`

**Step 1: Write tests**

`tests/fetch_test.rs`:

```rust
use unpm::fetch::Fetcher;

#[tokio::test]
async fn test_fetch_and_hash() {
    let fetcher = Fetcher::new();
    let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
    let result = fetcher.fetch(url).await.unwrap();
    assert!(!result.bytes.is_empty());
    assert!(!result.sha256.is_empty());
    assert_eq!(result.sha256.len(), 64); // hex-encoded SHA-256
}

#[tokio::test]
async fn test_verify_sha_match() {
    let fetcher = Fetcher::new();
    let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
    let result = fetcher.fetch(url).await.unwrap();
    // Fetch again and verify SHA matches
    let result2 = fetcher.fetch(url).await.unwrap();
    assert_eq!(result.sha256, result2.sha256);
}

#[tokio::test]
async fn test_verify_sha_mismatch() {
    let fetcher = Fetcher::new();
    let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
    let result = fetcher.fetch(url).await.unwrap();
    let verified = fetcher.verify(&result.bytes, "definitely_wrong_hash");
    assert!(!verified);
}
```

**Step 2: Implement Fetcher**

`src/fetch.rs`:

```rust
use anyhow::Context;
use sha2::{Digest, Sha256};

pub struct Fetcher {
    client: reqwest::Client,
}

pub struct FetchResult {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub size: u64,
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch(&self, url: &str) -> anyhow::Result<FetchResult> {
        let resp = self.client.get(url).send().await?;
        let bytes = resp.bytes().await.context("Failed to download")?.to_vec();
        let sha256 = Self::hash(&bytes);
        let size = bytes.len() as u64;
        Ok(FetchResult { bytes, sha256, size })
    }

    pub fn hash(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    pub fn verify(bytes: &[u8], expected_sha256: &str) -> bool {
        Self::hash(bytes) == expected_sha256
    }
}
```

Note: `verify` is a static method, but the test calls it as `fetcher.verify(...)`. Adjust the test or make it an associated method. The implementer should reconcile.

**Step 3: Add module, run tests, commit**

Run: `cargo test --test fetch_test`
Expected: All pass.

```bash
git add src/fetch.rs src/lib.rs tests/fetch_test.rs
git commit -m "Add HTTP fetcher with SHA-256 verification"
```

---

### Task 5: Vendor Module (File Placement)

**Files:**
- Create: `src/vendor.rs`
- Create: `tests/vendor_test.rs`
- Modify: `src/lib.rs`

**Step 1: Write tests**

`tests/vendor_test.rs`:

```rust
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
```

Add `tempfile` as a dev dependency in `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

**Step 2: Implement vendor module**

`src/vendor.rs`:

```rust
use anyhow::Context;
use std::path::Path;

pub fn place_file(output_dir: &Path, filename: &str, content: &[u8]) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create directory: {}", output_dir.display()))?;
    let dest = output_dir.join(filename);
    std::fs::write(&dest, content)
        .with_context(|| format!("Failed to write: {}", dest.display()))?;
    Ok(())
}

pub fn remove_file(output_dir: &Path, filename: &str) -> anyhow::Result<()> {
    let dest = output_dir.join(filename);
    if dest.exists() {
        std::fs::remove_file(&dest)
            .with_context(|| format!("Failed to remove: {}", dest.display()))?;
    }
    Ok(())
}
```

**Step 3: Run tests, commit**

Run: `cargo test --test vendor_test`

```bash
git add src/vendor.rs src/lib.rs tests/vendor_test.rs Cargo.toml Cargo.lock
git commit -m "Add vendor module for file placement"
```

---

### Task 6: CVE Checking (OSV.dev)

**Files:**
- Create: `src/cve.rs`
- Create: `tests/cve_test.rs`
- Modify: `src/lib.rs`

**Step 1: Write tests**

`tests/cve_test.rs`:

```rust
use unpm::cve::CveChecker;

#[tokio::test]
async fn test_check_no_vulnerabilities() {
    let checker = CveChecker::new();
    // htmx.org 2.0.4 should have no known vulns (adjust if this changes)
    let vulns = checker.check("htmx.org", "2.0.4").await.unwrap();
    assert!(vulns.is_empty());
}

#[tokio::test]
async fn test_check_known_vulnerability() {
    let checker = CveChecker::new();
    // lodash 4.17.20 has known vulnerabilities
    let vulns = checker.check("lodash", "4.17.20").await.unwrap();
    assert!(!vulns.is_empty());
    assert!(vulns[0].id.starts_with("GHSA") || vulns[0].id.starts_with("CVE"));
}
```

**Step 2: Implement CveChecker**

`src/cve.rs`:

```rust
use anyhow::Context;
use serde::{Deserialize, Serialize};

const OSV_API: &str = "https://api.osv.dev/v1/query";

pub struct CveChecker {
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Debug, Deserialize)]
struct OsvResponse {
    #[serde(default)]
    vulns: Vec<OsvVulnerability>,
}

#[derive(Debug, Deserialize)]
struct OsvVulnerability {
    id: String,
    summary: Option<String>,
    details: Option<String>,
    severity: Option<Vec<OsvSeverity>>,
}

#[derive(Debug, Deserialize)]
struct OsvSeverity {
    #[serde(rename = "type")]
    severity_type: String,
    score: String,
}

#[derive(Debug)]
pub struct Vulnerability {
    pub id: String,
    pub summary: String,
    pub severity: Option<String>,
}

impl CveChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn check(&self, package_name: &str, version: &str) -> anyhow::Result<Vec<Vulnerability>> {
        let query = OsvQuery {
            package: OsvPackage {
                name: package_name.to_string(),
                ecosystem: "npm".to_string(),
            },
            version: version.to_string(),
        };

        let resp: OsvResponse = self
            .client
            .post(OSV_API)
            .json(&query)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse OSV response")?;

        Ok(resp
            .vulns
            .into_iter()
            .map(|v| Vulnerability {
                id: v.id,
                summary: v.summary.unwrap_or_default(),
                severity: v.severity.and_then(|s| s.first().map(|s| s.score.clone())),
            })
            .collect())
    }
}
```

**Step 3: Run tests, commit**

Run: `cargo test --test cve_test`

```bash
git add src/cve.rs src/lib.rs tests/cve_test.rs
git commit -m "Add CVE checking via OSV.dev API"
```

---

### Task 7: CLI Structure

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`

**Step 1: Implement CLI argument parsing**

`src/cli.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "unpm", about = "Lightweight vendoring of static assets")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a dependency (interactive)
    Add {
        /// Exact npm package name
        package: String,
        /// Package version (default: latest)
        #[arg(long)]
        version: Option<String>,
        /// File path within the package
        #[arg(long)]
        file: Option<String>,
    },
    /// Fetch all dependencies
    Install,
    /// Verify vendored files and check for CVEs
    Check {
        /// Allow known vulnerabilities
        #[arg(long)]
        allow_vulnerable: bool,
    },
    /// Remove a dependency
    Remove {
        /// Package name to remove
        package: String,
    },
}
```

**Step 2: Wire up main.rs**

`src/main.rs`:

```rust
use clap::Parser;
use unpm::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Add { package, version, file } => {
            println!("Adding {package}...");
            unpm::commands::add(&package, version.as_deref(), file.as_deref()).await?;
        }
        Command::Install => {
            unpm::commands::install().await?;
        }
        Command::Check { allow_vulnerable } => {
            unpm::commands::check(allow_vulnerable).await?;
        }
        Command::Remove { package } => {
            unpm::commands::remove(&package)?;
        }
    }

    Ok(())
}
```

**Step 3: Create commands module stub**

Create `src/commands/mod.rs`:

```rust
mod add;
mod check;
mod install;
mod remove;

pub use add::add;
pub use check::check;
pub use install::install;
pub use remove::remove;
```

Create stubs for each: `src/commands/add.rs`, `src/commands/install.rs`, `src/commands/check.rs`, `src/commands/remove.rs`. Each should contain a placeholder function that returns `Ok(())`.

Example `src/commands/add.rs`:

```rust
pub async fn add(_package: &str, _version: Option<&str>, _file: Option<&str>) -> anyhow::Result<()> {
    todo!("implement add command")
}
```

**Step 4: Update lib.rs**

```rust
pub mod cli;
pub mod commands;
pub mod config;
pub mod cve;
pub mod fetch;
pub mod lockfile;
pub mod manifest;
pub mod registry;
pub mod vendor;
```

**Step 5: Verify it compiles**

Run: `cargo build`

**Step 6: Commit**

```bash
git add src/
git commit -m "Add CLI structure with clap and command stubs"
```

---

### Task 8: Implement `install` Command

**Files:**
- Modify: `src/commands/install.rs`

**Step 1: Implement install**

`src/commands/install.rs`:

```rust
use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::vendor;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub async fn install() -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);
    let fetcher = Fetcher::new();

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    let pb = ProgressBar::new(manifest.dependencies.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:30}] {pos}/{len}")
            .unwrap(),
    );
    pb.set_message("Installing");

    for (name, dep) in &manifest.dependencies {
        let locked = lockfile
            .dependencies
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("'{name}' is in unpm.toml but not in unpm.lock. Run `unpm add` first."))?;

        let result = fetcher.fetch(&locked.url).await?;

        if !Fetcher::verify(&result.bytes, &locked.sha256) {
            anyhow::bail!(
                "SHA mismatch for {name}!\nExpected: {}\nGot:      {}\nThe remote file may have been tampered with.",
                locked.sha256,
                result.sha256
            );
        }

        let filename = locked
            .url
            .rsplit('/')
            .next()
            .unwrap_or(name);

        vendor::place_file(output_dir, filename, &result.bytes)?;
        pb.inc(1);
    }

    pb.finish_with_message("Done");
    println!("Installed {} dependencies to {}", manifest.dependencies.len(), config.output_dir);
    Ok(())
}
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/commands/install.rs
git commit -m "Implement install command with SHA verification"
```

---

### Task 9: Implement `add` Command (Interactive)

**Files:**
- Modify: `src/commands/add.rs`

**Step 1: Implement add**

`src/commands/add.rs`:

```rust
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, Lockfile};
use crate::manifest::{Dependency, DependencySpec, Manifest};
use crate::registry::Registry;
use dialoguer::{Confirm, Select};

pub async fn add(package: &str, version: Option<&str>, file: Option<&str>) -> anyhow::Result<()> {
    let registry = Registry::new();

    // 1. Look up exact package name
    println!("Looking up {package}...");
    let pkg = registry.get_package(package).await?;

    // 2. Select version
    let version = if let Some(v) = version {
        v.to_string()
    } else {
        select_version(&pkg)?
    };

    // 3. Get file listing for this version
    let pkg_files = registry.get_package_files(package, &version).await?;

    // 4. Select file
    let file_path = if let Some(f) = file {
        f.to_string()
    } else {
        select_file(&pkg_files)?
    };

    // 5. Build URL and fetch
    let url = Registry::file_url(package, &version, &file_path);
    println!("Fetching {url}...");
    let fetcher = Fetcher::new();
    let result = fetcher.fetch(&url).await?;

    // 6. Confirm
    let filename = file_path.rsplit('/').next().unwrap_or(&file_path);
    println!("\nPackage:  {package}");
    println!("Version:  {version}");
    println!("File:     {file_path}");
    println!("Size:     {} bytes", result.size);
    println!("SHA-256:  {}", result.sha256);

    let is_interactive = atty::is(atty::Stream::Stdin);
    if is_interactive {
        if !Confirm::new().with_prompt("Add this dependency?").default(true).interact()? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // 7. Update manifest
    let mut manifest = Manifest::load()?;
    let dep = if file_path == pkg_files.default.as_deref().unwrap_or("") {
        Dependency::Short(version.clone())
    } else {
        Dependency::Extended(DependencySpec {
            version: version.clone(),
            file: Some(file_path.clone()),
            url: None,
            ignore_cves: vec![],
        })
    };
    manifest.dependencies.insert(package.to_string(), dep);
    manifest.save()?;

    // 8. Update lockfile
    let mut lockfile = Lockfile::load()?;
    lockfile.dependencies.insert(
        package.to_string(),
        LockedDependency {
            version,
            url: url.clone(),
            sha256: result.sha256,
            size: result.size,
        },
    );
    lockfile.save()?;

    // 9. Place file
    let config = crate::config::Config::load()?;
    crate::vendor::place_file(std::path::Path::new(&config.output_dir), filename, &result.bytes)?;

    println!("Added {package} → {}/{filename}", config.output_dir);
    Ok(())
}

fn select_version(pkg: &crate::registry::PackageInfo) -> anyhow::Result<String> {
    let latest = pkg.tags.latest.as_deref().unwrap_or(&pkg.versions[0].version);
    let is_interactive = atty::is(atty::Stream::Stdin);

    if !is_interactive {
        return Ok(latest.to_string());
    }

    let choices = vec![
        format!("{latest} (latest)"),
        "Choose a different version".to_string(),
    ];
    let selection = Select::new()
        .with_prompt("Version")
        .items(&choices)
        .default(0)
        .interact()?;

    if selection == 0 {
        Ok(latest.to_string())
    } else {
        let versions: Vec<&str> = pkg.versions.iter().map(|v| v.version.as_str()).collect();
        let idx = Select::new()
            .with_prompt("Select version")
            .items(&versions)
            .interact()?;
        Ok(versions[idx].to_string())
    }
}

fn select_file(pkg_files: &crate::registry::PackageFiles) -> anyhow::Result<String> {
    let is_interactive = atty::is(atty::Stream::Stdin);
    let default_file = pkg_files.default.as_deref();

    if !is_interactive {
        return default_file
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No default file found. Use --file to specify."));
    }

    // Collect all files (flatten the directory tree)
    let files: Vec<&str> = pkg_files
        .files
        .iter()
        .filter(|f| f.file_type == "file")
        .map(|f| f.name.as_str())
        .collect();

    let mut choices = vec![];
    if let Some(def) = default_file {
        choices.push(format!("Use default: {def}"));
    }
    choices.push("Select file manually".to_string());

    let selection = Select::new()
        .with_prompt("Which file?")
        .items(&choices)
        .default(0)
        .interact()?;

    if selection == 0 && default_file.is_some() {
        // Check for minified variant
        let def = default_file.unwrap();
        let min_variant = def.replace(".js", ".min.js");
        let has_min = files.iter().any(|f| *f == min_variant);
        if has_min && def != min_variant {
            let min_choices = vec!["Minified", "Unminified"];
            let min_sel = Select::new()
                .with_prompt("Minified or unminified?")
                .items(&min_choices)
                .default(0)
                .interact()?;
            if min_sel == 0 {
                return Ok(min_variant);
            }
        }
        Ok(def.to_string())
    } else {
        let idx = Select::new()
            .with_prompt("Select file")
            .items(&files)
            .interact()?;
        Ok(files[idx].to_string())
    }
}
```

Note: This uses the `atty` crate for TTY detection — add `atty = "0.2"` to `Cargo.toml`. The file tree flattening logic is simplified here; as noted in Task 3, the implementer needs to handle the recursive directory structure from jsdelivr's API and flatten it into a list of file paths with their full relative paths.

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Manual test**

Run: `cargo run -- add htmx.org`
Expected: Interactive flow works, creates `unpm.toml`, `unpm.lock`, and places file in `static/vendor/`.

**Step 4: Commit**

```bash
git add src/commands/add.rs Cargo.toml Cargo.lock
git commit -m "Implement interactive add command"
```

---

### Task 10: Implement `check` Command

**Files:**
- Modify: `src/commands/check.rs`

**Step 1: Implement check**

`src/commands/check.rs`:

```rust
use crate::config::Config;
use crate::cve::CveChecker;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::registry::Registry;
use std::path::Path;
use std::process;

pub async fn check(allow_vulnerable: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);

    let mut has_errors = false;
    let fetcher = Fetcher::new();
    let cve_checker = CveChecker::new();
    let registry = Registry::new();

    for (name, dep) in &manifest.dependencies {
        println!("Checking {name}...");

        // 1. Verify lockfile entry exists
        let locked = match lockfile.dependencies.get(name) {
            Some(l) => l,
            None => {
                eprintln!("  ✗ Not in lockfile. Run `unpm add {name}`.");
                has_errors = true;
                continue;
            }
        };

        // 2. Verify vendored file exists and SHA matches
        let filename = locked.url.rsplit('/').next().unwrap_or(name);
        let file_path = output_dir.join(filename);
        if file_path.exists() {
            let contents = std::fs::read(&file_path)?;
            if !Fetcher::verify(&contents, &locked.sha256) {
                eprintln!("  ✗ SHA mismatch for vendored file!");
                has_errors = true;
            } else {
                println!("  ✓ SHA verified");
            }
        } else {
            eprintln!("  ✗ Vendored file missing: {}", file_path.display());
            has_errors = true;
        }

        // 3. Check for CVEs
        let vulns = cve_checker.check(name, dep.version()).await?;
        if !vulns.is_empty() {
            let ignored = dep.ignore_cves();
            let unignored: Vec<_> = vulns
                .iter()
                .filter(|v| !ignored.contains(&v.id))
                .collect();

            if !unignored.is_empty() {
                for v in &unignored {
                    eprintln!("  ⚠ {} — {}", v.id, v.summary);
                }
                if !allow_vulnerable {
                    has_errors = true;
                }
            }
        }

        // 4. Check for newer versions
        if let Ok(pkg) = registry.get_package(name).await {
            if let Some(latest) = &pkg.tags.latest {
                if latest != dep.version() {
                    println!("  ℹ Newer version available: {latest}");
                }
            }
        }
    }

    if has_errors {
        eprintln!("\nCheck failed.");
        process::exit(1);
    }

    println!("\nAll checks passed.");
    Ok(())
}
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/commands/check.rs
git commit -m "Implement check command with SHA verification and CVE checking"
```

---

### Task 11: Implement `remove` Command

**Files:**
- Modify: `src/commands/remove.rs`

**Step 1: Implement remove**

`src/commands/remove.rs`:

```rust
use crate::config::Config;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::vendor;
use std::path::Path;

pub fn remove(package: &str) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut manifest = Manifest::load()?;
    let mut lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);

    if manifest.dependencies.remove(package).is_none() {
        anyhow::bail!("Package '{package}' not found in unpm.toml");
    }

    if let Some(locked) = lockfile.dependencies.remove(package) {
        let filename = locked.url.rsplit('/').next().unwrap_or(package);
        vendor::remove_file(output_dir, filename)?;
    }

    manifest.save()?;
    lockfile.save()?;

    println!("Removed {package}");
    Ok(())
}
```

**Step 2: Verify it compiles, commit**

```bash
git add src/commands/remove.rs
git commit -m "Implement remove command"
```

---

### Task 12: GitHub Action

**Files:**
- Create: `action.yml`
- Create: `action/entrypoint.sh`

**Step 1: Create action.yml**

```yaml
name: "unpm check"
description: "Verify vendored dependencies: SHA integrity and CVE scanning"
branding:
  icon: "shield"
  color: "green"

inputs:
  allow-vulnerable:
    description: "Allow known vulnerabilities (not recommended)"
    required: false
    default: "false"
  version:
    description: "unpm version to use"
    required: false
    default: "latest"

runs:
  using: "composite"
  steps:
    - name: Install unpm
      shell: bash
      run: |
        VERSION="${{ inputs.version }}"
        if [ "$VERSION" = "latest" ]; then
          URL=$(curl -s https://api.github.com/repos/unpm/unpm/releases/latest | grep browser_download_url | grep linux-x86_64 | cut -d '"' -f 4)
        else
          URL="https://github.com/unpm/unpm/releases/download/v${VERSION}/unpm-linux-x86_64"
        fi
        curl -sSL "$URL" -o /usr/local/bin/unpm
        chmod +x /usr/local/bin/unpm

    - name: Run unpm check
      shell: bash
      run: |
        ARGS="check"
        if [ "${{ inputs.allow-vulnerable }}" = "true" ]; then
          ARGS="$ARGS --allow-vulnerable"
        fi
        unpm $ARGS
```

**Step 2: Commit**

```bash
git add action.yml
git commit -m "Add GitHub Action for unpm check"
```

---

### Task 13: Release Build Setup

**Files:**
- Create: `.github/workflows/release.yml`
- Create: `.github/workflows/ci.yml`

**Step 1: CI workflow**

`.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
```

**Step 2: Release workflow**

`.github/workflows/release.yml`:

```yaml
name: Release
on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            artifact: unpm-linux-x86_64
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            artifact: unpm-linux-aarch64
          - target: x86_64-apple-darwin
            os: macos-latest
            artifact: unpm-darwin-x86_64
          - target: aarch64-apple-darwin
            os: macos-latest
            artifact: unpm-darwin-aarch64

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/unpm

  release:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4
      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            unpm-linux-x86_64/unpm
            unpm-linux-aarch64/unpm
            unpm-darwin-x86_64/unpm
            unpm-darwin-aarch64/unpm
```

**Step 3: Commit**

```bash
git add .github/
git commit -m "Add CI and release workflows"
```
