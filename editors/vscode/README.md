# `.llm` VS Code Support

This folder contains the first minimal VS Code package for `.llm` files.

## What It Does

- registers `.llm` as a language in VS Code
- associates `.llm` files with a TextMate grammar
- provides syntax highlighting for:
  - reserved top-level keys
  - mapping keys
  - bare scalars
  - single-quoted and double-quoted strings
  - list markers
  - `#` comments
- enables line comments and quote auto-closing
- provides live inline diagnostics as you type (300ms debounce)
  — requires `llm_format` binary on PATH or `llm.formatterPath`
  setting
- provides completion for top-level keys, sub-keys, and
  `{var}` references as you type
- provides hover documentation for keys and `{var}` references
- provides go-to-definition for `{var}` → `vars:` entry

## Formatter

The extension registers a document formatter for `.llm` files.

To use it:

- Build or install the `llm_format` binary (`cargo build --release`)
- Either add the binary to your PATH, or set `llm.formatterPath`
  in VS Code settings to the full binary path
- Format with `Shift+Alt+F` (Windows/Linux) or `Shift+Option+F` (Mac),
  or enable Format on Save in VS Code settings

If the binary is not found, a one-time informational message is shown.

When a file has validation errors, the formatter shows the diagnostic
output (including error codes) rather than applying edits.

## Live Validation

The extension validates `.llm` files as you type and shows inline
error squiggles.

Validation runs 300ms after you stop typing. It requires the same
`llm_format` binary as the formatter.

Error codes (e.g., `[E101]`) appear in the Problems panel alongside
the human-readable message. Hover over a squiggle to see the full
diagnostic.

## IntelliSense

The extension provides three IntelliSense features:

**Completion**
- Type at column 0 to get suggestions for all 8 top-level
  keys with snippet inserts
- Inside a `system:` or `user:` block, get common sub-key
  suggestions (`role`, `objective`, `tone`, etc.)
- Inside a quoted string after `{`, get completions for all
  vars defined in the current document

**Hover**
- Hover over any top-level key to see its type, required
  status, and a description
- Hover over a `{var_name}` reference to see its defined
  value, or a warning if it is undefined

**Go-to-Definition**
- Press `F12` (or right-click → Go to Definition) on a
  `{var_name}` reference to jump to its definition in the
  `vars:` block

## What It Does Not Yet Do

- no language server
- ✓ live validation: DiagnosticCollection + debounced
  onDidChangeTextDocument; squiggles appear within 300ms
  of stopping typing
- ✓ completion, hover, and go-to-definition: implemented via
  VS Code extension APIs (no language server required)
- ✓ formatter-on-save: DocumentFormattingEditProvider registered
  for the 'llm' language — requires llm_format binary on PATH or
  configured via llm.formatterPath setting
- no marketplace packaging or publishing workflow

## Icon

Icon source: `images/Promptus.svg` — export to `images/Promptus.png`
before marketplace publication. VS Code marketplace requires a PNG icon;
the SVG is stored here as the source of truth.

## Local Manual Verification

1. Open `D:\llm_format\editors\vscode` as the workspace folder in VS Code.
2. Press `F5` and launch `Run .llm Syntax Extension`.
3. In the Extension Development Host window, open `.llm` files such as:
   - repo fixture `examples/minimal.llm` from `..\..\examples\minimal.llm`
   - repo fixture `examples/quoted.llm` from `..\..\examples\quoted.llm`
   - repo fixture `examples/noncanonical/messy.llm` from `..\..\examples\noncanonical\messy.llm`
   - editor fixture `editors/vscode/fixtures/highlighting-sample.llm`
4. Confirm that the language mode is `llm` and that keys, strings, list markers, and comments are highlighted.

This extension package is intentionally self-contained and does not call into the Rust parser or validator.
