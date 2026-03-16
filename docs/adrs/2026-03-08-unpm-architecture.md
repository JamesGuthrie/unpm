# unpm Architecture Decisions

## 1. Vendor static assets directly into the repo

### Status
Accepted

### Context
Frontend projects that need a few static JS/CSS/SVG files (htmx, Alpine, etc.) are forced to choose between a full node_modules tree, runtime CDN links, or a build tool — all of which add complexity disproportionate to the task.

### Decision
unpm downloads files and commits them directly into the repository under a configurable vendor directory. No runtime fetching, no node_modules, no build step.

### Consequences
- Vendored files are visible in diffs and reviewable in PRs.
- No runtime dependency on any CDN or package registry — the repo is self-contained.
- Repository size increases with each vendored file, though these are typically small static assets.
- Updating a dependency means replacing a committed file rather than changing a version range.

---

## 2. jsdelivr as the sole CDN source

### Status
Accepted

### Context
We need a reliable way to resolve and fetch files from both npm packages and GitHub repositories. Using registries directly (npmjs.org, GitHub raw) would require handling multiple auth schemes, API formats, and URL patterns.

### Decision
All package resolution and file fetching goes through jsdelivr, which provides a unified API and CDN for both npm and GitHub-hosted packages (`data.jsdelivr.com/v1/packages/npm/...` and `.../gh/...`).

### Consequences
- Single API client covers both npm and GitHub sources.
- jsdelivr is a single point of failure — if it goes down, `add`, `install`, and `update` all fail. However, already-vendored files are unaffected since they live in the repo.
- We inherit jsdelivr's package availability. Packages not on npm or public GitHub repos are unsupported.

---

## 3. SHA-256 locking for integrity verification

### Status
Accepted

### Context
Vendored files are trusted at download time, but content could be tampered with in the lockfile, in the vendor directory, or even at the CDN between operations.

### Decision
Every fetched file is hashed with SHA-256 at download time and recorded in the lockfile. On `install`, hashes are verified against the lockfile. On `check`, hashes are additionally cross-checked against jsdelivr's independently computed server-side hash.

### Consequences
- Detects lockfile tampering, local file modification, and CDN content changes between `add` and `check`.
- The CDN cross-check still trusts jsdelivr as the authority — it does not verify against the original package author's signature.
- Hash mismatches cause immediate failure, which is the right default for a security-focused tool.

---

## 4. No transitive dependencies

### Status
Accepted

### Context
Traditional package managers pull in transitive dependency trees that are opaque, difficult to audit, and a frequent vector for supply-chain attacks. unpm's scope is limited to static frontend assets, which are typically self-contained single files.

### Decision
unpm resolves only the packages explicitly listed in `unpm.toml`. There is no dependency resolution, no transitive fetching, and no post-install scripts.

### Consequences
- The dependency list is flat and fully visible. What you declare is exactly what you get.
- If a package genuinely depends on another file, the user must add it manually.
- Eliminates an entire class of supply-chain attacks (transitive dependency hijacking, post-install script exploits).

---

## 5. Exact version pinning

### Status
Accepted

### Context
Semver ranges (^, ~) introduce ambiguity — the resolved version can change depending on when you install. For vendored static files committed to a repo, this ambiguity serves no purpose.

### Decision
Versions in `unpm.toml` are exact (e.g., `"2.0.7"`, not `"^2.0.7"`). Updates are explicit via `unpm update`, which defaults to staying within the same major version unless a target is specified.

### Consequences
- Builds are perfectly reproducible from the manifest alone.
- Users must run `unpm update` to get new versions — there is no automatic drift.
- The major-version-bounded default update behavior balances safety with convenience.

---

## 6. OSV.dev for CVE scanning

### Status
Accepted

### Context
Vendored files don't benefit from `npm audit` or GitHub Dependabot, which rely on package-lock.json or similar lockfiles. We need an independent vulnerability data source.

### Decision
`unpm check` queries the OSV.dev API by package name and version. Known vulnerabilities cause a non-zero exit. Per-dependency overrides (`ignore-cves`) and a global `--allow-vulnerable` flag are provided for acknowledged risks.

### Consequences
- CVE coverage depends on OSV.dev's database, which aggregates from multiple sources (GitHub Advisories, NVD, etc.) and has good coverage for npm packages.
- Can be integrated into CI via the GitHub Action to block PRs with known vulnerabilities.
- The override mechanism prevents false positives from blocking workflows permanently.

---

## 7. TOML manifest + JSON lockfile

### Status
Accepted

### Context
The manifest is user-authored and needs to be readable and easy to edit by hand. The lockfile is tool-generated and needs to be easy to parse and diff.

### Decision
The dependency manifest (`unpm.toml`) uses TOML for human authoring. The lockfile (`unpm.lock`) uses JSON for tool generation and consumption.

### Consequences
- TOML's inline table syntax gives a clean shorthand for simple deps (`htmx.org = "2.0.7"`) while supporting structured entries when needed.
- JSON lockfiles produce clean diffs in PRs — one key-value pair per line, deterministic ordering.
- Two serialization formats means two parsers, but both are well-supported in Rust via `toml` and `serde_json`.
