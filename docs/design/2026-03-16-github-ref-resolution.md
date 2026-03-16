# GitHub Git Ref Resolution

Implements specs: `add.version.github-ref`, `add.version.github-resolve`, `update.version.github-resolve`.

## Problem

GitHub packages currently only support semver tag versions. Users cannot pin to a branch (`main`) or a specific commit SHA. The version string is stored identically in both the manifest and lockfile, with no resolution step. This means lockfile URLs use mutable tag names rather than immutable commit SHAs.

## Design

### Version Resolution Flow

All GitHub package versions — whether semver tags, branch names, or commit SHAs — are resolved to a commit SHA via the GitHub API. The same flow applies to both `add` and `update`:

1. **Resolve via GitHub API** — call `GET https://api.github.com/repos/{user}/{repo}/commits/{ref}` with `Accept: application/vnd.github.sha`. This resolves tags, branch names, and commit SHAs to a full 40-character SHA. If the API returns 404/422, error with "version not found".

**Why always resolve, even for semver tags?** Git tags can be force-pushed, making them mutable. Resolving to a commit SHA ensures the lockfile always points to an immutable snapshot. This also simplifies the resolution logic — there's a single path for all GitHub versions rather than a special case for semver.

### Storage Model

| Scenario | Manifest stores | Lockfile stores | URLs use |
|----------|----------------|-----------------|----------|
| Semver tag | `"2.0.8"` | `"<resolved SHA>"` | `@<resolved SHA>` |
| Commit SHA | `"32b003..."` | `"32b003..."` | `@32b003...` |
| Branch name | `"main"` | `"<resolved SHA>"` | `@<resolved SHA>` |

The manifest always records the user's original input. The lockfile always records the resolved commit SHA. This means lockfile URLs are immutable (pointing to a specific commit) while the manifest preserves user intent.

### GitHub API Details

- **Endpoint**: `GET https://api.github.com/repos/{user}/{repo}/commits/{ref}`
- **Header**: `Accept: application/vnd.github.sha` (plain text response, no JSON parsing)
- **Auth**: Unauthenticated only (no `GITHUB_TOKEN` support for now)
- **Rate limit**: 60 requests/hour. On 403, error with a message indicating the rate limit and suggesting the user wait.

### Add Flow

Version selection in `add` for GitHub packages:

1. If version is explicitly specified (flag or `@` syntax), resolve via GitHub API regardless of whether it appears in the jsdelivr version list.
2. If version is not specified, select from the jsdelivr version list as normal (default/interactive), then resolve the selected version via GitHub API.
3. The resolved SHA is used for file listing via jsdelivr data API and for constructing CDN URLs.
4. Manifest records the original user input; lockfile records the resolved SHA.

### Update Flow

Uses the same resolution logic as add:

1. Read the manifest version for the package.
2. Determine the target version: explicit version (flag/`@`), major-boundary compatible version, `--latest`, or re-resolve the current manifest version.
3. For GitHub packages, resolve the target version to a SHA via GitHub API.
4. Compare the resolved SHA to the lockfile's current version.
5. If different, re-fetch files at the new SHA and update the lockfile.
6. If same, the package is already current.

### Error Handling

- **Invalid ref**: GitHub API returns 422 or 404 → "Version '{version}' not found for package '{name}'"
- **Rate limited**: GitHub API returns 403 → "GitHub API rate limit exceeded (60 requests/hour for unauthenticated requests)"
- **jsdelivr down**: Standard fetch error propagation (existing behavior)

### What This Does NOT Cover

- `GITHUB_TOKEN` / authenticated GitHub API requests
- `check.cve.git-rev` (CVE scanning for git-ref-based packages) — to be implemented separately
