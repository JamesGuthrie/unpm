# Configuration

Configuration controls tool-wide settings such as the output directory
and cleanup behavior. Settings are loaded from a dotfile in the project
root, with sensible defaults when the file is absent.

## File

r[config.file.name]
The configuration file MUST be named `.unpm.toml` and located in the
project root directory.

r[config.file.missing]
If `.unpm.toml` does not exist, the tool MUST proceed using default
values for all configuration fields.

r[config.format.toml]
The configuration file MUST be valid TOML. If the file exists but
contains invalid TOML, the tool MUST exit with an error.

r[config.format.empty]
An empty configuration file MUST be treated as valid and all fields
MUST fall back to their defaults.

## Output Directory

r[config.output-dir.default]
When `output_dir` is not specified, it MUST default to `static/vendor`.

r[config.output-dir.custom]
When `output_dir` is specified, the tool MUST use the provided value
as the directory for vendored files.

## Canonical Mode

r[config.canonical.default]
When `canonical` is not specified, it MUST default to `true`.

r[config.canonical.cleanup]
When `canonical` is `true`, untracked files in the output directory
MUST be removed after operations that modify vendored files.

r[config.canonical.disabled]
When `canonical` is `false`, the tool MUST NOT remove any files from
the output directory that it did not explicitly place or update.
