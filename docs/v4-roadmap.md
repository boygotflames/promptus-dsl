# v4 Roadmap

## Status

v3 is declared and complete. v4 work begins here.

## Guiding Principle

v1 through v3 built the core transpiler, proved provider
differentiation, made prompts expressive with `{vars}`, and
gave developers inline tooling (CI, diagnostics, completion,
hover). What remains is the adoption gap: the tooling cannot
be distributed (no .vsix), prompts cannot be composed across
files (every document is standalone), and the editor experience
has one visible hole (expanded var values are invisible while
authoring). v4 closes these three gaps. It makes `.llm` ready
to distribute, ready to compose, and more transparent to read.

The three pillars:

1. **VS Code marketplace packaging** — a .vsix build pipeline
   so the extension can be installed without cloning the repo
2. **Inlay hints** — show expanded `{var}` values as ghost
   text in the editor while authoring, without transpiling
3. **Multi-file includes** — `include:` key for sharing
   system instructions, tools, and vars across documents

v4 does not break v3. All v3-stable surfaces remain stable.

---

## Track G — VS Code Marketplace Packaging

### Problem

The VS Code extension (`editors/vscode/`) is feature-complete
as of v3: syntax highlighting, live diagnostics, formatter,
completion, hover, go-to-definition. But it cannot be installed
by anyone who does not clone this repository.

There is no `.vsix` artifact and no build pipeline. Installing
the extension requires:
1. cloning the repo
2. opening `editors/vscode/` in VS Code
3. pressing F5 to launch the Extension Development Host

This is a developer-mode workflow, not a user-facing one.
The extension is invisible to anyone who searches the VS Code
marketplace.

The specific blockers are:
1. `images/Promptus.png` is referenced in `package.json` as the
   icon but the README notes it needs to be exported from the SVG
   source — the PNG may not be present or may be placeholder-only
2. No `.vscodeignore` file exists — packaging without it will
   bundle the entire repo into the .vsix
3. No `vsce package` step exists in any script or workflow
4. No marketplace publish instructions in the README

### What v4 changes

One-time pipeline setup:

1. Confirm or generate `images/Promptus.png` from the SVG source
   (or use a placeholder 128×128 PNG with the L glyph)
2. Add `.vscodeignore` to the `editors/vscode/` directory,
   excluding everything except what the extension needs:
   - keep: `extension.js`, `package.json`, `language-configuration.json`,
     `syntaxes/`, `images/Promptus.png`, `README.md`
   - exclude: all Rust source, `target/`, `.github/`, `docs/`,
     `tests/`, `examples/`, `fixtures/` at the vscode level
3. Add `vsce` as a dev dependency in `package.json`
4. Add `package` and `build` scripts to `package.json`:
   - `"package": "vsce package"` → produces `llm-vscode-0.1.0.vsix`
   - `"build": "vsce package --no-dependencies"`
5. Update `editors/vscode/README.md`: add Installation section
   with .vsix local install instructions
6. Add `.github/workflows/vscode-package.yml`: on push to main,
   run `npm run package` and upload the .vsix as a workflow artifact
   (not a marketplace publish — that requires a manual secret)
7. Document the marketplace publish step (requires a publisher
   token secret) in a separate `docs/marketplace.md`

Marketplace publication (the actual `vsce publish`) is a manual
one-time step that requires a publisher token. The pipeline
automates the `.vsix` artifact; publication remains manual.

### Sequence

1. Inspect `images/Promptus.png` — confirm it exists and is
   valid 128×128 PNG; if not, create a minimal placeholder
2. Create `editors/vscode/.vscodeignore`
3. Update `editors/vscode/package.json`: add `vsce` devDependency,
   add `package` script
4. Run `npm run package` in `editors/vscode/` — confirm
   `llm-vscode-0.1.0.vsix` is produced without errors
5. Add `.github/workflows/vscode-package.yml`: artifact upload
6. Update `editors/vscode/README.md`: installation section
7. Create `docs/marketplace.md`: publisher token setup guide

### Estimated size

**Small** — one packet. No Rust changes. No SPEC changes.
Pure packaging configuration and CI workflow.

### v4 contract commitment

By end of Track G:
- `npm run package` in `editors/vscode/` produces a valid `.vsix`
- CI uploads the `.vsix` as a build artifact on every push to main
- Any user can install the extension from the `.vsix` without
  cloning the repo
- Marketplace publication process is documented (but requires a
  human-held publisher token to execute)

---

## Track H — Inlay Hints

### Problem

vars expansion is invisible while authoring. Consider:

```llm
system:
  objective: "transform {source_table} to {target_table}"
vars:
  source_table: orders_raw
  target_table: orders_clean
```

While editing, the user sees `{source_table}`. They cannot tell
what value it expands to without scrolling to the `vars:` block
or running `transpile --target plain`. For documents with 10+
vars, this cognitive load is real.

