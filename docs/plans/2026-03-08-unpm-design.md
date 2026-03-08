# unpm Design

Lightweight vendoring of static JS/CSS/SVG assets into a repository. No node_modules, no runtime fetching, no CDN dependency at runtime.

## File Layout

```
project/
├── .unpm.toml          # tool config (output dir)
├── unpm.toml           # dependency manifest (user-authored)
├── unpm.lock           # resolved URLs, SHAs, metadata (tool-generated)
└── static/vendor/      # vendored files (configurable path)
```

### .unpm.toml (tool config)

```toml
output_dir = "static/vendor"
```

### unpm.toml (dependency manifest)

```toml
[dependencies]
htmx.org = "2.0.7"
idiomorph = { version = "0.7.2", file = "dist/idiomorph.js" }
"gh:answerdotai/fasthtml-js" = { version = "1.0.12", file = "fasthtml.js" }
some-lib = { version = "1.0.0", url = "https://example.com/lib.min.js" }
```

Package names must be exact. npm packages use their registry name (e.g., `htmx.org` not `htmx`). GitHub-hosted packages use the `gh:user/repo` prefix. No fuzzy resolution — this prevents typosquatting/namesquatting attacks.

Short form (`htmx.org = "2.0.7"`) resolves the default entry point via the CDN. Extended form allows specifying `file` path, full `url` override, or `ignore-cves` list.

### unpm.lock (generated)

```json
{
  "htmx.org": {
    "version": "2.0.7",
    "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.7/dist/htmx.min.js",
    "sha256": "60231ae6...",
    "size": 51076,
    "filename": "htmx.min.js"
  }
}
```

The `filename` field tracks the vendored filename. Plain filenames are used by default; namespaced filenames (e.g., `package_file.js`) are used only when a collision would occur.

## Commands

- `unpm add <package[@version]>` — interactive flow to add a dependency
- `unpm install` — fetch all deps per manifest + lockfile
- `unpm check` — verify vendored files (lockfile SHA, CDN SHA, CVEs)
- `unpm list` — list all dependencies
- `unpm outdated` — show dependencies with newer versions available
- `unpm update <package[@version]>` — update a dependency to latest or a specific version
- `unpm remove <package>` — remove a vendored dependency

## Package Sources

All packages are resolved via jsdelivr. Two source types are supported:

- **npm** — `unpm add htmx.org` (API: `data.jsdelivr.com/v1/packages/npm/...`)
- **GitHub** — `unpm add gh:user/repo` (API: `data.jsdelivr.com/v1/packages/gh/...`)

The source type is inferred from the manifest key name (`gh:` prefix = GitHub, otherwise npm).

## `unpm add` Interactive Flow

1. Look up exact package name on jsdelivr — fail if not found (no fuzzy matching)
2. Select version (default: latest stable via semver, option to pick another)
3. Select file:
   - "Use default entry point" (from package.json main/browser/module)
   - "Select file manually" (browsable list)
4. Minification preference — if both `foo.js` and `foo.min.js` exist, ask which
5. Confirm — show summary, write to `unpm.toml` + `unpm.lock`, vendor file

Version can be specified inline: `unpm add htmx.org@2.0.7`

Non-interactive mode via flags: `unpm add htmx.org --version 2.0.7 --file dist/htmx.min.js`

## `unpm update`

Updates an existing dependency to a new version, preserving all other settings (file path, ignore-cves, etc.).

- `unpm update htmx.org` — update to latest stable version
- `unpm update htmx.org@2.0.6` — update to a specific version
- `unpm update htmx.org --version 2.0.6` — same, with flag syntax

The file path is extracted from the existing lockfile URL, so the same file within the package is fetched at the new version. Manifest, lockfile, and vendored file are all updated in place.

## Security & Verification

### SHA verification

- On `unpm add`, the fetched file is hashed (SHA-256) and recorded in `unpm.lock`
- On `unpm install`, every fetched file is hashed and compared against `unpm.lock`; mismatch → immediate failure
- On `unpm check`, vendored files are verified against both the lockfile SHA and the jsdelivr API hash (independent second source)

### CDN hash cross-checking

- `unpm check` fetches the per-file SHA-256 hash from jsdelivr's package API
- This hash is computed server-side by jsdelivr, independent of what was downloaded at `add` time
- Catches: lockfile tampering, CDN compromise at add-time, or silent CDN content changes
- Limitation: still relies on jsdelivr as the authority — not the package author

### CVE checking

- `unpm check` queries OSV.dev API by package name + version
- Known vulnerability → exit non-zero, print advisory ID and summary
- Per-dep override: `ignore-cves = ["GHSA-xxxx-xxxx-xxxx"]`
- Global override: `--allow-vulnerable` flag

### Freshness checking

- `unpm outdated` queries jsdelivr for the latest version of each dependency
- Uses semver parsing to find the highest stable (non-prerelease) version
- Reports outdated deps (informational only)

### Typosquatting prevention

- Package names must be exact — no alias resolution or fuzzy matching
- `unpm add htmx` fails; `unpm add htmx.org` succeeds
- Future: `unpm search` command for discovery (not in initial release)

### Supply-chain posture

- Lockfile committed to repo — diffs reviewable in PRs
- Vendored files committed — no runtime fetching
- No post-install scripts, no transitive dependencies — just static files
- CDN hash cross-checking provides independent verification beyond trust-on-first-download

## GitHub Action

```yaml
- uses: unpm/action@v1
```

The action downloads the `unpm` binary from GitHub releases, runs `unpm check`, and exits non-zero on SHA mismatches or CVEs. All logic lives in the CLI; the action is a thin wrapper.

## Project Structure

```
src/
├── main.rs              # CLI entry point (clap)
├── cli.rs               # command definitions & argument parsing
├── config.rs            # .unpm.toml parsing
├── manifest.rs          # unpm.toml parsing (custom inline-table serialization)
├── lockfile.rs          # unpm.lock read/write (JSON)
├── registry.rs          # jsdelivr API client (npm + GitHub sources)
├── fetch.rs             # HTTP fetching + SHA-256 hashing
├── cve.rs               # OSV.dev API client
├── vendor.rs            # file placement into output dir
└── commands/
    ├── add.rs           # interactive add flow
    ├── install.rs       # fetch all dependencies
    ├── check.rs         # SHA + CDN hash + CVE verification
    ├── list.rs          # list dependencies
    ├── outdated.rs      # check for newer versions
    ├── update.rs        # update a dependency version
    └── remove.rs        # remove a dependency
```

### Key dependencies

- `clap` — CLI parsing
- `reqwest` (with rustls) — HTTP client, no OpenSSL for easier static linking
- `serde` / `serde_json` / `toml` — serialization
- `sha2` + `hex` + `base64` — SHA-256 hashing and encoding
- `semver` — version parsing and comparison
- `dialoguer` — interactive prompts
- `indicatif` — progress bars
- `futures` — parallel async operations
- `tokio` — async runtime
- `anyhow` — error handling

### Distribution

- Static binaries via cross-compilation for linux/mac (x86_64 + aarch64)
- GitHub releases with prebuilt binaries
- GitHub Action pulls architecture-appropriate binary from releases
