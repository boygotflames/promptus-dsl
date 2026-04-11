# Changelog

All notable changes to the `.llm` transpiler are documented here.

This project is at v0. Until v1, entries track meaningful milestones
rather than semantic versioning increments. Breaking changes to stable
surfaces are always noted explicitly.

---

## [v0] — Active

### Stable surface (no breaking changes)
- Surface syntax: indentation-based DSL with 8 reserved top-level keys
- Canonical formatter: 2-space indentation, deterministic scalar quoting,
  key ordering follows Document field order
- `plain` output: deterministic, order-stable, whitespace-normalized
- `json-ir` output: deterministic JSON intermediate representation

### Implemented and usable (partial)
- Semantic validation: duplicate key detection, top-level shape rules,
  type constraints per key
- Provider selection: `generic` (default) and `openai` (explicit);
  `anthropic` is explicitly unsupported
- VS Code support: file association, syntax highlighting, language config

### Provisional (format may change)
- `shadow` output: compact machine-facing representation; deterministic
  within the current implementation but not a frozen public contract
- `bench` report: token counting and baseline comparison; report shape
  subject to revision

### Infrastructure
- span-aware AST with deterministic diagnostics
- CLI: `parse`, `validate`, `transpile`, `fmt`, `bench`
- conformance test suite covering the stable contract surface
- public compatibility matrix at docs/compatibility-matrix.md

---

## Notable Changes

### Validator: system key required + real-world fixtures
- `system` is now a required key — documents missing it produce
  Error E101 "missing required key: `system`"
- SPEC.md Required Keys: system added; two-key minimum documented;
  E101 description generalized to "missing required key"
- Three real-world example fixtures added:
  examples/code-reviewer.llm, data-pipeline.llm, support-agent.llm
- Matching baselines added; Token Efficiency table expanded to 7
  fixtures (avg savings 8.5% shadow vs Markdown)
- Conformance tests: system-required and two-key minimum added

### Internal cleanup + validator type constraint hardening
- src/provider.rs: ShadowProfile::ProvisionalV0 renamed to V0
  (shadow is stable; internal name updated to match)
- SPEC.md: vars key grammar constraint, system/user/output mapping
  key behavior documented explicitly in Type Constraints; key grammar
  enforced at parse time (E005), not validator — documented accurately
- examples/invalid/vars-invalid-key.llm: new fixture (key starting
  with digit triggers E005 at parse time)
- tests/conformance.rs: conformance_vars_key_grammar_is_enforced_at_parse_time

### Benchmarking: richer baselines and token savings proof
- examples/baselines/extractor.md: honest Markdown equivalent added
- examples/baselines/json-output.md: honest Markdown equivalent added
- examples/baselines/quoted.md: honest Markdown equivalent added
- README.md: Token Efficiency section added with actual savings table
- tests/conformance.rs: bench determinism and positive-savings
  conformance tests added for all four fixtures

### VS Code: formatter-on-save integration
- `editors/vscode/extension.js`: new entry point; registers
  `DocumentFormattingEditProvider` for the `llm` language; spawns
  `llm_format fmt <filepath>` (stdout is the default — no `--write`
  flag needed); resolves binary from `llm.formatterPath` setting or
  PATH; graceful ENOENT and non-zero-exit handling
- `editors/vscode/package.json`: `main`, `activationEvents`, and
  `llm.formatterPath` configuration contribution added
- `editors/vscode/README.md`: Formatter section added explaining
  how to configure; what-it-does list updated
- Note: `fmt` already outputs to stdout by default (no `--write`);
  no CLI change required

### Logo and visual identity
- `assets/Promptus.svg`: project logo added (Fibonacci spiral,
  dual-gradient visual identity)
- `editors/vscode/images/Promptus.svg`: logo source for VS Code
  extension (export to PNG before marketplace publication)
- `editors/vscode/package.json`: `icon` field added
- `README.md`: logo header added; `assets/` entry added to
  Repository Layout

### Validator: empty scalar enforcement
- `agent`, `memory`/`tools`/`constraints` sequence items, and
  `vars` scalar values now reject empty strings with E103
