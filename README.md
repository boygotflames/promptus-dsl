# `.llm` Transpiler

`.llm` is a spec-first prompt and configuration format for LLM-oriented workflows, paired with a Rust reference implementation that parses, validates, formats, benchmarks, and transpiles `.llm` documents into deterministic machine-facing outputs.

## What Problem It Solves

Markdown is readable, but it is a poor systems language for prompt orchestration. It is permissive, presentation-oriented, and structurally noisy. This repository explores a stricter alternative:

- a human-readable surface DSL for authoring
- a deterministic compiler pipeline
- compact machine-facing output targets
- measurable benchmarking against explicit baselines

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain, 2021 edition or later)
- Cargo (included with Rust)

### Build and test

```powershell
git clone <repo-url>
cd llm_format
cargo build
cargo test
```

### Run the CLI

```powershell
# Parse a .llm file and inspect the AST
cargo run -- parse examples/minimal.llm

# Validate a .llm file
cargo run -- validate examples/minimal.llm

# Transpile to plain text output
cargo run -- transpile examples/minimal.llm --target plain

# Transpile to JSON intermediate representation
cargo run -- transpile examples/minimal.llm --target json-ir

# Transpile to shadow representation (provisional)
cargo run -- transpile examples/minimal.llm --target shadow

# Format a .llm file to canonical form
cargo run -- fmt examples/noncanonical/messy.llm

# Benchmark token usage
cargo run -- bench examples/minimal.llm --baseline examples/baselines/minimal.md
```

## What Currently Works

- Surface parsing for reserved top-level keys:
  - `agent`
  - `system`
  - `user`
  - `memory`
  - `tools`
  - `output`
  - `constraints`
  - `vars`
- indentation-based structure with mappings, sequences, comments, and quoted scalars
- span-aware syntax diagnostics
- first-pass semantic validation
- deterministic output targets:
  - `plain`
  - `json-ir`
  - provisional `shadow`
- canonical formatting with 2-space indentation
- token counting and comparison against explicit baseline text files
- minimal VS Code file association and syntax highlighting

## Example: Input and Output

Given `examples/minimal.llm`:

```
agent: DataExtractor
system:
  role: financial_analyst
  output: json
memory:
  - user_history
```

**`--target plain`** (stable) — canonical normalized surface form:

```
agent: DataExtractor
system:
  role: financial_analyst
  output: json
memory:
  - user_history
```

**`--target json-ir`** (stable) — deterministic JSON intermediate representation:

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

**`--target shadow`** (provisional) — compact machine-facing representation:

```
@a="DataExtractor"
@s={role="financial_analyst";output="json"}
@m=["user_history"]
```

**Benchmark against `examples/baselines/minimal.md`:**

```
provider: generic
tokenizer: cl100k_base
source  | bytes=101 | tokens=27 | delta_bytes=+0  | delta_tokens=+0
plain   | bytes=94  | tokens=26 | delta_bytes=-7  | delta_tokens=-1
json-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19
shadow  | bytes=82  | tokens=23 | delta_bytes=-19 | delta_tokens=-4
```

## Current Scope

This repository currently acts as a reference implementation and proving ground for the v0 `.llm` language slice documented in [SPEC.md](SPEC.md).

The main public documents are:

- [Mission.md](Mission.md)
- [Plan.md](Plan.md)
- [SPEC.md](SPEC.md)
- [docs/compatibility-matrix.md](docs/compatibility-matrix.md)

## Public Contract Status

The current public contract boundary is intentionally split rather than treated as one giant frozen promise.

- `stable`
  - surface syntax
  - canonical formatter behavior
  - `plain` output
  - `json-ir` output
- `provisional`
  - `shadow` output
  - `bench` report shape
- `partial`
  - semantic validation breadth
  - provider-specific profile behavior
  - VS Code support
- `unsupported`
  - provider profiles without an explicit supported tokenizer/shadow path, currently `anthropic`

See [docs/compatibility-matrix.md](docs/compatibility-matrix.md) for the public matrix and contract notes.

## Provider Support Truth

Provider-aware behavior is intentionally narrow right now.

- `generic`
  - default provider profile
  - supported for `shadow` and `bench`
- `openai`
  - explicitly selectable
  - currently shares the same provisional shadow behavior as `generic`
  - bench uses the same current tokenizer path as the generic/default flow
- `anthropic`
  - explicitly unsupported in current provider-aware flows

This repo does not make claims of universal token behavior across providers.

## Editor Support Truth

Minimal VS Code support lives under [editors/vscode](editors/vscode).

Current editor support includes:

- `.llm` file association
- syntax highlighting
- basic language configuration

Current editor support does not include:

- LSP
- live validation
- completion
- hover help
- formatter-on-save integration

## Non-Goals Right Now

- remote provider execution
- cloud routing or orchestration
- API key/auth flows
- provider-specific runtime adapters beyond narrow profile plumbing
- language server implementation
- editor automation beyond minimal syntax support
- finalized public output compatibility guarantees for provisional targets

## Repository Layout

- [src](src): compiler, CLI, formatter, bench, provider, and transpile code
- [tests](tests): deterministic behavior coverage
- [examples](examples): valid, invalid, noncanonical, and benchmark baseline fixtures
- [editors/vscode](editors/vscode): minimal VS Code support package
- [docs](docs): compatibility matrix, versioning strategy

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for scope, development workflow, commit style, and the public contract stability model.

Before submitting anything, run:

```powershell
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All three must be clean.

## Status Note

The project is real, useful, and increasingly disciplined, but it is still in a standardization-prep phase. Treat current behavior as implemented reality, not as a forever-frozen contract unless the spec explicitly says so.
