# v3 Roadmap

## Status

v2 is declared and complete. v3 work begins here.

## Guiding Principle

v3 makes the format expressive and the tooling production-grade.
v2 proved the format works and the provider story is real.
v3 answers: can developers build non-trivial prompts with it,
and can teams adopt it with confidence in their CI pipelines?

The three pillars:

1. **vars expansion** — `{var}` references in system/user blocks
2. **CI and tooling maturity** — bench regression, GitHub Actions
3. **LSP groundwork** — completion and hover without a full language server

v3 does not break v2. All v2-stable surfaces remain stable.

---

## Track E — CI and Tooling Maturity

### Problem

The repository has no CI configuration of any kind. There is no
`.github/` directory. Every PR that changes shadow encoding, validator
behavior, or bench output is merged without automated verification.

The conformance suite already locks in exact bench byte counts and
token counts via constants (`BENCH_GENERIC_MINIMAL`, `BENCH_ANTHROPIC_MINIMAL`,
etc.) and asserts `conformance_bench_savings_are_positive_for_all_fixtures`.
But these checks only run locally, only when a developer remembers to
run `cargo test`. A shadow encoding regression or a tokenizer change
that silently inflates token counts is currently undetectable before
merge.

Three classes of regression are currently invisible on PRs:

1. **Token count inflation** — a change to shadow encoding increases
   token output vs the locked constant; no test fails without running
   `cargo test`
2. **Clippy regressions** — new warnings that signal real problems
   pass through unchecked
3. **Formatting drift** — code merged without `cargo fmt` runs silently

### What v3 changes

One file: `.github/workflows/ci.yml`. Four jobs:

1. **test** — `cargo test --all-targets`; catches all conformance and
   unit failures including the existing bench regression constants
2. **clippy** — `cargo clippy --all-targets --all-features -- -D warnings`;
   fails on any warning
3. **fmt** — `cargo fmt --check`; fails if formatting is not canonical
4. **bench-anthropic** — add locked bench constants for the
   `extractor.llm` and `json-output.llm` fixtures under the
   `anthropic` provider (currently only `minimal.llm` has a locked
   constant for anthropic); ensures three-fixture coverage for the
   newest provider

The regression threshold rule: the existing `cargo test` suite
enforces exact byte and token counts via `assert_eq!(result, CONSTANT)`.
No new threshold mechanism is needed — the constant-based approach is
already the right design. CI just needs to run the tests.

### Sequence

1. Capture exact bench output for `extractor.llm` and `json-output.llm`
   under `anthropic` provider; add `BENCH_ANTHROPIC_EXTRACTOR` and
   `BENCH_ANTHROPIC_JSON_OUTPUT` constants to `tests/conformance.rs`;
   add `conformance_bench_anthropic_extractor_is_deterministic` and
   `conformance_bench_anthropic_json_output_is_deterministic` tests
2. Create `.github/workflows/ci.yml` with the four jobs listed above;
   set `on: [push, pull_request]` targeting `main`
3. Run full local validation to confirm CI config is correct before commit
4. Update `docs/compatibility-matrix.md` bench row: `provisional → stable`
   once three-fixture coverage is confirmed across all supported providers

### v3 contract commitment

By end of Track E:
- Every PR to main runs `cargo test`, `clippy`, and `cargo fmt --check`
  automatically via GitHub Actions
- Bench output is conformance-locked for all three primary fixtures
  (`minimal`, `extractor`, `json-output`) across all three providers
  (`generic`, `openai`, `anthropic`)
- Shadow encoding regressions are detectable before merge
- `bench` report shape promoted from `provisional` to `stable`

Deferred beyond Track E:
- Performance benchmarks (binary size, compile time) in CI
- Release workflow (tagging, artifact publishing)
- Cross-platform CI matrix (Windows, macOS, Linux)

---

## Track D — vars Expansion

### Problem

The `vars:` key exists, is parsed, is validated, and is conformance-
tested. But it does nothing at runtime. The values in `vars` are never
used by any other part of the document. data-pipeline.llm has:

```llm
vars:
  source_table: orders_raw
  target_table: orders_clean
  batch_size: 1000
```

But `source_table` cannot be referenced from `system:`. The author
must write the literal value `orders_raw` everywhere they need it.
This makes multi-environment prompts (staging vs production table
names, region-specific endpoints) require manual copy-paste substitution.

`vars` was designed as a template variable store. Without expansion,
it is documentation at best.

### What v3 changes

**Syntax:** `{var_name}` inside any scalar value references the
corresponding `vars` entry. No surface DSL grammar change is required:
`{` is already excluded from bare scalars (the bare scalar charset
allows only letters, digits, `_`, `-`), so `{var}` can only appear
inside double-quoted strings. The formatter already quotes any scalar
containing `{`, so the canonical form is automatically correct.

Example:

