# Tracey-Compatible Specs Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reverse-engineer tracey-compatible behavioral spec files for all user-facing commands and data formats, reorganize docs directory, and create ADRs for existing features.

**Architecture:** Pure documentation task. Reorganize `docs/` into `adrs/`, `design/`, `impl/`, `specs/`. Write 2 ADRs from existing design docs. Write 10 spec files by reading source code and extracting observable behavioral rules into `r[requirement.id]` format.

**Tech Stack:** Markdown, tracey requirement marker syntax (`r[id]`)

**Spec:** `docs/design/2026-03-14-tracey-specs-design.md`

---

## Chunk 1: Directory Reorganization

### Task 1: Reorganize docs directory

**Files:**
- Move: `docs/plans/2026-03-08-unpm-design.md` → `docs/design/2026-03-08-unpm-design.md`
- Move: `docs/specs/2026-03-12-multi-file-deps-design.md` → `docs/design/2026-03-12-multi-file-deps-design.md`
- Move: `docs/plans/2026-03-08-unpm-implementation.md` → `docs/impl/2026-03-08-unpm-implementation.md`
- Move: `docs/plans/2026-03-12-multi-file-deps.md` → `docs/impl/2026-03-12-multi-file-deps.md`
- Move: `docs/design/2026-03-14-tracey-specs-adr.md` → `docs/adrs/2026-03-14-tracey-specs.md`
- Move: `docs/plans/2026-03-14-tracey-specs.md` → `docs/impl/2026-03-14-tracey-specs.md` (this plan file)
- Create: `docs/adrs/` directory

- [ ] **Step 1: Create new directories**

```bash
mkdir -p docs/adrs docs/impl
```

- [ ] **Step 2: Move design docs**

```bash
git mv docs/plans/2026-03-08-unpm-design.md docs/design/2026-03-08-unpm-design.md
git mv docs/specs/2026-03-12-multi-file-deps-design.md docs/design/2026-03-12-multi-file-deps-design.md
```

- [ ] **Step 3: Move implementation docs**

```bash
git mv docs/plans/2026-03-08-unpm-implementation.md docs/impl/2026-03-08-unpm-implementation.md
git mv docs/plans/2026-03-12-multi-file-deps.md docs/impl/2026-03-12-multi-file-deps.md
```

- [ ] **Step 4: Move ADR and plan file**

```bash
git mv docs/design/2026-03-14-tracey-specs-adr.md docs/adrs/2026-03-14-tracey-specs.md
git mv docs/plans/2026-03-14-tracey-specs.md docs/impl/2026-03-14-tracey-specs.md
```

- [ ] **Step 5: Remove empty plans and specs directories**

```bash
rmdir docs/plans docs/specs
```

- [ ] **Step 6: Verify final structure**

```bash
find docs -type f | sort
```

Expected output:
```
docs/adrs/2026-03-14-tracey-specs.md
docs/design/2026-03-08-unpm-design.md
docs/design/2026-03-12-multi-file-deps-design.md
docs/design/2026-03-14-tracey-specs-design.md
docs/impl/2026-03-08-unpm-implementation.md
docs/impl/2026-03-12-multi-file-deps.md
docs/impl/2026-03-14-tracey-specs.md
```

- [ ] **Step 7: Commit**

```bash
git add docs/
git commit -m "chore: reorganize docs into adrs/, design/, impl/"
```

---

## Chunk 2: Reverse-Engineered ADRs

### Task 2: Write architecture ADR

Reverse-engineer from `docs/design/2026-03-08-unpm-design.md` and current codebase state.

**Files:**
- Create: `docs/adrs/2026-03-08-unpm-architecture.md`
- Reference: `docs/design/2026-03-08-unpm-design.md`

- [ ] **Step 1: Write ADR**

Cover these decisions:
- Vendoring static assets directly into the repo (vs node_modules, CDN links, or build tools)
- jsdelivr as the CDN source for both npm and GitHub packages
- SHA-256 locking for integrity verification
- No transitive dependencies — only explicit, flat dependency list
- Exact version pinning (no ranges)
- OSV.dev for CVE scanning
- TOML manifest + JSON lockfile format choice

- [ ] **Step 2: Commit**

```bash
git add docs/adrs/2026-03-08-unpm-architecture.md
git commit -m "docs: add architecture ADR reverse-engineered from design doc"
```

### Task 3: Write multi-file deps ADR