The v3 definition provider (F12 on `{var}`) helps — but the
user has to ask. Inlay hints surface the answer passively:

```llm
  objective: "transform {source_table}‹orders_raw› to {target_table}‹orders_clean›"
```

The `‹value›` ghost text appears after each `{var_name}` reference,
showing the resolved value inline without changing the document.

This is the one visible IntelliSense gap that the v3 hover provider
only partially covers (hover requires the user to move the cursor).

### What v4 changes

One new function in `editors/vscode/extension.js`:

```javascript
vscode.languages.registerInlayHintsProvider(selector, {
  provideInlayHints(document, range) {
    return getInlayHints(document, range);
  }
});
```

`getInlayHints(document, range)` logic:
1. Parse the vars block from the document text (reuse `parseVarsBlock`)
2. For each line in `range`:
   - find all `{var_name}` patterns
   - if the var name is in the vars map:
     - create an `InlayHint` at the position after the closing `}`
     - label: `: value` (or `= value`, styled as type hint)
     - kind: `vscode.InlayHintKind.Type`
3. Return the array of hints

Undefined vars (E114) get no hint — the squiggle from live
validation already marks them.

No Rust changes. No SPEC changes. Pure JS extension work.

`vscode.InlayHintsProvider` is available since VS Code 1.67.
The extension already requires VS Code ^1.85.0, so no version
constraint issue.

### Sequence

1. Add `getInlayHints(document, range)` function to `extension.js`
2. Register `InlayHintsProvider` in `activate()`
3. Add a `fixtures/inlay-hints-sample.llm` fixture with several
   vars references for manual verification
4. Update `editors/vscode/README.md`: add Inlay Hints subsection
   under IntelliSense
5. Add a `vscode.rs` conformance test: read the extension.js text
   and confirm `registerInlayHintsProvider` is present (smoke test)

### Estimated size

**Small** — one packet. Reuses `parseVarsBlock` already in the
extension. No Rust changes. The VS Code API is straightforward.

### v4 contract commitment

By end of Track H:
- Expanded var values appear as ghost text after each `{var_name}`
  in the editor without requiring any user action
- Hints update when the vars block changes (on each document change
  event, same as diagnostics)
- Undefined vars produce no hint (squiggle from live validation
  is sufficient feedback)

---

## Track I — Multi-file Includes

### Problem

Every `.llm` document is a standalone unit. There is no way to
share:
- a common system prompt (e.g., a company-wide `base-system.llm`)
- a shared tool list (e.g., `standard-tools.llm`)
- shared vars (e.g., environment-specific `staging-vars.llm`)

In practice, this means prompt authors copy-paste shared sections
across files. When the base instructions change, every file that
copies them must be updated manually. For teams with 10+ agent
prompts, this is a real maintenance problem.

The feature exists in the post-v0 deferred work section of
SPEC.md under "Includes/imports and multi-file composition."
vars expansion (v3 Track D) is a prerequisite — without it,
the value of shared files is limited.

### What v4 changes

**Syntax:** A new top-level key `include:` takes a sequence
of relative file paths:

```llm
include:
  - shared/base-system.llm
  - shared/common-tools.llm

agent: DataPipeline
system:
  extra_instruction: run daily at midnight
vars:
  source_table: orders_raw
```

Or a single-file shorthand (scalar form):
```llm
include: shared/base-system.llm
```

**Scope:** Top-level only. No nested includes. No includes
inside mappings or sequences. Circular includes are an error.

**Merge semantics — explicit rules:**

| Key | Merge behavior |
|---|---|
| `agent` | Error if both parent and included define it |
| `system` | Error if both define it (no silent override) |
| `user` | Error if both define it |
| `output` | Error if both define it |
| `memory` | Concatenated: included items first, parent items second |
| `tools` | Concatenated: deduplicated by item value |
| `constraints` | Concatenated: included items first, parent second |
| `vars` | Merged: parent key wins on conflict (no error) |

Rationale: for unique scalar keys, silent override is dangerous
(hard to debug). Sequences can be composed. vars are naturally
overridable (the parent provides the environment-specific value).

**Var expansion across file boundaries:**
Each file expands its own vars block. An included file uses
its own vars during transpilation of its own content. Parent
vars are not injected into included files. The merged vars
block (parent wins on conflict) is what the final document
sees after merge.

**Parser change:** The parser must read `include:` at the
top level and resolve paths relative to the parent file's
directory. The resolved files are parsed and merged before
validation. stdin mode (`--stdin`) does not support includes
(no file path context) — `include:` in a stdin document is
a parse error.

**New diagnostic codes:**

| Code | Description |
|---|---|
| `E115` | include file not found |
| `E116` | circular include detected |
| `E117` | include key conflict (scalar key defined in both files) |

