# GitHub Git Ref Resolution Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow GitHub packages to be pinned to branch names or commit SHAs, not just semver tags.

**Architecture:** Add a `resolve_github_ref` method to `Registry` that resolves any git ref (tag, branch, SHA) to a commit SHA via the GitHub API. Thread a `(manifest_version, lockfile_version)` pair through the add and update flows so the user's input is stored in the manifest while resolved SHAs go in the lockfile.

**Tech Stack:** Rust, reqwest, GitHub REST API (`application/vnd.github.sha`)

---

## File Map

| File | Role |
|------|------|
| `src/registry.rs` | New `ResolvedVersion` struct, new `resolve_github_ref` method |
| `src/commands/add.rs` | Refactor `select_version` to return `ResolvedVersion`, thread manifest/lockfile versions through flow |
| `src/commands/update.rs` | Add branch re-resolution path, fix `extract_file_path` to use lockfile version |
| `tests/registry_test.rs` | Tests for `resolve_github_ref` (SHA, branch, invalid ref) |

---

## Task 1: Add `ResolvedVersion` and `resolve_github_ref` to Registry

**Files:**
- Modify: `src/registry.rs`
- Test: `tests/registry_test.rs`

The resolution flow:
1. Call GitHub API to resolve the ref to a commit SHA.
2. Return `(original_input, resolved_sha)` — manifest stores user intent, lockfile stores SHA.
3. If GitHub API returns 404/422 → error.

- [ ] **Step 1: Write failing tests**

In `tests/registry_test.rs`:

```rust
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

#[tokio::test]
async fn test_resolve_github_ref_not_found() {
    let registry = Registry::new();
    let source = PackageSource::parse("gh:jquery/jquery").unwrap();
    let result = registry
        .resolve_github_ref(&source, "this-ref-does-not-exist-xyz")
        .await;
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_resolve_github_ref -- --nocapture`
Expected: compilation error — `resolve_github_ref` doesn't exist yet.

- [ ] **Step 3: Implement `ResolvedVersion` and `resolve_github_ref`**

In `src/registry.rs`, add the return type:

```rust
/// The result of resolving a GitHub ref (branch, SHA, or tag).
pub struct ResolvedVersion {
    /// What the manifest should store (user's original input).
    pub manifest_version: String,
    /// What the lockfile should store (resolved SHA or tag).
    pub lockfile_version: String,
}
```

Add constant:

```rust
const GITHUB_API_BASE: &str = "https://api.github.com";
```

Add method to `impl Registry`:

```rust
// r[impl add.version.github-ref]
// r[impl add.version.github-resolve]
/// Resolve a GitHub version (tag, branch, or SHA) to a commit SHA.
pub async fn resolve_github_ref(
    &self,
    source: &PackageSource,
    version: &str,
) -> Result<ResolvedVersion> {
    let PackageSource::GitHub { user, repo } = source else {
        bail!("resolve_github_ref called on non-GitHub source");
    };

    let gh_url = format!(
        "{GITHUB_API_BASE}/repos/{user}/{repo}/commits/{version}"
    );
    log::debug!("GET {gh_url}");
    let resp = self
        .client
        .get(&gh_url)
        .header("Accept", "application/vnd.github.sha")
        .header("User-Agent", "unpm")
        .send()
        .await?;
    log::debug!("  -> {}", resp.status());

    if resp.status() == reqwest::StatusCode::FORBIDDEN {
        bail!(
            "GitHub API rate limit exceeded (60 requests/hour for unauthenticated requests). \
             Please wait and try again."
        );
    }

    if resp.status() == reqwest::StatusCode::NOT_FOUND
        || resp.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY
    {
        bail!("Version '{version}' not found for {source}");
    }

    let sha = resp.error_for_status()?.text().await?.trim().to_string();

    Ok(ResolvedVersion {
        manifest_version: version.to_string(),
        lockfile_version: sha,
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_resolve_github_ref -- --nocapture`
Expected: all 3 tests pass.

---

## Task 2: Update `add` command to support GitHub refs

**Files:**
- Modify: `src/commands/add.rs`

For GitHub packages, all versions must be resolved to a SHA via the GitHub API. `select_version` determines the version string, then resolves it. The rest of the `add` function threads manifest/lockfile versions separately.

- [ ] **Step 1: Refactor `select_version` to return `ResolvedVersion`**

Add import:

```rust
use crate::registry::ResolvedVersion;
```

Change `select_version` signature to:

```rust
async fn select_version(
    pkg_info: &crate::registry::PackageInfo,
    version_flag: Option<&str>,
    interactive: bool,
    source: &PackageSource,
    registry: &Registry,
) -> Result<ResolvedVersion>
```

Key changes inside the function:

1. When `version_flag` is provided and source is GitHub → call `registry.resolve_github_ref(source, v).await?` (resolves tags, branches, and SHAs to a commit SHA).
2. When `version_flag` is provided, NOT found, and source is NOT GitHub → error as before.
3. When `version_flag` is provided, found, and source is NOT GitHub → use as-is.
4. When no version flag, select version from list. For GitHub packages, resolve the selected version via `resolve_github_ref`. For npm, return as-is.

The version validation block (currently lines 435-441) becomes:

