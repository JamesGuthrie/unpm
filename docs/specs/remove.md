# `remove` command

The `remove` command removes a vendored dependency from the project. It
validates that the package exists, deletes its files from disk, updates
the manifest and lockfile, and optionally runs canonical cleanup.

## Input validation

r[remove.manifest.exists]
The named package MUST exist in the manifest's dependencies. If the
package is not found, the command MUST exit with an error stating the
package was not found in dependencies.

## Lockfile lookup

r[remove.lockfile.missing]
If the package has no corresponding lockfile entry, the command MUST
skip file deletion and continue without error.

## File deletion

r[remove.files.delete]
All files listed in the package's lockfile entry MUST be deleted from
disk relative to the configured output directory.

## State update

r[remove.state.manifest]
The package MUST be removed from the manifest and the manifest MUST be
saved to disk.

r[remove.state.lockfile]
The package MUST be removed from the lockfile and the lockfile MUST be
saved to disk.

## Cleanup

r[remove.cleanup.canonical]
After the manifest and lockfile are saved, canonical cleanup MUST run
against the configured output directory if the `canonical` configuration
option is enabled.

## Output

r[remove.output.confirmation]
Upon successful removal, the command MUST print a confirmation message
naming the removed package.
