# Multi-File Dependencies

## Status

Accepted

## Context

Some npm packages distribute multiple files that are used together (e.g. uPlot ships both JS and CSS). unpm's original model supported only one file per dependency, forcing users to add the same package under different keys or manually manage extra files. This was a recurring friction point for packages with paired assets.

## Decision

**Multiple files per dependency.** A dependency in the manifest can specify a `files` array of paths to vendor from the package, replacing the single-file-only model. The existing short form and single `file` field remain valid for backward compatibility.

**`file` and `files` are mutually exclusive.** Setting both on the same dependency is a deserialization error. This eliminates ambiguity about which field takes precedence and keeps the manifest schema simple.

**Lockfile always uses a `files` array.** Even single-file dependencies are stored as a one-element `files` array. A uniform format means every consumer of lockfile data has exactly one code path, rather than branching on whether the entry is single-file or multi-file.

**Auto-migration from old lockfile format.** When the lockfile reader encounters old flat fields (`url`, `sha256`, `size`, `filename` at the top level) without a `files` array, it wraps them into a single-element `files` array in memory. The next write persists the new format. If both flat fields and a `files` array are present, the lockfile is treated as corrupt and an error is raised.

**Merge-on-re-add.** Running `unpm add` for a package that already exists in the manifest appends new files to the existing entry and preserves the current version. Duplicate file paths are silently ignored. A conflicting `--version` flag is an error. This lets users incrementally build up a dependency's file list without losing existing state.

**Filename collision resolution.** Cross-package collisions (two packages producing the same output filename) use the existing package-name prefix strategy. Intra-package collisions (two source paths within the same package that share a basename) prefix with the immediate parent directory, escalating to more path segments if parents also collide. Collision detection runs at `add` time so that `install` can trust the filenames already recorded in the lockfile.

## Consequences

**Positive:**
- Packages with paired assets (JS + CSS, multiple bundles) are first-class citizens
- Uniform `files` array in the lockfile simplifies all command implementations — no branching on single vs. multi
- Auto-migration makes the lockfile format change invisible to users on first run
- Merge-on-re-add supports incremental workflows without destructive overwrites

**Negative:**
- `file` and `files` as separate manifest fields adds surface area to the schema; users may initially be confused about which to use
- Auto-migration is a one-way door — old unpm versions cannot read the new lockfile format
- Intra-package collision resolution with directory prefixes can produce surprising filenames that differ from what the user might expect
