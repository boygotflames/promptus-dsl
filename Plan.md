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
- ✓ `agent` required-key rule implemented and conformance-tested
- ✓ error-code strategy: `code: Option<&'static str>` field on
  `Diagnostic`, E001–E024 (parser/lexer) and E101–E110 (validator)
  vocabulary defined in SPEC.md, all emission sites carry codes,
  conformance-tested
- ✓ empty scalar enforcement: agent, sequence items
  (memory/tools/constraints), and vars values reject empty strings
  (E103); SPEC.md Type Constraints updated with explicit rule
- remaining: define required status for other keys (spec decision
  needed per key — deferred until use cases justify)
- remaining: richer semantic contracts beyond current conservative
  v0 checks

---

## Phase 3 — Transpiler Targets `[PARTIAL]`

### Objective
Generate useful outputs from the AST.

### Reality
- `plain` is implemented and deterministic
- `json-ir` is implemented and deterministic
- `shadow` is implemented and deterministic
- output ordering is test-backed

### Remaining Work
- ✓ `shadow` output stable: v0 format specified in SPEC.md,
  conformance tests added, compatibility matrix updated
- ✓ `plain` and `json-ir` output contracts formally specified in
  SPEC.md with prescriptive sections, encoding rules, and stability
  declarations; Public Contract Status updated with cross-links
- ✓ internal vs public contract boundary explicit in SPEC.md
  Public Contract Status (stable / partial / provisional /
  unsupported split with per-section links)
- remaining: provider differentiation — `plain`, `json-ir`, and
  `shadow` currently ignore provider; real per-provider behavior
  is a post-v0 concern

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
- ✓ visual identity: `assets/Promptus.svg` added, wired into README
  and VS Code extension source (`editors/vscode/images/Promptus.svg`)
- remaining: PNG export for marketplace publication
- remaining: formatter-on-save (requires TypeScript extension entry
  point — medium packet)
- remaining: LSP, live validation, completion (post-v0)

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

## Phase 9 — Ecosystem and Standardization `[COMPLETE]`

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
- Added CONTRIBUTING.md: scope, workflow, commit style, stability contract
- Added docs/versioning.md: v0 strategy, stability table, bump criteria
- Added CHANGELOG.md: v0 surface record and Phase 9 milestone log

### Remaining Work
- README onboarding depth (getting-started walkthrough, install steps,
  example output)
- SPEC.md deferred section cleanup and contract freeze language
- compatibility matrix cross-link consistency audit
- versioning and release process hygiene (now addressed in docs/versioning.md)
- contribution workflow (now addressed in CONTRIBUTING.md)

---

## Immediate Priorities

All Phase 9 immediate priorities are resolved:
1. ✓ Public docs, compatibility status, and conformance coverage are
   truthful and current.
2. ✓ Versioning and compatibility language defined in docs/versioning.md.
3. ✓ Conformance expanded to match the stable v0 contract surface.
4. ✓ Contribution and release expectations documented in CONTRIBUTING.md.
5. ✓ Stale scaffold residues (src/tokenizer.rs, src/transpiler.rs) removed.

---

## Next Phase

Phase 9 is complete. The repository has a clean public surface, honest
documentation, explicit contract boundaries, and a full test suite.

The next work track is **contract and capability deepening**:

- **Track 1 — Semantic validation depth** (Phase 2 remaining work):
  fuller required-structure rules, richer type constraints, error-code
  strategy
- **Track 2 — Shadow output stabilization** (Phase 3 remaining work):
  freeze shadow compatibility expectations, evolve provisional shadow
  toward a stable contract
- **Track 3 — Provider differentiation** (Phase 8 remaining work):
  real provider behavior beyond the current generic/openai stub,
  tokenizer/profile plurality
- **Track 4 — Editor and tooling** (Phase 7 remaining work):
  formatter-on-save, LSP groundwork assessment

Tracks are not sequential. The active track will be chosen based on
priority at the start of each packet.

---

## Working Principle

Each new phase or sub-phase should reduce ambiguity, not just add code. The repo is now mature enough that documentation drift, compatibility ambiguity, and public-surface sloppiness are real engineering problems, not secondary chores.
