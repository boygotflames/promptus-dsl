# v2 Roadmap

## Status

v1 is declared and frozen. v2 work begins here.

## Guiding Principle

v2 deepens what v1 proved. The three pillars:

1. Real provider differentiation — not a stub
2. Deeper semantic validation — catch real mistakes
3. Live editor feedback — squiggles without a full LSP

v2 does not break v1. All v1-stable surfaces remain stable.
New behavior is additive or provider-specific.

---

## Track A — Provider Differentiation

### Problem

`generic` and `openai` currently produce identical shadow output.
The `anthropic` provider is explicitly unsupported with no shadow
profile and no tokenizer. The provider abstraction exists in
`src/provider.rs` but carries no real behavioral variation.
`bench` reports wrong token counts for Anthropic documents because
it uses cl100k_base for all providers.

### What v2 changes

**Tokenizer plurality.**
Anthropic models use a different tokenizer than OpenAI's cl100k_base.
v2 adds `TokenizerProfile::O200kBase` as a proxy for Anthropic's
tokenizer (the best public approximation), enabling honest `bench`
output for Anthropic documents. This unblocks `anthropic` support
for the `bench` subcommand.

**Anthropic shadow profile (`ShadowProfile::V1`).**
Anthropic explicitly recommends XML-style tag delimiters for Claude
prompts — `<system>`, `<user>`, `<tools>` — because Claude's
training data uses these patterns. The generic v0 `@`-marker format
is OpenAI-neutral but not Claude-optimized.

A `ShadowProfile::V1` renders the same `.llm` document using XML
tags instead of compact `@` markers:

```
# V0 (generic/openai)
@s={role="assistant";tone="helpful"}
@u="Summarize the report"
@t=["web_search","calculator"]

# V1 (anthropic)
<system>role=assistant;tone=helpful</system>
<user>Summarize the report</user>
<tools>web_search,calculator</tools>
```

This is a real semantic difference: Claude processes `<system>` blocks
differently than inline `@s=` markers. The differentiation is tied
to provider training, not cosmetics.

**SPEC.md extension.**
The Shadow Representation section gains a V1 subsection documenting
the XML-tag encoding rules, stability promise, and the Anthropic
provider binding.

### Sequence

1. Add `TokenizerProfile::O200kBase` to `src/provider.rs`; bind
   `Provider::Anthropic` to it; update `tokenizer_support` to
   `Supported` for Anthropic; update `bench` conformance tests
2. Add `ShadowProfile::V1` to `src/provider.rs`; bind
   `Provider::Anthropic` to it; update `shadow_support` to
   `Supported` for Anthropic
3. Implement `render_v1_document` in `src/transpile/shadow.rs`
   using XML-tag syntax; dispatch from `render_document` match arm
4. Update SPEC.md Shadow Representation section with V1 encoding
   specification
5. Add conformance tests proving Anthropic shadow differs from
   generic/openai for the same document; add bench tests for
   Anthropic provider

### v2 contract commitment

By end of Track A:
- `anthropic` provider is no longer `unsupported` for shadow and bench
- `openai` and `generic` shadow output remains identical (V0)
- `anthropic` shadow output uses V1 XML-tag encoding
- V1 shadow format is stable and conformance-tested

Deferred beyond Track A:
- Real Anthropic tokenizer (proprietary; O200kBase remains a proxy)
- Provider-specific tool call encoding (e.g., OpenAI function-calling
  JSON format in `@t`) — requires a new output target, not a shadow variant
- Additional provider profiles beyond generic, openai, anthropic

---

## Track B — Semantic Validation Depth

### Problem

The validator enforces required keys (E101) and type shapes
(E104–E110). It does not catch several real authoring mistakes that
pass all current checks:

- `system: ""` passes (empty scalar; E103 only covers `agent`)
- `system:` with zero mapping entries passes (vacuous block)
- `tools: []` passes (empty sequence adds no value)
- `tools: [web_search, web_search]` passes (duplicate tool)

These are not edge cases — they occur in real prompt iteration.

### What v2 changes

Four concrete new validation rules, in order of impact:

**Rule 1 — Empty scalar for system/user (extends E103)**
`validate_prompt_field` currently checks only that system/user is a
scalar or mapping. It does not check scalar emptiness. Add an empty
scalar check: `system: ""` and `user: ""` emit E103 with a message
matching the existing agent pattern (`must not be empty`).
No new error code — E103 covers "field must not be empty" generically.

**Rule 2 — Empty mapping for system/user/output (E111)**
A mapping with zero entries is never intentional. If `system:` or
`user:` resolves to a mapping with no keys, emit E111:
`\`system\` mapping must not be empty`.
Same for `output:`. Catches copy-paste skeletons left incomplete.

**Rule 3 — Empty sequence for memory/tools/constraints (E112)**
`validate_sequence_field` currently allows zero-item sequences.
If a sequence key is present but has zero items, emit E112:
`\`tools\` sequence must not be empty`. A declared but empty
sequence key serves no purpose — the author should remove it.

**Rule 4 — Duplicate sequence items in tools/constraints (E113)**
`tools: [web_search, web_search]` is a redundant declaration.
Add a HashSet pass in `validate_sequence_field` for
`TopLevelKey::Tools` and `TopLevelKey::Constraints` only
(memory items may legitimately repeat; tools and constraints
should not). Emit E113: `duplicate item \`web_search\` in \`tools\``.

