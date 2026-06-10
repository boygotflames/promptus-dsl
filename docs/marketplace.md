# VS Code Marketplace Publishing Guide

This guide covers the repository's VS Code Marketplace publication
flow for the `.llm` extension package under `editors/vscode/`.

## Publisher

- Publisher ID: `MjirihYoussef`

The `publisher` field in [editors/vscode/package.json](../editors/vscode/package.json)
must continue to match the Marketplace publisher account.

## Required Secret

GitHub Actions requires this repository secret:

- `VSCE_PAT`

This secret is consumed by the tag-triggered publication workflow in
[.github/workflows/vscode-publish.yml](../.github/workflows/vscode-publish.yml).

## Azure DevOps PAT Scope

The PAT used for `VSCE_PAT` must be created with this scope:

- `Marketplace -> Manage`

Without that scope, `vsce publish` will fail even if the secret exists.

## Local Package Verification

Use the extension directory directly:

```bash
cd editors/vscode
npm install
npx vsce package --no-dependencies
```

This is the local verification path for `.vsix` packaging before any
tagged publish flow is used.

## CI Workflows

Two workflows now serve different purposes:

### 1. Packaging validation

[.github/workflows/vscode-package.yml](../.github/workflows/vscode-package.yml)
stays in place as the normal PR/main packaging check.

It:

- installs `@vscode/vsce`
- packages the extension with `npx vsce package --no-dependencies`
- uploads the resulting `.vsix` as a workflow artifact

Use this workflow to confirm packaging remains healthy during normal
development.

### 2. Tag-triggered Marketplace publication

[.github/workflows/vscode-publish.yml](../.github/workflows/vscode-publish.yml)
publishes the extension when a Git tag matching `v*` is pushed.

Flow:

1. A tag such as `v5.0.1` is pushed.
2. The workflow checks out the repo in `editors/vscode/`.
3. It installs `@vscode/vsce`.
4. It derives the extension version from the tag name by stripping the
   leading `v`.
5. It rewrites `editors/vscode/package.json` with that version for the
   publish job.
6. It runs:

```bash
npx vsce publish --pat ${{ secrets.VSCE_PAT }} --no-dependencies
```

This packet does not publish anything manually and does not create or
push a tag.

## Known Limitation

The extension currently exposes a broader implementation surface in
`extension.js` than its package metadata and help text fully describe.

Specifically:

- `include:` is implemented in the core language and parser
- extension completion and hover documentation do not yet fully surface
  `include:` alongside the older top-level key set

This is a documentation/metadata limitation, not a Marketplace
publishing blocker.
