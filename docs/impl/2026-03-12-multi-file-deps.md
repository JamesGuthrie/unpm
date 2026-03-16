# Multi-File Dependencies Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support vendoring multiple files from a single npm/GitHub package (e.g. uPlot JS + CSS).

**Architecture:** The lockfile migrates to always use a `files: [...]` array (with auto-migration of old format). The manifest gains an optional `files` field alongside the existing `file` field. All commands are updated to iterate over file arrays. A shared `extract_file_path` utility is extracted for reuse across `update` and `check`.

**Tech Stack:** Rust, serde, clap, dialoguer (MultiSelect), tokio

**Spec:** `docs/specs/2026-03-12-multi-file-deps-design.md`

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `src/lockfile.rs` | Modify | New `LockedFile` struct, `files` array, auto-migration |
| `src/manifest.rs` | Modify | Add `files` field to Extended, validation, serialization |
| `src/url.rs` | Create | Shared `extract_file_path` utility |
| `src/lib.rs` | Modify | Add `pub mod url;` |
| `src/vendor.rs` | Modify | Update `clean_if_canonical` for multi-file lockfile |
| `src/cli.rs` | Modify | `--file` becomes `Vec<String>` |
| `src/main.rs` | Modify | Pass `Vec` to `add` |
| `src/commands/add.rs` | Modify | Multi-file add, merge, multi-select |
| `src/commands/install.rs` | Modify | Iterate `files` array |
| `src/commands/check.rs` | Modify | Per-file integrity checks |
| `src/commands/update.rs` | Modify | Per-file update, use shared utility |
| `src/commands/remove.rs` | Modify | Remove all files |
| `src/commands/list.rs` | Modify | Tree output format |
| `tests/lockfile_test.rs` | Modify | New format tests, migration tests |
| `tests/manifest_test.rs` | Modify | `files` field tests, validation tests |

---

## Chunk 1: Lockfile Migration

### Task 1: New lockfile structs and serialization

**Files:**
- Modify: `src/lockfile.rs`
- Test: `tests/lockfile_test.rs`

- [ ] **Step 1: Write failing tests for new lockfile format**

In `tests/lockfile_test.rs`, add tests for the new format. Keep existing tests — they'll be updated in step 5.

```rust
use unpm::lockfile::{LockedDependency, LockedFile, Lockfile};

#[test]
fn new_format_roundtrip() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "htmx.org".to_string(),
        LockedDependency {
            version: "2.0.4".to_string(),
            files: vec![LockedFile {
                url: "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js".to_string(),
                sha256: "abc123".to_string(),
                size: 12345,
                filename: "htmx.min.js".to_string(),
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
    assert_eq!(dep.files[0].filename, "htmx.min.js");
}

#[test]
fn new_format_multi_file() {
    let mut lockfile = Lockfile::default();
    lockfile.dependencies.insert(
        "uplot".to_string(),
        LockedDependency {
            version: "1.6.31".to_string(),
            files: vec![
                LockedFile {
                    url: "https://cdn.jsdelivr.net/npm/uplot@1.6.31/dist/uPlot.min.js".to_string(),
                    sha256: "def456".to_string(),
                    size: 45000,
                    filename: "uPlot.min.js".to_string(),
                },
                LockedFile {
                    url: "https://cdn.jsdelivr.net/npm/uplot@1.6.31/dist/uPlot.min.css".to_string(),
                    sha256: "ghi789".to_string(),
                    size: 3200,
                    filename: "uPlot.min.css".to_string(),
                },
            ],
        },
    );

    let json = lockfile.to_json().unwrap();
    let reparsed = Lockfile::from_json(&json).unwrap();

    let dep = &reparsed.dependencies["uplot"];
    assert_eq!(dep.files.len(), 2);
    assert_eq!(dep.files[0].filename, "uPlot.min.js");
    assert_eq!(dep.files[1].filename, "uPlot.min.css");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test lockfile_test`
Expected: Compilation error — `LockedFile` does not exist, `LockedDependency` has no `files` field.

- [ ] **Step 3: Implement new lockfile structs**

Replace the contents of `src/lockfile.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(flatten)]
    pub dependencies: BTreeMap<String, LockedDependency>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LockedDependency {
    pub version: String,
    pub files: Vec<LockedFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LockedFile {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub filename: String,
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

- [ ] **Step 4: Update existing lockfile tests**

Update the existing tests in `tests/lockfile_test.rs` to use the new structs:

```rust
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
    assert_eq!(dep.files[0].sha256, "abc123");
    assert_eq!(dep.files[0].size, 12345);
    assert_eq!(dep.files[0].filename, "htmx.org_htmx.min.js");
}

#[test]
fn from_json_string() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4",
            "files": [
                {
                    "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
                    "sha256": "abc123",
                    "size": 12345,
                    "filename": "htmx.org_htmx.min.js"
                }
            ]
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
```

- [ ] **Step 5: Run lockfile tests to verify they pass**

Run: `cargo test --test lockfile_test`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/lockfile.rs tests/lockfile_test.rs
git commit -m "feat: migrate lockfile to files array format"
```

### Task 2: Lockfile auto-migration from old format

**Files:**
- Modify: `src/lockfile.rs`
- Test: `tests/lockfile_test.rs`

- [ ] **Step 1: Write failing test for old format migration**

