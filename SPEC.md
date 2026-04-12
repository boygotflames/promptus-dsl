# .llm Specification Draft (v0)

## Scope

This specification describes the v0 surface. It is implemented by the
reference transpiler in this repository. Behavior marked stable here
is covered by the conformance suite in tests/conformance.rs.

This draft defines the initial foundation for `.llm`, a human-readable prompt and configuration format with a machine-oriented shadow representation.

The v0 scope is intentionally small:

- a surface syntax for prompt/config authoring
- a normalized AST
- validation for the initial top-level keys
- pluggable transpilation targets

Provider-specific behavior, optimization passes, and execution semantics are out of scope for v0.

## Supported Top-Level Keys

The v0 parser recognizes exactly these top-level keys:

- `agent`
- `system`
- `user`
- `memory`
- `tools`
- `output`
- `constraints`
- `vars`
- `include`

The `include` key is a composition directive, not a prompt key.
It does not appear in transpiled output.

Unknown top-level keys are rejected.

## Surface Syntax

The current syntax is a small indentation-based DSL inspired by YAML, but it is not YAML-complete.

Rules:

- indentation uses spaces only
- nested blocks must increase indentation by exactly two spaces
- blank lines are ignored
- lines whose first non-space character is `#` are treated as comments
- mappings use `key: value` or `key:` followed by an indented block
- sequences use `- value`
- scalars may be bare text, single-quoted strings, or double-quoted strings

### Grammar Sketch

```text
document        := top_level_entry*
top_level_entry := key ":" scalar?
                 | key ":" newline block

block           := mapping | sequence
mapping         := indented_entry+
indented_entry  := key ":" scalar?
                 | key ":" newline block
sequence        := sequence_item+
sequence_item   := "- " scalar
                 | "-" newline block

key             := /[A-Za-z_][A-Za-z0-9_-]*/
scalar          := bare_scalar | single_quoted | double_quoted
bare_scalar     := any non-empty text after ":" or "- "
single_quoted   := "'" .* "'"
double_quoted   := "\"" .* "\""
```

## AST Model

The v0 AST has:

- a `Document` root with explicit fields for each supported top-level key
- a generic `Node` value model:
  - `Scalar(String)`
  - `Sequence(Vec<Node>)`
  - `Mapping(Vec<MappingEntry>)`
- a `MappingEntry` record for keyed children with their own spans
- every mapping stores `Vec<MappingEntry>` children rather than a raw map
- every `Node` carries a source `Span` with line and column information

The parser accepts a generic tree shape first. The validator then applies key-specific constraints.

## Canonical Formatting

The v0 formatter emits an AST-normalized surface form.

- top-level blocks follow the reserved key order in the `Document`
- mapping entries preserve AST/source insertion order
- indentation is always 2 spaces per nesting level
- scalars remain bare only when they are non-empty and contain only ASCII letters, digits, `_`, or `-`
- all other scalars are emitted as double-quoted strings with backslash escaping
- comments and blank-line layout are not preserved by the current formatter

## Validation Semantics

### Required Keys

The following top-level keys are required in every valid `.llm` document:

- `agent` — every document must declare an agent identity
- `system` — every document must provide system-level instructions

All other top-level keys are optional. A document containing only
`agent` and `system` is valid.

### Type Constraints

Scalar values that are syntactically present but empty (zero-length
after trimming) are treated as validation errors wherever the spec
requires a non-empty scalar.

Current validation rules:

- `agent` must be a non-empty scalar
- `system` and `user` must be either a scalar or a mapping
- `memory`, `tools`, and `constraints` must be sequences of
  non-empty scalar values
- `output` must be either a scalar or a mapping
- `vars` must be a mapping whose values are non-empty scalars
- mapping keys must be unique within each mapping block

Key name grammar (enforced at parse time, applies to all mapping keys
including `vars`, `system`, `user`, and `output` sub-keys):

- All mapping keys must match `/[A-Za-z_][A-Za-z0-9_-]*/`; keys
  that violate this grammar are rejected by the lexer (E005) before
  they reach the validator. The validator adds no further key-name
  constraints at v0.

Mapping key behavior per top-level block:

- `vars` mapping keys: any key matching the key grammar is accepted;
  no reserved key set is enforced at v0. Key grammar is enforced by
  the parser, not the validator.
- `system` and `user` mapping values: any keys matching the key
  grammar are accepted; no reserved key set is enforced at v0.
