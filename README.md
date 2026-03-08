# unpm

Vendor static JS, CSS, and SVG assets directly into your repository. No `node_modules`, no runtime CDN dependency, no build step.

unpm fetches versioned files from npm and GitHub packages via [jsdelivr](https://www.jsdelivr.com/), locks them with SHA-256 hashes, and checks for known vulnerabilities.

## Install

Download a prebuilt binary from [releases](https://github.com/unpm/unpm/releases), or build from source:

```
cargo install --path .
```

## Quick Start

```sh
# Add a package (interactive)
unpm add htmx.org

# Add with a specific version
unpm add htmx.org@2.0.7

# Add a GitHub-hosted package
unpm add gh:answerdotai/fasthtml-js

# Fetch all vendored files
unpm install

# Verify integrity and scan for CVEs
unpm check
```

Files are vendored into `static/vendor/` by default. Configure the output directory in `.unpm.toml`:

```toml
output_dir = "assets/vendor"
```

## Manifest

Dependencies are declared in `unpm.toml`:

```toml
[dependencies]
htmx.org = "2.0.7"
idiomorph = { version = "0.7.2", file = "dist/idiomorph.js" }
"gh:answerdotai/fasthtml-js" = { version = "1.0.12", file = "fasthtml.js" }
```

Short form uses the package's default entry point. Extended form allows specifying a `file` path within the package, a custom `url`, or a list of CVEs to ignore:

```toml
[dependencies]
lodash = { version = "4.17.21", file = "lodash.min.js", ignore-cves = ["GHSA-x5rq-j2xg-h7qm"] }
```

## Commands

| Command | Description |
|---------|-------------|
| `unpm add <package[@version]>` | Add a dependency (interactive) |
| `unpm install` | Fetch all dependencies |
| `unpm check` | Verify SHA integrity, cross-check against CDN, scan for CVEs |
| `unpm list` | List all dependencies |
| `unpm outdated` | Show dependencies with newer versions available |
| `unpm update <package[@version]>` | Update a dependency |
| `unpm remove <package>` | Remove a dependency |

### `unpm add`

Interactive flow: select version, pick a file from the package, choose between minified/unminified variants, confirm.

Non-interactive mode for CI:

```sh
unpm add htmx.org --version 2.0.7 --file dist/htmx.min.js
```

### `unpm check`

Runs three verifications per dependency:

1. **Lockfile SHA** -- vendored file matches the hash recorded at add time
2. **CDN SHA** -- vendored file matches the hash jsdelivr currently reports (independent second source)
3. **CVE scan** -- no known vulnerabilities via [OSV.dev](https://osv.dev/)

Exits non-zero if any check fails. Only failures are printed.

```sh
# Allow known vulnerabilities (not recommended)
unpm check --allow-vulnerable
```

### `unpm update`

Updates an existing dependency, preserving file path and other settings:

```sh
unpm update htmx.org          # update to latest
unpm update htmx.org@2.0.6    # update to specific version
```

## Package Sources

unpm supports two package sources, both resolved via jsdelivr:

- **npm** -- `unpm add htmx.org`
- **GitHub** -- `unpm add gh:user/repo`

Package names must be exact. No fuzzy matching or alias resolution -- this prevents typosquatting attacks.

## Security

- **SHA-256 integrity** -- every vendored file is hashed and locked. Tampering is detected on `check` and `install`.
- **CDN cross-checking** -- `check` verifies vendored files against jsdelivr's independently computed hashes, not just the hash from first download.
- **CVE scanning** -- queries the OSV.dev vulnerability database for each package/version.
- **No post-install scripts** -- unpm vendors static files only. No code execution, no transitive dependencies.
- **Exact package names** -- no fuzzy resolution. `unpm add htmx` fails; `unpm add htmx.org` succeeds.
- **Reviewable diffs** -- both `unpm.toml` and `unpm.lock` are committed to the repo, making dependency changes visible in PRs.

## GitHub Action

Add to your CI workflow to verify vendored dependencies on every push:

```yaml
- uses: unpm/action@v1
```

The action downloads the `unpm` binary and runs `unpm check`. It exits non-zero on SHA mismatches or known vulnerabilities.

### Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `allow-vulnerable` | Allow known vulnerabilities | `false` |
| `version` | unpm version to use | `latest` |

### Example workflow

```yaml
name: Verify vendored deps
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: unpm/action@v1
```
