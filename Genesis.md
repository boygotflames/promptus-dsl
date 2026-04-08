# Genesis

## Executive Snapshot

This repository is now a working, compiler-grade `.llm` toolchain for controlled internal use. It parses an indentation-based prompt DSL into a span-bearing AST, validates a first tranche of semantics, emits deterministic `plain`, `json-ir`, and provisional `shadow` outputs, exposes those flows through a CLI, benchmarks the current representations with a fixed tokenizer path, formats files canonically, and ships minimal VS Code syntax support.

It is not yet a public standard, not a provider-execution platform, not an IDE-integrated language stack, and not a frozen interoperability contract. The practical maturity level is "solid internal prototype with unusually good determinism and test coverage, but still missing the boring public-facing contracts that make software trustworthy outside the lab."

## Founding Intent

The original intent was not to build "yet another prompt file." It was to replace improvised Markdown prompt blobs with a spec-first, machine-legible format that can be authored by humans and normalized for downstream LLM runtimes. The core architectural bet was dual-layer:

- Surface layer: readable `.llm` DSL for humans.
- Shadow layer: compact, normalized machine-facing representation.

Future brand note: `LLM Promptus` appears to be a future-facing branding idea, not a current repo rename. Treat it as possible later packaging/identity only.

Latin note status: the user asked for the previously provided Latin note to be carried forward, but that line was not recoverable from tracked repository state or git history during this audit. It should be inserted later verbatim from the original source, not guessed here.

## Compact Overview of Mission.md

`Mission.md` defines the big swing clearly: `.llm` exists because Markdown is too loose, ambiguous, and machine-hostile for serious prompt pipelines. The mission argues for a deterministic, spec-first format with separate author-facing and machine-facing layers, implemented in Rust, with pluggable targets and measurable proof rather than hand-wavy efficiency claims. The document is strong on engineering values: deterministic execution, clear module boundaries, spec-first discipline, and provider-agnostic design.

The mission is ambitious but mostly still directional. It names a real product philosophy, not just tasks. The strongest parts are the dual-layer framing and the refusal to confuse benchmarks with truth. The weakest part is that the repo has outrun the mission's operational translation into public docs.

## Compact Overview of Plan.md

`Plan.md` is useful as an original roadmap and almost useless as a status document now. It still claims Phase 1 is in progress and marks Phases 2 through 9 as pending, even though the git history and tracked tree show substantial work through provider-aware plumbing and editor support.

As a roadmap, it is coherent:

- Phase 0: scaffold.
- Phase 1: parser/AST.
- Phase 2: semantic validation.
- Phase 3: transpiler targets.
- Phase 4: CLI.
- Phase 5: benchmarking.
- Phase 6: formatter.
- Phase 7: editor support.
- Phase 8: provider-aware adapters.
- Phase 9: ecosystem/standardization.

As reality, it is stale enough to be dangerous. A new contributor reading only `Plan.md` would underestimate the implementation surface and overestimate what remains unstarted.

## Compact Overview of SPEC.md

`SPEC.md` is the most useful authoritative document in the repo. It defines the v0 surface syntax, reserved top-level keys, indentation model, AST concepts, canonical formatting rules, first-pass validation semantics, and the current provisional shadow representation. It also explicitly marks deferred work, which helps keep implementation honest.

The spec is still a draft, but it has become the de facto contract for most implemented behavior. The main remaining weakness is not absence of content so much as lack of frozen compatibility language around outputs, validation growth, and long-term versioning.

## Whole-Repo Spider Summary

- Docs: `Mission.md` and `SPEC.md` still matter; `Plan.md` is materially stale; root `README.md` is missing; `Genesis.md` did not previously exist as tracked documentation.
- Compiler core: real lexer, parser, AST, diagnostics, and validator are in place with deterministic behavior and span-aware errors.
- CLI: `parse`, `validate`, `transpile`, `fmt`, and `bench` all exist and are wired as thin orchestration over library code.
- Bench: deterministic measurement exists for `source`, `plain`, `json-ir`, and `shadow`, with optional external baseline comparison and provider-visible reporting.
- Formatter: canonical AST-based formatter exists and drives the `plain` target, which keeps canonical rendering single-sourced.
- Editor support: minimal VS Code grammar/config/package exist and are isolated from Rust internals.
- Tests: coverage is broad for a young repo; most contracts are enforced through exact-string tests and fixture-based failure checks.
- Examples: fixtures are useful and intentional, including invalid cases, noncanonical formatting input, and benchmark baselines.
- Provider layer: real abstraction exists, but only `generic` and `openai` are meaningfully supported today; `anthropic` is explicitly unsupported rather than faked.

## File-by-File Compact Inventory