- `output` mapping values: any keys matching the key grammar are
  accepted; no reserved key set is enforced at v0.

Additional validation rules added in v2:

- `system` and `user` scalar values must not be empty (zero-length
  after trimming) — E103
- A `system`, `user`, or `output` mapping block must contain at
  least one entry. An empty mapping block is a validation error — E111
- `memory`, `tools`, and `constraints` sequences must contain at
  least one item. An empty sequence is a validation error — E112
- `tools` and `constraints` sequences must not contain duplicate
  items (case-sensitive comparison). Duplicate items are a
  validation error — E113. `memory` is exempt from this rule.

The constraints listed above constitute the complete v1 validation contract. New constraints affecting previously-valid documents require a version bump. Constraints may be added for keys that are currently unconstrained only if the document remains valid under the new rule (i.e., additive-only within v1).

### Diagnostic Codes

Every diagnostic emitted by the parser or validator carries a structured
error code. Codes are stable identifiers for specific error conditions,
intended for programmatic use, editor integration, and documentation.

Code format: `E` followed by a 3-digit number.

- `E001`–`E099`: parser and lexer errors (syntax phase)
- `E101`–`E199`: validator errors (semantic phase)

The code is exposed as the `code: Option<&'static str>` field on the
`Diagnostic` type and is included in the human-readable display format
as a `[E001]` prefix before the message text when a code is present.

#### Parser and Lexer Codes

| Code | Description |
|---|---|
| `E001` | tabs are not allowed; use two-space indentation |
| `E002` | indentation must be a multiple of two spaces |
| `E003` | expected a space after `-` in a list item |
| `E004` | expected a mapping entry or list item |
| `E005` | expected an identifier at the start of a mapping entry |
| `E006` | expected `:` after mapping key |
| `E007` | expected a scalar value |
| `E008` | unexpected trailing characters after quoted scalar |
| `E009` | unterminated escape sequence in quoted scalar |
| `E010` | unsupported escape sequence |
| `E011` | unterminated quoted scalar |
| `E012` | top-level key must start at column 1 |
| `E013` | unknown top-level key |
| `E014` | duplicate top-level key |
| `E015` | unexpected end of input while expecting a top-level entry |
| `E016` | top-level entry must be a mapping, not a list item |
| `E017` | unexpected end of input after `:` |
| `E018` | wrong indentation level |
| `E019` | unexpected indentation inside a sequence |
| `E020` | list item found where a mapping entry was expected |
| `E021` | unexpected end of input inside a mapping |
| `E022` | expected a mapping entry but found a list item |
| `E023` | expected an indented block after `:` |
| `E024` | nested block indented by wrong number of spaces |

#### Validator Codes

| Code | Description |
|---|---|
| `E101` | missing required key (message carries the specific key name) |
| `E102` | duplicate key in mapping |
| `E103` | scalar field value must not be empty |
| `E104` | field must be a scalar value |
| `E105` | field must be a scalar or mapping (`system`, `user`) |
| `E106` | field must be a sequence |
| `E107` | sequence field may only contain scalar items |
| `E108` | `output` must be a scalar or mapping |
| `E109` | `vars` must be a mapping |
| `E110` | `vars` entry must be a scalar value |
| `E111` | mapping block has no entries |
| `E112` | sequence has no items |
| `E113` | duplicate item in sequence |
| `E114` | undefined var reference: `{name}` is not defined in `vars` |
| `E115` | invalid `include` path (absolute path or empty path) |
| `E116` | circular include detected |
| `E117` | key conflict during include merge: scalar key defined in both files |

Diagnostic codes are part of the stable public contract. Once assigned,
a code's meaning does not change. New codes may be added; existing codes
are never reused for a different error.

## Public Contract Status

The current public v0 boundary is intentionally split:

