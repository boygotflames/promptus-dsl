# VS Code Marketplace Publishing Guide

This document describes how to publish the `.llm` VS Code extension
to the Visual Studio Code Marketplace.

## Prerequisites

1. A Visual Studio Marketplace publisher account registered at
   https://marketplace.visualstudio.com/manage
2. A Personal Access Token (PAT) from Azure DevOps with
   **Marketplace → Manage** scope
3. Node.js 18+ installed
4. `@vscode/vsce` installed (`npm install -g @vscode/vsce` or use
   the local devDependency via `npx vsce`)

## One-time publisher setup

The `package.json` currently declares `"publisher": "llm-format"`.
This publisher ID must match your registered Marketplace publisher
account. If the account name differs, update `package.json` before
packaging.

## Building the .vsix locally

From the `editors/vscode/` directory:

```bash
npm install          # installs @vscode/vsce as devDependency
npm run package      # runs: vsce package
```

This produces `llm-vscode-0.1.0.vsix` in the current directory.

To verify the .vsix before publishing, install it locally in VS Code:

```
code --install-extension llm-vscode-0.1.0.vsix
```

## Publishing to the Marketplace

```bash
npx vsce publish --pat <YOUR_PAT>
```

Or set the PAT as an environment variable:

```bash
VSCE_PAT=<YOUR_PAT> npx vsce publish
```

## GitHub Actions — automated packaging

The workflow `.github/workflows/vscode-package.yml` runs on every
push to `main` or `LLM-Promptus` that touches `editors/vscode/`.
It produces a `.vsix` artifact named `llm-vscode-vsix` available
for download from the Actions run page.

To automate marketplace publishing from CI, add your PAT as a
GitHub Actions secret named `VSCE_PAT` and extend the workflow:

```yaml
- name: Publish to Marketplace
  if: github.ref == 'refs/heads/main'
  run: npx vsce publish --pat ${{ secrets.VSCE_PAT }}
  working-directory: editors/vscode
```

## Version bumping

Before each release, update `"version"` in `editors/vscode/package.json`.
Follow semantic versioning: `MAJOR.MINOR.PATCH`.

The .vsix filename reflects the version: `llm-vscode-0.2.0.vsix`.

## Icon note

The extension icon is at `editors/vscode/images/Promptus 128x128.png`.
VS Code Marketplace requires a PNG icon at 128×128 pixels. The SVG
source is at `editors/vscode/images/Promptus.svg`.
