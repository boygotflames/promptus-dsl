# Changelog

All notable changes to the `.llm` transpiler are documented here.

This project is at v0. Until v1, entries track meaningful milestones
rather than semantic versioning increments. Breaking changes to stable
surfaces are always noted explicitly.

---

## [v0] — Active

### Stable surface (no breaking changes)
- Surface syntax: indentation-based DSL with 8 reserved top-level keys
- Canonical formatter: 2-space indentation, deterministic scalar quoting,
  key ordering follows Document field order
- `plain` output: deterministic, order-stable, whitespace-normalized
- `json-ir` output: deterministic JSON intermediate representation

### Implemented and usable (partial)
- Semantic validation: duplicate key detection, top-level shape rules,
  type constraints per key
- Provider selection: `generic` (default) and `openai` (explicit);
  `anthropic` is explicitly unsupported
- VS Code support: file association, syntax highlighting, language config

### Provisional (format may change)
- `shadow` output: compact machine-facing representation; deterministic
  within the current implementation but not a frozen public contract
- `bench` report: token counting and baseline comparison; report shape
  subject to revision

### Infrastructure
- span-aware AST with deterministic diagnostics
- CLI: `parse`, `validate`, `transpile`, `fmt`, `bench`
- conformance test suite covering the stable contract surface
- public compatibility matrix at docs/compatibility-matrix.md

---

## Notable Changes

### Phase 9 kickoff (standardization prep)
- Added root README.md
- Added docs/compatibility-matrix.md
- Added tests/conformance.rs
- Established stable/provisional/partial/unsupported boundary language
- Removed empty scaffold residues (src/tokenizer.rs, src/transpiler.rs)
- Added CONTRIBUTING.md, docs/versioning.md, CHANGELOG.md
