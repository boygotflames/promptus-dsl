# Contributing to `.llm`

## Status

This project is in an active standardization phase. The core language,
parser, and output targets are implemented. Current work focuses on
hardening the public contract, improving documentation, and preparing
for broader adoption.

External contributions are welcome under the conditions below.

---

## Before You Contribute

Read these files first. They are the authoritative control plane:

- [Mission.md](Mission.md) — why this project exists and what it is building
- [Plan.md](Plan.md) — current phase, status, and roadmap
- [SPEC.md](SPEC.md) — the language specification and public contract

Do not submit changes that conflict with the current phase goals in
Plan.md or the language semantics in SPEC.md.

---

## What is In Scope Right Now

Phase 9 is active. In-scope contributions include:

- documentation improvements (README, SPEC.md clarity, examples)
- test coverage additions for existing stable behavior
- conformance test additions for the public contract surface
- bug fixes in existing compiler passes (parser, validator, transpile, fmt)
- editor support improvements (VS Code syntax, language configuration)

---

## What is Out of Scope Right Now

Do not submit contributions that:

- add new top-level keys or change the v0 surface syntax without a
  corresponding SPEC.md change and explicit maintainer agreement
- change `shadow` output format (it is provisional and under active revision)
- add new provider profiles beyond the current `generic`/`openai` stubs
- implement LSP, remote execution, cloud routing, or API auth flows
- introduce `unsafe` Rust without documented justification
- change public contract behavior (syntax, `plain` output, `json-ir` output)
  without updating SPEC.md first

---

## Development Workflow

### Setup
```
git clone <repo>
cd llm_format
cargo build
cargo test
```

### Before submitting

Run this sequence and confirm it is fully clean:
```
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All three must pass with zero warnings and zero failures.

### Spec-facing changes

If your change affects spec-visible behavior, update in this order:

1. `SPEC.md`
2. `examples/`
3. `tests/`
4. implementation

Never update the implementation before updating the spec.

---

## Commit Style

Use short, imperative commit messages.

Examples:
- `fix: correct span offset in nested mapping parser`
- `feat: add sequence support to output validator`
- `docs: clarify plain output contract in SPEC.md`
- `test: add conformance case for duplicate top-level key`
- `chore: update compatibility matrix for shadow provisional status`

Reference the relevant phase from Plan.md in the commit body when the
change is phase-aligned work.

---

## Public Contract Stability

The current stable public contract is defined in
[docs/compatibility-matrix.md](docs/compatibility-matrix.md).

Before changing anything marked `stable`, open a discussion first.
Provisional and partial surfaces can evolve with normal pull requests.

---

## Questions

If you are unsure whether a contribution is in scope, open an issue
describing what you want to change and why before writing code.
