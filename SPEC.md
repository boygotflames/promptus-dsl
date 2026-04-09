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

All other top-level keys are optional. A document containing only
`agent` is valid.

### Type Constraints

Current validation rules:

- `agent` must be a non-empty scalar
- `system` and `user` must be either a scalar or a mapping
- `memory`, `tools`, and `constraints` must be sequences of scalar values
- `output` must be either a scalar or a mapping
- `vars` must be a mapping whose values are scalars
- mapping keys must be unique within each mapping block

These constraints are intentionally conservative and may expand in later versions.

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
| `E101` | missing required key: `agent` |
| `E102` | duplicate key in mapping |
| `E103` | scalar field value must not be empty |
| `E104` | field must be a scalar value |
| `E105` | field must be a scalar or mapping (`system`, `user`) |
| `E106` | field must be a sequence |
| `E107` | sequence field may only contain scalar items |
| `E108` | `output` must be a scalar or mapping |
| `E109` | `vars` must be a mapping |
| `E110` | `vars` entry must be a scalar value |

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
- `partial`
  - current semantic validation breadth
  - provider-selection plumbing
  - minimal VS Code syntax support
- `provisional`
  - `bench` report behavior
- `unsupported`
  - provider profiles without an explicit supported tokenizer/shadow path

The public compatibility summary lives in [docs/compatibility-matrix.md](docs/compatibility-matrix.md). The current public contract is backed by a dedicated conformance layer in [tests/conformance.rs](tests/conformance.rs).

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

- `generic` — default; uses the v0 shadow encoding described above
- `openai` — currently maps to the same v0 shadow encoding as `generic`
- other profiles — fail explicitly with a descriptive error; no silent
  fallback

Provider profiles are a plumbing hook for future differentiation.
At v0, `generic` and `openai` produce identical output.

### Stability

The shadow format described in this section is **stable** as of v0.

The same `.llm` input will always produce the same shadow output.
Downstream tooling may depend on this format. Breaking changes to
the shadow format require a version bump and a CHANGELOG entry.

### Example

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
- **Semantic normalization passes** — Deferred to post-v0. The current
  validator enforces structure constraints only. Value normalization,
  interpolation, and derived fields are not part of the v0 contract.
- **Canonical formatter** — Implemented in v0. The `fmt` CLI command and
  `src/formatter.rs` provide canonical AST-based formatting with
  deterministic scalar quoting and 2-space indentation. See
  [Canonical Formatting](#canonical-formatting) above.
- **Provider-specific emission layers** — Partial in v0. The `generic`
  and `openai` profiles exist and currently share the same v0 shadow
  encoding. Real per-provider divergence is deferred to post-v0.
- **Includes/imports and multi-file composition** — Deferred to post-v0.
  Each `.llm` document is a standalone unit at v0. Cross-file references
  are not part of the v0 contract.
