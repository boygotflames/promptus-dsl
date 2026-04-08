---
name: rust-memory-methodology
description: >
  Invoke this skill for any Rust task where ownership, allocation, borrowing,
  aliasing, thread boundaries, or data-shape choices materially affect
  correctness or maintainability. Covers ownership models, lifetime design,
  stack vs heap layout, allocation strategy, clone tradeoffs, interior
  mutability, Rc/Arc selection, Send/Sync boundaries. Especially applicable to
  parsers, compilers, transpilers, ASTs, IRs, and diagnostics systems.
---

# Rust Memory Methodology

## When to Use This Skill

Use this skill when:
- designing or reviewing Rust types where ownership or layout matters
- refactoring ownership, clones, lifetimes, or allocation surfaces
- working in a parser, compiler, transpiler, AST, IR, or diagnostics codebase
- debugging borrow checker friction that might signal a design problem
- deciding between `Arc`, `Rc`, `Box`, `Clone`, `Cow`, or lifetime-bound references

---

## Task Modes

### DESIGN MODE
1. Identify the lifetime horizon: parse-stage (short-lived) vs long-lived (outlives parse).
2. Default to owned data. Borrow only when lifetime topology is simple and local.
3. Does this type outlive its input buffer? If yes → own.
4. Will this type cross thread boundaries? If yes → `Send`-compatible from the start.
5. Is this a recursive structure? If yes → `Box` the back-edge.

### REVIEW MODE
1. Run the Review Checklist.
2. Flag: unnecessary `Arc`, `RefCell` as design crutch, deep clone chains, unstable map ordering in output-sensitive code.

### REFACTOR MODE
1. Do not reshape the ownership model unless there is a demonstrated problem.
2. Prefer minimal diffs.
3. If borrow checker friction is the problem, consider moving to owned data before adding lifetime bounds.

### PERFORMANCE MODE
1. Do not optimize without identifying the hot path first.
2. Target repeated deep clone chains before micro-optimizing allocation counts.
3. Consider `Cow<'a, str>` only when both owned and borrowed states are genuinely common.

### DEBUG MODE
1. Treat persistent borrow errors as a design signal, not a puzzle to solve with annotations.
2. Common root causes: aliased mutable state, lifetime parameters spreading across modules, mixed ownership strategies, `RefCell`/`Rc` masking a real ownership problem.
3. If the fix requires 3+ new lifetime annotations in unrelated types, treat that as a design failure.

---

## Core Rules

- Prefer clear ownership over clever lifetime gymnastics.
- Prefer owned data when lifetime coupling would make the design brittle.
- Prefer borrowing when data is short-lived and tied to a local buffer.
- Avoid unnecessary cloning; do not contort APIs just to eliminate a cheap local clone.
- Make aliasing and mutation boundaries explicit.
- Treat heap allocation as a cost to justify, not a default to fear blindly.

---

## Review Checklist

- [ ] Who owns each major value?
- [ ] How long does each value need to live?
- [ ] Is borrowing reducing real cost, or just increasing complexity?
- [ ] Are clones cheap and local, or large and repeated on hot paths?
- [ ] Is interior mutability truly necessary, or a design shortcut?
- [ ] Is reference counting truly necessary, or an early generalization?
- [ ] Does the design preserve deterministic behavior?
- [ ] Is insertion order preserved where output must be deterministic?

---

## Reporting Contract

```
Memory / Ownership Report
Mode: [design | review | refactor | performance | debug]
Component: [type, module, or subsystem]
Ownership model: [owned / borrowed / mixed]
Lifetime model: [none / local / cross-module]
Allocation surfaces: [where heap allocation occurs and why]
Clone surfaces: [where cloning occurs, cost estimate]
Mutation boundaries: [where mutation is permitted and how guarded]
Thread-safety assumption: [single-threaded / Send-compatible / Arc-guarded]
Determinism implications: [ordering-sensitive data structures or outputs]
Alternatives considered: [alternative - rejected because reason]
Residual risks: [anything fragile or likely to break under future changes]
Design priority: [ergonomics / safety / speed / simplicity]
```