Reverse-engineer from `docs/design/2026-03-12-multi-file-deps-design.md` and `docs/impl/2026-03-12-multi-file-deps.md`.

**Files:**
- Create: `docs/adrs/2026-03-12-multi-file-deps.md`
- Reference: `docs/design/2026-03-12-multi-file-deps-design.md`, `docs/impl/2026-03-12-multi-file-deps.md`

- [ ] **Step 1: Write ADR**

Cover these decisions:
- Support multiple files per dependency (vs one-dep-per-file)
- `files` array in lockfile (always, even for single-file deps)
- Auto-migration from old flat lockfile format
- `file` and `files` mutually exclusive in manifest
- Merge-on-re-add behavior (preserve existing files, append new)
- Filename collision resolution strategy (intra-package: directory prefix; cross-package: package name prefix)

- [ ] **Step 2: Commit**

```bash
git add docs/adrs/2026-03-12-multi-file-deps.md
git commit -m "docs: add multi-file deps ADR reverse-engineered from design doc"
```

---

## Chunk 3: Command Specs — check, add, update

### Task 4: Write check.md spec

The check command has the richest behavioral surface. Read `src/commands/check.rs` and `src/fetch.rs` for the implementation.

**Files:**
- Create: `docs/specs/check.md`
- Reference: `src/commands/check.rs`, `src/cve.rs`, `src/fetch.rs`, `src/registry.rs`, `src/url.rs`

- [ ] **Step 1: Write spec**

Aspects to cover:

**Integrity:**
- Each manifest dep MUST have a lockfile entry (line 70-76 of check.rs)
- Each locked file MUST exist on disk (line 82-99)
- SHA-256 of vendored file MUST match lockfile hash (line 85-89)
- Local SHA-256 MUST be cross-verified against CDN hash via base64-to-hex conversion (lines 157-183, 237-255)
- Base64 decode failure, missing CDN file, and network errors all report as integrity errors (lines 242-255)

**CVE Scanning:**
- Each dep MUST be checked against OSV.dev using npm package name or manifest name for non-npm (lines 114-125)
- Vulnerabilities in `ignore_cves` list MUST be excluded (lines 216-219)
- `--allow-vulnerable` flag MUST suppress CVE failures (line 221)
- CVE query errors MUST be reported as vulnerability findings (lines 228-230)

**Freshness:**
- Each dep MUST be compared against latest stable registry version (lines 184-197)
- Outdated deps are always printed when found (informational by default) (lines 289-297)
- Outdated deps MUST only cause failure when `--fail-on-outdated` is set (lines 293-296)
- CDN hash verification is per-file, not per-package (lines 102-111)

**General:**
- All async checks (CVE, CDN hash, outdated) MUST run concurrently, max 5 in-flight (line 201)
- Empty manifest MUST print "No dependencies to check." and exit success (lines 58-61)
- MUST exit error if any integrity errors or unignored vulnerabilities exist (lines 299-301)
- MUST print "All checks passed." on success (line 303)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/check.md
git commit -m "docs: add check command spec"
```

### Task 5: Write add.md spec

The most complex command. Read `src/commands/add.rs` for the implementation.

**Files:**
- Create: `docs/specs/add.md`
- Reference: `src/commands/add.rs`, `src/vendor.rs`, `src/registry.rs`

- [ ] **Step 1: Write spec**

Aspects to cover:

**Package Resolution:**
- Package name MAY include version via `@` syntax (line 15-21 of add.rs)
- `--version` flag overrides `@` version (line 15-21)
- Package source inferred from name prefix (`gh:` for GitHub); no `--source` CLI flag exists (registry.rs lines 65-72)

**Non-Interactive Mode:**
- MUST require both `--version` and `--file` flags (lines 14-27)
- All specified files MUST exist in the package file listing (lines 280-298)
- File paths MUST be normalized (leading `/` stripped) (line 297)
- Explicitly specified version MUST exist in the package's version list (lines 414-418)

**Interactive Mode:**
- Version selection: list versions descending by semver, show stable/latest tags (lines 409-455)
- Default file: offer package's default entry point if available (lines 457-476)
- Minification preference: if default is minified, offer unminified counterpart and vice versa (lines 478-495)
- Multi-select: when adding to existing dep or no default, show file picker with existing files marked "(already added)" (lines 354-391)
- Confirmation: show version, filename(s), and ask to confirm; skip if version was pre-specified or adding to existing dep (lines 131-151)

**Merge Behavior:**
- Adding files to an existing dep MUST preserve the existing version; error if `--version` specifies a different version (lines 36-49)
- Existing files from lockfile MUST be preserved; new files appended (lines 195-212)
- If all specified files are already vendored, MUST print "All specified files are already vendored" and exit success (lines 89-92)

**Filename Collisions:**
- Intra-package collision (same filename from different paths): prefix with parent directory segments joined by `_` (lines 256-274)
- Cross-package collision (same filename from different package): prefix with package name (lines 256-274)
- Check both existing lockfile and current batch for collisions (lines 110-116)

**Manifest Form:**
- Single file matching package default → Short form (version only) (lines 153-190)
- Single file not matching default → Extended with `file` field (lines 153-190)
- Multiple files → Extended with `files` array (lines 153-190)

**Vendoring:**
- All files MUST be fetched before any are written (lines 94-128)
- Files placed via vendor with path traversal protection (lines 214-231)
- Canonical cleanup runs after placement if enabled (line 229)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/add.md
git commit -m "docs: add add command spec"
```

