Development Plan for the .llm Transpiler

This roadmap tracks the phases of the .llm project. Status markers like [COMPLETED] and [IN PROGRESS] reflect our current progress and will be updated as we advance.

Phase 0 — Scaffold and Foundation [COMPLETED]

Goals: Establish a minimal yet production‑grade project scaffold.

Deliverables:

Create initial Rust crate with Cargo.toml and .gitignore.
Establish directory structure (src/, examples/, tests/, docs/).
Add placeholder modules (ast.rs, parser.rs, validator.rs, diagnostics.rs, transpile/, cli/, bench/).
Draft initial SPEC.md and provide examples.
Ensure the project compiles and tests run.

Status: Completed (see Codex report for details). The foundation includes AST definitions, diagnostics scaffolding, CLI skeleton, basic examples and tests, but no full parser or validator logic.

Phase 1 — Parser and AST [IN PROGRESS]

Goals: Turn the Surface .llm DSL into a stable abstract syntax tree.

Tasks:

Implement a lexer using logos or a custom iterator.
Implement a deterministic parser handling scalar values, nested maps, lists and indentation. Support top‑level blocks (agent, system, user, memory, tools, output, constraints, vars) and nested structures.
Track source spans (line and column) for all AST nodes to facilitate precise diagnostics.
Produce clear syntax errors with contextual messages.
Create golden tests for valid and invalid .llm samples to ensure the parser is robust.
Update SPEC.md as grammar details solidify.

Success Criteria: The parser can round‑trip sample files into AST snapshots and the tests pass.

Phase 2 — Semantic Validation [PENDING]

Goals: Enforce semantic correctness and provide meaningful diagnostics.

Tasks:

Implement a validation layer that checks for duplicate keys, unknown blocks and required fields.
Validate lists, maps and scalars according to the spec.
Define and implement error codes and messages.
Phase 3 — Transpiler Targets [PENDING]

Goals: Generate useful outputs from the AST.

Tasks:

Define a TargetEmitter trait and implement PlainEmitter, ShadowEmitter and JsonIrEmitter.
Make target encoding pluggable to allow provider‑specific Shadow encodings.
Ensure deterministic output for each target.
Phase 4 — Command‑Line Interface [PENDING]

Goals: Provide a user‑friendly CLI.

Tasks:

Implement commands: parse, validate, transpile, bench, fmt, explain-error.
Ensure commands integrate with parser, validator and transpiler modules.
Provide helpful usage messages and handle file inputs/outputs.
Phase 5 — Benchmarking and Proof [PENDING]

Goals: Validate performance and token reductions.

Tasks:

Abstract over tokenizer implementations (e.g., tiktoken).
Build fixtures for comparing .llm files against Markdown and other formats.
Generate reports on token counts, byte sizes and compile times.
Phase 6 — Formatter and Language Ergonomics [PENDING]

Goals: Improve authoring experience.

Tasks:

Implement a formatter that normalizes .llm files according to canonical style rules.
Add syntax highlighting grammar for editors.
Provide example library demonstrating best practices.
Phase 7 — Editor Support [PENDING]

Goals: Increase adoption.

Tasks:

Develop a VS Code extension offering syntax highlighting, validation on save and transpilation previews.
Explore integrations with other editors and IDEs.
Phase 8 — Provider‑Aware Adapters [PENDING]

Goals: Optimize output across model providers.

Tasks:

Implement provider adapters for OpenAI, Anthropic and others.
Make token optimization tables configurable.
Support features like function calling or system vs user prompts as provider capabilities evolve.
Phase 9 — Ecosystem and Standardization [PENDING]

Goals: Establish .llm as a public standard.

Tasks:

Formalize versioning and publish a compatibility matrix.
Provide conformance tests and migration guides.
Engage with the community through open‑source governance and contributions.