```rust
#[test]
fn migrate_old_format() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4",
            "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
            "sha256": "abc123",
            "size": 12345,
            "filename": "htmx.min.js"
        }
    }"#;
    let lockfile = Lockfile::from_json(json).unwrap();
    let dep = &lockfile.dependencies["htmx.org"];
    assert_eq!(dep.version, "2.0.4");
    assert_eq!(dep.files.len(), 1);
    assert_eq!(dep.files[0].url, "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js");
    assert_eq!(dep.files[0].sha256, "abc123");
    assert_eq!(dep.files[0].size, 12345);
    assert_eq!(dep.files[0].filename, "htmx.min.js");
}

#[test]
fn reject_corrupt_lockfile_both_formats() {
    let json = r#"{
        "htmx.org": {
            "version": "2.0.4",
            "url": "https://example.com/htmx.min.js",
            "sha256": "abc123",
            "size": 12345,
            "filename": "htmx.min.js",
            "files": [
                { "url": "https://example.com/htmx.min.js", "sha256": "abc123", "size": 12345, "filename": "htmx.min.js" }
            ]
        }
    }"#;
    assert!(Lockfile::from_json(json).is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test lockfile_test migrate_old_format reject_corrupt`
Expected: FAIL — old format doesn't parse into new struct.

- [ ] **Step 3: Implement auto-migration with custom deserializer**

In `src/lockfile.rs`, add a custom deserializer for `LockedDependency`:

```rust
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(flatten)]
    pub dependencies: BTreeMap<String, LockedDependency>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct LockedDependency {
    pub version: String,
    pub files: Vec<LockedFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LockedFile {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub filename: String,
}

impl<'de> Deserialize<'de> for LockedDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            version: String,
            #[serde(default)]
            files: Option<Vec<LockedFile>>,
            // Old format fields
            url: Option<String>,
            sha256: Option<String>,
            size: Option<u64>,
            filename: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let has_old = raw.url.is_some() || raw.sha256.is_some() || raw.size.is_some() || raw.filename.is_some();
        let has_new = raw.files.is_some();

        if has_old && has_new {
            return Err(serde::de::Error::custom(
                "lockfile entry has both old flat fields and files array — lockfile is corrupt",
            ));
        }

        let files = if let Some(files) = raw.files {
            files
        } else if has_old {
            vec![LockedFile {
                url: raw.url.ok_or_else(|| serde::de::Error::missing_field("url"))?,
                sha256: raw.sha256.ok_or_else(|| serde::de::Error::missing_field("sha256"))?,
                size: raw.size.ok_or_else(|| serde::de::Error::missing_field("size"))?,
                filename: raw.filename.ok_or_else(|| serde::de::Error::missing_field("filename"))?,
            }]
        } else {
            return Err(serde::de::Error::custom(
                "lockfile entry has neither files array nor old flat fields",
            ));
        };

        Ok(LockedDependency {
            version: raw.version,
            files,
        })
    }
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

- [ ] **Step 4: Run all lockfile tests**

Run: `cargo test --test lockfile_test`
Expected: All pass (new format, old format migration, corrupt detection).

- [ ] **Step 5: Commit**

```bash
git add src/lockfile.rs tests/lockfile_test.rs
git commit -m "feat: auto-migrate old lockfile format to files array"
```

---

## Chunk 2: Manifest and Shared Utilities

### Task 3: Add `files` field to manifest

**Files:**
- Modify: `src/manifest.rs`
- Test: `tests/manifest_test.rs`

- [ ] **Step 1: Write failing tests for `files` field**

Add to `tests/manifest_test.rs`:

```rust
#[test]
fn parse_files_form() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", files = ["dist/uPlot.min.js", "dist/uPlot.min.css"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["uplot"];
    assert_eq!(dep.version(), "1.6.31");
    assert_eq!(
        dep.files(),
        Some(&["dist/uPlot.min.js".to_string(), "dist/uPlot.min.css".to_string()][..])
    );
    assert_eq!(dep.file(), None);
}

#[test]
fn files_single_element_valid() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", files = ["dist/uPlot.min.js"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    let dep = &manifest.dependencies["uplot"];
    assert_eq!(dep.files().unwrap().len(), 1);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test manifest_test parse_files_form files_single_element`
Expected: Compilation error — `files()` method doesn't exist.

- [ ] **Step 3: Add `files` field to `Dependency::Extended`**

In `src/manifest.rs`, add the field and accessor:

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    Short(String),
    Extended {
        version: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        files: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, rename = "ignore-cves", skip_serializing_if = "Vec::is_empty")]
        ignore_cves: Vec<String>,
    },
}
```

Add the accessor method in `impl Dependency`:

```rust
pub fn files(&self) -> Option<&[String]> {
    match self {
        Dependency::Short(_) => None,
        Dependency::Extended { files, .. } => files.as_deref(),
    }
}
```

Update `Manifest::save()` to handle the `files` field. In the `Extended` match arm, add destructuring for `files` and add the serialization branch:

```rust
Dependency::Extended {
    version,
    source,
    file,
    files,
    url,
    ignore_cves,
} => {
    let mut fields = vec![format!("version = {}", toml_string(version))];
    if let Some(s) = source {
        fields.push(format!("source = {}", toml_string(s)));
    }
    if let Some(f) = file {
        fields.push(format!("file = {}", toml_string(f)));
    }
    if let Some(fs) = files {
        let items: Vec<String> = fs.iter().map(|f| toml_string(f)).collect();
        fields.push(format!("files = [{}]", items.join(", ")));
    }
    if let Some(u) = url {
        fields.push(format!("url = {}", toml_string(u)));
    }
    if !ignore_cves.is_empty() {
        let cves: Vec<String> =
            ignore_cves.iter().map(|c| toml_string(c)).collect();
        fields.push(format!("ignore-cves = [{}]", cves.join(", ")));
    }
    writeln!(out, "{key} = {{ {} }}", fields.join(", "))?;
}
```

- [ ] **Step 4: Write test for `Manifest::save()` roundtrip with `files`**

Add to `tests/manifest_test.rs`:

```rust
#[test]
fn save_roundtrip_with_files() {
    use std::collections::BTreeMap;
    let manifest = Manifest {
        dependencies: BTreeMap::from([
            ("htmx.org".to_string(), Dependency::Short("2.0.4".to_string())),
            ("uplot".to_string(), Dependency::Extended {
                version: "1.6.31".to_string(),
                source: None,
                file: None,
                files: Some(vec!["dist/uPlot.min.js".to_string(), "dist/uPlot.min.css".to_string()]),
                url: None,
                ignore_cves: Vec::new(),
            }),
        ]),
    };

    // Use save() which writes via the manual serializer, then load it back
    manifest.save().unwrap();
    let contents = std::fs::read_to_string("unpm.toml").unwrap();
    let reparsed: Manifest = toml::from_str(&contents).unwrap();

    assert_eq!(reparsed.dependencies["uplot"].version(), "1.6.31");
    assert_eq!(
        reparsed.dependencies["uplot"].files(),
        Some(&["dist/uPlot.min.js".to_string(), "dist/uPlot.min.css".to_string()][..])
    );
}
```

Note: this test writes to the working directory. Run it in isolation or use a temp dir wrapper if needed.

- [ ] **Step 5: Run manifest tests**

Run: `cargo test --test manifest_test`
Expected: All pass (including existing tests).

- [ ] **Step 6: Commit**

```bash
git add src/manifest.rs tests/manifest_test.rs
git commit -m "feat: add files field to manifest dependency"
```

### Task 4: Manifest validation (file + files, url + files, empty files)

**Files:**
- Modify: `src/manifest.rs`
- Test: `tests/manifest_test.rs`

- [ ] **Step 1: Write failing validation tests**

```rust
#[test]
fn reject_file_and_files() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", file = "dist/uPlot.min.js", files = ["dist/uPlot.min.css"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn reject_url_and_files() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", url = "https://example.com/uplot.js", files = ["dist/uPlot.min.css"] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn reject_empty_files() {
    let toml = r#"
[dependencies]
uplot = { version = "1.6.31", files = [] }
"#;
    let manifest: Manifest = toml::from_str(toml).unwrap();
    assert!(manifest.validate().is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test manifest_test reject_`
Expected: Compilation error — `validate()` doesn't exist.

- [ ] **Step 3: Implement validate method**

Add to `impl Manifest` in `src/manifest.rs`:

```rust
pub fn validate(&self) -> anyhow::Result<()> {
    for (name, dep) in &self.dependencies {
        if let Dependency::Extended { file, files, url, .. } = dep {
            if file.is_some() && files.is_some() {
                anyhow::bail!("{name}: `file` and `files` are mutually exclusive");
            }
            if url.is_some() && files.is_some() {
                anyhow::bail!("{name}: `url` and `files` are mutually exclusive");
            }
            if let Some(fs) = files {
                if fs.is_empty() {
                    anyhow::bail!("{name}: `files` must not be empty");
                }
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Run validation tests**

Run: `cargo test --test manifest_test reject_`
Expected: All pass.

- [ ] **Step 5: Add `validate()` call to `Manifest::load()`**

In `Manifest::load()`, call validate after parsing:

```rust
pub fn load() -> anyhow::Result<Self> {
    let path = std::path::Path::new("unpm.toml");
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&contents)?;
        manifest.validate()?;
        Ok(manifest)
    } else {
        Ok(Self {
            dependencies: BTreeMap::new(),
        })
    }
}
```

- [ ] **Step 6: Run all manifest tests**

Run: `cargo test --test manifest_test`
Expected: All pass.

- [ ] **Step 7: Commit**

```bash
git add src/manifest.rs tests/manifest_test.rs
git commit -m "feat: validate manifest file/files/url mutual exclusion"
```

### Task 5: Extract shared `extract_file_path` utility

**Files:**
- Create: `src/url.rs`
- Modify: `src/lib.rs`
- Modify: `src/commands/update.rs`

- [ ] **Step 1: Create `src/url.rs` with the extracted function**

```rust
use anyhow::{Result, bail};

