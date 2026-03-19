# Install Script Design

## Overview

A POSIX shell script (`install.sh`) that downloads the correct `unpm` binary from the latest GitHub release and installs it to `~/.local/bin`. The script is served from the repository (via `raw.githubusercontent.com`), not bundled with releases — this ensures bug fixes to the installer apply immediately without re-releasing.

## Usage

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/JamesGuthrie/unpm/main/install.sh | sh
```

## Platform Detection

The script uses `uname -s` and `uname -m` to determine the OS and architecture, then maps to the release artifact name:

| `uname -s` | `uname -m` | Artifact              |
|-------------|------------|-----------------------|
| Linux       | x86_64     | `unpm-linux-x86_64`  |
| Linux       | aarch64    | `unpm-linux-aarch64` |
| Darwin      | x86_64     | `unpm-darwin-x86_64` |
| Darwin      | arm64      | `unpm-darwin-aarch64` |

Note: macOS reports `arm64` for Apple Silicon, but the artifact uses `aarch64`. The script handles this mapping.

Unsupported OS/arch combinations produce a clear error listing the supported platforms.

## Version Resolution

The script always installs the latest release. It queries the GitHub API endpoint `/repos/JamesGuthrie/unpm/releases/latest` to determine the current release tag. The extracted field is `tag_name`.

No `jq` dependency — the tag is extracted from the JSON response using `sed`/`grep`.

## Download

The binary is downloaded from:

```
https://github.com/JamesGuthrie/unpm/releases/download/{tag}/{artifact}
```

Using `curl` with `--proto '=https' --tlsv1.2 -fSL` flags for security and redirect-following. Trust is based on HTTPS to GitHub's CDN; no additional checksum or attestation verification is performed by the script.

## Installation

1. Create `~/.local/bin` if it doesn't exist.
2. Download to a temporary file first, then `mv` into `~/.local/bin/unpm`. This avoids leaving a partial binary if the download is interrupted.
3. `chmod +x ~/.local/bin/unpm`.
4. If `~/.local/bin/unpm` already exists, it is silently overwritten (this is an installer, not a package manager).
5. Check if `~/.local/bin` is on `$PATH`:
   - If yes: print success message.
   - If no: print success message plus instructions for adding `~/.local/bin` to `$PATH` for common shells:
     - **bash**: `echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc`
     - **zsh**: `echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc`
     - **fish**: `fish_add_path ~/.local/bin`

## Dependencies

The script requires only standard POSIX utilities plus `curl`:

- `curl` (required, for downloading)
- `uname` (for platform detection)
- `chmod`, `mkdir` (for installation)
- `grep`, `sed` (for parsing the GitHub API response)

## Error Handling

The script uses `set -eu` to fail on any uncaught error or undefined variable.

- **Unsupported platform**: Clear message listing supported OS/arch combinations.
- **Network failure / HTTP error**: `curl -f` returns nonzero on HTTP errors (e.g. 404); the script exits with an error message.
- **Cannot write to install directory**: Error message if `~/.local/bin` creation or file write fails.
- **`curl` not found**: Error message asking the user to install `curl`.

## Release Workflow Changes

None required. The script is served directly from the repository, not as a release artifact.

## File Location

The script lives at the repository root: `install.sh`.
