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
| Semantic validator | `partial` | Current conservative validation rules are implemented and deterministic, but the full semantic language contract is still intentionally narrow. | `tests/conformance.rs` |
| Canonical formatter (`fmt`) | `stable` | Canonical 2-space indentation, reserved top-level ordering, source-preserved mapping order, and explicit quoting rules are treated as the current v0 formatting contract. | `tests/conformance.rs` |
| `transpile --target plain` | `stable` | The plain target is the canonical normalized surface rendering for the current v0 slice. Encoding rules and stability declaration are specified in [SPEC.md §Plain Output](../SPEC.md#plain-output). | `tests/conformance.rs` |
| `transpile --target json-ir` | `stable` | The JSON IR is deterministic and provider-agnostic for the current v0 slice. It is treated as a public contract for the presently implemented fields. Encoding rules and stability declaration are specified in [SPEC.md §JSON-IR Output](../SPEC.md#json-ir-output). | `tests/conformance.rs` |
| `transpile --target shadow` | `stable` | The shadow target is deterministic and provider-aware. The v0 shadow format is fully specified as a stable contract in [SPEC.md §Shadow Representation](../SPEC.md#shadow-representation) and covered by the conformance suite. | `tests/conformance.rs` |
| `bench` report | `provisional` | Bench output is deterministic for supported providers, but it remains tied to the current tokenizer/profile path and may evolve as benchmark discipline grows. | `tests/conformance.rs` |
| Provider profile: `generic` | `partial` | Supported for current `shadow` and `bench` flows. The profile exists, but provider-specific divergence is still intentionally limited. | `tests/conformance.rs` |
| Provider profile: `openai` | `partial` | Explicitly selectable and supported, but currently shares the same v0 shadow encoding and tokenizer path as `generic`. | `tests/conformance.rs` |
| Provider profile: `anthropic` | `unsupported` | Current provider-aware flows must fail explicitly rather than silently falling back. | `tests/conformance.rs` |
| VS Code syntax support | `partial` | File association, syntax highlighting, and basic language configuration exist. LSP, live diagnostics, completion, and formatter integration do not. | `tests/vscode.rs` |

## Boundary Notes

- The stable v0 contract covers surface syntax, canonical formatting, `plain`, `json-ir`, and `shadow`.
- `shadow` is now stable: the v0 format is fully specified in SPEC.md and backed by the conformance suite.
- `bench` remains provisional because its report shape is still tied to a single tokenizer path and may evolve.
- Provider support is honest by design: explicit support where it exists, explicit failure where it does not.
- The compatibility matrix should evolve only when the implementation and conformance tests justify it.
