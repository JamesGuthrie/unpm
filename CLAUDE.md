## Project Documentation

- Most non-bugfix changes to this repo begin as an ADR (`docs/adrs`)
- Complex features may warrant a design doc (`docs/design`)
- Complex features may warrant an implementation doc (`docs/impl`)
- We use [Tracey](https://github.com/bearcove/tracey) to record behavioural specifications, and link them to source code and tests.
- When designing new features, the existing specs should be taken into account to navigate the design space.

## Git Commits

- Use [Conventional Commits](https://www.conventionalcommits.org/) format. This is required for release-please to generate changelogs and determine version bumps.
  Common prefixes: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`. Use `!` after the prefix for breaking changes (e.g. `feat!:`).
- **One commit per feature.** Do not commit after each task or implementation step. Make a single, self-contained commit when the feature is complete, all tests pass, and the code compiles. This avoids dead code from intermediate states.
- The only exception is documentation-only commits (ADR, design doc) that precede implementation — these may be committed separately if the feature spans multiple sessions.


