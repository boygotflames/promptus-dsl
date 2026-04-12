'use strict';

const vscode = require('vscode');
const cp = require('child_process');

// Track whether the binary-not-found message has been shown for diagnostics,
// so we don't spam the user on every keystroke.
let binaryNotFoundShownForDiagnostics = false;

function getFormatterPath() {
    const config = vscode.workspace.getConfiguration('llm');
    const configured = config.get('formatterPath');
    if (configured && configured.trim().length > 0) {
        return configured.trim();
    }
    return 'llm_format';
}

// Debounce helper — no external dependencies.
function debounce(fn, ms) {
    let timer;
    return function (...args) {
        clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, args), ms);
    };
}

// Parse stderr output from `llm_format validate --stdin` into
// an array of vscode.Diagnostic objects.
//
// The Rust CLI emits lines in this format:
//   syntax error at LINE:COL: [ECODE] MESSAGE
//   semantic error at LINE:COL: [ECODE] MESSAGE
//
// Line and column are 1-based; VS Code Positions are 0-based.
function parseDiagnostics(stderr) {
    const diagRegex =
        /^(syntax|semantic) error at (\d+):(\d+): \[([EW]\d+)\] (.+)$/;
    const diagnostics = [];

    for (const line of stderr.split('\n')) {
        const match = line.match(diagRegex);
        if (!match) continue;

        const [, kind, lineStr, colStr, code, message] = match;
        const lineNum = parseInt(lineStr, 10) - 1;   // 0-based
        const colNum = parseInt(colStr, 10) - 1;     // 0-based

        // Use a single-character range at the reported position.
        const position = new vscode.Position(lineNum, colNum);
        const range = new vscode.Range(position, position);

        const severity =
            kind === 'syntax' || kind === 'semantic'
                ? vscode.DiagnosticSeverity.Error
                : vscode.DiagnosticSeverity.Warning;

        const diag = new vscode.Diagnostic(range, message, severity);
        diag.source = 'llm';
        diag.code = code;
        diagnostics.push(diag);
    }

    return diagnostics;
}

// Run `llm_format validate --stdin`, pipe the document text, and update
// the diagnostic collection for the document's URI.
function validateDocument(document, diagnosticCollection) {
    if (document.languageId !== 'llm') return;

    const binaryPath = getFormatterPath();
    const text = document.getText();

    let child;
    try {
        child = cp.spawn(binaryPath, ['validate', '--stdin'], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });
    } catch (err) {
        if (err.code === 'ENOENT') {
            if (!binaryNotFoundShownForDiagnostics) {
                binaryNotFoundShownForDiagnostics = true;
                vscode.window.showInformationMessage(
                    'llm_format binary not found. ' +
                    'Build with `cargo build --release` and add to PATH, ' +
                    'or set llm.formatterPath in VS Code settings.'
                );
            }
        }
        diagnosticCollection.set(document.uri, []);
        return;
    }

    let stderr = '';
    child.stderr.on('data', (data) => {
        stderr += data.toString();
    });

    child.on('error', (err) => {
        if (err.code === 'ENOENT') {
            if (!binaryNotFoundShownForDiagnostics) {
                binaryNotFoundShownForDiagnostics = true;
                vscode.window.showInformationMessage(
                    'llm_format binary not found. ' +
                    'Build with `cargo build --release` and add to PATH, ' +
                    'or set llm.formatterPath in VS Code settings.'
                );
            }
        }
        diagnosticCollection.set(document.uri, []);
    });

    child.on('close', () => {
        const diagnostics = parseDiagnostics(stderr);
        diagnosticCollection.set(document.uri, diagnostics);
    });

    // Pipe the current editor buffer content to the binary's stdin.
    child.stdin.write(text);
    child.stdin.end();
}

// ── IntelliSense data ─────────────────────────────────────────────────────────

const TOP_LEVEL_KEY_DOCS = {
    agent: {
        required: true,
        type: 'scalar',
        description: 'Declares the agent identity for this document.'
    },
    system: {
        required: true,
        type: 'scalar or mapping',
        description: 'Provides system-level instructions for the model.'
    },
    user: {
        required: false,
        type: 'scalar or mapping',
        description: 'Contains the user request or conversational context.'
    },
    memory: {
        required: false,
        type: 'sequence of scalars',
        description: 'Lists memory items available to the agent.'
    },
    tools: {
        required: false,
        type: 'sequence of scalars',
        description: 'Declares tools available for the agent to use.'
    },
    output: {
        required: false,
        type: 'scalar or mapping',
        description: 'Specifies the expected output format or schema.'
    },
    constraints: {
        required: false,
        type: 'sequence of scalars',
        description: 'Lists behavioral constraints the agent must follow.'
    },
    vars: {
        required: false,
        type: 'mapping of scalars',
        description: 'Defines template variables for use as {var_name} references.'
    }
};

