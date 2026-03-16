# `add` command

The `add` command vendors a package file (or files) into the project. It handles two main scenarios: adding a new dependency, and adding additional files to an existing dependency. It operates in interactive mode (when stdin is a terminal) or non-interactive mode (when it is not).

## Input parsing

r[add.input.package-name]
The command MUST accept a package name as its first positional argument (e.g., `unpm add htmx.org`). The package name identifies which package to fetch from a registry.

r[add.input.at-syntax]
The package name MAY include a version using `@` syntax (e.g., `htmx.org@2.0.4`). The portion after the last `@` is treated as the version, provided neither the name nor version portion is empty.

r[add.input.version-flag]
The `--version` flag, when provided, MUST take precedence over any version specified via `@` syntax in the package name.

r[add.input.source]
The package source MUST be inferred from the package name prefix. Names beginning with `gh:` MUST be resolved as GitHub packages in `gh:user/repo` format. All other names MUST be resolved as npm packages. There is no `--source` CLI flag.

r[add.input.github-validation]
A GitHub package specifier MUST contain both a non-empty user and a non-empty repo separated by `/`. The command MUST error if this format is not met.

## Package resolution

r[add.resolve.not-found]
When the package cannot be found on the registry, the command MUST error with a message indicating that the package was not found.

## Existing dependency detection

r[add.existing.preserve-version]
When adding files to an existing dependency, the command MUST use the existing dependency's version. The user MUST NOT be prompted for version selection.

r[add.existing.version-conflict]
If `--version` specifies a version different from the existing dependency's version, the command MUST error with a message indicating the version conflict.

r[add.existing.already-vendored]
If all specified files are already vendored for the existing dependency, the command MUST print "All specified files are already vendored for {package}." and exit successfully without modifying any files.

r[add.existing.preserve-files]
Existing files from the lockfile MUST be preserved. New files MUST be appended to the existing file list.

## Version selection

r[add.version.validation]
An explicitly specified version MUST exist in the package's version list. The command MUST error if the version is not found.

r[add.version.github-ref]
For GitHub packages, the `--version` flag and `@` syntax also accept any valid git ref (commit SHA or branch name) that does not appear in the version list.

r[add.version.github-branch-resolve]
When a GitHub package version is a branch name, the lockfile MUST record the resolved commit SHA, not the branch name. The manifest MUST record the original branch name as specified by the user.

r[add.version.default]
When no version is specified, the command MUST offer a default version. The default MUST be the `latest` tag if present, otherwise the highest stable (non-prerelease) semver version, otherwise the first version in descending semver order.

r[add.version.list]
If the user declines the default version, the command MUST present a list of all versions sorted in descending semver order. Versions that do not parse as semver MUST be sorted lexicographically and placed after valid semver versions.

## File selection

r[add.files.default-offer]
If the package has a default entry point and no files are already added, the command MUST offer a choice between using the default entry point and selecting files manually.

r[add.files.default-resolution]
When the package's declared default file does not exist in the file listing, the command MUST attempt to resolve it by checking for an unminified counterpart (e.g., replacing `.min.js` with `.js`). This applies to `.js` and `.css` extensions.

r[add.files.min-counterpart]
When the user chooses the default entry point, the command MUST check whether both a minified and unminified counterpart exist (for `.js` and `.css` files). If both exist, the command MUST prompt the user to choose between them, defaulting to whichever variant the original default was.

r[add.files.multi-select]
When adding files to an existing dependency, or when the user opts for manual selection, the command MUST present a multi-select file picker. Files already added to the dependency MUST be marked with "(already added)" and pre-selected.

r[add.files.no-selection]
If the user selects no files in the multi-select picker, the command MUST error.

## Confirmation

r[add.confirm.prompt]
When the version was not pre-specified and the package is not an existing dependency, the command MUST display the package name, version, file path(s), file sizes, and SHA-256 hashes, then prompt for confirmation before proceeding.

r[add.confirm.skip]
The confirmation prompt MUST be skipped when the version was pre-specified via `--version` or `@` syntax, or when adding files to an existing dependency.

r[add.confirm.abort]
If the user declines confirmation, the command MUST print "Aborted." and exit without modifying any files, the manifest, or the lockfile.

## Non-interactive mode

When stdin is not a terminal, the command operates in non-interactive mode. All selections that would normally be prompted for interactively MUST instead be specified via flags.

r[add.noninteractive.required-flags]
The command MUST require both `--version` and `--file` flags. The command MUST error if either is missing.

r[add.noninteractive.file-validation]
All file paths specified via `--file` MUST exist in the package's file listing. The command MUST error if any specified file is not found.

r[add.noninteractive.path-normalization]
File paths specified via `--file` MUST be normalized by stripping any leading `/` before matching against the package file listing.

## Manifest update

r[add.manifest.short-form]
When the dependency has exactly one file and that file matches the package's default entry point, the manifest entry MUST use the short form (version string only).

r[add.manifest.extended-file]
When the dependency has exactly one file that does not match the package's default entry point, the manifest entry MUST use the extended form with a `file` field.

r[add.manifest.extended-files]
When the dependency has multiple files, the manifest entry MUST use the extended form with a `files` array containing all file paths (both existing and new).

r[add.manifest.preserve-fields]
When updating an existing dependency, the manifest entry MUST preserve the existing `source` and `ignore_cves` fields.

## Filename resolution

r[add.filename.intra-package]
When a file's basename collides with another file in the same batch, the command MUST prefix the filename with parent directory segments joined by `_`, increasing the number of prefix segments until the name is unique.

r[add.filename.cross-package]
When a file's basename collides with an existing filename in the lockfile (from any package), the command MUST prefix the filename with the package's manifest key (with `/` and `:` replaced by `-`).

r[add.filename.batch-awareness]
Filename collision checks MUST consider both existing filenames in the lockfile and filenames of files already resolved in the current add batch.

## Vendoring

r[add.vendor.fetch-before-write]
All files MUST be fetched from the registry before any files are written to disk.

r[add.vendor.path-traversal]
File placement MUST reject path traversal attempts. Only the basename of the filename MUST be used when constructing the destination path.

r[add.vendor.output-directory]
The output directory MUST be created if it does not already exist before files are placed.

r[add.vendor.canonical-cleanup]
After placing files, the command MUST run canonical cleanup if the `canonical` configuration option is enabled. Canonical cleanup removes any files in the output directory that are not tracked in the lockfile.
