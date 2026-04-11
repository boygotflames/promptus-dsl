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
| `bench` report | `provisional` | Bench output is deterministic for supported providers, but it remains tied to the current tokenizer/profile path and may evolve as benchmark discipline grows. | `tests/conformance.rs` |
| Provider profile: `generic` | `stable (scope-limited)` | Generic profile is stable. Provider-specific divergence (different shadow encoding per provider) is explicitly deferred to post-v1. At v1, all profiles share the v0 shadow encoding. | `tests/conformance.rs` |
| Provider profile: `openai` | `stable (scope-limited)` | openai profile is stable as an alias for generic at v1. Real provider differentiation (tokenizer plurality, distinct shadow encoding) is explicitly deferred to post-v1. | `tests/conformance.rs` |
| Provider profile: `anthropic` | `unsupported` | Current provider-aware flows must fail explicitly rather than silently falling back. | `tests/conformance.rs` |
| VS Code syntax support | `stable (scope-limited)` | Syntax highlighting, file association, and formatter-on-save are stable. LSP, live validation, and completion are explicitly deferred to post-v1. | `tests/vscode.rs` |

## Boundary Notes

- The v1 contract covers surface syntax, canonical formatting, `plain`, `json-ir`, `shadow`, semantic validation, provider profiles (`generic`/`openai`), and VS Code syntax support.
- `shadow` is stable since v0 (Packet 6): the format is fully specified in SPEC.md and backed by the conformance suite.
- Semantic validation contract is frozen at v1: new constraints affecting previously-valid documents require a version bump.
- Provider differentiation (distinct shadow encoding per provider, tokenizer plurality) and VS Code LSP/completion are explicitly deferred to post-v1.
- `bench` remains provisional because its report shape is still tied to a single tokenizer path and may evolve.
- Provider support is honest by design: explicit support where it exists, explicit failure where it does not.
- The compatibility matrix should evolve only when the implementation and conformance tests justify it.