// Snippet text for top-level key completions.
const TOP_LEVEL_KEY_SNIPPETS = {
    agent:       'agent: $1',
    system:      'system:\n  $1',
    user:        'user: $1',
    memory:      'memory:\n  - $1',
    tools:       'tools:\n  - $1',
    output:      'output: $1',
    constraints: 'constraints:\n  - $1',
    vars:        'vars:\n  $1: $2'
};

const TOP_LEVEL_KEY_DETAILS = {
    agent:       'Agent identity — required',
    system:      'System instructions — required',
    user:        'User request or context',
    memory:      'Memory items (sequence)',
    tools:       'Available tools (sequence)',
    output:      'Output format or schema',
    constraints: 'Behavioral constraints (sequence)',
    vars:        'Template variables'
};

const SYSTEM_USER_SUBKEYS = [
    { label: 'role',         detail: 'Agent role or persona' },
    { label: 'objective',    detail: 'Primary objective' },
    { label: 'tone',         detail: 'Response tone or style' },
    { label: 'format',       detail: 'Response format' },
    { label: 'context',      detail: 'Background context' },
    { label: 'instructions', detail: 'Detailed instructions' }
];

// ── Vars parsing ──────────────────────────────────────────────────────────────

// Extract all var key→value pairs from a document's vars: block.
// Returns a Map<string, string>.
function parseVarsBlock(text) {
    const vars = new Map();
    const lines = text.split('\n');
    let inVars = false;
    for (const line of lines) {
        if (/^vars:/.test(line)) {
            inVars = true;
            continue;
        }
        if (inVars) {
            // Top-level key ends the vars block
            if (/^\S/.test(line) && line.trim().length > 0 && !line.startsWith(' ')) {
                break;
            }
            // Match indented key: value pairs
            const m = line.match(/^  ([\w][\w-]*):\s*(.*)$/);
            if (m) {
                vars.set(m[1], m[2].trim());
            }
        }
    }
    return vars;
}

// ── Completion ────────────────────────────────────────────────────────────────

function getCompletionItems(document, position) {
    const lineText = document.lineAt(position.line).text;
    const textUpToCursor = lineText.substring(0, position.character);
    const docText = document.getText();

    // --- {var} reference completion inside a quoted string ---
    // Trigger: cursor is after a `{` that follows a `"` on the same line
    const openBraceIdx = textUpToCursor.lastIndexOf('{');
    if (openBraceIdx !== -1) {
        const textBeforeBrace = textUpToCursor.substring(0, openBraceIdx);
        const lastQuote = textBeforeBrace.lastIndexOf('"');
        if (lastQuote !== -1) {
            // We are inside a quoted string after a `{` — offer var names
            const vars = parseVarsBlock(docText);
            if (vars.size > 0) {
                const items = [];
                for (const [key, value] of vars) {
                    const item = new vscode.CompletionItem(
                        key,
                        vscode.CompletionItemKind.Variable
                    );
                    // `{` is already typed; insert just `key}`
                    item.insertText = `${key}}`;
                    item.detail = value ? `= "${value}"` : 'var reference';
                    item.documentation = new vscode.MarkdownString(
                        `**{${key}}** — defined in \`vars\`\n\nValue: \`${value}\``
                    );
                    items.push(item);
                }
                return items;
            }
        }
    }

    // --- Sub-key completion inside system: or user: block ---
    if (/^\s+/.test(lineText) && position.character >= 2) {
        // Line is indented — check if we're inside system: or user:
        for (let i = position.line - 1; i >= 0; i--) {
            const prevLine = document.lineAt(i).text;
            if (/^(system|user):/.test(prevLine)) {
                return SYSTEM_USER_SUBKEYS.map(({ label, detail }) => {
                    const item = new vscode.CompletionItem(
                        label,
                        vscode.CompletionItemKind.Field
                    );
                    item.insertText = new vscode.SnippetString(`${label}: $1`);
                    item.detail = detail;
                    return item;
                });
            }
            // Stop if we hit another top-level key
            if (/^\S/.test(prevLine) && prevLine.trim().length > 0) {
                break;
            }
        }
    }

    // --- Top-level key completion at column 0 ---
    if (position.character === 0 || !/.*:/.test(textUpToCursor)) {
        return Object.keys(TOP_LEVEL_KEY_DOCS).map((key) => {
            const item = new vscode.CompletionItem(
                key,
                vscode.CompletionItemKind.Keyword
            );
            item.insertText = new vscode.SnippetString(TOP_LEVEL_KEY_SNIPPETS[key]);
            item.detail = TOP_LEVEL_KEY_DETAILS[key];
            const doc = TOP_LEVEL_KEY_DOCS[key];
            item.documentation = new vscode.MarkdownString(
                `**\`${key}\`** — ${doc.required ? 'required' : 'optional'}\n\n` +
                `Type: \`${doc.type}\`\n\n${doc.description}`
            );
            return item;
        });
    }

    return [];
}