```llm
agent: DataPipeline
system:
  query: "SELECT * FROM {source_table} WHERE region = {region}"
  target: "INSERT INTO {target_table}"
vars:
  source_table: orders_raw
  target_table: orders_clean
  region: apac
```

**Scope:** References are recognized in scalar values at any position:
`system` (scalar or mapping values), `user` (scalar or mapping values),
`output` mapping values, `memory` sequence items, `tools` sequence
items, `constraints` sequence items.

`vars` values themselves do NOT support `{var}` references in v3.
Chained substitution is deferred to post-v3. This eliminates circular
reference checking entirely and keeps the implementation minimal.

**New SPEC.md subsection:** "Var References" under Surface Syntax.
Defines the `{identifier}` syntax, the scope of expansion, the vars-
in-vars exclusion, and the stability commitment.

**No AST change required.** `Node::Scalar { value: String, .. }`
already holds the raw string. The `{var}` pattern is recognized at
validation and transpile time, not parse time.

**New validator pass:** `validate_var_references` scans all scalar
values in all positions for `{identifier}` patterns and emits E114
for each reference to a var name not present in `vars`. This pass
runs after all existing validation passes.

**Transpiler change:** A helper `expand_vars(value: &str, vars: &HashMap<&str, &str>) -> String`
performs `{name}` → value substitution. Called at each scalar emit
site in `src/transpile/plain.rs`, `src/transpile/json_ir.rs`, and
`src/transpile/shadow.rs`. The formatter (`src/formatter.rs`) does
NOT expand — it preserves `{var}` verbatim.

### New diagnostic codes

| Code | Description |
|---|---|
| `E114` | undefined var reference: `{name}` is not defined in `vars` |

E115 (circular reference in vars) is deferred — vars-in-vars
substitution is not in scope for v3.

### Sequence

1. Update SPEC.md: add "Var References" subsection to Surface Syntax;
   add E114 to the Diagnostic Codes table; freeze the var reference
   contract as stable from v3
2. Add `validate_var_references` to `src/validator.rs`; collect var
   names from `document.vars`, scan all scalar values for `{identifier}`
   patterns using a simple regex or manual parser, emit E114 for each
   undefined reference
3. Implement `expand_vars` helper in a new `src/transpile/vars.rs`
   module; import and call it from plain, json-ir, and shadow emitters
4. Create conformance fixtures: `examples/vars-expansion.llm` (a
   document with `{var}` references that expand correctly) and
   `examples/invalid/undefined-var.llm` (E114 fixture)
5. Add conformance tests:
   - `conformance_vars_references_expand_in_plain_output`
   - `conformance_vars_references_expand_in_shadow_output`
   - `conformance_vars_references_expand_in_json_ir_output`
   - `conformance_e114_undefined_var_reference_is_rejected`
   - `conformance_formatter_preserves_var_references_verbatim`
6. Update `docs/compatibility-matrix.md`: `vars` expansion row added as stable

### v3 contract commitment

By end of Track D:
- `{var_name}` inside any quoted scalar is treated as a var reference
- Undefined var references emit E114
- `plain`, `json-ir`, and `shadow` targets expand `{var}` before emitting
- `fmt` preserves `{var}` verbatim (format does not expand)
- `vars` values themselves do not expand (no chaining in v3)
- All new behavior is conformance-tested

Deferred beyond Track D:
- Chained/nested vars substitution (`vars` values referencing other vars)
- Default values for vars (`{name:default}`)
- Environment variable injection (`{env:VAR_NAME}`)
- Multi-file shared vars (deferred to v4 with multi-file composition)

---

## Track F — LSP Groundwork

### Problem

The current spawn-and-parse approach (Track C) gives inline error
squiggles and formatter-on-save. It cannot give:

- **Completion** — no `textDocument/completion` without LSP (or a VS Code
  `CompletionItemProvider`)
- **Hover documentation** — no `textDocument/hover` without LSP (or a
  `HoverProvider`)
- **Go-to-definition** — no jump-to-vars-entry for `{var}` references
  without LSP (or a `DefinitionProvider`)

Of these, **completion is highest value**. `.llm` has exactly 8
top-level keys. New authors look them up constantly. Offering
`agent:`, `system:`, `user:`, `memory:`, `tools:`, `output:`,
`constraints:`, `vars:` as completions at column 1 eliminates the
most common friction point. Common sub-keys for `system` and `user`
(`role`, `objective`, `tone`, `format`, `language`) are a natural
second tier.

**No language server required.** VS Code exposes
`vscode.CompletionItemProvider` and `vscode.HoverProvider` as
extension APIs that work independently of the Language Server Protocol.
The `.llm` document structure is simple enough — 8 known top-level
keys, fixed grammar, no dynamic completion from runtime state — that
a static completion list in JavaScript covers 95% of the user-visible
value.

The spawn-and-parse pattern from Track C already handles the hard
part (error feedback). Track F extends the extension with three new
lightweight providers.

### What v3 changes

