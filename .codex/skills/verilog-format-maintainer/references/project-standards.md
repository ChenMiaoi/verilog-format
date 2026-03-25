# Project Standards

## How To Apply These Rules

- Treat every `must` rule in this file as mandatory.
- Treat every `recommended` rule as the default choice unless repository
  reality, compatibility, or user direction requires otherwise.
- The repository is currently Java plus Maven. Apply the same engineering
  intent to Java code today.
- The Rust-specific section becomes mandatory only when the task edits Rust code
  or introduces Rust tooling into this repository.

## General Rules

### Must

- `descriptive-names`: names must be self-explanatory at the point of use.
- `accurate-names`: names must reflect actual behavior and side effects.
- `encode-units`: encode units in names when types do not express them, such as
  `*_bytes` or `*_ms`.
- `bool-names`: boolean names should read like assertions, such as `is_*`,
  `has_*`, `can_*`, or `should_*`.
- `explain-why`: comments should explain why, not restate what the code does.
- `design-decisions`: record non-obvious design decisions and rejected
  alternatives near the decision point or in nearby documentation.
- `one-concept-per-file`: each file should carry one main concept; split large
  files before they become catch-all containers.
- `top-down-reading`: code should read from high-level flow into lower-level
  helpers.
- `logical-paragraphs`: organize function bodies into logical steps.
- `error-message-format`: error messages must be specific, actionable, and
  stylistically consistent.

### Recommended

- `semantic-line-breaks`: prefer semantic line breaks in Markdown and doc
  comments.
- `cite-sources`: cite external specifications or algorithm sources when an
  implementation depends on them.
- `familiar-conventions`: follow familiar Rust and Linux conventions when they
  fit the repository and language in use.

## Rust-Specific Rules

Apply this section when the task touches Rust code.
For Java code, follow the same intent with Java idioms, type modeling, and API
design.

### Must

- `camel-case-acronyms`: types, traits, and acronyms must follow Rust naming
  conventions.
- `minimize-nesting`: prefer early returns and reduce nesting depth.
- `small-functions`: keep functions focused on one job.
- `no-bool-args`: avoid unclear boolean parameters; prefer enums or config
  structs.
- `rust-type-invariants`: use the type system to express invariants whenever
  possible.
- `propagate-errors`: prefer `?` for fallible paths instead of manual plumbing.
- `narrow-visibility`: default to the narrowest viable visibility, preferably
  private or `pub(crate)`.
- `narrow-lint-suppression`: keep lint suppression as small and local as
  possible.
- `debug-assert`: use `debug_assert!` only for correctness checks that can be
  omitted in release builds.

### Conditionally Required

- `justify-unsafe-use`: every `unsafe` block must include a `// SAFETY:`
  explanation.
- `document-safety-conds`: any `unsafe fn` or unsafe contract must document a
  `# Safety` section.
- `module-boundary-safety`: reason about safety at module boundaries, not only
  at individual call sites.

### Recommended

- `explain-variables`: split complex expressions into semantically named
  intermediates.
- `block-expressions`: use block expressions to constrain temporary state and
  lifetimes.
- `checked-arithmetic`: prefer checked or saturating arithmetic when overflow is
  plausible.
- `enum-over-dyn`: prefer enums over trait objects for closed sets.
- `getter-encapsulation`: avoid leaking mutable internal state.
- `module-docs`: add module-level docs for major modules.
- `macros-as-last-resort`: prefer functions and traits before macros.
- `minimize-copies`: avoid unnecessary copies and allocations in hot paths.
- `no-premature-optimization`: do not optimize without profiling or benchmark
  evidence.

## Testing Rules

### Must

- `add-regression-tests`: add regression tests when fixing a bug whenever the
  repository structure allows it.
- `test-visible-behavior`: test user-visible behavior, not internal
  implementation details.
- `use-assertions`: use assertion helpers rather than manual print comparison.
- `test-cleanup`: clean up files, processes, and other resources created by
  tests.

### Current Repository Notes

- For formatter behavior changes, favor tests or smoke checks against the final
  formatted Verilog output.
- Decoder-specific guidance from the request should be interpreted here as:
  when parser or formatter behavior is fixed, add repository-native regression
  tests that check the observable output.

## Validation Commands

### Current Java And Maven Repository

Mandatory baseline:

```sh
mvn clean package
```

Recommended when behavior changes:

- Run a representative CLI invocation against a sample Verilog file.
- Inspect the formatted output rather than relying only on build success.

### Rust Workflow If Rust Is Added

Mandatory before merge:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

If any of these are unavailable, call out the exact missing file, tool, or
configuration instead of marking validation as passed.

## Git And Pull Request Rules

### Must

- `atomic-commits`: one commit per logical change.
- `refactor-then-feature`: split refactors from feature work.
- `focused-prs`: keep each PR on one topic.
- `signed-commits`: repository commits must use `git commit -s`.
- `large-commit-body`: simple commits may be title-only; larger commits must
  include a body with bullet points describing the main changes.

### Commit Message Policy

- Keep Conventional Commit prefixes: `feat:`, `fix:`, `docs:`, `refactor:`,
  `test:`, and `chore:`.
- Optional scopes are allowed, such as `feat(parser): ...`.
- Write the subject line in the imperative mood.
- Keep the subject line within roughly 72 characters when practical.
- For any commit created by the agent, use `git commit -s`.
- Treat a commit as large when it changes at least 6 files or when insertions
  plus deletions reach at least 150 lines.
- Large commits must include a body with concise bullet points.

## Operating Principle

- Never fake tool output, passing checks, or repository capabilities.
- When a user rule conflicts with repository reality, surface the conflict
  clearly and either apply the repository-native equivalent now or state what
  must be added before the stricter rule can be enforced.
