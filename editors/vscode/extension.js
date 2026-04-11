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

    // --- Live diagnostics (new) ---

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
}

function deactivate() {}

module.exports = { activate, deactivate };
