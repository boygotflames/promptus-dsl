---
name: repo-diagram
description: >
  Generates a deterministic, horizontal architectural diagram of the repository. 
  Maps core modules, data flow, boundaries, and key abstractions into a single 
  SVG or self-contained HTML visual. Automatically respects repo hygiene and 
  saves to a specified output path outside the working tree.
---

# Repo Diagram Skill

## 1. Mission
When invoked, your objective is to analyze the current repository state and generate a clean, high-fidelity architectural diagram. You are mapping the actual implementation reality, not the theoretical roadmap.

## 2. Execution Protocol
1. **Spider the Source:** Read the core source tree to understand module boundaries, entry points (CLI/API), data flows, and output targets.
2. **Ignore Noise:** Strictly ignore build directories (`target/`, `node_modules/`), hidden agent folders (`.claude/`, `.git/`), and benchmark/fixture data unless explicitly requested.
3. **Map the Architecture:** Identify the following layers:
   - Input / Entry Points (e.g., CLI, standard input)
   - Core Processing (e.g., Lexer, Parser, AST, Validator)
   - Data Models / State
   - Emitters / Output Targets
4. **Generate the Visual:** Construct the diagram using a reliable text-to-visual syntax (e.g., Mermaid.js wrapped in a self-contained HTML file, or a perfectly formed raw SVG).

## 3. Layout and Styling Constraints
- **Orientation:** Must be strictly horizontal (Left-to-Right data flow). If using Mermaid, enforce `graph LR` or `flowchart LR`.
- **Clarity:** Group related components inside sub-graphs or bounding boxes to clearly delineate module boundaries (e.g., "Compiler Core" vs "CLI Layer").
- **Conciseness:** Do not map every single utility function. Map the primary architectural organs and how data moves between them.

## 4. Output Contract
- **No Repo Pollution:** The generated file MUST be saved to the absolute or relative path explicitly provided by the user. If the path is outside the repository, respect that boundary.
- **Silent Execution:** Once the file is successfully written to disk, do not output the raw SVG/HTML code into the chat. Simply confirm the file path where the diagram was saved and state that the task is complete.