**SPEC.md change:** Remove the "Includes/imports deferred"
bullet from Deferred Work; add `include:` to Supported Top-Level
Keys and a new "Multi-file Includes" section.

### Sequence

This is a multi-packet track. Estimated 3 packets:

**Packet A — Parser and file resolution**
1. Add `include` to the top-level key set in the parser
2. Add `document.include` field (sequence of paths) to the AST
3. Implement `resolve_includes(document, base_path) -> Result<Document>`
   that loads, parses, and merges included files
4. Detect and error on circular includes
5. E115 (file not found) and E116 (circular)

**Packet B — Merge semantics and validator**
1. Implement merge logic per the rules table above
2. E117 (key conflict) in the validator
3. vars merge: parent wins
4. Sequence concatenation and tools deduplication
5. Update `validate_document` to run after merge

**Packet C — Tests and SPEC update**
1. Add conformance fixtures: `examples/includes/` directory
2. Add 8+ conformance tests covering all merge cases
3. SPEC.md: add `include:` to key list and new section
4. README.md: document multi-file workflow
5. examples/: add multi-file example set

### Estimated size

**Large** — 3 packets. The parser change is non-trivial (file
resolution, circular detection). The merge semantics have edge
cases. The conformance suite needs substantial new fixtures.

### v4 contract commitment

By end of Track I:
- `include:` at top level takes a sequence of relative .llm paths
- Files are merged before validation and transpilation
- Circular includes are detected and rejected (E116)
- Missing files are rejected (E115)
- Scalar key conflicts are rejected (E117)
- vars merge: parent wins on conflict
- Sequence keys (memory, tools, constraints) are concatenated
- Each file's vars expand independently; merged vars apply to
  the final composed document
- `--stdin` mode rejects documents with `include:` keys

---

## What is explicitly NOT in v4

**Full LSP server** — Not needed. The spawn-and-parse approach
covers all five high-value IntelliSense features (diagnostics,
completion, hover, definition, inlay hints). A full LSP would
add rename-symbol and workspace-wide indexing. Rename is useful
but niche; workspace indexing only becomes necessary after
multi-file includes (Track I) has real adoption. Defer to v5
or later if demand materializes.

**vars chaining / conditional expansion** — No concrete use
case that flat single-level vars cannot solve. Chaining (`a: "{b}"`)
was deliberately made non-recursive in v3. Conditionals require
an expression language, which is substantial complexity with
speculative value. Deferred indefinitely.

**CI bench regression percentage threshold** — The constant-based
approach already catches regressions exactly. A percentage
threshold (e.g., "fail if token count increases by >5%") adds
complexity for no additional safety. Deferred indefinitely.

**Additional provider profiles** — The generic V0 format works
for all non-Anthropic providers that accept plain-text prompts.
There is no provider currently on the market that requires a
different encoding from V0 generic. Add new profiles only in
response to a concrete provider requirement.

**Python / JS programmatic API** — Adoption via CLI subprocess
is sufficient for the current stage. Building a PyO3 binding or
WASM module is significant engineering work. The right time is
after the CLI interface is fully stable and there is demonstrated
demand. Not v4 scope.

---

## Track Sequencing

**Recommended order: G → H → I**

**Track G first.** One packet, no dependencies, no risk.
Makes the v3 work distributable. Anyone can now install
the extension with a .vsix rather than cloning the repo.
CI artifact upload means every main-branch push produces
a fresh .vsix automatically.

**Track H second.** One packet, builds directly on the
`parseVarsBlock` helper from v3 Track F. Completes the
IntelliSense story (the one visible gap: var values are
invisible while authoring). No Rust changes.

**Track I third.** Multi-packet, the largest work item.
Benefits from G and H being settled so the CI pipeline
and editor experience are solid before the parser changes
land. Track I also benefits from vars expansion (v3 Track D)
being stable — shared vars files only make sense now that
vars actually do something.

Do NOT start Track I until Track G is complete —
includes require file-system access that affects the
`.github/workflows/ci.yml` test matrix (stdin mode must
be confirmed to reject `include:` gracefully).

---

## v4 Success Criteria

v4 is complete when:

- [ ] `npm run package` in `editors/vscode/` produces
      `llm-vscode-0.1.0.vsix` without errors
- [ ] CI uploads the `.vsix` as a build artifact on main
- [ ] Expanded `{var}` values appear as inlay hints in VS Code
      without any user action
- [ ] `include:` key accepts a sequence of relative .llm paths
- [ ] Included files are merged before validation and transpilation
- [ ] E115, E116, E117 are defined, implemented, and
      conformance-tested
- [ ] `--stdin` mode rejects `include:` keys cleanly
- [ ] SPEC.md: `include:` added to key list; multi-file section added
- [ ] CHANGELOG.md v4 entry complete and dated
