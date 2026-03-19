# One-Line Install Script

## Status

Accepted

## Context

Users need a way to install unpm without cloning the repository or manually downloading release artifacts. A shell-based installer served from the repository (not bundled with releases) allows bug fixes to the installer to take effect immediately without re-releasing.

## Decision

Provide a POSIX shell script (`install.sh`) at the repository root that downloads the correct platform binary from the latest GitHub release and installs it to `~/.local/bin`. The script is fetched via `raw.githubusercontent.com` and uses the GitHub API to resolve the latest release tag — no hard-coded versions.

Platform detection uses `uname -s` / `uname -m` to map to release artifact names. The only non-POSIX dependency is `curl`.

## Consequences

**Positive:**
- Single `curl | sh` command to install on any supported platform.
- Installer fixes ship instantly — no release cycle required.
- No dependency on package managers, Homebrew taps, or language-specific toolchains.

**Negative:**
- `curl | sh` is a trust-on-first-use pattern — users must trust the repository contents at fetch time.
- No checksum or signature verification of the downloaded binary beyond HTTPS transport security.
- Depends on GitHub API availability for version resolution and GitHub Releases CDN for binary download.