/// Extract the file path portion from a jsdelivr CDN URL.
/// e.g. "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js" -> "dist/htmx.min.js"
pub fn extract_file_path(url: &str, version: &str) -> Result<String> {
    let marker = format!("@{version}/");
    let idx = url
        .find(&marker)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse file path from URL: {url}"))?;
    let path = &url[idx + marker.len()..];
    if path.is_empty() {
        bail!("No file path found in URL: {url}");
    }
    Ok(path.to_string())
}
```

- [ ] **Step 2: Add `pub mod url;` to `src/lib.rs`**

Add after the existing modules:
```rust
pub mod url;
```

- [ ] **Step 3: Update `src/commands/update.rs` to use the shared utility**

Remove the local `extract_file_path` function (lines 170-181). Change the import and call site:

Replace:
```rust
let file_path = extract_file_path(&locked.url, &old_version)?;
```
With:
```rust
let file_path = crate::url::extract_file_path(&locked.url, &old_version)?;
```

(The `locked.url` reference will need updating in a later task when update is migrated to multi-file. For now, just move the function.)

- [ ] **Step 4: Write unit tests for `extract_file_path`**

Add `#[cfg(test)]` module at the bottom of `src/url.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_npm_path() {
        let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js";
        assert_eq!(extract_file_path(url, "2.0.4").unwrap(), "dist/htmx.min.js");
    }

    #[test]
    fn extracts_github_path() {
        let url = "https://cdn.jsdelivr.net/gh/user/repo@1.0.0/dist/lib.js";
        assert_eq!(extract_file_path(url, "1.0.0").unwrap(), "dist/lib.js");
    }

    #[test]
    fn errors_on_missing_version() {
        let url = "https://cdn.jsdelivr.net/npm/htmx.org/dist/htmx.min.js";
        assert!(extract_file_path(url, "2.0.4").is_err());
    }

    #[test]
    fn errors_on_empty_path() {
        let url = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/";
        assert!(extract_file_path(url, "2.0.4").is_err());
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test url::tests`
Expected: All pass.

- [ ] **Step 6: Verify full compilation**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 7: Commit**

```bash
git add src/url.rs src/lib.rs src/commands/update.rs
git commit -m "refactor: extract shared extract_file_path utility"
```

---

## Chunk 3: Update Commands for Multi-File Lockfile

All commands currently reference `locked.url`, `locked.sha256`, `locked.size`, and `locked.filename` directly. They need to iterate over `locked.files` instead.

### Task 6: Update `vendor.rs` for multi-file lockfile

**Files:**
- Modify: `src/vendor.rs`

- [ ] **Step 1: Update `clean_if_canonical`**

Change the `known` set construction from:
```rust
let known: HashSet<&str> = lockfile
    .dependencies
    .values()
    .map(|l| l.filename.as_str())
    .collect();
```
To:
```rust
let known: HashSet<&str> = lockfile
    .dependencies
    .values()
    .flat_map(|l| l.files.iter().map(|f| f.filename.as_str()))
    .collect();
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src/vendor.rs
git commit -m "feat: update canonical clean for multi-file lockfile"
```

### Task 7: Update `install` command

**Files:**
- Modify: `src/commands/install.rs`

- [ ] **Step 1: Update install to iterate `files` array**

Replace the contents of `src/commands/install.rs`:

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
    let client = reqwest::Client::new();
    let fetcher = Fetcher::with_client(client);

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    let total_files: u64 = lockfile
        .dependencies
        .values()
        .map(|l| l.files.len() as u64)
        .sum();

    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:30}] {pos}/{len}")
            .unwrap(),
    );
    pb.set_message("Installing");

    for (name, dep) in &manifest.dependencies {
        let locked = lockfile.dependencies.get(name).ok_or_else(|| {
            anyhow::anyhow!("'{name}' is in unpm.toml but not in unpm.lock. Run `unpm add` first.")
        })?;

        for locked_file in &locked.files {
            // Use custom URL from manifest if specified (single-file deps only)
            let url = if locked.files.len() == 1 {
                dep.url().unwrap_or(&locked_file.url)
            } else {
                &locked_file.url
            };

            let result = fetcher.fetch(url).await?;

            if !Fetcher::verify(&result.bytes, &locked_file.sha256) {
                anyhow::bail!(
                    "SHA mismatch for {name} ({})!\nExpected: {}\nGot:      {}\nThe remote file may have been tampered with.",
                    locked_file.filename,
                    locked_file.sha256,
                    result.sha256
                );
            }

            vendor::place_file(output_dir, &locked_file.filename, &result.bytes)?;
            pb.inc(1);
        }
    }

    pb.finish_with_message("Done");

    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

    println!(
        "Installed {} dependencies to {}",
        manifest.dependencies.len(),
        config.output_dir
    );
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src/commands/install.rs
git commit -m "feat: update install command for multi-file lockfile"
```

### Task 8: Update `check` command

**Files:**
- Modify: `src/commands/check.rs`

- [ ] **Step 1: Update check to iterate per file**

The key changes in `src/commands/check.rs`:

1. The local SHA verification loop needs to iterate `locked.files`:

Replace the SHA verification block (the `for (name, dep)` loop body, lines 67-131) with:

```rust
for (name, dep) in &manifest.dependencies {
    log::debug!("checking {name} v{}", dep.version());

    let locked = match lockfile.dependencies.get(name) {
        Some(l) => l,
        None => {
            integrity_errors.push(format!("  {name}: not in lockfile, run `unpm add` first"));
            continue;
        }
    };

    for locked_file in &locked.files {
        log::debug!("  lockfile filename: {}", locked_file.filename);

        let file_path = output_dir.join(&locked_file.filename);

        let local_sha256 = match std::fs::read(&file_path) {
            Ok(bytes) => {
                let hash = Fetcher::hash(&bytes);
                if hash != locked_file.sha256 {
                    integrity_errors
                        .push(format!("  {name}: SHA mismatch for {}", locked_file.filename));
                }
                hash
            }
            Err(_) => {
                integrity_errors.push(format!(
                    "  {name}: vendored file not found ({})",
                    file_path.display()
                ));
                continue;
            }
        };

        // Queue CDN hash verification (per file)
        if let Ok(source) = PackageSource::from_manifest(name, dep.source()) {
            tasks.push(CheckTask::CdnHash {
                name: name.clone(),
                source,
                version: dep.version().to_string(),
                local_sha256,
                url: locked_file.url.clone(),
            });
        }
    }

    // Queue CVE check (per package)
    let source = PackageSource::from_manifest(name, dep.source()).ok();
    let cve_name = match &source {
        Some(PackageSource::Npm(n)) => n.clone(),
        _ => name.clone(),
    };
    tasks.push(CheckTask::Cve {
        name: name.clone(),
        cve_name,
        version: dep.version().to_string(),
        ignore_cves: dep.ignore_cves().to_vec(),
    });

    // Queue outdated check (per package)
    if let Ok(source) = PackageSource::from_manifest(name, dep.source()) {
        tasks.push(CheckTask::Outdated {
            name: name.clone(),
            current: dep.version().to_string(),
            source,
        });
    }
}
```

2. Update the `CdnHash` variant in `CheckTask` to store the file URL instead of the vendored filename:

```rust
CdnHash {
    name: String,
    source: PackageSource,
    version: String,
    local_sha256: String,
    url: String,  // was: filename: String
},
```

3. Update the CDN hash check in the async block to use `extract_file_path` from the URL:

```rust
CheckTask::CdnHash {
    name,
    source,
    version,
    local_sha256,
    url,
} => {
    let result = async {
        let file_path = crate::url::extract_file_path(&url, &version)?;
        let pkg_files = registry.get_package_files(&source, &version).await?;
        log::debug!(
            "{name}: looking for path '{file_path}' in CDN file list"
        );
        let entry = pkg_files.files.iter().find(|f| f.path == file_path);
        if entry.is_none() {
            log::debug!(
                "{name}: no matching file found in {} CDN entries",
                pkg_files.files.len()
            );
        }
        Ok(entry.map(|e| e.hash.clone()))
    }
    .await;
    CheckResult::CdnHash {
        name,
        result,
        local_sha256,
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src/commands/check.rs
git commit -m "feat: update check command for multi-file lockfile"
```

### Task 9: Update `update` command

**Files:**
- Modify: `src/commands/update.rs`

- [ ] **Step 1: Update to iterate files with atomic failure**

The key changes in the update loop (the `for name in &packages` body):

Replace the single-file fetch/update block (from `let file_path = ...` through the lockfile insert) with a per-file loop:

```rust
struct FetchedFile {
    locked: crate::lockfile::LockedFile,
    bytes: Vec<u8>,
}

// Fetch all files at the new version
let mut fetched: Vec<FetchedFile> = Vec::new();
for locked_file in &locked.files {
    let file_path = crate::url::extract_file_path(&locked_file.url, &old_version)?;
    let url = Registry::file_url(&source, &new_version, &file_path);

    let result = match fetcher.fetch(&url).await {
        Ok(r) => r,
        Err(e) => {
            bail!(
                "{name}: failed to fetch '{file_path}' at version {new_version}: {e}\n\
                 Adjust the `files` list in unpm.toml and retry."
            );
        }
    };

    fetched.push(FetchedFile {
        locked: crate::lockfile::LockedFile {
            url,
            sha256: result.sha256,
            size: result.size,
            filename: locked_file.filename.clone(),
        },
        bytes: result.bytes.to_vec(),
    });
}

let new_dep = match dep {
    Dependency::Short(_) => Dependency::Short(new_version.clone()),
    Dependency::Extended {
        source, file, files, url: url_override, ignore_cves, ..
    } => Dependency::Extended {
        version: new_version.clone(),
        source: source.clone(),
        file: file.clone(),
        files: files.clone(),
        url: url_override.clone(),
        ignore_cves: ignore_cves.clone(),
    },
};

manifest.dependencies.insert(name.clone(), new_dep);
lockfile.dependencies.insert(
    name.clone(),
    crate::lockfile::LockedDependency {
        version: new_version.clone(),
        files: fetched.iter().map(|f| f.locked.clone()).collect(),
    },
);

for f in &fetched {
    vendor::place_file(output_dir, &f.locked.filename, &f.bytes)?;
}
println!("{name}: {old_version} -> {new_version}");
```

Also update the `locked` variable binding. Currently it's:
```rust
let locked = lockfile.dependencies.get(name.as_str()).ok_or_else(...)?;
```
This borrows `lockfile` immutably. Since we later call `lockfile.dependencies.insert`, we need to clone or restructure. Clone the locked data before the loop:
```rust
let locked = lockfile.dependencies.get(name.as_str())
    .ok_or_else(|| anyhow::anyhow!("'{name}' not found in lockfile"))?
    .clone();
```
(`Clone` derive was already added to `LockedDependency` in Task 2.)

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add src/commands/update.rs src/lockfile.rs
git commit -m "feat: update update command for multi-file lockfile"
```

### Task 10: Update `remove` command

**Files:**
- Modify: `src/commands/remove.rs`

- [ ] **Step 1: Update remove to delete all files**

Replace:
```rust
if let Some(locked) = lockfile.dependencies.remove(package) {
    vendor::remove_file(Path::new(&config.output_dir), &locked.filename)?;
}
```
With:
```rust
if let Some(locked) = lockfile.dependencies.remove(package) {
    for file in &locked.files {
        vendor::remove_file(Path::new(&config.output_dir), &file.filename)?;
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src/commands/remove.rs
git commit -m "feat: update remove command for multi-file lockfile"
```

### Task 11: Update `list` command

**Files:**
- Modify: `src/commands/list.rs`

- [ ] **Step 1: Update to tree format**

Replace contents of `src/commands/list.rs`:

```rust
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;

pub fn list() -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;

    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return Ok(());
    }

    for (name, dep) in &manifest.dependencies {
        let version = dep.version();
        println!("{name}@{version}");

        match lockfile.dependencies.get(name) {
            Some(locked) => {
                for file in &locked.files {
                    println!("  {}", file.filename);
                }
            }
            None => {
                println!("  (not installed)");
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src/commands/list.rs
git commit -m "feat: update list command to tree format"
```

---

## Chunk 4: Update `add` Command and CLI

### Task 12: Update CLI for multi-file flag

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Change `--file` to accept multiple values**

In `src/cli.rs`, change the `Add` variant:

```rust
Add {
    /// Package specifier: npm name (e.g. htmx.org) or gh:user/repo
    package: String,
    /// Package version (default: latest)
    #[arg(long)]
    version: Option<String>,
    /// File path(s) within the package (repeatable)
    #[arg(long, action = clap::ArgAction::Append)]
    file: Vec<String>,
},
```

- [ ] **Step 2: Update `src/main.rs` to pass Vec**

Change the `Command::Add` match arm:

```rust
Command::Add {
    package,
    version,
    file,
} => {
    unpm::commands::add(&package, version.as_deref(), &file).await?;
}
```

- [ ] **Step 3: Update `add` function signature**

In `src/commands/add.rs`, change the signature:

```rust
pub async fn add(package: &str, version: Option<&str>, files: &[String]) -> Result<()> {
```

And update the non-interactive guard:
```rust
if !interactive && (version.is_none() || files.is_empty()) {
    bail!("Non-interactive mode requires both --version and --file flags");
}
```

For now, use only the first file to keep existing single-file behavior working (multi-file add logic comes in next task):
```rust
let file = files.first().map(|s| s.as_str());
```

Then use `file` where `file` was previously used in the function.

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/main.rs src/commands/add.rs
git commit -m "feat: change --file to accept multiple values"
```

### Task 13: Update `add` command for multi-file support

**Files:**
- Modify: `src/commands/add.rs`

This is the most complex task. The `add` function needs to:
1. Handle multiple `--file` flags
2. Support interactive multi-select
3. Merge with existing deps
4. Write `files` form in manifest
5. Write multiple entries in lockfile

- [ ] **Step 1: Rewrite the add function**

Replace the contents of `src/commands/add.rs`. Key changes:

1. **Multi-file from flags**: When multiple `--file` flags are provided, validate all exist in the package, skip minification prompt.

2. **Interactive multi-select**: Replace `Select` with `dialoguer::MultiSelect` for the file picker when user chooses manual selection. (`dialoguer` 0.11 includes `MultiSelect`.)

3. **Merge logic**: Before the registry lookup, check if the package already exists in the manifest/lockfile. If so, resolve existing files from lockfile, check version compatibility, and append new files.

4. **Manifest form**: If total files > 1, use `files` array. If 1 file == default, use short form. If 1 file != default, use `file` form.

5. **Lockfile**: Build `Vec<LockedFile>` from all files (existing + new), insert as `LockedDependency`.

Here's the full replacement for `add.rs`:

```rust
use std::io::IsTerminal;
use std::path::Path;

use anyhow::{Result, bail};
use dialoguer::{Confirm, MultiSelect, Select};

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, LockedFile, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::{PackageSource, Registry, latest_stable};
use crate::vendor;

pub async fn add(package: &str, version: Option<&str>, files_flag: &[String]) -> Result<()> {
    let (package, version) = match version {
        Some(_) => (package, version),
        None => match package.rsplit_once('@') {
            Some((pkg, ver)) if !pkg.is_empty() && !ver.is_empty() => (pkg, Some(ver)),
            _ => (package, None),
        },
    };

    let interactive = std::io::stdin().is_terminal();

    if !interactive && (version.is_none() || files_flag.is_empty()) {
        bail!("Non-interactive mode requires both --version and --file flags");
    }

    let source = PackageSource::parse(package)?;
    let manifest_key = source.display_name();
    let client = reqwest::Client::new();
    let registry = Registry::with_client(client.clone());
    let fetcher = Fetcher::with_client(client);

    // Check for existing dep (merge mode)
    let mut manifest = Manifest::load()?;
    let mut lockfile = Lockfile::load()?;
    let existing = manifest.dependencies.get(&manifest_key);

    if let Some(existing_dep) = existing {
        if let Some(v) = version {
            if v != existing_dep.version() {
                bail!(
                    "{manifest_key} already exists at version {}. \
                     Cannot add files at version {v}.",
                    existing_dep.version()
                );
            }
        }
    }

    // Step 1: Look up package
    println!("Looking up {source}...");
    let pkg_info = registry.get_package(&source).await?;

    // Step 2: Select version
    let selected_version = if let Some(existing_dep) = existing {
        existing_dep.version().to_string()
    } else {
        select_version(&pkg_info, version, interactive)?
    };

    // Step 3: Get file listing
    println!("Fetching file list for {source}@{selected_version}...");
    let pkg_files = registry
        .get_package_files(&source, &selected_version)
        .await?;

    // Resolve existing files from lockfile
    let existing_file_paths: Vec<String> = lockfile
        .dependencies
        .get(&manifest_key)
        .map(|l| {
            l.files
                .iter()
                .filter_map(|f| {
                    crate::url::extract_file_path(&f.url, &selected_version).ok()
                })
                .collect()
        })
        .unwrap_or_default();

    // Step 4: Select file(s)
    let selected_files = select_files(
        &pkg_files,
        files_flag,
        interactive,
        &existing_file_paths,
    )?;

    // Filter out files that already exist
    let new_files: Vec<String> = selected_files
        .into_iter()
        .filter(|f| !existing_file_paths.contains(f))
        .collect();

    if new_files.is_empty() && existing.is_some() {
        println!("All specified files are already vendored for {manifest_key}.");
        return Ok(());
    }

    // Step 5: Fetch all new files
    let config = Config::load()?;
    let output_dir = Path::new(&config.output_dir);
    let mut fetched_files: Vec<(String, LockedFile, Vec<u8>)> = Vec::new();

    for file_path in &new_files {
        let url = Registry::file_url(&source, &selected_version, file_path);
        println!("Fetching {url}...");
        let result = fetcher.fetch(&url).await?;

        let original_filename = Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let vendored_filename = resolve_filename(
            &original_filename,
            file_path,
            &manifest_key,
            &lockfile,
            &fetched_files,
        );

        fetched_files.push((
            file_path.clone(),
            LockedFile {
                url,
                sha256: result.sha256.clone(),
                size: result.size,
                filename: vendored_filename,
            },
            result.bytes.to_vec(),
        ));
    }

    // Step 6: Confirm (if interactive and version wasn't pre-specified)
    if interactive && version.is_none() && existing.is_none() {
        println!();
        println!("  Package:  {source}");
        println!("  Version:  {selected_version}");
        for (path, locked_file, _) in &fetched_files {
            println!("  File:     {path} -> {}", locked_file.filename);
            println!("    Size:   {} bytes", locked_file.size);
            println!("    SHA:    {}", locked_file.sha256);
        }
        println!();

        let confirm = Confirm::new()
            .with_prompt("Add this dependency?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Step 7: Determine manifest form
    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d));

    let all_file_paths: Vec<&str> = existing_file_paths
        .iter()
        .map(|s| s.as_str())
        .chain(new_files.iter().map(|s| s.as_str()))
        .collect();

    let existing_source = existing.and_then(|d| d.source().map(|s| s.to_string()));
    let existing_cves = existing
        .map(|d| d.ignore_cves().to_vec())
        .unwrap_or_default();

    let dep = if all_file_paths.len() == 1 && default_path == Some(all_file_paths[0]) {
        Dependency::Short(selected_version.clone())
    } else if all_file_paths.len() == 1 {
        Dependency::Extended {
            version: selected_version.clone(),
            source: existing_source.clone(),
            file: Some(all_file_paths[0].to_string()),
            files: None,
            url: None,
            ignore_cves: existing_cves.clone(),
        }
    } else {
        Dependency::Extended {
            version: selected_version.clone(),
            source: existing_source.clone(),
            file: None,
            files: Some(all_file_paths.iter().map(|s| s.to_string()).collect()),
            url: None,
            ignore_cves: existing_cves.clone(),
        }
    };

    manifest.dependencies.insert(manifest_key.clone(), dep);
    manifest.save()?;

    // Step 8: Update lockfile
    let mut all_locked_files: Vec<LockedFile> = lockfile
        .dependencies
        .get(&manifest_key)
        .map(|l| l.files.clone())
        .unwrap_or_default();
    for (_, locked_file, _) in &fetched_files {
        all_locked_files.push(locked_file.clone());
    }

    lockfile.dependencies.insert(
        manifest_key.clone(),
        LockedDependency {
            version: selected_version.clone(),
            files: all_locked_files,
        },
    );
    lockfile.save()?;

    // Step 9: Place files
    for (_, locked_file, bytes) in &fetched_files {
        vendor::place_file(output_dir, &locked_file.filename, bytes)?;
    }

    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

    if new_files.len() == 1 {
        println!(
            "Added {source}@{selected_version} -> {}/{}",
            config.output_dir, fetched_files[0].1.filename
        );
    } else {
        println!("Added {source}@{selected_version}:");
        for (_, locked_file, _) in &fetched_files {
            println!("  {}/{}", config.output_dir, locked_file.filename);
        }
    }

    Ok(())
}

/// Resolve vendored filename, avoiding collisions with existing lockfile entries,
/// other files being added in the same batch, and intra-package collisions (same
/// basename from different directories within one package).
fn resolve_filename(
    original: &str,
    file_path: &str,
    manifest_key: &str,
    lockfile: &Lockfile,
    batch: &[(String, LockedFile, Vec<u8>)],
) -> String {
    let existing_filenames: Vec<&str> = lockfile
        .dependencies
        .values()
        .flat_map(|l| l.files.iter().map(|f| f.filename.as_str()))
        .chain(batch.iter().map(|(_, f, _)| f.filename.as_str()))
        .collect();

    if !existing_filenames.contains(&original) {
        return original.to_string();
    }

    // Check if collision is intra-package (same batch has same basename)
    let batch_has_same_basename = batch.iter().any(|(_, f, _)| {
        Path::new(&f.filename).file_name().map(|n| n.to_string_lossy().to_string())
            == Some(original.to_string())
    });

    if batch_has_same_basename {
        // Intra-package collision: prefix with parent directory segments
        let parts: Vec<&str> = file_path.split('/').collect();
        // Use immediate parent first, then progressively more segments
        for depth in 1..parts.len() {
            let prefix = parts[parts.len() - 1 - depth..parts.len() - 1].join("_");
            let candidate = format!("{prefix}_{original}");
            if !existing_filenames.contains(&candidate.as_str()) {
                return candidate;
            }
        }
    }

    // Cross-package collision: namespace with package name
    format!(
        "{}_{}",
        manifest_key.replace(['/', ':'], "-"),
        original
    )
}

fn select_files(
    pkg_files: &crate::registry::PackageFiles,
    files_flag: &[String],
    interactive: bool,
    existing_files: &[String],
) -> Result<Vec<String>> {
    if !files_flag.is_empty() {
        // Validate all specified files exist
        for f in files_flag {
            let path = f.strip_prefix('/').unwrap_or(f);
            if !pkg_files.files.iter().any(|fe| fe.path == path) {
                bail!("File {f} not found in package");
            }
        }
        return Ok(files_flag
            .iter()
            .map(|f| f.strip_prefix('/').unwrap_or(f).to_string())
            .collect());
    }

    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d).to_string())
        .map(|d| resolve_default_path(&d, &pkg_files.files));

    if !interactive {
        return default_path
            .map(|d| vec![d])
            .ok_or_else(|| anyhow::anyhow!("No default entry point; use --file"));
    }

    // If there are existing files, skip the default prompt and go straight to multi-select
    if !existing_files.is_empty() {
        return interactive_multi_select(pkg_files, existing_files);
    }

    if let Some(ref default) = default_path {
        let items = &[
            format!("Use default entry point ({default})"),
            "Select file(s) manually".to_string(),
        ];
        let selection = Select::new()
            .with_prompt("File selection")
            .items(&items[..])
            .default(0)
            .interact()?;

        if selection == 0 {
            // Check for min counterpart
            let file_paths: Vec<&str> = pkg_files.files.iter().map(|f| f.path.as_str()).collect();
            let final_file = if let Some((min_file, full_file)) = find_min_counterpart(default, &file_paths) {
                let items = &[
                    format!("{min_file} (minified)"),
                    format!("{full_file} (unminified)"),
                ];
                let default_idx = if *default == min_file { 0 } else { 1 };
                let selection = Select::new()
                    .with_prompt("Both minified and unminified versions exist")
                    .items(items)
                    .default(default_idx)
                    .interact()?;
                if selection == 0 { min_file } else { full_file }
            } else {
                default.clone()
            };
            return Ok(vec![final_file]);
        }
    }

    interactive_multi_select(pkg_files, existing_files)
}

fn interactive_multi_select(
    pkg_files: &crate::registry::PackageFiles,
    existing_files: &[String],
) -> Result<Vec<String>> {
    let file_labels: Vec<String> = pkg_files
        .files
        .iter()
        .map(|f| {
            let marker = if existing_files.contains(&f.path) {
                " (already added)"
            } else {
                ""
            };
            format!("{} ({} bytes){marker}", f.path, f.size)
        })
        .collect();

    let defaults: Vec<bool> = pkg_files
        .files
        .iter()
        .map(|f| existing_files.contains(&f.path))
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("Select file(s) (space to toggle, enter to confirm)")
        .items(&file_labels)
        .defaults(&defaults)
        .interact()?;

    if selections.is_empty() {
        bail!("No files selected");
    }

    Ok(selections
        .into_iter()
        .map(|i| pkg_files.files[i].path.clone())
        .collect())
}

fn sorted_versions(versions: &[crate::registry::VersionInfo]) -> Vec<&str> {
    let mut parsed: Vec<(&str, Option<semver::Version>)> = versions
        .iter()
        .map(|v| (v.version.as_str(), semver::Version::parse(&v.version).ok()))
        .collect();

    parsed.sort_by(|a, b| match (&b.1, &a.1) {
        (Some(bv), Some(av)) => bv.cmp(av),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.0.cmp(b.0),
    });

    parsed.into_iter().map(|(s, _)| s).collect()
}

fn select_version(
    pkg_info: &crate::registry::PackageInfo,
    version_flag: Option<&str>,
    interactive: bool,
) -> Result<String> {
    if let Some(v) = version_flag {
        if !pkg_info.versions.iter().any(|vi| vi.version == v) {
            bail!("Version {v} not found for {}", pkg_info.name);
        }
        return Ok(v.to_string());
    }

    let stable = latest_stable(&pkg_info.versions);
    let default_version = pkg_info
        .tags
        .latest
        .as_deref()
        .or(stable.as_deref())
        .or_else(|| sorted_versions(&pkg_info.versions).first().copied())
        .ok_or_else(|| anyhow::anyhow!("No versions found for {}", pkg_info.name))?;

    if !interactive {
        return Ok(default_version.to_string());
    }

    let label = if pkg_info.tags.latest.is_some() {
        format!("Use latest version ({default_version})?")
    } else {
        format!("Use latest stable version ({default_version})?")
    };

    let use_default = Confirm::new().with_prompt(label).default(true).interact()?;

    if use_default {
        return Ok(default_version.to_string());
    }

    let versions = sorted_versions(&pkg_info.versions);

    let selection = Select::new()
        .with_prompt("Select version")
        .items(&versions)
        .default(0)
        .interact()?;

    Ok(versions[selection].to_string())
}

fn resolve_default_path(default: &str, files: &[crate::registry::FileEntry]) -> String {
    if files.iter().any(|f| f.path == default) {
        return default.to_string();
    }

    for ext in &[".js", ".css"] {
        let min_ext = format!(".min{ext}");
        if let Some(stem) = default.strip_suffix(&min_ext) {
            let unminified = format!("{stem}{ext}");
            if files.iter().any(|f| f.path == unminified) {
                log::debug!(
                    "default path '{default}' not in file listing, using '{unminified}' instead"
                );
                return unminified;
            }
        }
    }

    default.to_string()
}

fn find_min_counterpart(selected: &str, all_files: &[&str]) -> Option<(String, String)> {
    for ext in &[".js", ".css"] {
        let min_ext = format!(".min{ext}");

        if selected.ends_with(&min_ext) {
            let unminified = format!("{}{ext}", &selected[..selected.len() - min_ext.len()]);
            if all_files.contains(&unminified.as_str()) {
                return Some((selected.to_string(), unminified));
            }
        } else if let Some(stripped) = selected.strip_suffix(ext) {
            let minified = format!("{}{min_ext}", stripped);
            if all_files.contains(&minified.as_str()) {
                return Some((minified, selected.to_string()));
            }
        }
    }
    None
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src/commands/add.rs
git commit -m "feat: multi-file add with merge support and multi-select"
```

---

## Chunk 5: Final Verification

### Task 14: Full build and test suite

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings.

- [ ] **Step 3: Run formatter**

Run: `cargo fmt --check`
Expected: No changes needed.

- [ ] **Step 4: Fix any issues found in steps 1-3**

If any test failures, lint warnings, or format issues: fix and re-run.

- [ ] **Step 5: Manual smoke test**

Run a quick manual test to verify the happy path:
```bash
# In a temp directory
mkdir /tmp/unpm-test && cd /tmp/unpm-test
cargo run --manifest-path /Users/james/Development/unpm/Cargo.toml -- add htmx.org --version 2.0.4 --file dist/htmx.min.js
cat unpm.toml
cat unpm.lock
cargo run --manifest-path /Users/james/Development/unpm/Cargo.toml -- list
```

- [ ] **Step 6: Commit any remaining fixes**

```bash
git add -A
git commit -m "chore: fix lint and test issues"
```
