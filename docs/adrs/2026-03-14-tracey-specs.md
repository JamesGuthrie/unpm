# Tracey-Compatible Behavioral Specs

## Status

Accepted

## Context

unpm is primarily LLM-developed. Existing documentation (ADRs, design docs, implementation plans) provides thorough historical context but is verbose and spread across multiple files. There is no single artifact that concisely describes unpm's current behavior. When an LLM needs to understand what a command should do — to implement a new feature or avoid breaking existing behavior — it must reconstruct intent from ~3,700 lines of prose that mixes rationale, rejected alternatives, and current requirements.

Tracey is a spec-coverage tool that uses a `r[requirement.id]` marker syntax in markdown to define requirements. These can later be linked to implementation and test code via comment annotations, with tooling for staleness detection, coverage tracking, and CI validation.

## Decision

Adopt the tracey requirement marker format (`r[requirement.id]`) for behavioral spec files in `docs/specs/`. One spec file per user-facing command and one per user-facing data format (manifest, lockfile, config).

Specs document observable behavior in terse, RFC-style normative language. They do not contain implementation details, rationale, or examples — those remain in design docs, ADRs, and the README respectively.

Phase 1 covers spec authoring only. Tracey tooling (code annotations, CI integration, MCP server) is deferred to future phases, allowing the specs to settle before investing in annotation maintenance.

## Consequences

**Positive:**
- LLMs get a high signal-to-noise behavioral reference (~30 lines per command vs hundreds of lines of design prose)
- Specs serve as a contract: an LLM modifying a command can check the spec to see what invariants it must preserve
- The tracey marker format is forward-compatible — when tracey tooling is adopted later, specs require no changes
- Requirement IDs create a shared vocabulary for discussing behavior across docs, code, and conversation

**Negative:**
- Another set of docs to maintain alongside ADRs, design docs, and README
- Risk of specs drifting from implementation without tracey's staleness detection (mitigated in future phases)
- Overhead of establishing and following the requirement ID convention
