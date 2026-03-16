# `list` command

The `list` command displays all declared dependencies from the manifest
along with their vendored files.

## Preconditions

r[list.empty]
When the manifest contains no dependencies, the command MUST print
"No dependencies." and exit successfully.

## Output

r[list.output.entry]
Each manifest dependency MUST be printed as a line in the format
`{name}@{version}`.

r[list.output.files]
For each dependency that has a corresponding lockfile entry, the
command MUST print each locked filename on its own line, indented
with two leading spaces.

r[list.output.not-installed]
For each dependency that has no corresponding lockfile entry, the
command MUST print `  (not installed)` on a single indented line.