- `Mission.md`: founding product and engineering philosophy.
- `Plan.md`: roadmap skeleton; currently stale as status tracking.
- `SPEC.md`: actual behavioral control plane for v0.
- `Cargo.toml`: lean dependency surface; mostly CLI, lexing, tokenizer, and test helpers.
- `.gitignore`: now hardened beyond the original single-line `target/`.
- `src/lib.rs`: public module/export surface for the library crate.
- `src/main.rs`: minimal binary entry point.
- `src/ast.rs`: explicit `Document`, `Node`, `MappingEntry`, `TopLevelKey`, and span helpers.
- `src/diagnostics.rs`: syntax vs semantic diagnostic formatting and collection.
- `src/lexer.rs`: line-oriented lexer with indentation, comments, quoted scalars, and deterministic failures.
- `src/parser.rs`: manual parser for the v0 surface DSL; owns unknown/duplicate top-level rejection.
- `src/validator.rs`: first semantic tranche; duplicate nested keys and top-level shape checks.
- `src/formatter.rs`: canonical surface formatter and scalar quoting rules.
- `src/provider.rs`: provider/profile abstraction with explicit support boundaries.
- `src/transpile/mod.rs`: target selection and shared emitter boundary.
- `src/transpile/plain.rs`: canonical surface-style output via formatter reuse.
- `src/transpile/json_ir.rs`: deterministic JSON IR renderer.
- `src/transpile/shadow.rs`: provisional v0 shadow renderer with explicit marker table.
- `src/bench/mod.rs`: report model, row computation, and rendering.
- `src/bench/tokenizer.rs`: tokenizer wrapper around `cl100k_base`.
- `src/cli/*.rs`: command-specific thin orchestration with consistent parse/validate-before-action flows.
- `src/tokenizer.rs`: tracked empty scaffold residue; functionally dead today.
- `src/transpiler.rs`: tracked empty scaffold residue; functionally dead today.
- `tests/parse_golden.rs`: stable AST assertions for valid fixtures.
- `tests/parse_errors.rs`: parser failure contracts and precise diagnostics.
- `tests/validate.rs`: semantic validation contracts.
- `tests/transpile.rs`: deterministic output contracts and transpile CLI behavior.
- `tests/bench.rs`: deterministic bench output/report behavior.
- `tests/fmt.rs`: formatter exact-output and idempotence checks.
- `tests/vscode.rs`: editor asset sanity checks without a JS test harness.
- `examples/*.llm`: canonical positive fixtures.
- `examples/invalid/*.llm`: negative parser/validator fixtures.
- `examples/noncanonical/messy.llm`: formatter normalization fixture.
- `examples/baselines/*.md`: explicit benchmark comparison artifacts, not language semantics.
- `editors/vscode/*`: minimal syntax-highlighting package, docs, and launch config.

## Phase-by-Phase Roadmap Reality

| Phase | Intended | Actual | Status |
|---|---|---|---|
| 0 | Scaffold and foundation | Cargo project, module layout, examples, initial docs landed | Complete |
| 1 | Parser and AST | Real lexer/parser/AST with spans, quoted scalars, lists, nested maps, comments, and deterministic parse diagnostics | Complete |
| 2 | Semantic validation | Initial semantic tranche exists, but required-structure depth is intentionally limited by spec | Partial |
| 3 | Transpiler targets | `plain`, `json-ir`, and provisional `shadow` are implemented and tested | Partial |
| 4 | CLI | `parse`, `validate`, `transpile`, `fmt`, and `bench` all exist; transpile file output is hardened | Complete for current scope |
| 5 | Benchmarking and proof | Internal representation counting and external baseline comparison exist; proof is still narrow and tokenizer-specific | Partial |
| 6 | Formatter and ergonomics | Canonical formatter and write-back flow exist; style preservation is intentionally absent | Partial |
| 7 | Editor support | Minimal VS Code association/grammar/config/package exist; no LSP or live validation | Partial |
| 8 | Provider-aware adapters | Provider/profile plumbing exists for shadow and bench; only `generic`/`openai` are meaningful today | Partial |
| 9 | Ecosystem and standardization | No serious ecosystem packaging, versioning, governance, or external contract freeze yet | Not started |

## Intended vs Actual

The percentages below are qualitative judgment calls, not scientific measurements.

- Mission realization as an internal toolchain: about 70%.
- Mission realization as a public technology/standard: about 25%.
- Spec-backed surface/compiler correctness for current v0 slice: about 75%.
- Public-contract readiness for outputs and compatibility promises: about 30%.
- Provider-aware maturity: about 20%.
- Ecosystem/readme/onboarding maturity: about 15%.

Translation: the compiler skeleton became real faster than the surrounding product surface became legible. That is better than vaporware, but still a mismatch.

## Drift Analysis

The repo mostly followed the roadmap linearly through parser, validator, output targets, CLI hardening, benchmarking, formatter, editor support, and provider plumbing. That is good drift: sequencing stayed close to the intended phase order.

The main branches and side-streams were:

- CLI hardening became more serious than a bare phase checklist item.
- Bench grew into external-baseline comparison before any public README existed.
- VS Code support arrived before ecosystem/standardization prep.
- Provider plumbing arrived while provider behavior still intentionally mostly does not diverge.

Why the drift happened:

- the easiest wins were implementation wins, not documentation wins
- the project favored proving internals over publishing contracts
- each phase packet was kept narrow, which helped code quality but let high-level docs fall behind