// ── Hover ─────────────────────────────────────────────────────────────────────

function getHoverInfo(document, position) {
    const lineText = document.lineAt(position.line).text;
    const docText = document.getText();

    // --- {var_name} hover ---
    // Check if cursor is on a {var_name} pattern
    const varRefRegex = /\{([\w][\w-]*)\}/g;
    let m;
    while ((m = varRefRegex.exec(lineText)) !== null) {
        const start = m.index;
        const end = m.index + m[0].length;
        if (position.character >= start && position.character <= end) {
            const varName = m[1];
            const vars = parseVarsBlock(docText);
            const range = new vscode.Range(
                position.line, start,
                position.line, end
            );
            if (vars.has(varName)) {
                const value = vars.get(varName);
                return new vscode.Hover(
                    new vscode.MarkdownString(
                        `**{${varName}}** = \`"${value}"\`\n\nDefined in \`vars\``
                    ),
                    range
                );
            } else {
                return new vscode.Hover(
                    new vscode.MarkdownString(
                        `**{${varName}}** — ⚠ undefined var reference (\`E114\`)`
                    ),
                    range
                );
            }
        }
    }

    // --- Top-level key hover (only at column 0) ---
    const topLevelMatch = lineText.match(/^(\w[\w-]*):/);
    if (topLevelMatch) {
        const key = topLevelMatch[1];
        const doc = TOP_LEVEL_KEY_DOCS[key];
        if (doc) {
            const range = new vscode.Range(
                position.line, 0,
                position.line, key.length
            );
            return new vscode.Hover(
                new vscode.MarkdownString(
                    `**\`${key}\`** — ${doc.required ? 'required' : 'optional'}\n\n` +
                    `Type: \`${doc.type}\`\n\n${doc.description}`
                ),
                range
            );
        }
    }

    return null;
}

// ── Definition ────────────────────────────────────────────────────────────────

function getDefinition(document, position) {
    const lineText = document.lineAt(position.line).text;
    const docText = document.getText();

    // Find a {var_name} pattern under the cursor
    const varRefRegex = /\{([\w][\w-]*)\}/g;
    let m;
    while ((m = varRefRegex.exec(lineText)) !== null) {
        const start = m.index;
        const end = m.index + m[0].length;
        if (position.character >= start && position.character <= end) {
            const varName = m[1];
            // Find the vars: block, then the var_name: line
            const lines = docText.split('\n');
            let inVars = false;
            for (let i = 0; i < lines.length; i++) {
                if (/^vars:/.test(lines[i])) {
                    inVars = true;
                    continue;
                }
                if (inVars) {
                    if (/^\S/.test(lines[i]) && lines[i].trim().length > 0) {
                        break; // left vars block
                    }
                    const keyMatch = lines[i].match(/^  ([\w][\w-]*):/);
                    if (keyMatch && keyMatch[1] === varName) {
                        const targetPos = new vscode.Position(i, 2);
                        return new vscode.Location(document.uri, targetPos);
                    }
                }
            }
            return null;
        }
    }

    return null;
}

// ── Inlay hints ───────────────────────────────────────────────────────────────

