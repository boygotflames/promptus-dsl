# VS Code Extension Distribution Guide

> Automated VS Code Marketplace publication is deferred indefinitely.
> The extension is distributed as a `.vsix` package through local
> packaging and, when available, GitHub Release artifacts.
> No `VSCE_PAT` or Azure DevOps Marketplace token is required for the
> current distribution path.

This guide covers the current `.vsix`-based distribution workflow for
the `.llm` VS Code extension under `editors/vscode/`.

## Package Metadata

The repository metadata in
[editors/vscode/package.json](../editors/vscode/package.json) should
remain:

```json
"repository": {
  "type": "git",
  "url": "https://github.com/boygotflames/promptus-dsl",
  "directory": "editors/vscode"
}
```

That URL is part of the extension package metadata and should continue
to point at this repository, even though Marketplace publication is not
currently active.

## Local Packaging

From the extension directory:

```bash
cd editors/vscode
npm install
npx vsce package --no-dependencies
```

Expected output:

```text
llm-vscode-1.0.0.vsix
```

## Manual Installation in VS Code

After packaging, install the extension manually:

```bash
code --install-extension llm-vscode-1.0.0.vsix
```

You can also install through the VS Code UI:

1. Open the Extensions panel.
2. Open the `...` menu.
3. Choose `Install from VSIX...`.
4. Select the generated `.vsix` file.

## CI Packaging Workflow

[.github/workflows/vscode-package.yml](../.github/workflows/vscode-package.yml)
remains the active validation and packaging workflow.

It:

- installs `@vscode/vsce`
- packages the extension with `npx vsce package --no-dependencies`
- uploads the resulting `.vsix` as a workflow artifact

That workflow is the current automated packaging path for normal repo
changes.

## Release Artifacts

The preferred public distribution model is:

- local `.vsix` packaging for direct/manual installation
- GitHub Actions `.vsix` artifacts from the packaging workflow
- GitHub Release assets when the release workflow is extended to ship
  the packaged `.vsix`

At the moment, the release-binary workflow is separate from the VS Code
packaging workflow, so `.vsix` release attachment should be treated as
an optional future improvement rather than an active guarantee.

## Historical / Deferred Publisher Context

The extension package still carries the publisher field:

- `MjirihYoussef`

Treat that as historical/deferred metadata for future Marketplace use,
not as a current setup requirement. No Azure DevOps PAT creation, no
Marketplace onboarding, and no first-publish steps are part of the
current distribution plan.

## Known Limitation

The extension implementation surface in `extension.js` is broader than
the package help text fully describes.

Specifically:

- `include:` is implemented in the core language and parser
- extension completion and hover documentation do not yet fully surface
  `include:` alongside the older top-level key set

This is a documentation/metadata gap, not a packaging blocker.
