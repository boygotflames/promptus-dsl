# Compatibility Matrix

This matrix describes the current public-facing contract status for the v0 `.llm` slice.

It is intentionally narrower than the full internal test suite. The goal is to say, in public, what we are prepared to stand behind today without pretending every implemented behavior is frozen forever.

## Status Meanings

- `stable` — deterministic and treated as the current public v0 contract
- `provisional` — implemented and test-backed, but explicitly subject to revision
- `partial` — materially implemented, but not broad enough to claim as a fully closed contract surface
- `unsupported` — explicitly not supported and expected to fail cleanly

## Current Matrix

| Capability | Status | Notes | Contract Coverage |
| --- | --- | --- | --- |
| Surface syntax parser | `stable` | Reserved top-level keys, indentation-based mappings/sequences, comments, quoted/bare scalars, and deterministic failure on known invalid fixtures are the current public v0 syntax contract. | `tests/conformance.rs` |
| Semantic validator | `stable` | v1 contract frozen. See SPEC.md Validation Semantics. Additive-only constraints within v1. | `tests/conformance.rs` |
| Canonical formatter (`fmt`) | `stable` | Canonical 2-space indentation, reserved top-level ordering, source-preserved mapping order, and explicit quoting rules are treated as the current v0 formatting contract. | `tests/conformance.rs` |
| `transpile --target plain` | `stable` | The plain target is the canonical normalized surface rendering for the current v0 slice. Encoding rules and stability declaration are specified in [SPEC.md §Plain Output](../SPEC.md#plain-output). | `tests/conformance.rs` |
| `transpile --target json-ir` | `stable` | The JSON IR is deterministic and provider-agnostic for the current v0 slice. It is treated as a public contract for the presently implemented fields. Encoding rules and stability declaration are specified in [SPEC.md §JSON-IR Output](../SPEC.md#json-ir-output). | `tests/conformance.rs` |
| `transpile --target shadow` | `stable` | The shadow target is deterministic and provider-aware. The v0 shadow format is fully specified as a stable contract in [SPEC.md §Shadow Representation](../SPEC.md#shadow-representation) and covered by the conformance suite. | `tests/conformance.rs` |
| `bench` report | `stable (scope-limited)` | Report shape is stable. Tokenizer profiles (`cl100k_base` for `generic`/`openai`, `o200k_base` for `anthropic`) are stable. Exact output is conformance-locked for all primary fixtures across all three providers. CI integration added in v3. | `tests/conformance.rs` |
| Provider profile: `generic` | `stable` | Generic profile uses the V0 compact `@`-marker shadow encoding. Encoding is fully specified in [SPEC.md §Shadow Representation](../SPEC.md#shadow-representation). | `tests/conformance.rs` |
| Provider profile: `openai` | `stable` | openai profile uses the same V0 shadow encoding as `generic`. Identical output for all documents. | `tests/conformance.rs` |
| Provider profile: `anthropic` | `stable` | Anthropic profile uses the V1 XML-tag shadow encoding and `o200k_base` tokenizer (proxy for Anthropic's tokenizer). V1 format fully specified in [SPEC.md §V1 Anthropic Shadow Encoding](../SPEC.md#v1-anthropic-shadow-encoding). | `tests/conformance.rs` |
| VS Code syntax support | `stable (scope-limited)` | Syntax highlighting, file association, and formatter-on-save are stable. LSP, live validation, and completion are explicitly deferred to post-v1. | `tests/vscode.rs` |

## Boundary Notes

- The v2 contract adds: V1 Anthropic shadow encoding, `o200k_base` tokenizer profile for `anthropic`, and live VS Code diagnostics (`validate --stdin`).
- The v1 contract covers surface syntax, canonical formatting, `plain`, `json-ir`, `shadow` (V0), semantic validation, provider profiles (`generic`/`openai`), and VS Code syntax support.
- `shadow` V0 is stable since v0 (Packet 6): the format is fully specified in SPEC.md and backed by the conformance suite.
- `shadow` V1 (Anthropic XML-tag encoding) is stable as of v2: fully specified in SPEC.md and backed by conformance tests.
- Semantic validation contract is frozen at v1: new constraints affecting previously-valid documents require a version bump.
- VS Code LSP/completion are explicitly deferred to post-v2.
- `bench` is stable (scope-limited) as of v3: report shape is locked, tokenizer profiles are stable, and exact output is conformance-tested for all primary fixtures.
- Provider support is honest by design: explicit support where it exists, documented clearly.
- The compatibility matrix should evolve only when the implementation and conformance tests justify it.
