# `check` command

The `check` command verifies the integrity of vendored files, checks
dependencies for known CVEs, and reports outdated dependencies.

## Preconditions

r[check.empty]
When the manifest contains no dependencies, the command MUST print
"No dependencies to check." and exit successfully.

## Integrity

r[check.integrity.lockfile-presence]
Each manifest dependency MUST have a corresponding lockfile entry.
A missing lockfile entry MUST be reported as an integrity error.

r[check.integrity.file-exists]
Each file listed in a dependency's lockfile entry MUST exist on disk
at the configured output directory. A missing file MUST be reported
as an integrity error.

r[check.integrity.sha-match]
The SHA-256 hash of each vendored file MUST match the hash stored in
the lockfile. A mismatch MUST be reported as an integrity error.

r[check.integrity.cdn-verify]
The SHA-256 hash of each vendored file MUST be cross-verified against
the hash reported by the CDN. A mismatch MUST be reported as an
integrity error. Verification is performed per file, not per package.

r[check.integrity.cdn-decode-failure]
If the CDN-reported hash cannot be parsed, the command MUST report an
integrity error.

r[check.integrity.cdn-missing-file]
If a vendored file cannot be found in the CDN's file listing, the
command MUST report an integrity error.

r[check.integrity.cdn-network-error]
If the CDN file listing cannot be fetched (network error or HTTP
failure), the command MUST report an integrity error.

## CVE Scanning

r[check.cve.query]
Each dependency MUST be checked for known vulnerabilities by querying
OSV.dev. For npm-sourced packages, the query MUST use the `npm`
ecosystem with the npm package name. For GitHub-sourced packages, the
query MUST use the `GIT` ecosystem with the full repository URL
(e.g., `https://github.com/user/repo.git`) as the package name.

r[check.cve.git-rev]
When a GitHub-sourced dependency's version is a commit SHA rather than
a tag, the CVE query MUST use the OSV.dev commit query instead of an
ecosystem/version query.

r[check.cve.ignore]
Vulnerabilities whose ID appears in the dependency's `ignore_cves`
list MUST be excluded from reported findings.

r[check.cve.allow-vulnerable]
When the `--allow-vulnerable` flag is set, discovered vulnerabilities
(after filtering by `ignore_cves`) MUST NOT be reported and MUST NOT
cause failure.

r[check.cve.query-error]
If a CVE query fails (network error or API failure), the error MUST
be reported as a vulnerability finding regardless of the
`--allow-vulnerable` flag.

## Freshness

r[check.freshness.compare]
Each dependency MUST be compared against the latest stable version
from the registry. The latest stable version is determined by the
registry's "latest" tag, falling back to the highest stable (non-prerelease)
semver version.

r[check.freshness.print]
Outdated dependencies MUST always be printed when found.

r[check.freshness.fail-on-outdated]
Outdated dependencies MUST only cause a non-zero exit when the
`--fail-on-outdated` flag is set.

## Concurrency

r[check.concurrency]
CVE queries, CDN hash verification, and freshness lookups MUST run
concurrently.

## Exit Behavior

r[check.exit.failure]
The command MUST exit with an error if any integrity errors exist or
any unignored vulnerability findings are present.

r[check.exit.success]
When all checks pass, the command MUST print "All checks passed." and
exit successfully.
