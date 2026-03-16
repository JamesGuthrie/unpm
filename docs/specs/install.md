# `install` command

The `install` command reads the manifest and lockfile, fetches each dependency's files from their locked URLs, verifies their integrity against the stored hashes, and places them into the configured output directory. It operates entirely from the lockfile — if a manifest dependency is not yet locked, the command directs the user to `unpm add` rather than resolving on the fly.

## Preconditions

r[install.preconditions.empty-manifest]
When the manifest contains no dependencies, the command MUST print "No dependencies to install." and exit successfully without fetching or writing any files.

r[install.preconditions.missing-lock-entry]
When a manifest dependency has no corresponding entry in the lockfile, the command MUST error with a message indicating the package is in the manifest but not the lockfile, and suggest running `unpm add`.

## Fetching

r[install.fetch.progress-total]
The progress bar MUST count the total number of individual files across all dependencies, not the number of dependencies.

## Integrity

r[install.integrity.sha256]
The SHA-256 hash of the fetched content MUST be verified against the hash stored in the lockfile for each file.

r[install.integrity.mismatch]
When the SHA-256 hash of fetched content does not match the lockfile hash, the command MUST fail immediately with a message containing the expected and actual hashes.

## Vendoring

r[install.vendor.placement]
Each fetched file MUST be placed in the configured output directory using the filename from the lockfile entry.

r[install.vendor.canonical-cleanup]
After all files are placed, the command MUST run canonical cleanup if the configuration enables it.

r[install.vendor.success-message]
On successful completion, the command MUST print "Installed {N} dependencies to {DIR}" where N is the number of manifest dependencies and DIR is the configured output directory.
