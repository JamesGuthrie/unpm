# `update` command

The `update` command updates vendored dependencies to newer versions,
respecting major version boundaries by default.

## Input validation

r[update.precondition.in-manifest]
If a package name is specified, it MUST exist in the manifest. If it
does not, the command MUST error with "Package '{name}' not found in
dependencies".

r[update.precondition.in-lockfile]
Each package selected for update MUST have a corresponding lockfile
entry. A missing lockfile entry MUST error with "'{name}' not found
in lockfile".

## Target Selection

r[update.target.at-syntax]
The package argument MAY include a target version using `@` syntax
(e.g., `htmx.org@2.1.0`). The `@` is only parsed when no explicit
`--version` flag is provided.

r[update.target.version-requires-package]
The `--version` flag without a package name MUST error with "--version
requires a package name".

r[update.target.all]
When no package name is specified, the command MUST update all
dependencies listed in the manifest.

## Version Resolution

r[update.version.major-boundary]
By default, the command MUST resolve the highest stable
(non-prerelease) version that shares the same major version as the
current version.

r[update.version.held-back]
When the current version is already the latest within its major
version and a newer stable version exists in a higher major version,
the command MUST print a message indicating the package is held back,
naming the available version and suggesting `--latest`.

r[update.version.latest-flag]
When the `--latest` flag is set, the command MUST resolve the target
from the registry's "latest" tag, falling back to the highest stable (non-prerelease) semver
version. This allows crossing major version boundaries.

r[update.version.explicit]
An explicit version (via `@` syntax or `--version`) MUST be used
as-is, bypassing both the major version constraint and registry
lookup.

r[update.version.github-branch-resolve]
When a GitHub package's manifest version is a branch name, the command
MUST re-resolve the branch to the current commit SHA and update the
lockfile accordingly.

r[update.version.already-current-single]
When updating a single named package that is already at the target
version, the command MUST print "{name} is already at {version}".

r[update.version.already-current-all]
When updating all packages, any package already at its target version
MUST be silently skipped.

## File Handling

r[update.files.path-extraction]
Each file in the lockfile entry MUST be fetched at the target version.

r[update.files.fetch-failure]
If any file fails to fetch at the new version, the command MUST bail
with an error naming the package, the file path, and the target
version.

r[update.files.atomic]
All files for a package MUST be fetched before any are written to
disk. A fetch failure for any file MUST prevent all writes for that
package.

## Vendoring

r[update.vendor.placement]
Updated files MUST be written to the configured output directory
using the filename from the lockfile entry.

r[update.vendor.cleanup]
After all packages are processed, canonical cleanup MUST run against
the output directory if the `canonical` configuration option is enabled.

## Manifest update

r[update.manifest.short-form]
A dependency using the short form (version string only) MUST remain
in short form after update.

r[update.manifest.extended-form]
A dependency using the extended form MUST preserve all fields
(`source`, `file`, `files`, `ignore-cves`) except `version`,
which MUST be set to the new version.

r[update.persist.manifest]
The manifest MUST be saved after all packages are processed,
regardless of whether any packages were updated.

r[update.persist.lockfile]
The lockfile MUST be saved after all packages are processed,
regardless of whether any packages were updated.

## Output

r[update.output.success]
For each successfully updated package, the command MUST print
"{name}: {old_version} -> {new_version}".