### Task 6: Write update.md spec

Read `src/commands/update.rs` for the implementation.

**Files:**
- Create: `docs/specs/update.md`
- Reference: `src/commands/update.rs`, `src/registry.rs`

- [ ] **Step 1: Write spec**

Aspects to cover:

**Preconditions:**
- Package in manifest but missing from lockfile MUST error with "not found in lockfile" (lines 49-53)

**Target Selection:**
- Package name MAY include target version via `@` syntax (lines 11-27)
- `--version` without package name MUST error (lines 11-27)
- No package specified → update all manifest dependencies (lines 37-45)

**Version Resolution:**
- MUST respect major version boundary by default: find highest stable version with same major (lines 175-190)
- If already at latest within major, report "held back" with latest available version (lines 58-97)
- `--latest` flag MUST allow crossing major version boundaries (lines 58-97)
- Explicit `@version` MUST bypass major version constraint (lines 58-97)
- When updating a single package already at target, MUST print "{name} is already at {version}" (lines 100-102)
- When updating all packages, already-at-target packages are silently skipped (line 100 checks `package.is_some()`)

**File Handling:**
- MUST fetch each file from lockfile at new version by extracting file path from old URL (lines 106-130)
- If ANY file fails to fetch at new version, MUST bail with error naming the package, file path, and version (lines 115-120)
- Atomic semantics: all files for a package fetched before any are written (lines 106-130)

**Manifest Preservation:**
- Short form dependencies MUST remain Short form (lines 132-149)
- Extended form dependencies MUST preserve all fields except version (lines 132-149)

**Vendoring:**
- Updated files placed in vendor directory (lines 151-162)
- Canonical cleanup runs after placement (line 169)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/update.md
git commit -m "docs: add update command spec"
```

---

## Chunk 4: Command Specs — install, remove, list, outdated

### Task 7: Write install.md spec

**Files:**
- Create: `docs/specs/install.md`
- Reference: `src/commands/install.rs`

- [ ] **Step 1: Write spec**

Aspects to cover:

**Preconditions:**
- Empty manifest MUST print "No dependencies to install." and exit (lines 9-19)
- Manifest dep without lockfile entry MUST error suggesting `unpm add` (lines 36-38)

**Fetching:**
- Progress bar MUST count total files across all deps, not dep count (lines 22-34)
- Custom URL override (`dep.url()`) MUST apply only when locked entry has exactly one file (lines 40-61)
- Otherwise, use the URL stored in each locked file entry (lines 40-61)

**Integrity:**
- SHA-256 of fetched content MUST match lockfile hash (lines 50-57)
- Mismatch MUST fail immediately with expected vs actual hash (lines 50-57)

**Vendoring:**
- Files placed in output directory (lines 59-66)
- Canonical cleanup runs after placement (lines 59-66)
- MUST print "Installed N dependencies to DIR" on success (lines 68-72)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/install.md
git commit -m "docs: add install command spec"
```

### Task 8: Write remove.md spec

**Files:**
- Create: `docs/specs/remove.md`
- Reference: `src/commands/remove.rs`

- [ ] **Step 1: Write spec**

