'use strict';

const vscode = require('vscode');
const cp = require('child_process');

function getFormatterPath() {
    const config = vscode.workspace.getConfiguration('llm');
    const configured = config.get('formatterPath');
    if (configured && configured.trim().length > 0) {
        return configured.trim();
    }
    return 'llm_format';
}

function activate(context) {
    const selector = { language: 'llm', scheme: 'file' };

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
}

function deactivate() {}

module.exports = { activate, deactivate };
