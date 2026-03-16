# Reverse-Engineering Tracey-Compatible Specs for unpm

## Goal

Create a set of tracey-compatible spec files that serve as the single source of truth for unpm's current behavior. These specs are primarily intended to give LLMs a concise, structured behavioral reference when designing and implementing features — replacing the need to reconstruct intent from verbose ADRs, design docs, and implementation docs.

## Context

unpm is primarily LLM-developed. Existing documentation (design docs, implementation plans) is thorough but verbose and mixes historical rationale with current requirements. There is no single artifact that says "here is what unpm does right now" in a form optimized for LLM consumption.

Tracey specs use a `r[requirement.id]` marker syntax in markdown to define requirements that can later be linked to implementation and test code via comment annotations. Even without the tracey toolchain installed, the spec format provides a terse, navigable behavioral contract.

## Phasing

### Phase 1 (this plan)

- Reorganize `docs/` directory structure
- Reverse-engineer ADRs from existing design docs
- Reverse-engineer tracey-compatible spec files from source code, tests, and existing docs
- No tracey installation, no code annotations

### Phase 2 (future)

- Install tracey, add `.config/tracey/config.styx`
- Annotate source and test code with tracey markers (exact syntax per tracey docs at time of adoption)

### Phase 3 (future)

- CI integration with `tracey query validate --deny warnings`
- MCP integration for Claude Code sessions

## Docs Directory Reorganization

Current structure:

```
docs/
  plans/
    2026-03-08-unpm-design.md
    2026-03-08-unpm-implementation.md
    2026-03-12-multi-file-deps.md
  specs/
    2026-03-12-multi-file-deps-design.md
```

Target structure:

```
docs/
  adrs/
    2026-03-08-unpm-architecture.md       (new, reverse-engineered)
    2026-03-12-multi-file-deps.md         (new, reverse-engineered)
    2026-03-14-tracey-specs.md            (moved from design/)
  design/
    2026-03-08-unpm-design.md             (moved from plans/)
    2026-03-12-multi-file-deps-design.md  (moved from specs/)
    2026-03-14-tracey-specs-design.md     (this document)
  impl/
    2026-03-08-unpm-implementation.md     (moved from plans/)
    2026-03-12-multi-file-deps.md         (moved from plans/)
  specs/
    add.md
    check.md
    config.md
    install.md
    list.md
    lockfile.md
    manifest.md
    outdated.md
    remove.md
    update.md
```

`docs/plans/` is removed after moving its contents.

## ADR Format

```markdown
# Title

## Status
Accepted

## Context
What prompted the decision.

## Decision
What was decided.

## Consequences
What follows from the decision — both positive and negative.
```

Two ADRs will be reverse-engineered from the existing design docs:

- **`2026-03-08-unpm-architecture.md`** — core architectural decisions: vendoring approach, jsdelivr as CDN, SHA-256 locking, no transitive dependencies, exact version pinning, OSV.dev for CVE scanning.
- **`2026-03-12-multi-file-deps.md`** — decision to support multiple files per dependency, lockfile migration strategy, merge-on-re-add behavior.

## Spec File Design

### Scope

One spec file per user-facing command (7 files) plus one per user-facing data format (3 files). Core modules (fetch, registry, url, cve) are covered implicitly through the command specs that exercise them.

### Requirement ID Convention

Pattern: `r[<command>.<aspect>.<detail>]`

- Commands use the command name: `r[add.interactive.version-select]`
- Data formats use the format name: `r[manifest.dependency.short-form]`
- Aspects group related requirements: `check.integrity`, `check.cve`, `check.freshness`
- Maximum 3 segments. No deeper nesting.

### Spec Content Rules

Each requirement:

- Describes one observable behavior
- Uses RFC-style normative language (MUST, MUST NOT, SHOULD)
- Is testable — you could write a test from reading it
- Is grouped under markdown headings by aspect

Specs do NOT contain:

- Implementation details (which function, which crate)
- Rationale or history (belongs in ADRs/design docs)
- Examples or tutorials (belongs in README)

### Example Spec Fragment

```markdown
# Check Command

## Integrity

r[check.integrity.lockfile-presence]
Each manifest dependency MUST have a corresponding lockfile entry.
Missing entries are reported as integrity errors.

r[check.integrity.sha-match]
The SHA-256 of each vendored file MUST match the hash stored in
the lockfile.

## CVE Scanning

r[check.cve.ignore-list]
Vulnerabilities whose ID appears in the dependency's `ignore_cves`
list MUST be excluded from results.
```

### Writing Order

Ordered by behavioral complexity (most invariants first):

**Command specs:**

1. `check.md`
2. `add.md`
3. `update.md`
4. `install.md`
5. `remove.md`
6. `list.md`
7. `outdated.md`

**Data format specs:**

8. `manifest.md`
9. `lockfile.md`
10. `config.md`

### Authoring Process (per file)

1. Read the relevant source file(s) and test file(s)
2. Read any existing design/impl docs covering the area
3. Extract every observable behavioral rule into a `r[...]` requirement
4. Write requirement text — terse, one behavior per requirement
5. Group under markdown headings by aspect
