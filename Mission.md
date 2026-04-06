Mission: .llm Format
Problem Statement

Large language model prompting currently relies on Markdown (.md), a format built for HTML rendering rather than machine logic. When used with token-hungry models like Claude 3 Opus or GPT‑4, Markdown's structural syntax leads to significant token waste; a typical system prompt can lose around 15 % of its tokens to the ```json, ### and --- delimiters. Markdown lacks rigid routing primitives, forcing developers to embed verbose XML-like tags to express state transitions, which bloats prompts further. Worse, Markdown has no strict grammar, so models sometimes hallucinate formatting or leave code blocks unclosed, causing downstream parse errors.

Vision & Solution

Our mission is to solve these inefficiencies by defining a new dual‑layer .llm format and building an accompanying Rust transpiler. The .llm specification embraces a mirror architecture: the human‑readable Surface layer is a concise DSL that uses indentation and clear keywords to describe agents, system roles, memory and tooling. When compiled, this Surface is transformed into a Shadow layer: a normalized machine‑facing representation that uses single‑token ASCII or Unicode markers to encode structural boundaries. This mirror design decouples developer ergonomics from machine efficiency and reduces structural token cost by 40–60 %.

Goals

We aim to establish .llm as the definitive open standard for multi‑provider agent prompts. The core deliverables are:

An unambiguous specification (SPEC.md) defining the Surface grammar, semantics and reserved keywords.
A safe, high‑performance Rust transpiler that parses .llm files into an abstract syntax tree (AST), validates their semantics and emits multiple targets (Shadow payloads, plain prompts and a JSON intermediate representation).
A benchmarking suite to demonstrate token cost reductions relative to equivalent Markdown prompts and to measure performance across model tokenizers.
An open ecosystem that invites contributions and fosters adoption across editors, frameworks and providers.
Engineering Tenets

Our engineering principles are:

Specification‑First: The grammar and semantics are defined up front in a living specification. Code, tools and examples must adhere to the spec.
Dual‑Layer Architecture: All compilation stages respect the separation between Surface and Shadow. The Surface must remain readable and writeable by humans; the Shadow must remain deterministic and token‑efficient.
Deterministic Execution: Parsing, validation and transpilation must be fully deterministic. The same .llm input always produces the same AST and output; error messages include precise line/column diagnostics.
Pluggable Targets: The transpiler exposes a trait‑based interface for new emitters so that providers can supply optimized Shadow encodings without changing the core grammar.
Memory‑Safe Performance: We use Rust with crates like logos for lexing and nom or custom iterators for parsing, guaranteeing memory safety and predictable performance.
Open but Disciplined: The specification, compiler and tooling are open source to encourage community contributions; however, enterprise routing and orchestration may become commercial offerings【959362995327066†L67-L80】.
Licensing & Philosophy

We embrace the "Open Claw" philosophy. .llm and its tooling will be released under a permissive license (MIT or Apache‑2.0). This encourages viral adoption and prevents vendor lock‑in. Monetization will focus on optional enterprise routers and cloud orchestration services rather than on the file format itself.

Commitment

We commit to an iterative, transparent development process. Every phase will be documented in PLAN.md, with milestones tracked openly. The result will be a robust ecosystem that lowers API costs, improves prompt stability and empowers developers to build sophisticated agentic workflows without worrying about token bloat or ambiguous parsing.