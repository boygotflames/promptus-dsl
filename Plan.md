# Development Plan for the `.llm` Transpiler

## Purpose

This roadmap defines the implementation phases for the `.llm` project and tracks progress from initial scaffold to ecosystem standardization.

It is intended to be a living project document.

---

## Status Legend

- `[COMPLETED]` — finished and stable enough to build on
- `[IN PROGRESS]` — actively being implemented
- `[PENDING]` — planned but not started
- `[BLOCKED]` — waiting on decisions, dependencies, or upstream work

---

## Current Project Status

### Phase 0 — Scaffold and Foundation
**Status:** `[COMPLETED]`

The project foundation exists and is usable.

Current baseline includes:

- initial Rust crate structure
- core module scaffolding
- AST definitions
- diagnostics scaffolding
- CLI skeleton
- examples and basic tests

This phase established the project shape, but not the full parser/validator/transpiler implementation.

### Phase 1 — Parser and AST
**Status:** `[IN PROGRESS]`

Current effort is focused on turning the Surface `.llm` syntax into a stable, deterministic AST with strong source diagnostics.

---

# Roadmap

## Phase 0 — Scaffold and Foundation `[COMPLETED]`

### Objective
Establish a minimal but production-grade project skeleton.

### Deliverables
- Create the initial Rust crate with `Cargo.toml` and `.gitignore`
- Establish the base directory structure:
  - `src/`
  - `examples/`
  - `tests/`
  - `docs/`
- Add initial placeholder modules:
  - `ast.rs`
  - `parser.rs`
  - `validator.rs`
  - `diagnostics.rs`
  - `transpile/`
  - `cli/`
  - `bench/`
- Draft the initial `SPEC.md`
- Add basic examples
- Ensure the project compiles and tests run

### Outcome
Foundation completed. The project is now ready for full parser and compiler work.

---

## Phase 1 — Parser and AST `[IN PROGRESS]`

### Objective
Turn the Surface `.llm` DSL into a deterministic and inspectable abstract syntax tree.

### Tasks
- Implement a lexer using `logos` or a custom iterator
- Implement a deterministic parser for:
  - scalar values
  - nested maps
  - lists
  - indentation-based structure
- Support top-level blocks such as:
  - `agent`
  - `system`
  - `user`
  - `memory`
  - `tools`
  - `output`
  - `constraints`
  - `vars`
- Track source spans for all AST nodes:
  - line
  - column
  - source range
- Produce clear syntax errors with useful context
- Add golden tests for valid and invalid `.llm` samples
- Refine `SPEC.md` as grammar details become stable

### Exit Criteria
- Sample `.llm` files parse successfully into AST snapshots
- Invalid syntax produces deterministic diagnostics
- Parser tests pass consistently

---

## Phase 2 — Semantic Validation `[PENDING]`

### Objective
Enforce semantic correctness beyond syntax.

### Tasks
- Detect duplicate keys
- Detect unknown blocks or unsupported structures
- Enforce required fields
- Validate scalar, list, and map shapes against the spec
- Define a consistent error-code system
- Implement semantic diagnostics with actionable messages

### Exit Criteria
- Valid documents pass semantic validation
- Invalid documents fail with precise, spec-aligned errors

---

## Phase 3 — Transpiler Targets `[PENDING]`

### Objective
Generate useful outputs from the AST.

### Tasks
- Define a `TargetEmitter` trait
- Implement:
  - `PlainEmitter`
  - `ShadowEmitter`
  - `JsonIrEmitter`
- Ensure output generation is deterministic
- Make target encoding pluggable for provider-specific Shadow formats

### Exit Criteria
- A parsed AST can be emitted reliably into all supported output targets
- Emission behavior is stable and test-covered

---

## Phase 4 — Command-Line Interface `[PENDING]`

### Objective
Provide a usable developer-facing CLI.

### Tasks
- Implement commands such as:
  - `parse`
  - `validate`
  - `transpile`
  - `bench`
  - `fmt`
  - `explain-error`
- Connect CLI commands to parser, validator, and transpiler modules
- Handle file input/output paths cleanly
- Provide strong help text and failure messages

### Exit Criteria
- The CLI supports core workflows end-to-end
- Output and errors are readable and predictable

---

## Phase 5 — Benchmarking and Proof `[PENDING]`

### Objective
Quantify the value of `.llm`.

### Tasks
- Abstract over tokenizer implementations
- Build benchmark fixtures comparing:
  - `.llm`
  - Markdown
  - other structured prompt formats
- Measure:
  - token count
  - byte size
  - compile time
- Generate repeatable reports for internal validation and public proof

### Exit Criteria
- Benchmark suite is automated
- Results clearly demonstrate the efficiency profile of `.llm`

---

## Phase 6 — Formatter and Language Ergonomics `[PENDING]`

### Objective
Improve authoring quality and consistency.

### Tasks
- Implement a canonical formatter for `.llm`
- Normalize file layout according to official style rules
- Add syntax-highlighting grammar for editors
- Build a library of examples and best-practice patterns

### Exit Criteria
- `.llm` files can be auto-formatted into canonical form
- Authoring becomes easier and less error-prone

---

## Phase 7 — Editor Support `[PENDING]`

### Objective
Support real developer workflows.

### Tasks
- Build a VS Code extension
- Provide:
  - syntax highlighting
  - validation on save
  - transpilation previews
- Explore support for additional editors and IDEs

### Exit Criteria
- Authors can work comfortably with `.llm` inside mainstream tooling

---

## Phase 8 — Provider-Aware Adapters `[PENDING]`

### Objective
Optimize output across model vendors.

### Tasks
- Implement adapters for major providers such as:
  - OpenAI
  - Anthropic
  - others as needed
- Make token-optimization tables configurable
- Support provider-specific capabilities as they evolve, including:
  - function/tool calling
  - system/user channel differences
  - structured output conventions

### Exit Criteria
- `.llm` output can be specialized without changing core source documents

---

## Phase 9 — Ecosystem and Standardization `[PENDING]`

### Objective
Turn `.llm` into a durable public standard.

### Tasks
- Formalize versioning strategy
- Publish a compatibility matrix
- Add conformance tests
- Publish migration guides
- Establish contribution and governance processes
- Grow ecosystem participation around tooling and adoption

### Exit Criteria
- `.llm` is documented, versioned, testable, and adoptable beyond the core project

---

## Immediate Priorities

1. Complete deterministic parsing for the Surface DSL
2. Lock down AST structure and source-span tracking
3. Expand parser test coverage
4. Refine the spec in parallel with implementation
5. Prepare the validation layer as the next execution phase

---

## Working Principle

Each phase should produce artifacts that are:

- testable
- deterministic
- spec-aligned
- extensible
- useful on their own

The project should evolve in layers, with each completed phase reducing ambiguity for the next one.