---
name: verilog-format-maintainer
description: Use this skill when modifying the verilog-format repository. It defines the project workflow, code quality bar, validation order, testing expectations, and git or PR rules for this repo, while also explaining how to apply the requested Rust-specific rules in a repository that currently uses Java 8 and Maven instead of Cargo.
---

# Verilog Format Maintainer

## Overview

Use this skill for any change in this repository.
It captures the project-specific working agreement for design, naming, tests,
validation, and commits.

Read [references/project-standards.md](references/project-standards.md)
before changing code, tests, build logic, or commit structure.

## Repo Reality

- The current repository is Java 8 plus Maven, not Rust plus Cargo.
- Primary implementation code lives under `src/main/java/`.
- The repository currently has no `Cargo.toml` and no Cargo workspace.
- Do not claim `cargo fmt` or `cargo clippy` compliance when those commands do
  not exist in the working tree.
- Apply the Rust-only rules from the reference file when the task touches real
  Rust code or introduces Rust tooling.
- For the current Java and Maven codebase, follow the same design intent using
  repository-native Java idioms and Maven validation.

## Workflow

1. Inspect the affected code paths, CLI behavior, and build flow before editing.
2. Keep each change focused on one logical concern.
3. Prefer design cleanup before feature work when both are needed.
4. Preserve low coupling, strong encapsulation, and top-down readability.
5. Add or update tests for user-visible behavior whenever behavior changes.
6. Run the required validation commands before finishing.
7. If a mandated command or script is missing, report the gap explicitly rather
   than silently substituting it.

## Validation Order

### Current repository baseline

Run these for normal changes in the current Java and Maven repository:

```sh
mvn clean package
```

When formatter output or CLI behavior changes, also run a targeted smoke check
against a representative Verilog input and review the user-visible output.

### Additional Rust-only validation

If the repository gains Rust code or the task edits Rust code, the following
become mandatory before the change is considered complete:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

If the user requires Rust validation but the repository still lacks the
necessary Cargo files, stop short of claiming compliance and state exactly what
is missing.

## Behavioral Guardrails

- Optimize for readable, maintainable code over clever shortcuts.
- Use self-explanatory names at the point of use.
- Keep files centered on one main concept; split oversized files early.
- Prefer narrow visibility and narrow lint suppression.
- Explain non-obvious design decisions and "why" comments, not line-by-line
  narration of what the code already says.
- Prefer tests that assert user-visible formatter behavior over internal
  implementation details.

## Git And PR Rules

- Use atomic commits when the user asks for commits.
- Split refactors from behavior changes whenever practical.
- Keep each PR focused on one topic.
- Use `git commit -s` for repository commits so the commit includes a valid
  `Signed-off-by:` trailer.
- Large commits need a body with bullet points summarizing the main changes.
- If no commit was requested, do not create one on your own.

## When To Load References

- Load [references/project-standards.md](references/project-standards.md) for
  naming, file structure, testing, validation, and commit policy details.
- Re-read the validation section before final verification to avoid claiming
  checks that are unavailable in the current tree.
