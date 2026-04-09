# Development Plan for the `.llm` Transpiler

## Purpose

This roadmap tracks the project from scaffold to standardization. It is both a planning document and a status document, so it must reflect what the repository actually contains, not what we vaguely hoped to build later.

---

## Status Legend

- `[COMPLETED]` — landed and stable enough to build on
- `[PARTIAL]` — materially implemented, but not closed against the original phase ambition
- `[IN PROGRESS]` — the active focus right now
- `[PENDING]` — planned but not started
- `[BLOCKED]` — waiting on a decision or prerequisite

---

## Current Reality Snapshot

The repository is no longer a scaffold. Current tracked reality includes:

- a deterministic parser and span-bearing AST
- initial semantic validation
- deterministic `plain`, `json-ir`, and provisional `shadow` outputs
- a working CLI for `parse`, `validate`, `transpile`, `fmt`, and `bench`
- benchmarking with external baseline comparison
- a canonical formatter
- minimal VS Code syntax support
- provider/profile selection for `shadow` and `bench`
- a public compatibility matrix
- a dedicated conformance test layer for the currently claimed contract surface

The biggest remaining weakness is not runtime functionality. It is public-surface truth: documentation, compatibility clarity, standardization prep, release readiness, and governance/contribution structure.

---

# Roadmap

## Phase 0 — Scaffold and Foundation `[COMPLETED]`

### Objective
Establish a minimal but production-structured Rust project.

### Reality
- crate and module structure exist
- examples and tests exist
- initial spec/docs surface exists
- basic project shape is stable

### Remaining Notes
Scaffold residues src/tokenizer.rs and src/transpiler.rs have been
removed. Phase 0 is fully closed.

---

## Phase 1 — Parser and AST `[COMPLETED]`

### Objective
Turn the Surface `.llm` DSL into a deterministic and inspectable AST.

### Reality
- indentation-based parsing is implemented
- reserved top-level keys are enforced
- scalars, quoted strings, lists, and nested maps are supported
- comments are supported
- spans are carried through the AST
- syntax failures produce deterministic diagnostics

### Remaining Notes
The parser is a real milestone now, not an in-progress stub.

---

## Phase 2 — Semantic Validation `[PARTIAL]`

### Objective
Enforce semantic correctness beyond raw syntax.

### Reality
- duplicate keys in mapping scope are detected
- top-level shape rules for current keys exist
- semantic diagnostics are separated from syntax diagnostics
- duplicate and unknown top-level keys remain parser-owned by design

### Remaining Work
- fuller required-structure rules once the spec is explicit
- richer semantic contracts beyond current conservative v0 checks
- any future error-code strategy

---

## Phase 3 — Transpiler Targets `[PARTIAL]`

### Objective
Generate useful outputs from the AST.

### Reality
- `plain` is implemented and deterministic
- `json-ir` is implemented and deterministic
- provisional `shadow` is implemented and deterministic
- output ordering is test-backed

### Remaining Work
- freeze output compatibility expectations
- clarify which outputs are internal/intermediate vs public contract
- evolve `shadow` without pretending the current provisional form is final

---

## Phase 4 — Command-Line Interface `[PARTIAL]`

### Objective
Provide a usable developer-facing CLI.

### Reality
- `parse`, `validate`, `transpile`, `fmt`, and `bench` exist
- transpile supports stdout and file output with safe overwrite behavior
- parse/validate/transpile/fmt/bench flows are test-backed

### Remaining Work
- stronger public CLI documentation
- decide whether `explain-error` is real roadmap or dead wish-list
- packaging/help polish for external users

---

## Phase 5 — Benchmarking and Proof `[PARTIAL]`

### Objective
Quantify the value of `.llm`.

### Reality
- bench measures `source`, `plain`, `json-ir`, and `shadow`
- token counting uses an explicit tokenizer path
- external baseline comparison is implemented
- reports are deterministic and provider-visible

### Remaining Work
- conformance around what proof claims are allowed publicly
- broader benchmark discipline beyond one tokenizer family
- release-quality benchmark fixtures and reporting conventions

---

## Phase 6 — Formatter and Language Ergonomics `[PARTIAL]`

### Objective
Improve authoring quality and canonical normalization.

### Reality
- canonical formatter exists
- `fmt` CLI exists
- formatter is idempotent
- canonical quoting and indentation rules are implemented

### Remaining Work
- decide whether comments/blank-line preservation matters
- expand public style guidance
- separate canonical language contract from mere implementation convenience where needed

---

## Phase 7 — Editor Support `[PARTIAL]`

### Objective
Support real developer workflows in editors.

### Reality
- minimal VS Code package exists
- `.llm` association exists
- grammar and language configuration exist
- extension assets have basic sanity coverage

### Remaining Work
- no LSP
- no live validation
- no formatter integration
- no broader editor story

---

## Phase 8 — Provider-Aware Adapters `[PARTIAL]`

### Objective
Introduce explicit provider-profile plumbing without faking provider behavior.

### Reality
- provider abstraction exists
- `generic` is the default path
- `openai` is explicit and currently maps to the same current behavior
- `anthropic` is explicitly unsupported
- bench and `shadow` both expose provider selection

### Remaining Work
- real provider differentiation
- tokenizer/profile plurality beyond the current single concrete path
- spec language for what provider-aware behavior is allowed to affect

---

## Phase 9 — Ecosystem and Standardization `[IN PROGRESS]`

### Objective
Turn the repository from a strong internal prototype into a public-facing, disciplined, adoptable project surface.

### Kickoff Reality
This kickoff packet is focused on documentation truth and public/private boundary cleanup:

- create a real root `README.md`
- update this plan to match the actual repository
- privatize `Genesis.md` as local continuity rather than public repo surface
- prepare the repo for standardization work instead of pretending standardization is already done

The first public-contract hardening slice has now also landed:

- add a public compatibility matrix
- add a dedicated conformance suite for the claimed contract surface
- make stable vs provisional boundaries explicit in the public docs

### Remaining Work
- versioning strategy
- contribution workflow
- release/process hygiene
- public adoption polish

---

## Immediate Priorities

1. Keep public docs, compatibility status, and conformance coverage truthful and current.
2. Decide what versioning and compatibility language mean for the stable v0 contract surface.
3. Expand conformance only where the repo is actually ready to make public promises.
4. Clarify contribution, release, and process expectations for external use.
5. Annotate or remove stale scaffold residues that confuse the source tree.

---

## Predicted Next Roadmap After This Phase 9 Kickoff

This forecast is a judgment call based on current repo reality, not a guarantee.

### Likely Next Track 1 — Contract Hardening
- define versioning language for the project and outputs
- keep sharpening what is public contract vs internal implementation detail
- tighten `SPEC.md` where behavior is currently implied only by tests or matrix notes

### Likely Next Track 2 — Conformance and Release Readiness
- maintain and expand conformance only where the contract is explicit
- tighten release hygiene, contribution guidance, and packaging expectations

### Likely Next Track 3 — Adoption Polish
- improve README/onboarding examples
- decide how much editor support should grow before LSP work is justified
- make provider-support truth legible without overselling it

---

## Working Principle

Each new phase or sub-phase should reduce ambiguity, not just add code. The repo is now mature enough that documentation drift, compatibility ambiguity, and public-surface sloppiness are real engineering problems, not secondary chores.