- `stable`
  - surface syntax
  - canonical formatting
  - `plain` output — see [Plain Output](#plain-output)
  - `json-ir` output — see [JSON-IR Output](#json-ir-output)
  - `shadow` output — see [Shadow Representation](#shadow-representation)
- `stable (scope-limited)` — at v1; post-v1 expansion is explicitly deferred
  - semantic validation (v1 contract frozen; see §Validation Semantics)
  - provider profiles: `generic` and `openai` (differentiation deferred post-v1)
  - VS Code syntax support (LSP/completion deferred post-v1)
- `provisional`
  - `bench` report behavior
- `unsupported`
  - provider profiles without an explicit supported tokenizer/shadow path

The public compatibility summary lives in [docs/compatibility-matrix.md](docs/compatibility-matrix.md). The current public contract is backed by a dedicated conformance layer in [tests/conformance.rs](tests/conformance.rs).

## vars Expansion

### Template References

Scalar values in `system`, `user`, `output`, `memory`, `tools`, and
`constraints` may contain template references of the form `{var_name}`,
where `var_name` matches the key grammar `/[A-Za-z_][A-Za-z0-9_-]*/`.

Template references must appear inside double-quoted strings on the
surface layer — the bare scalar charset excludes `{`, so `{var}` cannot
appear in an unquoted value. The canonical formatter preserves `{var_name}`
verbatim without modification.

`vars` values themselves do not support template references. Expansion
is non-recursive: a substituted value is not scanned for further `{var}`
references.

### Expansion Behavior

Template expansion occurs at **transpile time**, not at parse time or
format time.

- The `fmt` formatter preserves `{var_name}` verbatim. Source files
  remain editable with references intact.
- The `plain`, `json-ir`, and `shadow` transpile targets substitute
  each `{var_name}` with the corresponding value from the `vars` mapping
  before emitting.
- If a `vars` mapping is absent, no substitution occurs and references
  are passed through verbatim.
- Expansion is non-recursive: substituted values are not scanned for
  further `{var}` references.
- For documents without `{var}` references, `plain` output is identical
  to `fmt` output.

### Validation

The validator checks for undefined var references:

- If a scalar value contains `{var_name}` and no `vars` key named
  `var_name` is defined, a validation error is emitted with code E114.
- One E114 is emitted per unique undefined reference name per scalar node.
- If no `vars` block exists and no references appear, no error is emitted.
- If no `vars` block exists but references do appear, E114 is emitted
  for each undefined reference.

### Stability

The vars expansion behavior is **stable** as of v3. The `{var_name}`
reference syntax is stable. The expansion-at-transpile-time contract
is stable. Breaking changes require a version bump and a CHANGELOG entry.

## Multi-file Includes

### Surface Syntax

The `include` key accepts a scalar (single file) or a sequence
(multiple files):

```text
include: shared/base-system.llm

# or multiple files:
include:
  - shared/base-system.llm
  - shared/common-vars.llm
```

Paths are relative to the directory of the including file.
Absolute paths are not supported. `include` as a mapping is not
supported.

### Merge Semantics

When an included file is merged into the parent document:

- **Scalar keys** (`agent`, `system`, `user`, `output`): conflict
  is a validation error (E117) if both parent and included file
  declare the same key. The parent always wins if the key appears
  in both — but the error is still emitted unless suppressed.
- **Sequence keys** (`memory`, `tools`, `constraints`): sequences
  are concatenated. `tools` and `constraints` items are deduplicated
  after merge (first occurrence wins). `memory` items are not
  deduplicated.
- **`vars`**: merged with parent winning on key conflict. No error
  on duplicate var keys — parent silently overrides.
- **`include`**: the `include` key itself is consumed during
  composition and never appears in the merged document or any
  transpiled output.

### var Expansion Across Boundaries

Each file expands using its own vars only. After merge, the composed
document's unified vars map (parent wins) is used for final expansion.

### Circular Include Detection

Circular includes are a validation error (E116). Detection uses the
inclusion chain (stack of file paths currently being resolved). If a
file appears twice in the chain, E116 is emitted and resolution stops.

### Diagnostic Codes

| `E115` | validator | invalid include path (absolute, empty, or missing) |
| `E116` | validator | circular include detected |
| `E117` | validator | key conflict: scalar key defined in both parent and included file |

### Stability

Multi-file includes are **stable** as of v4.

## Shadow Representation

The v0 transpiler emits a Shadow target intended as a compact,
deterministic, machine-facing representation of a `.llm` document.

This specification describes the `v0` shadow format. It is implemented
by `src/transpile/shadow.rs` and covered by the conformance suite in
`tests/conformance.rs`. The format is stable as of this document.

### Marker Table

Each supported top-level key maps to a two-character reserved marker:

| Key | Marker | Value encoding |
|---|---|---|
| `agent` | `@a` | `="<scalar>"` |
| `system` | `@s` | scalar: `="<value>"` / mapping: `={key="val";...}` |
| `user` | `@u` | scalar: `="<value>"` / mapping: `={key="val";...}` |
| `memory` | `@m` | `=["item1","item2",...]` |
| `tools` | `@t` | `=["item1","item2",...]` |
| `output` | `@o` | scalar: `="<value>"` / mapping: `={key="val";...}` |
| `constraints` | `@c` | `=["item1","item2",...]` |
| `vars` | `@v` | `={key="val";...}` |

### Encoding Rules

1. **Scalar values** are always double-quoted: `"value"`. The following
   characters are backslash-escaped inside quoted scalars: `\`, `"`,
   newline (`\n`), carriage return (`\r`), tab (`\t`).
2. **Mapping values** use `{` `}` delimiters with `;`-separated
   `key="value"` pairs. Mapping keys are emitted bare (unquoted).
   Mapping values are recursively encoded.
3. **Sequence values** use `[` `]` delimiters with `,`-separated
   encoded items. Sequence items are recursively encoded.
4. **Absent keys** are omitted entirely — no empty marker is emitted
   for a top-level key that is not present in the document.
5. **Key ordering** follows the Document field order defined in the AST:
   `agent`, `system`, `user`, `memory`, `tools`, `output`,
   `constraints`, `vars`. Source insertion order does not affect output
   order.
6. **Each marker** appears on its own line. Lines are joined with `\n`
   (no trailing newline).

### Provider Profiles

The shadow emitter accepts a provider profile parameter:

- `generic` — default; uses the V0 shadow encoding described above
- `openai` — uses the same V0 shadow encoding as `generic`
- `anthropic` — uses the V1 shadow encoding described below

### V1 Anthropic Shadow Encoding

The `anthropic` provider uses an XML-tag shadow encoding. Anthropic models
process `<system>`, `<user>`, and similar XML tags natively because Claude's
training data uses these patterns. The V0 `@`-marker format is OpenAI-neutral
but not Claude-optimized; V1 replaces markers with named XML tags.

**Tag table:**

| Key | XML tag | Item tag |
|---|---|---|
| `agent` | `<agent>` | — |
| `system` | `<system>` | — |
| `user` | `<user>` | — |
| `memory` | `<memory>` | `<item>` |
| `tools` | `<tools>` | `<tool>` |
| `output` | `<output>` | — |
| `constraints` | `<constraints>` | `<rule>` |
| `vars` | `<vars>` | — |

**Encoding rules:**

1. **Scalar key** (`agent`): `<agent>value</agent>` on a single line.
   Value is emitted bare (no quoting).
2. **Scalar prompt** (`system`, `user`, `output` when scalar):
   `<system>value</system>` on a single line.
3. **Mapping prompt** (`system`, `user`, `output` when mapping):
   ```
   <system>
   key1: val1
   key2: val2
   </system>
   ```
   Entries are `key: value` lines, one per entry, no indentation inside
   the tag.
4. **Sequence** (`memory`, `tools`, `constraints`):
   ```
   <tools>
   <tool>web_search</tool>
   <tool>calculator</tool>
   </tools>
   ```
   Each item is wrapped in the key-specific item tag on its own line.
5. **Vars** (mapping of scalars):
   ```
   <vars>
   region: apac
   currency: usd
   </vars>
   ```
   Entries are `key: value` lines, one per entry, no indentation.
6. **Absent keys** are omitted entirely — same rule as V0.
7. **Key ordering** follows Document field order — same rule as V0.
8. **Blocks are joined** with `\n` (no trailing newline) — same rule as V0.

**Example** (minimal.llm with `--provider anthropic`):

```text
<agent>DataExtractor</agent>
<system>
role: financial_analyst
output: json
</system>
<memory>
<item>user_history</item>
</memory>
```

**Stability:** The V1 Anthropic shadow format is **stable** as of v2.
The same `.llm` input with `--provider anthropic` will always produce
the same V1 shadow output. Breaking changes require a version bump and
a CHANGELOG entry.

### V0 Stability

The V0 shadow format described in the Marker Table and Encoding Rules
above is **stable** as of v0. `generic` and `openai` both use V0.

The same `.llm` input will always produce the same shadow output.
Downstream tooling may depend on this format. Breaking changes to
the shadow format require a version bump and a CHANGELOG entry.

### V0 Example

```text
@a="DataExtractor"
@s={role="financial_analyst";output="json"}
@m=["user_history"]
```

## Example

```text
agent: DataExtractor
system:
  role: financial_analyst
  output: json
memory:
  - user_history
```

## Plain Output

The `plain` target emits the canonical formatter output for a document.
Its encoding is defined as identical to the output of the `fmt` command.
See [Canonical Formatting](#canonical-formatting) for the full encoding
rules.

Key properties:

- Top-level keys follow Document field order: `agent`, `system`, `user`,
  `memory`, `tools`, `output`, `constraints`, `vars`
- Absent keys are omitted entirely
- Mapping entries preserve source insertion order
- Indentation is always 2 spaces per nesting level
- Scalars remain bare only when they are non-empty and contain only ASCII
  letters, digits, `_`, or `-`; all other scalars are double-quoted with
  backslash escaping
- Comments and blank-line layout are not preserved
- Lines are joined with `\n`; there is no trailing newline

The provider parameter is ignored for `plain` output.

### Stability

The `plain` format described in this section is **stable** as of v0.
The same `.llm` input will always produce the same `plain` output.
Downstream tooling may depend on this format. Breaking changes require
a version bump and a CHANGELOG entry.

### Example

Input:

```text
agent: DataExtractor
system:
  role: financial_analyst
  output: json
memory:
  - user_history
```

Output:

```text
agent: DataExtractor
system:
  role: financial_analyst
  output: json
memory:
  - user_history
```

---

## JSON-IR Output

The `json-ir` target emits a JSON object representation of the document.
It is intended as a stable intermediate representation for programmatic
consumers that prefer structured JSON over the surface DSL syntax.

Key properties:

- The top-level output is a JSON object `{...}`
- Top-level keys follow Document field order: `agent`, `system`, `user`,
  `memory`, `tools`, `output`, `constraints`, `vars`
- Absent keys are omitted entirely — no `null`-valued key is emitted
- Mapping entries preserve source insertion order
- Indentation is 2 spaces per nesting level
- All object keys are double-quoted strings
- Scalar values are double-quoted strings; the following characters are
  backslash-escaped inside quoted strings: `\`, `"`, newline (`\n`),
  carriage return (`\r`), tab (`\t`)
- Sequence values use JSON array syntax `[...]` with each item on its own
  indented line; a comma appears after each item except the last
- Mapping values use JSON object syntax `{...}` with each entry on its
  own indented line; a comma appears after each entry except the last
- An empty sequence emits `[]` on a single line
- An empty mapping emits `{}` on a single line
- Lines are joined with `\n`; there is no trailing newline

The provider parameter is ignored for `json-ir` output.

### Stability

The `json-ir` format described in this section is **stable** as of v0.
The same `.llm` input will always produce the same `json-ir` output.
Downstream tooling may depend on this format. Breaking changes require
a version bump and a CHANGELOG entry.

### Example

Input:

```text
agent: DataExtractor
system:
  role: financial_analyst
  output: json
memory:
  - user_history
```

Output:

```json
{
  "agent": "DataExtractor",
  "system": {
    "role": "financial_analyst",
    "output": "json"
  },
  "memory": [
    "user_history"
  ]
}
```

---

## Deferred Work

The following are explicitly out of scope for v0. They are recorded here
so readers know they are intentionally deferred, not overlooked.

- **Richer scalar typing** — Deferred to post-v0. Current scalars are
  untyped strings. Numeric, boolean, and date types are not part of the
  v0 contract.
- **Richer list item structures** — Deferred to post-v0. Sequence items
  are currently scalars only (or nested blocks). Structured list items
  with typed sub-fields are not part of the v0 contract.
- **Semantic normalization passes — interpolation/vars expansion** —
  Implemented in v3. `{var_name}` references in scalar values are
  expanded at transpile time across `plain`, `json-ir`, and `shadow`
  targets. See [vars Expansion](#vars-expansion). Remaining deferred
  items: value normalization (trimming, casing) and derived fields
  (computed from other fields) are deferred to post-v3.
- **Canonical formatter** — Implemented in v0. The `fmt` CLI command and
  `src/formatter.rs` provide canonical AST-based formatting with
  deterministic scalar quoting and 2-space indentation. See
  [Canonical Formatting](#canonical-formatting) above.
- **Provider-specific emission layers** — Implemented in v2. `generic`
  and `openai` use the V0 compact `@`-marker shadow encoding.
  `anthropic` uses the V1 XML-tag shadow encoding with the `o200k_base`
  tokenizer profile. See [Provider Profiles](#provider-profiles) and
  [V1 Anthropic Shadow Encoding](#v1-anthropic-shadow-encoding).
- **Includes/imports and multi-file composition** — Deferred to post-v0.
  Each `.llm` document is a standalone unit at v0. Cross-file references
  are not part of the v0 contract.