## Entropy and Spaghetti Rate

These are qualitative judgment calls, not objective metrics.

- Core compiler entropy: low to moderate, roughly 3.5/10.
- Repo narrative entropy: high, roughly 7/10.
- Spaghetti rate in runtime code: low, roughly 3/10.
- Spaghetti rate in project signaling/docs: medium-high, roughly 6.5/10.

Evidence that entropy was reduced:

- clean module separation between AST, parser, validator, diagnostics, transpile, CLI, bench, and formatter
- deterministic outputs and diagnostics backed by focused tests
- explicit unsupported-provider behavior instead of fake support
- editor support isolated from Rust internals

Evidence that complexity accumulated:

- `Plan.md` lies about project status
- root `README.md` is missing entirely
- empty tracked scaffold leftovers (`src/tokenizer.rs`, `src/transpiler.rs`) still exist
- output contracts are tested but not yet frozen as public compatibility promises
- the repo has many capabilities now, but the top-level story does not explain them

## Brutal Critique

### Product owner

The ambition was good, but the governance discipline was sloppy. Shipping eight phases of capability without keeping the roadmap/status docs current is how teams accidentally gaslight themselves. The obsession with future value propositions was mostly healthy, but the boring work of "can a stranger understand this repo in ten minutes?" was neglected for too long.

### Architect / controller

The architecture is mostly sane, but there was an avoidable habit of pushing the frontier before freezing contracts. Provider awareness was added before provider differentiation was real. VS Code support landed before the repo had a proper README. That is not fatal, but it is how good prototypes become confusing products.

### Codex execution

Codex did a decent job keeping packets narrow and deterministic, but it also left classic agent residue: stale placeholder files still tracked, status docs not reconciled after implementation, and too much implicit reliance on tests as the real documentation. Good at shipping slices, weaker at periodic consolidation.

### The repo itself

The repo is better engineered than it is explained. That is a compliment and an indictment. The core is not spaghetti. The messaging layer absolutely is. Right now the code deserves more trust than the docs, and that is backwards for a spec-first project.

## What We Can Do Right Now

- Parse `.llm` files into a stable AST with source spans.
- Reject malformed syntax with precise line/column diagnostics.
- Run first-pass semantic validation.
- Emit deterministic `plain`, `json-ir`, and provisional `shadow` outputs.
- Select provider profiles for `shadow` and `bench` where supported.
- Benchmark `source`, `plain`, `json-ir`, and `shadow`, optionally against explicit baseline text files.
- Canonically format `.llm` files to stdout or in place.
- Load minimal `.llm` syntax highlighting support in VS Code.

## How to Test the Stack Right Now

Rust/CLI checks:

```powershell
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- parse examples/minimal.llm
cargo run -- validate examples/minimal.llm
cargo run -- transpile examples/minimal.llm --target plain
cargo run -- transpile examples/minimal.llm --target json-ir
cargo run -- transpile examples/minimal.llm --target shadow
cargo run -- bench examples/minimal.llm
cargo run -- bench examples/minimal.llm --baseline examples/baselines/minimal.md
cargo run -- fmt examples/noncanonical/messy.llm
```

Provider smoke checks:

```powershell
cargo run -- transpile examples/minimal.llm --target shadow --provider generic
cargo run -- bench examples/minimal.llm --provider openai
cargo run -- bench examples/minimal.llm --provider anthropic
```

The last command should fail explicitly today.

VS Code smoke check:

1. Open `D:\llm_format\editors\vscode` in VS Code.
2. Press `F5`.
3. In the Extension Development Host, open `examples/minimal.llm` and `examples/quoted.llm`.
4. Confirm `.llm` language mode, key highlighting, string highlighting, list markers, and `#` comments.

## What Still Blocks Broader Adoption

- no root `README.md`
- stale roadmap/status document
- no frozen compatibility story for `json-ir` or `shadow`
- provisional shadow format with no proof of long-term stability
- provider architecture ahead of real provider differentiation
- no schema/value typing beyond current generic nodes
- no comment-preserving formatter strategy
- no language server or live editor diagnostics
- no packaging, governance, or versioning story for standardization

## Recommended Next Moves

Phase 9 should not be "ecosystem hype." It should be consolidation and contract freezing.

Recommended next moves from current reality:

1. Rewrite `Plan.md` so status matches the repo.
2. Add a real root `README.md` with install, commands, examples, and scope boundaries.
3. Decide what v0 compatibility actually means for `plain`, `json-ir`, and `shadow`.
4. Remove or explicitly annotate dead scaffold leftovers like `src/tokenizer.rs` and `src/transpiler.rs`.
5. Tighten `SPEC.md` around required structure, output contracts, and provider/profile guarantees.
6. Decide whether Phase 9 is public standardization or a Phase 8.5 consolidation milestone. Right now, consolidation is the honest move.

If the team skips that consolidation step and jumps straight to ecosystem expansion, it will be scaling ambiguity, not scaling a product.
