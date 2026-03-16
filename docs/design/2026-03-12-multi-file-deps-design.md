# Multi-File Dependencies

Adds support for vendoring multiple files from a single npm/GitHub package.

## Motivation

Some packages distribute multiple file types that are used together (e.g. uPlot ships both JS and CSS). Today unpm supports only one file per dependency, forcing users to work around this by adding the same package under different keys or manually managing extra files.

## Manifest (`unpm.toml`)

Three forms coexist:

```toml
[dependencies]
# Short form (unchanged) — single file, default entry point
htmx.org = "2.0.7"

# Extended, single file (unchanged)
idiomorph = { version = "0.7.2", file = "dist/idiomorph.js" }

# Extended, multiple files (new)
uplot = { version = "1.6.31", files = ["dist/uPlot.min.js", "dist/uPlot.min.css"] }
```

### Rules

- `file` and `files` are mutually exclusive. Error if both are set.
- `files` with a single element is valid (no normalization to `file`).
- `url` is incompatible with `files`. Custom URL override is single-file only.
- Empty `files` array is an error.
- `source` (GitHub source override) works with all forms including `files`.

### Implementation

`Dependency::Extended` gains `files: Option<Vec<String>>`. Mutual-exclusion validated at deserialization time. Existing fields (`source`, `ignore_cves`, `url`, `file`) are unchanged. The manual TOML serializer in `Manifest::save()` needs a new branch to write `files` as an inline array.

## Lockfile (`unpm.lock`)

Migrates to a `files` array, always — even for single-file dependencies:

```json
{
  "htmx.org": {
    "version": "2.0.7",
    "files": [
      { "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.7/dist/htmx.min.js", "sha256": "abc...", "size": 12345, "filename": "htmx.min.js" }
    ]
  },
  "uplot": {
    "version": "1.6.31",
    "files": [
      { "url": "https://cdn.jsdelivr.net/npm/uplot@1.6.31/dist/uPlot.min.js", "sha256": "def...", "size": 45000, "filename": "uPlot.min.js" },
      { "url": "https://cdn.jsdelivr.net/npm/uplot@1.6.31/dist/uPlot.min.css", "sha256": "ghi...", "size": 3200, "filename": "uPlot.min.css" }
    ]
  }
}
```

### Structs

```rust
pub struct LockedDependency {
    pub version: String,
    pub files: Vec<LockedFile>,
}

pub struct LockedFile {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub filename: String,
}
```

### Auto-migration

When reading a lockfile with old flat fields (`url`, `sha256`, `size`, `filename` at the top level) and no `files` array, convert to a single-element `files` array in memory. The next write persists the new format. If both old flat fields and a `files` array are present, error — the lockfile is corrupt.

## CLI Changes

### `unpm add`

- `--file` changes from `Option<String>` to `Vec<String>` (clap `action = Append`), accepting multiple values: `unpm add uplot@1.6.31 --file dist/uPlot.min.js --file dist/uPlot.min.css`
- Non-interactive mode requires `--version` and at least one `--file`.
- Interactive mode: file picker becomes multi-select (space to toggle, enter to confirm).
- Minification preference prompt is skipped in multi-select mode (user is explicitly choosing files).
- Default entry point: when only one file is selected interactively and it matches the default, use the short manifest form. Multiple files always use `files`.

### Merge behavior

When adding files to a package that already exists in the manifest:

- Version is preserved. Error if `--version` is passed and conflicts with the existing version.
- Existing file(s) are resolved from the lockfile entry (works regardless of whether the manifest uses short form, `file`, or `files`).
- New files are appended. Duplicate file paths are silently ignored.
- Manifest entry is rewritten: if total files > 1, use `files` array form.

### `unpm install`

Iterates over each dependency's `files` array. For each file: fetch URL, verify SHA-256, write to output dir. Progress bar counts total files across all deps (not dep count). Custom URL override (`dep.url()`) applies only to single-file deps and overrides the sole file's URL.

### `unpm check`

- Integrity checks (local SHA match, CDN cross-check) run per file within each dependency (one task per file, not per dep). CDN cross-check uses the file's `url` field to derive the original CDN path for lookup (not the vendored `filename`, which may have been renamed due to collisions). The existing `extract_file_path` helper in `update.rs` should be extracted into a shared utility for this.
- CVE and freshness checks run per package (version-level, unchanged).
- `outdated` command is unaffected (version-level only).

### `unpm update`

For each file in the locked entry: extract the file path from the URL, construct a new URL at the new version, fetch, and hash. If any file fails to fetch (e.g. removed in the new version), the entire update for that package fails — no files are changed. The error message identifies which file is missing so the user can adjust their `files` list manually before retrying.

### `unpm remove`

`unpm remove <package>` removes the entire package and all its files from manifest, lockfile, and output dir. The remove loop iterates `locked.files` to delete each vendored file. No per-file removal — edit `unpm.toml` by hand and run `install` to drop individual files.

### `unpm list`

Uniform tree format for all deps:

```
htmx.org@2.0.7
  htmx.min.js
uplot@1.6.31
  uPlot.min.js
  uPlot.min.css
```

### Canonical clean

`clean_if_canonical` in `vendor.rs` collects known filenames by flatmapping over all dependencies' `files` arrays (rather than reading a single `filename` field).

## Edge Cases

### Filename collisions

Same cross-package collision logic as today (namespace with package name prefix). For collisions within a multi-file dep (two source paths with the same basename, e.g. `dist/uPlot.min.js` and `legacy/uPlot.min.js`), prefix with the immediate parent directory: `dist_uPlot.min.js` and `legacy_uPlot.min.js`. If immediate parents also collide, use progressively more path segments (`dist_v1_app.js` vs `dist_v2_app.js`).

Intra-package collision detection runs at `add` time when generating vendored filenames. On `install`, filenames are already resolved in the lockfile.

### Validation summary

| Condition | Result |
|---|---|
| `file` + `files` both set | Error |
| `url` + `files` both set | Error |
| `files` is empty array | Error |
| `--version` conflicts with existing package version on merge | Error |
| Duplicate file path on merge | Silently ignored |
