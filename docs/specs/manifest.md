# Manifest

The manifest declares all vendored dependencies for a project.

## File

r[manifest.file]
The manifest file MUST be named `unpm.toml` and located in the project root.

r[manifest.file.missing]
A missing manifest file MUST be treated as an empty manifest with no dependencies.

## Dependency Forms

r[manifest.dep.short]
A dependency MAY be declared in short form as `package = "version"`, where the value is a version string.

r[manifest.dep.extended]
A dependency MAY be declared in extended form as an inline table with a required `version` field and optional `source`, `file`, `files`, and `ignore-cves` fields.

## Validation

r[manifest.validation.file-files]
The `file` and `files` fields MUST NOT both be present on the same dependency.

r[manifest.validation.files-empty]
The `files` array MUST NOT be empty if present.

## Package Sources

r[manifest.source.default]
A dependency with no `source` field and no `gh:` key prefix MUST default to the npm registry.

r[manifest.source.github-prefix]
A dependency key prefixed with `gh:user/repo` MUST be treated as a GitHub source.

r[manifest.source.field]
An explicit `source` field MUST take precedence over the key name when determining package origin.

## Field Names

r[manifest.field.ignore-cves]
The `ignore_cves` field MUST be serialized and deserialized using the hyphenated TOML key name `ignore-cves`.

## Serialization

r[manifest.serial.short]
Short form dependencies MUST serialize as `name = "version"`.

r[manifest.serial.extended]
Extended form dependencies MUST serialize as a TOML inline table.

r[manifest.serial.key-quoting]
Package names containing characters other than ASCII alphanumerics, hyphens, and underscores MUST be quoted in the serialized output.

r[manifest.serial.escaping]
String values MUST escape backslashes and double quotes in the serialized output.

r[manifest.serial.order]
Dependencies MUST be serialized in lexicographic order by package name.

r[manifest.serial.omit-empty]
Optional fields that are absent or empty MUST be omitted from the serialized inline table.