- `validate_sequence_field`: empty-scalar branch added before
  the non-scalar-type branch
- `validate_vars_field`: empty-scalar branch added in the
  per-entry match
- SPEC.md Type Constraints: introductory empty-scalar rule added;
  sequence and vars bullets updated to say "non-empty"
- `examples/invalid/empty-agent.llm`: new invalid fixture
  (`agent: ""` — empty quoted scalar)
- `tests/validate.rs`: `empty_agent_scalar_is_rejected` added
- `tests/conformance.rs`: `conformance_validation_rejects_empty_agent_scalar` added

### Diagnostic error codes (E0xx/E1xx)
- `Diagnostic` struct gains `code: Option<&'static str>` field and
  `with_code()` builder method
- Error code vocabulary defined: E001–E024 (lexer/parser errors),
  E101–E110 (validator errors) — see SPEC.md Diagnostic Codes section
- All lexer, parser, and validator emission sites carry codes
- Display format updated: `[E001] message` prefix when code is present
- SPEC.md: `### Diagnostic Codes` subsection added to Validation
  Semantics with full code table and stability declaration
- Conformance tests: `conformance_missing_agent_diagnostic_carries_e101_code`
  and `conformance_diagnostic_codes_are_present_on_all_validator_errors`
  added
- Existing parse-error and validate tests updated to reflect new display
  format with code prefix

### Compatibility matrix cross-link audit
- `plain` row: SPEC.md §Plain Output cross-link added
- `json-ir` row: SPEC.md §JSON-IR Output cross-link added
- `shadow` row: SPEC.md §Shadow Representation direct link added
- `openai` row: stale "provisional shadow mapping" language corrected
- Boundary Notes: stable contract summary updated to include `shadow`

### Plain and JSON-IR output contracts formally specified
- SPEC.md: added `## Plain Output` section — prescriptive encoding rules,
  stability declaration, and example; cross-links added from Public
  Contract Status
- SPEC.md: added `## JSON-IR Output` section — full structural encoding
  rules (object/array/scalar layout, indentation, quoting, absent-key
  omission), stability declaration, and example
- SPEC.md: `shadow` moved from `provisional` to `stable` in Public
  Contract Status (shadow was stabilized in a prior packet; this corrects
  the status entry that was missed at that time)
- SPEC.md: stale "provisional shadow mapping" language in Deferred Work
  updated to reflect current stable status
- Plan.md: Phase 3 remaining work fully resolved; provider differentiation
  noted as the remaining post-v0 concern

### Semantic validation: agent key required
- `agent` is now a required key — documents missing it produce an
  Error diagnostic with message "missing required key: `agent`"
- SPEC.md Validation Semantics section restructured: Required Keys
  paragraph added before Type Constraints list
- New invalid fixture: examples/invalid/missing-agent.llm
- Conformance and validate tests added

### Shadow output stabilization
- Shadow format promoted from `provisional` to `stable`
- SPEC.md Shadow Representation section rewritten as a prescriptive
  contract: full marker table, encoding rules, provider profile
  behavior, stability declaration
- docs/compatibility-matrix.md shadow row updated to `stable`
- Conformance tests added for stable shadow contract (extractor fixture,
  absent-key omission, full marker coverage)

### Phase 9 completion
- Added docs/versioning.md: v0 stability strategy and v1 bump criteria
- Added CONTRIBUTING.md: contribution scope, workflow, commit style
- Added CHANGELOG.md
- Removed src/tokenizer.rs and src/transpiler.rs (empty scaffold residues)
- README.md: Getting Started, input/output example, Contributing section
- SPEC.md: Mapping node documented, Deferred Work explicitly marked
  post-v0, conformance anchor added
- docs/compatibility-matrix.md: cross-link audit completed, consistent
  with SPEC.md contract status

### Phase 9 kickoff (standardization prep)
- Added root README.md
- Added docs/compatibility-matrix.md
- Added tests/conformance.rs
- Established stable/provisional/partial/unsupported boundary language
- Removed empty scaffold residues (src/tokenizer.rs, src/transpiler.rs)
- Added CONTRIBUTING.md, docs/versioning.md, CHANGELOG.md
