# .llm Specification Draft (v0)

## Scope

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
scalar          := any non-empty text after ":" or "- "
```

## AST Model

The v0 AST has:

- a `Document` root with explicit fields for each supported top-level key
- a generic `Node` value model:
  - `Scalar(String)`
  - `Sequence(Vec<Node>)`
- a `MappingEntry` record for keyed children with their own spans
- every mapping stores `Vec<MappingEntry>` children rather than a raw map
- every `Node` carries a source `Span` with line and column information

The parser accepts a generic tree shape first. The validator then applies key-specific constraints.

## Validation Semantics

Current validation rules:

- `agent` must be a non-empty scalar
- `system` and `user` must be either a scalar or a mapping
- `memory`, `tools`, and `constraints` must be sequences of scalar values
- `output` must be either a scalar or a mapping
- `vars` must be a mapping whose values are scalars
- mapping keys must be unique within each mapping block

These constraints are intentionally conservative and may expand in later versions.

## Shadow Representation

The shadow layer is a normalized path-based projection of the AST. Example:

```text
agent = "DataExtractor"
system.role = "financial_analyst"
system.output = "json"
memory[0] = "user_history"
```

This representation is intended to be deterministic and easy for downstream tooling to consume.

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

- richer scalar typing
- richer list item structures
- semantic normalization passes
- formatter and editor tooling
- provider-specific emission layers