function getInlayHints(document, range) {
    const vars = parseVarsBlock(document.getText());
    if (vars.size === 0) return [];

    const hints = [];
    const varRefPattern = /\{([A-Za-z_][A-Za-z0-9_-]*)\}/g;

    for (let i = range.start.line; i <= range.end.line; i++) {
        const lineText = document.lineAt(i).text;
        let match;
        varRefPattern.lastIndex = 0;

        while ((match = varRefPattern.exec(lineText)) !== null) {
            const varName = match[1];
            if (!vars.has(varName)) continue; // undefined — squiggle handles it

            const value = vars.get(varName);
            const position = new vscode.Position(i, match.index + match[0].length);

            const hint = new vscode.InlayHint(
                position,
                `= "${value}"`,
                vscode.InlayHintKind.Type
            );
            hint.tooltip = new vscode.MarkdownString(
                `**${varName}** is defined in \`vars\` as \`"${value}"\``
            );
            hints.push(hint);
        }
    }

    return hints;
}

// ── Extension entry point ─────────────────────────────────────────────────────

function activate(context) {
    const selector = { language: 'llm', scheme: 'file' };

    // --- Formatter (existing) ---

    const provider = vscode.languages.registerDocumentFormattingEditProvider(
        selector,
        {
            provideDocumentFormattingEdits(document) {
                return new Promise((resolve) => {
                    const formatterPath = getFormatterPath();
                    const filePath = document.uri.fsPath;

                    cp.execFile(
                        formatterPath,
                        ['fmt', filePath],
                        { encoding: 'utf8' },
                        (error, stdout, stderr) => {
                            if (error) {
                                if (error.code === 'ENOENT') {
                                    vscode.window.showInformationMessage(
                                        'llm_format binary not found. ' +
                                        'Build with `cargo build --release` and add to PATH, ' +
                                        'or set llm.formatterPath in VS Code settings.'
                                    );
                                    resolve([]);
                                    return;
                                }
                                if (stderr && stderr.trim().length > 0) {
                                    vscode.window.showErrorMessage(
                                        `llm format error: ${stderr.trim()}`
                                    );
                                }
                                resolve([]);
                                return;
                            }

                            const fullRange = new vscode.Range(
                                document.lineAt(0).range.start,
                                document.lineAt(document.lineCount - 1).range.end
                            );
                            resolve([vscode.TextEdit.replace(fullRange, stdout)]);
                        }
                    );
                });
            }
        }
    );

    context.subscriptions.push(provider);

    // --- Live diagnostics (existing) ---

    const diagnosticCollection =
        vscode.languages.createDiagnosticCollection('llm');
    context.subscriptions.push(diagnosticCollection);

    const debouncedValidate = debounce((document) => {
        validateDocument(document, diagnosticCollection);
    }, 300);

    // Validate when a .llm file is opened.
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((document) => {
            validateDocument(document, diagnosticCollection);
        })
    );

    // Validate (debounced) as the user types.
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument((event) => {
            debouncedValidate(event.document);
        })
    );

    // Clear diagnostics when a .llm file is closed.
    context.subscriptions.push(
        vscode.workspace.onDidCloseTextDocument((document) => {
            diagnosticCollection.delete(document.uri);
        })
    );

    // Validate any already-open .llm documents at activation time.
    vscode.workspace.textDocuments.forEach((document) => {
        validateDocument(document, diagnosticCollection);
    });

    // --- Completion ---

    const completionProvider =
        vscode.languages.registerCompletionItemProvider(
            selector,
            {
                provideCompletionItems(document, position) {
                    return getCompletionItems(document, position);
                }
            },
            ':', ' ', '\n', '{'
        );
    context.subscriptions.push(completionProvider);

    // --- Hover ---

    const hoverProvider =
        vscode.languages.registerHoverProvider(
            selector,
            {
                provideHover(document, position) {
                    return getHoverInfo(document, position);
                }
            }
        );
    context.subscriptions.push(hoverProvider);

    // --- Definition (go-to-definition for {var} refs) ---

    const definitionProvider =
        vscode.languages.registerDefinitionProvider(
            selector,
            {
                provideDefinition(document, position) {
                    return getDefinition(document, position);
                }
            }
        );
    context.subscriptions.push(definitionProvider);

    // --- Inlay Hints ({var} expanded values as ghost text) ---

    const inlayHintsProvider =
        vscode.languages.registerInlayHintsProvider(
            selector,
            {
                provideInlayHints(document, range) {
                    return getInlayHints(document, range);
                }
            }
        );
    context.subscriptions.push(inlayHintsProvider);
}

function deactivate() {}

module.exports = { activate, deactivate };
