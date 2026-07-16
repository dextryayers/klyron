import * as vscode from 'vscode';
import * as path from 'path';
import * as cp from 'child_process';

let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Klyron');

    const config = vscode.workspace.getConfiguration('klyron');
    const binaryPath = config.get<string>('binaryPath', 'klyron');

    context.subscriptions.push(
        vscode.commands.registerCommand('klyron.run', () => runFile(binaryPath)),
        vscode.commands.registerCommand('klyron.test', () => runTests(binaryPath)),
        vscode.commands.registerCommand('klyron.build', () => buildProject(binaryPath)),
        vscode.commands.registerCommand('klyron.dev', () => startDevServer(binaryPath)),
        vscode.commands.registerCommand('klyron.format', () => formatFile(binaryPath)),
        vscode.commands.registerCommand('klyron.lint', () => lintFile(binaryPath)),
    );

    // Auto-format on save
    if (config.get<boolean>('formatOnSave', true)) {
        context.subscriptions.push(
            vscode.workspace.onDidSaveTextDocument((doc) => {
                if (doc.languageId === 'javascript' || doc.languageId === 'typescript') {
                    formatFile(binaryPath);
                }
            })
        );
    }

    // Auto-start dev server
    if (config.get<boolean>('autoStartDev', false) && vscode.workspace.workspaceFolders) {
        startDevServer(binaryPath);
    }

    outputChannel.appendLine('Klyron extension activated');
}

function getActiveFilePath(): string | undefined {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active file');
        return undefined;
    }
    return editor.document.fileName;
}

function getWorkspaceRoot(): string | undefined {
    return vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
}

function execKlyron(binaryPath: string, args: string[], cwd?: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const workspaceRoot = cwd || getWorkspaceRoot();
        const options: cp.ExecOptions = {
            cwd: workspaceRoot,
            maxBuffer: 10 * 1024 * 1024,
        };

        cp.exec(`${binaryPath} ${args.join(' ')}`, options, (error, stdout, stderr) => {
            if (error) {
                reject(new Error(stderr || error.message));
                return;
            }
            resolve(stdout.trim());
        });
    });
}

async function runFile(binaryPath: string) {
    const filePath = getActiveFilePath();
    if (!filePath) return;

    outputChannel.show();
    outputChannel.appendLine(`Running: ${filePath}`);

    try {
        const result = await execKlyron(binaryPath, ['run', `"${filePath}"`]);
        outputChannel.appendLine(result);
    } catch (error: any) {
        outputChannel.appendLine(`Error: ${error.message}`);
        vscode.window.showErrorMessage(`Klyron run failed: ${error.message}`);
    }
}

async function runTests(binaryPath: string) {
    const workspaceRoot = getWorkspaceRoot();
    if (!workspaceRoot) return;

    outputChannel.show();
    outputChannel.appendLine('Running tests...');

    try {
        const result = await execKlyron(binaryPath, ['test']);
        outputChannel.appendLine(result);
    } catch (error: any) {
        outputChannel.appendLine(`Tests failed: ${error.message}`);
    }
}

async function buildProject(binaryPath: string) {
    const workspaceRoot = getWorkspaceRoot();
    if (!workspaceRoot) return;

    outputChannel.show();
    outputChannel.appendLine('Building project...');

    try {
        const result = await execKlyron(binaryPath, ['build']);
        outputChannel.appendLine(result);
        vscode.window.showInformationMessage('Build complete');
    } catch (error: any) {
        outputChannel.appendLine(`Build failed: ${error.message}`);
        vscode.window.showErrorMessage(`Klyron build failed: ${error.message}`);
    }
}

async function startDevServer(binaryPath: string) {
    const workspaceRoot = getWorkspaceRoot();
    if (!workspaceRoot) return;

    outputChannel.show();
    outputChannel.appendLine('Starting dev server...');

    try {
        await execKlyron(binaryPath, ['dev', '--open']);
        outputChannel.appendLine('Dev server started');
    } catch (error: any) {
        outputChannel.appendLine(`Dev server error: ${error.message}`);
    }
}

async function formatFile(binaryPath: string) {
    const filePath = getActiveFilePath();
    if (!filePath) return;

    try {
        const result = await execKlyron(binaryPath, ['format', `"${filePath}"`]);
        const editor = vscode.window.activeTextEditor;
        if (editor && result) {
            editor.edit((editBuilder) => {
                const fullRange = new vscode.Range(
                    editor.document.positionAt(0),
                    editor.document.positionAt(editor.document.getText().length)
                );
                editBuilder.replace(fullRange, result);
            });
        }
    } catch (error: any) {
        vscode.window.showErrorMessage(`Format failed: ${error.message}`);
    }
}

async function lintFile(binaryPath: string) {
    const filePath = getActiveFilePath();
    if (!filePath) return;

    try {
        const result = await execKlyron(binaryPath, ['lint', `"${filePath}"`]);
        outputChannel.show();
        outputChannel.appendLine(result || 'No issues found');
    } catch (error: any) {
        outputChannel.appendLine(`Lint error: ${error.message}`);
    }
}

export function deactivate() {
    outputChannel?.dispose();
}
