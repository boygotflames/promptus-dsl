# Changelog

All notable changes to the `.llm` transpiler are documented here.

This project is at v0. Until v1, entries track meaningful milestones
rather than semantic versioning increments. Breaking changes to stable
surfaces are always noted explicitly.

---

## [v3] — 2026-04-12

### Summary
v3 makes the format expressive and the tooling production-grade.
Developers can now write template prompts with `{var}` references,
get inline completions and hover docs in VS Code, and rely on
CI to catch regressions before merge.

### What changed from v2 to v3
- vars expansion: `{var_name}` references in system/user/output/memory/
  tools/constraints — substituted at transpile time
- E114: undefined var reference validation
- CI: GitHub Actions workflow (test, lint, bench-sanity)
- Bench constants locked for anthropic provider across all fixtures;
  bench promoted to stable
- VS Code: completion (8 keys + sub-keys + `{var}`), hover (docs +
  values), go-to-definition (`{var}` → vars block)
- SPEC.md: vars Expansion section; Deferred Work cleaned up

### Explicit post-v3 deferrals
- Multi-file composition (includes/imports)
- Full LSP server
- vars chaining / conditional expansion
- Marketplace packaging

---

### v3 Track F — LSP groundwork (completion, hover, definition)
- `editors/vscode/extension.js`: `CompletionItemProvider` (8 top-level
  keys with snippets, system/user sub-keys, `{var}` reference completion),
  `HoverProvider` (key type/required/description + `{var}` value or E114
  warning), `DefinitionProvider` (F12 on `{var}` → vars block line)
- No LSP required — all three features implemented via VS Code extension
  APIs within existing spawn architecture
- `editors/vscode/README.md`: IntelliSense section added

### v3 Track D — vars expansion
- `src/transpile/vars.rs`: `expand()` helper; substitutes `{var_name}`
  references at transpile time; non-recursive; unknown references passed
  through verbatim
- `src/validator.rs`: `validate_var_references` pass; E114 emitted for
  each undefined var reference in `system`, `user`, `output`, `memory`,
  `tools`, or `constraints` scalars
- `plain`, `shadow` (V0 + V1Anthropic), `json_ir` emitters: vars expanded
  before output is written
- `fmt` formatter: preserves `{var_name}` verbatim (no change)
- `SPEC.md`: vars Expansion section added with full specification;
  E114 added to diagnostic codes table; Deferred Work updated
- `examples/data-pipeline.llm`: updated to use `{source_table}` and
  `{target_table}` references
- `examples/invalid/undefined-var-ref.llm`: new invalid fixture for E114
- Conformance: 7 new tests — E114 rejection, expansion in plain/shadow
  V0/V1 Anthropic, fmt preservation, non-recursive expansion

### v3 Track E — CI and tooling maturity
- `.github/workflows/ci.yml` created: 3 jobs (test, lint, bench-sanity);
  triggers on push and PR to `LLM-Promptus` and `main`; bench regression
  detection runs `cargo test conformance_bench --all-targets`
- `tests/conformance.rs`: `BENCH_ANTHROPIC_EXTRACTOR`, `BENCH_ANTHROPIC_JSON_OUTPUT`,
  `BENCH_ANTHROPIC_QUOTED` constants added; 4 new tests:
  `conformance_bench_anthropic_extractor_is_deterministic`,
  `conformance_bench_anthropic_json_output_is_deterministic`,
  `conformance_bench_anthropic_quoted_is_deterministic`,
  `conformance_bench_anthropic_token_counts_differ_from_generic`
- `docs/compatibility-matrix.md`: `bench` row promoted from `provisional`
  to `stable (scope-limited)`
- `SPEC.md`: stale "provider-specific emission layers — deferred to post-v0"
  note updated to reflect v2 implementation status

---

## [v2] — 2026-04-12

### Summary
v2 adds real provider differentiation (Anthropic XML shadow encoding +
o200k_base tokenizer), deeper semantic validation (E103 extension,
E111–E113), and live VS Code diagnostics (`validate --stdin` +
DiagnosticCollection squiggles). All v1 stable surfaces remain unchanged.

### v2 Track A — Provider differentiation
- Cargo.toml: tiktoken-rs upgraded 0.9.1 → 0.11.0 (adds o200k_base)
- src/provider.rs: `ShadowProfile::V1Anthropic` added; `TokenizerProfile::O200kBase`
  added; `Provider::Anthropic` now returns `Supported` for both shadow and
  tokenizer; `shadow_profile()` returns `V1Anthropic`; `tokenizer_profile()`
  returns `O200kBase`; unused `anyhow` import removed
