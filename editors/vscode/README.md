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

## Formatter

The extension registers a document formatter for `.llm` files.

To use it:

- Build or install the `llm_format` binary (`cargo build --release`)
- Either add the binary to your PATH, or set `llm.formatterPath`
  in VS Code settings to the full binary path
- Format with `Shift+Alt+F` (Windows/Linux) or `Shift+Option+F` (Mac),
  or enable Format on Save in VS Code settings

If the binary is not found, a one-time informational message is shown.

## What It Does Not Yet Do

- no language server
- no live parse or validation diagnostics
- no completion, hover, or code actions
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
