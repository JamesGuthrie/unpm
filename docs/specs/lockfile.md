# Lockfile

The lockfile records the exact resolved version, download URLs, and integrity hashes for every vendored file, ensuring that installs are reproducible across machines and over time.

## File

r[lockfile.file.name]
The lockfile MUST be named `unpm.lock` and located in the project root.

r[lockfile.file.format]
The lockfile MUST be JSON, serialized with pretty-printing.

r[lockfile.file.missing]
A missing lockfile MUST be treated as an empty lockfile with no dependencies.

## Structure

r[lockfile.structure.top-level]
The top-level JSON object MUST map package names directly to locked dependency objects. Package names MUST be serialized in sorted order.

r[lockfile.structure.dependency]
Each locked dependency object MUST contain a `version` string and a `files` array.

r[lockfile.structure.file-entry]
Each entry in the `files` array MUST contain `url` (string), `sha256` (string), `size` (unsigned integer), and `filename` (string) fields.

r[lockfile.structure.multi-file]
A locked dependency MAY contain multiple entries in its `files` array.

## Migration

r[lockfile.migration.old-format]
An entry using the old flat format (with `url`, `sha256`, `size`, and `filename` as direct fields alongside `version`) MUST be auto-migrated into a single-element `files` array on read. All four flat fields MUST be present; if any are missing the read MUST error.

r[lockfile.migration.conflict]
An entry containing both old flat fields and a `files` array MUST be rejected as a corrupt lockfile.

r[lockfile.migration.no-file-data]
An entry containing neither old flat fields nor a `files` array MUST be rejected as invalid.

## Serialization

r[lockfile.serialization.roundtrip]
A lockfile written by `save` MUST be readable by `load` without data loss.

r[lockfile.serialization.canonical]
The lockfile MUST always be written in the current format (with `files` array), regardless of whether it was read from the old flat format.