- src/transpile/shadow.rs: `V1Anthropic` match arm dispatches to
  `render_v1_anthropic_document`; XML helpers added: `xml_tag_for`,
  `render_v1_node`, `sequence_item_tag`, `node_to_plain`
- src/bench/tokenizer.rs: `o200k_base` import added; `from_profile`
  now matches on profile variant, uses `cl100k_base` or `o200k_base`
- SPEC.md Shadow Representation: Provider Profiles updated; new
  `### V1 Anthropic Shadow Encoding` subsection added (tag table,
  encoding rules, stability declaration, example)
- docs/compatibility-matrix.md: anthropic row `unsupported → stable`;
  generic/openai rows updated to note V0 encoding; Boundary Notes updated
- tests/conformance.rs: 4 new anthropic tests; SHADOW_V1_MINIMAL,
  SHADOW_V1_EXTRACTOR, BENCH_ANTHROPIC_MINIMAL constants added;
  two old `rejects_unsupported` tests replaced/renamed
- tests/transpile.rs: two old `unsupported` tests updated to verify V1
  output instead of error
- tests/bench.rs: `bench_cli_rejects_unsupported_provider_selection`
  replaced with `bench_cli_anthropic_provider_is_supported`
- docs/v2-roadmap.md: Track A Sequence items ✓; Status: COMPLETE

### v2 Track C — Live editor feedback
- src/cli/validate.rs: `--stdin` flag added; reads from stdin,
  emits same diagnostic format, exits 0/1/2; mutually exclusive
  with file path argument
- editors/vscode/extension.js: DiagnosticCollection + debounced
  onDidChangeTextDocument (300ms); parses `[Exxx]` stderr output;
  inline squiggles for all parse and semantic errors; ENOENT
  handled gracefully with one-time message
- editors/vscode/README.md: Live Validation section added;
  what-it-does list updated
- tests/conformance.rs: stdin valid/invalid conformance tests added
- docs/v2-roadmap.md: Track C marked COMPLETE

### v2 Track B — Semantic validation depth
- E103 extended: system and user empty scalars now rejected
- E111 (new): empty mapping block rejected for system/user/output
  (defensive guard — unreachable via parse_str; parser requires
  at least one entry to produce a Mapping node)
- E112 (new): empty sequence rejected for memory/tools/constraints
  (defensive guard — unreachable via parse_str; same reason as E111)
- E113 (new): duplicate items rejected in tools and constraints
  (memory exempt — duplicate history items are valid)
- SPEC.md: four new Type Constraint bullets added; E111/E112/E113
  added to Diagnostic Codes table
- Invalid fixtures: empty-system.llm, empty-mapping.llm,
  empty-sequence.llm, duplicate-tools.llm
- Conformance tests: 3 new tests (E103 system, E113 tools, E113
  memory exemption); E111/E112 documented as defensive guards

---

## [v1] — 2026-04-12

### Summary
v1 declares the `.llm` format and reference transpiler as a
stable, frozen public contract. All v1 surfaces are conformance-
tested. Breaking changes require a version bump.

### What changed from v0 to v1
- `agent` and `system` are required keys (E101)
- Diagnostic error codes (E001–E110) on all emission sites
- Shadow output stabilized and fully specified
- Plain and JSON-IR output contracts fully specified in SPEC.md
- CLI: `validate` ✓/✗ output, exit codes 0/1/2
- CLI: `parse --summary` flag
- VS Code: formatter-on-save (DocumentFormattingEditProvider)
- 7 example fixtures with Markdown baselines; 8.5% avg token savings
- Conformance suite: 80+ tests covering all stable surfaces

### Explicit post-v1 deferrals
- Provider differentiation (distinct shadow encoding per provider)
- Tokenizer plurality beyond cl100k
- VS Code LSP, live validation, completion
- `bench` structured output format / CI integration
- Additional required keys beyond agent + system

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

### CLI output polish
- `validate`: success now prints `✓ valid <file>` to stdout;
  failure prints diagnostics + `✗ invalid <file> (N error(s))`
  summary to stderr; exit codes: 0=valid, 1=invalid, 2=IO error
- `parse`: `--summary` flag added for compact key/node summary
  (`✓ parsed <file>` / `keys: ...` / `nodes: N`); failure prints
  `✗ parse failed <file>` to stderr
- editors/vscode/README.md: formatter error behavior documented

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