- Package MUST exist in manifest; error if not found (lines 10-17)
- MUST tolerate missing lockfile entry (skip file deletion if no lockfile entry) (line 19)
- All locked files for the package MUST be deleted from disk (lines 19-23)
- Package MUST be removed from both manifest and lockfile (lines 25-28)
- Canonical cleanup MUST run after removal (line 27)
- Confirmation message MUST be printed (line 28)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/remove.md
git commit -m "docs: add remove command spec"
```

### Task 9: Write list.md spec

**Files:**
- Create: `docs/specs/list.md`
- Reference: `src/commands/list.rs`

- [ ] **Step 1: Write spec**

- Empty manifest MUST print "No dependencies." and exit (lines 8-11)
- MUST list all manifest dependencies with their versions (lines 4-27)
- Each dependency MUST show its locked filenames (lines 4-27)
- Dependencies without a lockfile entry MUST show "(not installed)" (lines 4-27)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/list.md
git commit -m "docs: add list command spec"
```

### Task 10: Write outdated.md spec

**Files:**
- Create: `docs/specs/outdated.md`
- Reference: `src/commands/outdated.rs`

- [ ] **Step 1: Write spec**

- Empty manifest MUST print "No dependencies." and exit (lines 8-11)
- MUST query latest versions for all dependencies concurrently, max 5 in-flight (line ~20 of outdated.rs)
- MUST print "Outdated dependencies:" header only if any are outdated (lines 5-58)
- Each outdated dep shown as `name: current -> latest` (lines 5-58)
- If none outdated, MUST print "All dependencies are up to date." (lines 5-58)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/outdated.md
git commit -m "docs: add outdated command spec"
```

---

## Chunk 5: Data Format Specs

### Task 11: Write manifest.md spec

**Files:**
- Create: `docs/specs/manifest.md`
- Reference: `src/manifest.rs`, `tests/manifest_test.rs`

- [ ] **Step 1: Write spec**

Aspects to cover:

**File:**
- Manifest is `unpm.toml` in project root (line 91 of manifest.rs)
- Missing file treated as empty manifest (lines 94-96)

**Dependency Forms:**
- Short form: `package = "version"` (Dependency::Short enum variant)
- Extended form: inline table with `version` and optional `source`, `file`, `files`, `url`, `ignore_cves` fields

**Validation:**
- `file` and `files` MUST NOT both be present (lines 108-128)
- `url` and `files` MUST NOT both be present (lines 108-128)
- `files` array MUST NOT be empty if present (lines 108-128)

**Package Sources:**
- `source` field specifies package origin
- `gh:user/repo` prefix indicates GitHub source
- No `source` field defaults to npm

**Field Names:**
- `ignore_cves` field uses hyphenated TOML key name `ignore-cves` (serde rename on line 19)

**Serialization:**
- Short form deps MUST serialize as `name = "version"` (lines 130-178)
- Extended deps MUST serialize as inline table with proper TOML escaping (lines 130-178)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/manifest.md
git commit -m "docs: add manifest spec"
```

### Task 12: Write lockfile.md spec

**Files:**
- Create: `docs/specs/lockfile.md`
- Reference: `src/lockfile.rs`, `tests/lockfile_test.rs`

- [ ] **Step 1: Write spec**

Aspects to cover:

**File:**
- Lockfile is `unpm.lock` in project root (line 89 of lockfile.rs)
- JSON format, pretty-printed (lines 100-103)
- Missing file treated as empty lockfile (lines 91-94)

**Structure:**
- Top-level object maps package names to locked dependency objects
- Each locked dependency has `version` string and `files` array
- Each file entry has `url`, `sha256`, `size`, and `filename` fields

**Migration:**
- Old flat format (url/sha256/size/filename as direct fields) MUST be auto-migrated to `files` array on read (lines 58-80)
- Presence of BOTH old flat fields and `files` array MUST error as corruption (lines 41-50)
- Entries with NEITHER old flat fields nor `files` array MUST error (lines 81-85)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/lockfile.md
git commit -m "docs: add lockfile spec"
```

### Task 13: Write config.md spec

**Files:**
- Create: `docs/specs/config.md`
- Reference: `src/config.rs`, `tests/config_test.rs`

- [ ] **Step 1: Write spec**

- Config file is `.unpm.toml` in project root (line 29 of config.rs)
- Missing file MUST use defaults (lines 31-33)
- `output_dir`: directory for vendored files, default `"static/vendor"` (lines 4-9)
- `canonical`: boolean, when true untracked files in output_dir are removed after operations, default `true` (lines 4-9)

- [ ] **Step 2: Commit**

```bash
git add docs/specs/config.md
git commit -m "docs: add config spec"
```
