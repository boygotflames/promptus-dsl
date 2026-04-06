# Mission: `.llm` Format

## Overview

`.llm` is a new prompt format designed for large language models.

Its purpose is to replace Markdown as the default authoring layer for structured prompting when the target is not HTML rendering, but machine execution, deterministic parsing, and token efficiency.

The project combines two core deliverables:

- a formal `.llm` specification
- a Rust-based transpiler that compiles human-readable `.llm` source into machine-optimized output

---

## Problem Statement

Modern prompting still relies heavily on Markdown (`.md`), even though Markdown was designed for presentation and document publishing, not for machine-native prompt orchestration.

This creates several problems:

- **Token waste**  
  Structural syntax such as headings, fenced code blocks, separators, and wrapper markup consumes tokens without adding semantic value to the model.

- **Weak routing semantics**  
  Markdown has no built-in primitives for agent routing, memory boundaries, execution modes, tool contracts, or state transitions.

- **Prompt bloat through workaround syntax**  
  Developers often compensate with XML-like wrappers, nested delimiters, and verbose conventions that further increase token overhead.

- **Lack of strict grammar**  
  Markdown is permissive and ambiguous. Models can hallucinate formatting, close blocks incorrectly, or generate malformed structures that break downstream systems.

In short: Markdown is readable, but it is not a reliable systems language for LLM workflows.

---

## Vision

We are building a **dual-layer prompt architecture**:

### Surface Layer
A human-readable DSL for authoring structured prompts with clarity and minimal noise.

### Shadow Layer
A normalized machine-facing representation produced by compilation, using compact markers and deterministic encoding optimized for LLM execution.

This mirror design separates:

- **developer ergonomics** from **runtime efficiency**
- **authoring clarity** from **machine compactness**
- **human intent** from **provider-specific prompt serialization**

The result is a format that is easier to write, safer to parse, and substantially more token-efficient.

---

## Core Solution

The `.llm` system consists of:

1. **A formal specification** that defines grammar, semantics, reserved keywords, and canonical structure.
2. **A Rust transpiler** that parses `.llm` into an AST, validates semantics, and emits multiple targets.
3. **A benchmarking layer** that proves the token and performance advantages of `.llm` over Markdown-based prompting.
4. **An open ecosystem** for tooling, editor support, adapters, and future standardization.

---

## Goals

We aim to establish `.llm` as an open standard for multi-provider, structured, agent-oriented prompting.

### Primary goals

- Define an unambiguous specification in `SPEC.md`
- Build a safe, high-performance Rust compiler/transpiler
- Support multiple output targets, including:
  - Shadow payloads
  - plain prompt output
  - JSON intermediate representation
- Demonstrate measurable token savings against Markdown equivalents
- Enable adoption across editors, frameworks, and model providers

### Success criteria

A successful `.llm` ecosystem should make prompts:

- cheaper to send
- easier to author
- safer to validate
- more deterministic to execute
- easier to adapt across providers

---

## Engineering Tenets

### 1. Specification First
The grammar and semantics come first.

Implementation must follow the specification, not invent it on the fly. Code, tooling, examples, and tests must remain aligned with the evolving spec.

### 2. Dual-Layer Architecture
Surface and Shadow are distinct by design.

The Surface must remain readable and writable by humans.  
The Shadow must remain deterministic, compact, and optimized for machine consumption.

### 3. Deterministic Execution
Compilation must be stable and reproducible.

The same `.llm` input must always produce the same AST and the same emitted output. Diagnostics must be precise and source-aware.

### 4. Pluggable Targets
The transpiler must support extensible emission targets.

Provider-specific Shadow encodings or downstream formats should be addable without changing the core language.

### 5. Memory-Safe Performance
The implementation language is Rust.

We prioritize:
- memory safety
- predictable performance
- explicit error handling
- strong compiler guarantees

### 6. Open but Disciplined
The format and tooling should be open source, but the project must still maintain architectural discipline, versioning clarity, and quality control.

Openness is not an excuse for ambiguity.

---

## Licensing and Philosophy

`.llm` will follow an open ecosystem model.

### Licensing direction

A permissive license such as:

- MIT
- Apache-2.0

### Philosophy

We follow an **open core / open standard** mindset:

- the format should remain open
- the compiler and tooling should remain accessible
- adoption should not depend on vendor lock-in

Commercial value, if introduced, should come from higher-level infrastructure such as:

- enterprise routing
- hosted orchestration
- proprietary control planes
- deployment and observability layers

Not from restricting the format itself.

---

## Commitment

We are building `.llm` as an iterative and transparent engineering effort.

We commit to:

- documenting progress in `PLAN.md`
- evolving the specification in public
- benchmarking real gains, not hand-waving them
- building tooling that is practical, deterministic, and production-oriented

The end state is a robust prompt-engineering ecosystem that reduces token waste, improves structural reliability, and gives developers a format built for LLM systems rather than retrofitted from document markup.