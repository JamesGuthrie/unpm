# unpm Design

Lightweight vendoring of static JS/CSS/SVG assets into a repository. No node_modules, no runtime fetching, no CDN dependency at runtime.

## File Layout

```
project/
├── .unpm.toml          # tool config (output dir, CDN preference)
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
htmx = "2.0.4"
d3 = { version = "7.9.0", file = "dist/d3.min.js" }
some-lib = { version = "1.0.0", url = "https://example.com/lib.min.js" }
```

Short form (`htmx = "2.0.4"`) uses convention to resolve via the default CDN. Extended form allows specifying `file` path or full `url` override.

### unpm.lock (generated)

```json
{
  "htmx": {
    "version": "2.0.4",
    "url": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.4/dist/htmx.min.js",
    "sha256": "abc123...",
    "size": 49012
  }
}
```

## Commands

- `unpm add <package>` — interactive flow to add a dependency
- `unpm install` — fetch all deps per manifest + lockfile
- `unpm check` — verify vendored files match lockfile SHAs, check for CVEs, report outdated deps
- `unpm remove <package>` — remove a vendored dependency

## `unpm add` Interactive Flow

1. Resolve package name via jsdelivr API (handle name mapping, e.g. `htmx` → `htmx.org`)
2. Select version (default: latest stable, option to pick another)
3. Select file:
   - "Use default entry point" (from package.json main/browser/module)
   - "Select file manually" (browsable list)
4. Minification preference — if both `foo.js` and `foo.min.js` exist, ask which
5. Confirm — show summary, write to `unpm.toml` + `unpm.lock`

Non-interactive mode via flags: `unpm add htmx --version 2.0.4 --file dist/htmx.min.js`

## Security & Verification

### SHA verification

- On `unpm install`, every fetched file is hashed (SHA-256) and compared against `unpm.lock`
- SHA mismatch → immediate failure with clear error
- `unpm add` records SHA at add-time
- `unpm check` re-downloads and verifies vendored files match lockfile

### CVE checking

- `unpm check` queries OSV.dev API by package name + version
- Known vulnerability → exit non-zero, print CVE ID, severity, description
- Per-dep override: `ignore-cves = ["CVE-2024-1234"]`
- Global override: `--allow-vulnerable` flag

### Freshness checking

- `unpm check` queries jsdelivr for latest version of each dep
- Reports outdated deps (informational, doesn't fail by default)

### Supply-chain posture

- Lockfile committed to repo — diffs reviewable in PRs
- Vendored files committed — no runtime fetching
- No post-install scripts, no transitive dependencies — just static files

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
├── manifest.rs          # unpm.toml parsing
├── lockfile.rs          # unpm.lock read/write
├── registry.rs          # jsdelivr API client
├── fetch.rs             # HTTP fetching + SHA verification
├── cve.rs               # OSV.dev API client
├── vendor.rs            # file placement into output dir
└── interactive.rs       # terminal UI for `unpm add`
```

### Key dependencies

- `clap` — CLI parsing
- `reqwest` (with rustls) — HTTP client, no OpenSSL for easier static linking
- `serde` / `serde_json` / `toml` — serialization
- `sha2` — SHA-256 hashing
- `dialoguer` — interactive prompts
- `indicatif` — progress bars

### Distribution

- Static binaries via `cross` or `cargo-zigbuild` for linux/mac/windows
- GitHub releases with prebuilt binaries
- GitHub Action pulls binary from releases
