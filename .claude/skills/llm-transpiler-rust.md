---
name: llm-transpiler-rust
description: >
  Invoke this skill for any implementation, review, refactor, debug, or
  performance work in a Rust `.llm` DSL transpiler repository. Covers all
  compiler stack layers: lexer, parser, AST, validator, diagnostics, transpile,
  CLI, bench, and tests. Applies specification-first, phase-aligned,
  deterministic workflow anchored to Mission.md, Plan.md, and SPEC.md as the
  authoritative control plane. Use this skill, not generic Rust help, when the
  task touches transpiler correctness, spec-facing behavior, module boundaries,
  diagnostics design, or output determinism. Combine with
  rust-memory-methodology for any task where ownership, allocation, or
  data-shape choices are architectural questions.
---

# .llm Transpiler Rust

## When to Use This Skill

Use this skill when:
- you are implementing, reviewing, or refactoring any layer of the `.llm` transpiler stack
- you are adding or changing spec-facing behavior (syntax, semantics, diagnostics, output)
- you are working on any of: `ast`, `parser`, `validator`, `diagnostics`, `transpile`, `cli`, `bench`
- you need to stay phase-aligned with `Plan.md` and spec-aligned with `SPEC.md`
- you are writing, updating, or debugging tests for compiler behavior

Also invoke `rust-memory-methodology` when:
- you are designing new AST or IR types
- you are refactoring ownership or allocation surfaces
- borrow checker friction is a symptom of a design problem
- a decision about `Clone`, `Box`, `Rc`, `Arc`, lifetimes, or interior mutability is architectural

Do not use this skill for: generic Rust projects unrelated to the `.llm` transpiler, or for memory and ownership reasoning in isolation; use `rust-memory-methodology` for that.

---

## Task Modes

### DESIGN MODE
1. Read `SPEC.md` first. Do not design what is not specced.
2. Confirm which phase in `Plan.md` this work belongs to.
3. Design type shapes before writing implementation code.
4. Define span strategy for any new syntax-facing structure before writing the parser.
5. Define expected diagnostics output before writing the validator.
6. Add tests before or alongside implementation, not after.

### REVIEW MODE
1. Read `Mission.md`, `Plan.md`, and `SPEC.md` before forming any opinion.
2. Check actual behavior against spec claims.
3. Run the Review Checklist.
4. Flag: spec drift, missing span coverage, non-deterministic output, missing error cases, module boundary violations.

### REFACTOR MODE
1. Confirm the refactor does not change any spec-facing behavior.
2. Run the full validation suite before touching anything. Establish a green baseline.
3. Prefer minimal diffs.
4. Re-run validation after every meaningful change.

### DEBUG MODE
1. Reproduce the failure with the smallest possible input before changing anything.
2. Distinguish: compiler-internal bug vs spec ambiguity vs test expectation drift.
3. For borrow checker friction: invoke `rust-memory-methodology` in DEBUG MODE.
4. Fix the root cause. Do not suppress errors with `#[allow(...)]` or `unwrap()` replacements.

### PERFORMANCE MODE
1. Do not optimize without a failing bench or a measured regression.
2. Do not change spec-facing behavior in the name of performance.

---

## Start Protocol

Before writing any code, regardless of mode:

1. Read `Mission.md`, `Plan.md`, and `SPEC.md`.
2. Inspect the relevant repository surfaces: modules touched, tests covering them, relevant examples.
3. Re-anchor on the current phase in `Plan.md`.
4. Preserve all manual or previously generated work unless explicitly requested otherwise.
5. State the narrow milestone before writing any code.

---

## Governing Rules

- Treat `Mission.md`, `Plan.md`, and `SPEC.md` as authoritative.
- Do not let implementation outrun the spec.
- Keep behavior deterministic and explainable at every layer.
- Never introduce speculative features. Leave `// TODO:` for deferred ideas.
- Use git-aware, non-interactive workflows.

---

## Implementation Rules

- Favor stable Rust, explicit types, deterministic control flow.
- Avoid `unsafe` unless explicitly required and justified.
- Preserve or add source spans on every syntax-facing structure.
- Module boundaries:
  - `ast` — data shapes only
  - `parser` — produces AST from tokens; no semantic validation
  - `validator` — enforces semantic rules; no parsing
  - `diagnostics` — diagnostic types and span rendering; no domain logic
  - `transpile` — consumes validated AST; no re-parsing
  - `cli` — entry point only; no domain logic
  - `bench` — performance measurement only; no test assertions

---

## Review Checklist

- [ ] Does the implementation match `SPEC.md` exactly?
- [ ] Does every syntax-facing structure carry a span?
- [ ] Are diagnostics structurally separated from semantic payload?
- [ ] Is output order deterministic? (no `HashMap` iteration in output paths)
- [ ] Are module boundaries respected?
- [ ] Are error cases tested, not just happy paths?
- [ ] Does the CLI exit with correct exit codes?

---

## Validation

Run in this order for every implementation task:
```
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

When a change affects spec-facing behavior, update in this order:
1. `SPEC.md`
2. examples
3. tests
4. then implementation

---

## Reporting Contract

```
Transpiler Task Report
Mode: [design | review | refactor | debug | performance]
Phase: [phase name from Plan.md]
Component(s): [modules touched]
Spec alignment: [what SPEC.md requires vs what was implemented]
Files changed: [list]
Tests added or updated: [list]
Commands run and results:
  cargo fmt: [passed / issues fixed]
  cargo clippy: [passed / warnings resolved]
  cargo test: [N passed, N failed, N ignored]
  CLI smoke checks: [description and result]
Design decisions: [decision - rationale]
Open gaps or risks: [anything deferred or requiring future spec clarification]
Phase status: [on track / blocked / ready to advance]
```
