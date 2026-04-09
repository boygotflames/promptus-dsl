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

Current validation rules:

- `agent` must be a non-empty scalar
- `system` and `user` must be either a scalar or a mapping
- `memory`, `tools`, and `constraints` must be sequences of scalar values
- `output` must be either a scalar or a mapping
- `vars` must be a mapping whose values are scalars
- mapping keys must be unique within each mapping block

These constraints are intentionally conservative and may expand in later versions.

## Public Contract Status

The current public v0 boundary is intentionally split:

- `stable`
  - surface syntax
  - canonical formatting
  - `plain` output
  - `json-ir` output
- `partial`
  - current semantic validation breadth
  - provider-selection plumbing
  - minimal VS Code syntax support
- `provisional`
  - `shadow` output
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
  and `openai` profiles exist and share a single provisional shadow
  mapping. Real per-provider divergence is deferred to post-v0.
- **Includes/imports and multi-file composition** — Deferred to post-v0.
  Each `.llm` document is a standalone unit at v0. Cross-file references
  are not part of the v0 contract.
