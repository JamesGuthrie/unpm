# `outdated` command

The `outdated` command compares each dependency in the manifest against its
registry source and reports which packages have a newer version available.

## Preconditions

r[outdated.empty]
When the manifest contains no dependencies, the command MUST print
"No dependencies." and exit successfully.

## Resolution

r[outdated.resolution.source]
Each manifest dependency MUST be resolved to a package source. Dependencies
whose source cannot be resolved MUST be silently skipped.

r[outdated.resolution.latest]
The latest version MUST be determined by the registry's "latest" tag,
falling back to the highest stable (non-prerelease) semver version.

## Comparison

r[outdated.comparison]
A dependency MUST be considered outdated only when the registry returns
a latest version that differs from the current manifest version. A
dependency whose latest version cannot be determined MUST NOT be
reported as outdated.

## Concurrency

r[outdated.concurrency]
Registry lookups for all dependencies MUST run concurrently.

## Output

r[outdated.output.header]
The command MUST print "Outdated dependencies:" before the first
outdated entry. The header MUST NOT be printed when no dependencies
are outdated.

r[outdated.output.entry]
Each outdated dependency MUST be printed as `  name: current -> latest`
where `current` is the version from the manifest and `latest` is the
version from the registry.

r[outdated.output.up-to-date]
When no dependencies are outdated, the command MUST print
"All dependencies are up to date."
