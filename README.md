# unpm

Vendor static JS, CSS, and SVG assets directly into your repository. No `node_modules`, no runtime CDN dependency, no build step.

unpm fetches versioned files from npm and GitHub packages via [jsdelivr](https://www.jsdelivr.com/), locks them with SHA-256 hashes, and checks for known vulnerabilities.

## Install

Download a prebuilt binary from [releases](https://github.com/JamesGuthrie/unpm/releases), or build from source:

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

Files are vendored into `static/vendor/` by default. Configure in `.unpm.toml`:

```toml
output_dir = "assets/vendor"
canonical = true  # remove untracked files from output dir (default)
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
| `unpm check` | Verify integrity, CVEs, and freshness |
| `unpm list` | List all dependencies |
| `unpm outdated` | Show dependencies with newer versions available |
| `unpm update [package[@version]]` | Update one or all dependencies (same major) |
| `unpm remove <package>` | Remove a dependency |

### `unpm add`

Interactive flow: select version, pick a file from the package, choose between minified/unminified variants, confirm.

Non-interactive mode for CI:

```sh
unpm add htmx.org --version 2.0.7 --file dist/htmx.min.js
```

### `unpm check`

Runs four checks per dependency:

1. **Lockfile SHA** -- vendored file matches the hash recorded at add time
2. **CDN SHA** -- vendored file matches the hash jsdelivr currently reports (independent second source)
3. **CVE scan** -- no known vulnerabilities via [OSV.dev](https://osv.dev/)
4. **Freshness** -- whether newer versions are available

Output is grouped by category (Integrity, Vulnerabilities, Outdated). Only problems are printed.

```sh
unpm check --allow-vulnerable    # ignore CVEs
unpm check --fail-on-outdated    # treat outdated deps as errors
```

### `unpm update`

Updates dependencies within the same major version by default. If a newer major version exists but can't be installed, unpm tells you:

```
htmx.org: 1.9.12 held back (2.0.7 available, use --latest to update across major versions)
```

```sh
unpm update                   # update all (same major version)
unpm update --latest          # update all to latest, crossing major versions
unpm update htmx.org          # update one dependency (same major)
unpm update htmx.org@3.0.0   # pin to an explicit version
```

## Versioning

The version in `unpm.toml` is the exact version that gets installed — there are no version ranges or specifiers. When you run `unpm update`, it upgrades to the latest version within the same major version (e.g. `1.9.12` → `1.9.14`, but not `1.9.12` → `2.0.0`). This keeps updates safe by default, since major version bumps typically indicate breaking changes.

To update across major versions, use `unpm update --latest` or specify the version explicitly with `unpm update package@version`.

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
- uses: JamesGuthrie/unpm@v1
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
      - uses: JamesGuthrie/unpm@v1
```