```rust
if let Some(v) = version_flag {
    // r[impl add.version.github-ref]
    // r[impl add.version.github-resolve]
    if matches!(source, PackageSource::GitHub { .. }) {
        return registry.resolve_github_ref(source, v).await;
    }
    if pkg_info.versions.iter().any(|vi| vi.version == v) {
        return Ok(ResolvedVersion {
            manifest_version: v.to_string(),
            lockfile_version: v.to_string(),
        });
    }
    bail!("Version {v} not found for {}", pkg_info.name);
}
```

All other return paths wrap in `ResolvedVersion` with identical fields, e.g.:
```rust
Ok(ResolvedVersion {
    manifest_version: default_version.to_string(),
    lockfile_version: default_version.to_string(),
})
```

- [ ] **Step 2: Update the `add` function to use `ResolvedVersion`**

The call site changes from:

```rust
let selected_version = if let Some(existing_dep) = existing {
    existing_dep.version().to_string()
} else {
    select_version(&pkg_info, version, interactive)?
};
```

To:

```rust
let resolved = if let Some(existing_dep) = existing {
    ResolvedVersion {
        manifest_version: existing_dep.version().to_string(),
        lockfile_version: lockfile
            .dependencies
            .get(&manifest_key)
            .map(|l| l.version.clone())
            .unwrap_or_else(|| existing_dep.version().to_string()),
    }
} else {
    select_version(&pkg_info, version, interactive, &source, &registry).await?
};
```

Then replace all uses of `selected_version` throughout the function:

Use `resolved.lockfile_version` for:
- File listing: `registry.get_package_files(&source, &resolved.lockfile_version)`
- CDN URLs: `Registry::file_url(&source, &resolved.lockfile_version, file_path)`
- Lockfile entry: `LockedDependency { version: resolved.lockfile_version.clone(), ... }`
- `extract_file_path` calls: use `&resolved.lockfile_version`

Use `resolved.manifest_version` for:
- Manifest entry: `Dependency::Short(resolved.manifest_version.clone())` etc.
- Display/print output: `println!("Added {source}@{resolved.manifest_version} ...")`
- Confirmation prompt version display

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: no compilation errors.

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: all existing tests still pass.

---

## Task 3: Update `update` command to support GitHub ref re-resolution

**Files:**
- Modify: `src/commands/update.rs`

The update command determines the target version (via explicit flag, major-boundary logic, or re-resolution), then for GitHub packages resolves it to a SHA via the GitHub API.

- [ ] **Step 1: Refactor update's version resolution for GitHub refs**

Add import:

```rust
use crate::registry::ResolvedVersion;
```

The current logic at line 68 does `semver::Version::parse(&old_version)`. When this fails (non-semver) and the source is GitHub, the update command should re-resolve the manifest version via `registry.resolve_github_ref()`.

The key insight: for update, the **manifest version doesn't change** (it stays `"main"`). Only the lockfile version (SHA) changes.

Replace the version resolution block (lines 63-106) with logic that tracks `(manifest_version, lockfile_version)`:

For explicit version (`Some(v)`):
- If source is GitHub → resolve via `resolve_github_ref`
- Otherwise → use as-is (both fields identical)

For no explicit version (`None`):
- If semver parses → existing major-boundary logic to select target version. For GitHub packages, resolve the selected version via `resolve_github_ref`.
- If semver fails and source is GitHub:
  ```rust
  // r[impl update.version.github-resolve]
  let resolved = registry.resolve_github_ref(&source, &old_version).await?;
  (old_version.clone(), resolved.lockfile_version)
  ```
- Otherwise → existing `--latest` logic. For GitHub packages, resolve the selected version via `resolve_github_ref`.

Then use `lockfile_version` for file URLs, fetching, lockfile entry, and the "already current" check (compare against `locked.version`).

Use `manifest_version` for manifest entry and success message display.

- [ ] **Step 2: Fix `extract_file_path` to use lockfile version**

Currently line 126 does:
```rust
let file_path = crate::url::extract_file_path(&locked_file.url, &old_version)?;
```

But `old_version` comes from the manifest (e.g., `"main"`), while the lockfile URL contains the SHA. Change to:

```rust
let file_path = crate::url::extract_file_path(&locked_file.url, &locked.version)?;
```

This is correct for both semver (manifest == lockfile version) and refs (URL contains the SHA from the lockfile).

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: no compilation errors.

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: all tests pass.

---

## Task 4: Final verification and commit

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 2: Run tracey query to verify coverage**

Run: `tracey query uncovered`
Expected: `add.version.github-ref`, `add.version.github-resolve`, and `update.version.github-resolve` are no longer listed as uncovered. Only `check.cve.git-rev` should remain.

- [ ] **Step 3: Commit all changes**

```bash
git add src/registry.rs src/commands/add.rs src/commands/update.rs tests/registry_test.rs
git commit -m "feat: support git refs (branches and SHAs) for GitHub packages

GitHub packages now accept branch names and commit SHAs as versions
via --version or @ syntax. Resolution tries jsdelivr first, falling
back to the GitHub API for branch-to-SHA resolution.

Branch names are stored in the manifest (user intent) while resolved
SHAs are stored in the lockfile (reproducibility). On update, branches
are re-resolved to their current commit SHA."
```