**`editors/vscode/extension.js` additions:**

1. **`CompletionItemProvider` for top-level keys** — at column 1
   (or a line that starts at col 1 with nothing yet), offer all 8
   top-level keys as `CompletionItemKind.Keyword` items. Each item
   includes a documentation string describing the key's purpose and
   type.

2. **`CompletionItemProvider` for common sub-keys** — when the cursor
   is inside a `system:` or `user:` block (indented 2 spaces), offer
   common mapping keys: `role`, `objective`, `tone`, `format`,
   `context`, `language`. These are the most-used sub-keys in the
   existing fixtures and examples.

3. **`HoverProvider`** — when hovering over any top-level key word,
   show a markdown tooltip: the key name, its type (`scalar` /
   `scalar or mapping` / `sequence of scalars` / `mapping of scalars`),
   and a one-line description. Uses the static key metadata already
   implied by the SPEC.

4. **`DefinitionProvider` for var references** — when the cursor is
   inside `{var_name}` in a quoted string, find the `vars:` block
   in the document and jump to the line where `var_name:` is defined.
   Implemented with a regex scan of the document text — no Rust
   required, pure JS.

All four providers are registered within the existing `activate`
function. No new process spawning, no socket, no protocol.

**`editors/vscode/package.json`:** no changes needed; providers
are registered via `vscode.languages.*` APIs, not contribution points.

### Sequence

1. Add `TOP_LEVEL_KEY_DOCS` static table in `extension.js`:
   key → `{ type, description }` for all 8 keys
2. Implement and register `CompletionItemProvider` for top-level keys;
   add a test: `conformance_vscode_completion_offers_top_level_keys`
   (at minimum a smoke test that the completion array has 8 items)
3. Implement and register `CompletionItemProvider` for system/user
   sub-keys; add fixture-driven sub-key completion test
4. Implement and register `HoverProvider` for top-level keys
5. Implement and register `DefinitionProvider` for `{var}` references
   (requires Track D to be meaningful; may be implemented speculatively
   or deferred to post-Track D depending on sequencing)
6. Update `editors/vscode/README.md`: add Completion and Hover sections
7. Update `docs/compatibility-matrix.md`: VS Code support row from
   `stable (scope-limited)` → `stable` once completion + hover are present

### v3 contract commitment

By end of Track F:
- Top-level key completion works at column 1 in VS Code
- Common system/user sub-key completion works in indented blocks
- Hover over any top-level key shows type and description
- `{var}` references support go-to-definition (jumping to `vars:` entry)
- All features work without a language server process

Deferred beyond Track F:
- Full LSP (tower-lsp or lsp-types in Rust) — still deferred; the
  extension-API approach covers the highest-value features without it
- Code actions / quick fixes (add missing required key, remove duplicate)
- Rename symbol (rename var key and update all references)
- Semantic token highlighting (color var references differently)

---

## Track Sequencing

**Recommended order: E → D → F**

**Track E first.** Pure tooling — one YAML file and a few new test
constants. Zero language changes, zero risk, zero SPEC updates.
Delivers immediate CI safety so that Track D and F changes are
automatically verified before merge. The conformance constants
already do the right thing; CI just runs them on every PR. Single
packet, done.

**Track D second.** vars expansion is the most important capability
gap: `vars` is parsed, validated, and displayed — but does nothing.
Every document with a `vars:` block is currently carrying dead weight.
Track D has a real SPEC change, a new validator pass, and emitter
changes across three output targets. It benefits from Track E's CI
coverage being in place before landing. Track D also unblocks Track F's
`DefinitionProvider` (go-to-var) from being useful.

**Track F last.** The completion and hover providers are pure
JavaScript extension work, no Rust, no SPEC changes. They benefit
from Track D being complete because `DefinitionProvider` for `{var}`
references is only meaningful after vars expansion is a real feature.
Track F can begin speculatively while Track D is in progress, but
the go-to-definition feature needs Track D to land first.

---

## v3 Success Criteria

v3 is complete when:

- [ ] `{var_name}` references inside quoted scalars expand at transpile
      time in `plain`, `json-ir`, and `shadow` targets
- [ ] Undefined var references emit E114 and fail validation
- [ ] vars expansion is conformance-tested across all three output targets
- [ ] `fmt` preserves `{var}` references verbatim (no formatter expansion)
- [ ] GitHub Actions CI runs `cargo test`, `cargo clippy -- -D warnings`,
      and `cargo fmt --check` on every push and PR
- [ ] Bench output is conformance-locked for all three primary fixtures
      across all three providers (9 constants total)
- [ ] `bench` report shape promoted from `provisional` to `stable`
- [ ] Top-level key completion works in VS Code (8 keys, at column 1)
- [ ] Hover over any top-level key shows type and description
- [ ] `{var}` references support go-to-definition in VS Code
- [ ] CHANGELOG.md v3 entry added
- [ ] docs/versioning.md v3 section added
