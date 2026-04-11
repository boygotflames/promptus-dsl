# Versioning Strategy

## Current Version

This project is at **v0**.

v0 is an intentional designation, not a placeholder. It means:

- the core language surface is defined and implemented
- the public contract has an explicit stable boundary
- the project is not yet making cross-version compatibility guarantees
  beyond what is marked `stable` in the compatibility matrix
- the format and tooling are production-usable for controlled internal
  workflows
- the project is not yet frozen as a public standard

---

## What v0 Means for Stability

The public contract at v0 is intentionally split. See
[compatibility-matrix.md](compatibility-matrix.md) for the
full matrix.

| Surface | Status | Commitment |
|---|---|---|
| Surface syntax | stable | No breaking changes without a version bump |
| Canonical formatter behavior | stable | No breaking changes without a version bump |
| `plain` output | stable | No breaking changes without a version bump |
| `json-ir` output | stable | No breaking changes without a version bump |
| Semantic validation breadth | partial | May expand; existing valid inputs stay valid |
| Provider selection plumbing | partial | Interface stable; behavior may expand |
| VS Code support | partial | May improve; no compatibility promise |
| `shadow` output | stable | Stable as of v0 (Packet 6); see SPEC.md §Shadow Representation |
| `bench` report shape | provisional | Format may change without notice |
| `anthropic` provider profile | unsupported | Not implemented |

**Stable** means: the same `.llm` input will always produce the same
output for that target. Breaking changes to stable surfaces require a
version bump and a changelog entry.

**Provisional** means: deterministic within a session, but the format
itself may be revised as the spec matures. Do not build downstream
tooling that depends on provisional output format.

**Partial** means: implemented and usable, but the surface is not fully
closed against the original phase ambition. It may expand.

**Unsupported** means: explicitly not implemented. Attempting to use
it will fail with an explicit error, not a silent fallback.

---

## Version Bump Criteria

A version bump from v0 to v1 requires:

- all current `partial` surfaces resolved or explicitly deferred with
  documented rationale
- `shadow` output promoted from `provisional` to `stable` or explicitly
  deferred
- SPEC.md frozen against the v1 surface
- conformance suite coverage matching the full stable surface
- a changelog entry documenting all changes from v0

There is no timeline commitment for v1. The project will advance when
the criteria are met, not on a schedule.

---

### v1 Status

All v1 criteria are met as of this commit:

1. ✓ Partial surfaces resolved or explicitly deferred:
   - Semantic validation → stable (contract frozen in SPEC.md)
   - Provider generic/openai → stable (scope-limited; differentiation
     deferred post-v1)
   - VS Code support → stable (scope-limited; LSP deferred post-v1)
2. ✓ Shadow output stable (since Packet 6)
3. ✓ SPEC.md frozen against v1 surface
4. ✓ Conformance suite covers all stable surface behaviors
5. ✓ CHANGELOG.md documents v0→v1 changes

---

## Compatibility Guarantee at v0

Within v0:

- inputs that are valid today will remain valid
- stable outputs will not change
- provisional outputs may change
- new top-level keys will not be added without a SPEC.md update
- the CLI interface for `parse`, `validate`, `transpile`, `fmt`, and
  `bench` will not change incompatibly

---

## Changelog

See [CHANGELOG.md](../CHANGELOG.md) for a record of notable changes.