### Sequence

1. Extend `validate_prompt_field` to check empty scalar (E103)
2. Add `validate_empty_mapping` helper; call from prompt/output
   validators (E111)
3. Add empty-sequence check in `validate_sequence_field` (E112)
4. Add duplicate-item check in `validate_sequence_field` for
   tools and constraints (E113)
5. Update SPEC.md Type Constraints section and Diagnostic Codes table
6. Add conformance tests for each new rule
7. Add example invalid fixtures for each new rule

### v2 contract commitment

By end of Track B:
- E103 covers agent, system, and user empty scalars
- E111–E113 are stable error codes with documented semantics
- All new rules are conformance-tested
- No previously-valid document is invalidated by Rule 1–4
  (all four rules reject inputs that were never meaningful)

---

## Track C — Live Editor Feedback

### Problem

The VS Code extension has syntax highlighting and formatter-on-save
(`DocumentFormattingEditProvider`). Errors only appear when the user
runs `llm_format validate` in the terminal. There are no inline
squiggles. This is the biggest ergonomic gap between `.llm` and
any modern language tool.

### What v2 changes

**No LSP required.** VS Code exposes a `DiagnosticCollection` API
that is completely independent of the Language Server Protocol. The
same spawn-and-parse pattern used for formatting works for
diagnostics.

**CLI addition: `validate --stdin`.**
`validate` gains a `--stdin` flag that reads the document from stdin
instead of a file path. This lets the extension pipe the current
editor buffer (which may not match the saved file) without writing
a temp file. The existing diagnostic output format
(`syntax error at LINE:COL: [ECODE] MESSAGE`) is already
machine-parseable. No new output format is needed.

**extension.js additions:**

```js
// 1. Create a diagnostic collection once at activate
const diagnostics = vscode.languages.createDiagnosticCollection('llm');

// 2. Register a debounced on-change listener
workspace.onDidChangeTextDocument(debounce((event) => {
    if (event.document.languageId !== 'llm') return;
    runValidation(event.document, diagnostics);
}, 300));

// 3. runValidation: spawn `llm_format validate --stdin`, pipe buffer,
//    parse stderr lines matching the diagnostic format, call
//    diagnostics.set(uri, [vscode.Diagnostic, ...])
```

The diagnostic line parser maps
`(syntax|semantic) error at LINE:COL: [ECODE] MESSAGE`
to `new vscode.Diagnostic(range, message, severity)`.

This gives **genuine as-you-type squiggles** using only the existing
CLI binary — no socket, no protocol, no language server process.

### Sequence

1. Add `--stdin` flag to `validate` CLI (`src/cli/validate.rs`);
   when set, read from `io::stdin()` instead of a file path;
   update CLI help text; add unit test
2. Add `DiagnosticCollection` and `onDidChangeTextDocument` listener
   to `editors/vscode/extension.js` with 300ms debounce
3. Implement `runValidation`: spawn `validate --stdin`, pipe buffer
   content, parse diagnostic lines, call `diagnostics.set()`
4. Handle edge cases: binary not found (clear diagnostics, show
   info message), empty document (clear diagnostics)
5. Update `editors/vscode/package.json` — no new fields needed
6. Add tests in `tests/validate.rs` for `--stdin` behavior
7. Update `editors/vscode/README.md` to document squiggles

### v2 contract commitment

By end of Track C:
- Parse and validation errors show as inline squiggles in VS Code
  as the user types (300ms debounce)
- `validate --stdin` is a stable CLI surface
- Behavior is documented in the VS Code README

Deferred beyond Track C:
- Hover-over diagnostic details (requires LSP `textDocument/hover`)
- Completion/autocomplete (requires LSP)
- Code actions / quick fixes (requires LSP)
- Diagnostic severity distinctions beyond error/warning (LSP feature)

---

## Track Sequencing

**Recommended order: B → C → A**

**Track B first.**
Pure Rust, no architectural design decisions, each rule is
independent and can be implemented in a single packet. Zero
risk of breaking the v1 contract. Delivers immediate value —
authors get better error messages on real mistakes. Strengthens
the validator before new providers rely on it.

**Track C second.**
Requires only one small CLI addition (`--stdin`) plus the
extension.js changes. Benefits from Track B's richer diagnostics
being immediately visible in the editor. The spawn-and-parse
pattern is already proven by the formatter. Two files, two packets.

**Track A last.**
Broadest surface: touches `src/provider.rs`, `src/transpile/shadow.rs`,
SPEC.md, the conformance suite, and the bench subsystem. Requires
the most design validation. Benefits from Track B's richer validator
running against Anthropic documents before the Anthropic profile is
declared stable.

---

## v2 Success Criteria

v2 is complete when:

- [ ] openai shadow encoding meaningfully differs from generic
      _(currently: same; v2 target: openai stays V0, anthropic uses V1)_
- [ ] anthropic profile is no longer unsupported
- [ ] at least 3 new semantic validation rules catch real mistakes
      (E103 extension, E111, E112, E113)
- [ ] inline squiggles work in VS Code for parse and validation errors
- [ ] `validate --stdin` is a stable CLI surface
- [ ] all new behavior is conformance-tested
- [ ] SPEC.md updated for V1 shadow encoding and new validation rules
- [ ] CHANGELOG.md v2 entry added
- [ ] docs/versioning.md v2 section added
