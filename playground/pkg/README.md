<p align="center">
  <img src="assets/Promptus.svg" width="220"
       alt="promptus-dsl" />
</p>
<p align="center">
  <a href="https://github.com/boygotflames/promptus-dsl/releases">
    <img src="https://img.shields.io/github/v/release/boygotflames/promptus-dsl?label=version&color=0057FF"
         alt="Latest release" />
  </a>
  <a href="https://github.com/boygotflames/promptus-dsl/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/boygotflames/promptus-dsl/ci.yml?label=CI&branch=main"
         alt="CI status" />
  </a>
</p>
<br>

# promptus-dsl

**A prompt compiler for LLM workflows.**

Write structured prompts in `.llm` files. Validate them
before spending tokens. Compile them to the exact JSON
format each provider API expects.

```
agent: SentimentAnalyzer
system:
  role: classifier
  instruction: "Classify the input as positive, negative, or neutral."
user: "{input_text}"
output:
  sentiment: "one of: positive, negative, neutral"
  confidence: "float between 0 and 1"
vars:
  input_text: "The product broke on day one."
```

```powershell
# Validate before you spend
llm_format validate sentiment.llm

# Compile to Anthropic's native API format
llm_format transpile sentiment.llm --target anthropic-messages

# Compile to OpenAI's chat format
llm_format transpile sentiment.llm --target openai-chat
```

## The Problem

Prompts live as raw strings scattered across codebases. No validation
before execution. No reuse. No clear separation between what you write
and what the API receives.

promptus-dsl gives prompts the same discipline as code: structure,
validation, versioning, and composability.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 2021 edition or later)

### Build

```powershell
git clone https://github.com/boygotflames/promptus-dsl
cd promptus-dsl
cargo build --release
cargo test
```

### Your first prompt

```powershell
cargo run -- validate examples/minimal.llm
cargo run -- transpile examples/minimal.llm --target plain
cargo run -- transpile examples/minimal.llm --target json-ir
```

## What the Compiler Does

### validate

Catches problems before you spend tokens:

- Missing required keys (`agent`, `system`)
- Type mismatches (sequence where scalar expected)
- Empty required fields
- Undefined `{var}` references
- Circular includes
- Merge conflicts in multi-file prompts

```powershell
cargo run -- validate myprompt.llm
# ✓ valid  myprompt.llm
# OR
# semantic error at 3:1: [E101] missing required key: `system`
# ✗ invalid  myprompt.llm  (1 error(s))
```

### transpile

Compiles to output targets:

```powershell
# Plain normalized text
cargo run -- transpile prompt.llm --target plain

# JSON intermediate representation
cargo run -- transpile prompt.llm --target json-ir

# Shadow compact encoding (stable, backwards compatible)
cargo run -- transpile prompt.llm --target shadow
cargo run -- transpile prompt.llm --target shadow --provider anthropic

# Native provider JSON (ready for direct API calls)
cargo run -- transpile prompt.llm --target openai-chat
cargo run -- transpile prompt.llm --target anthropic-messages
```

### run

Execute a prompt against a live API:

```powershell
# Run against Anthropic (uses ANTHROPIC_API_KEY env var)
cargo run -- run examples/minimal.llm `
  --provider anthropic `
  --model claude-3-5-sonnet-20241022

# Run against OpenAI (uses OPENAI_API_KEY env var)
cargo run -- run examples/minimal.llm `
  --provider openai `
  --model gpt-4o

# Pass the API key explicitly
cargo run -- run examples/minimal.llm `
  --provider anthropic `
  --model claude-3-5-sonnet-20241022 `
  --key $env:MY_KEY

# Preview the API payload without sending (no key needed)
cargo run -- run examples/minimal.llm `
  --provider openai --model gpt-4o --dry-run
```

The `run` command validates the file first, then transpiles
to the provider's native JSON format, injects `model` and
`max_tokens`, and POSTs to the API. Response is printed as
pretty JSON. If the document declares an `output:` schema
and the response is JSON, missing fields are reported as
warnings.

### fmt

Canonical formatting — normalize any `.llm` file:

```powershell
cargo run -- fmt myprompt.llm
```

### lint

Static analysis of prompt quality before execution:

```powershell
cargo run -- lint myprompt.llm
```

Checks:
- **L001** — Missing `output:` schema declaration
- **L002** — Missing `user:` turn
- **L003** — System prompt is very short (< 20 chars)
- **L004** — System prompt is very long (> 4000 chars)
- **L005** — Contradictory constraints (concise + verbose)
- **L006** — Duplicate constraints

Warnings are printed to stderr; the command exits with
code 1 when warnings are found. They do not block
`transpile` or `run`.

### bench

Token count analysis against a Markdown baseline:

```powershell
cargo run -- bench myprompt.llm --baseline baseline.md
```

## Format Reference

### Required keys

- `agent` — identity of this prompt/agent
- `system` — instructions for the model

### Optional keys

- `user` — user turn content
- `memory` — context items (sequence)
- `tools` — available tools (sequence)
- `output` — expected output schema
- `constraints` — behavioral rules (sequence)
- `vars` — template variables for `{var_name}` expansion

### Multi-file composition

```
include: shared/base-system.llm

system:
  objective: "Answer questions about billing."

vars:
  product_name: "Acme Pro"
```

### Variable expansion

```
system: "You handle {product_name} support for {region}."
vars:
  product_name: Acme
  region: EU
```

Variables expand at transpile time in all output targets.

## VS Code Extension

Install from `editors/vscode/` for:

- Syntax highlighting
- Live validation (inline error squiggles as you type)
- Completion for all keys and `{var}` references
- Hover documentation
- Go-to-definition for `{var}` references
- `{var}` expanded values as ghost text
- Format on save

## Provider Support

| Provider | Shadow encoding | Tokenizer |
|---|---|---|
| `generic` (default) | V0 `@`-marker | cl100k_base |
| `openai` | V0 `@`-marker | cl100k_base |
| `anthropic` | V1 XML-tag | o200k_base |

All three providers are stable. Use `--provider anthropic` for
Anthropic-optimized shadow output and `o200k_base` token counts.

## Diagnostic Codes

All errors carry structured codes (`E001`–`E116`).
See [SPEC.md](SPEC.md) for the full reference.

## Repository Layout

- [`src/`](src): compiler — lexer, parser, AST, validator, transpile targets, CLI, bench
- [`tests/`](tests): conformance suite (131 tests)
- [`examples/`](examples): valid, invalid, and baseline fixtures
- [`editors/vscode/`](editors/vscode): VS Code extension
- [`docs/`](docs): compatibility matrix, versioning, roadmaps

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

```powershell
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the full version history (v1–v4).
