"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = require("path");
const vscode = require("vscode");
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
const child_process_1 = require("child_process");
let client;
let outputChannel;
function activate(context) {
    outputChannel = vscode.window.createOutputChannel('Ifa-Lang');
    outputChannel.appendLine('Ifa-Lang extension activating...');
    const config = vscode_1.workspace.getConfiguration('ifa');
    const lspEnabled = config.get('languageServer.enable', true);
    // Register commands
    registerCommands(context);
    // Register document formatting provider
    registerFormattingProvider(context);
    // Start LSP if enabled
    if (lspEnabled) {
        startLanguageServer(context);
    }
    // Register Debug Adapter
    const factory = new IfaDebugAdapterDescriptorFactory();
    context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory('ifa', factory));
    // Register Debug Configuration Provider
    context.subscriptions.push(vscode.debug.registerDebugConfigurationProvider('ifa', new IfaDebugConfigurationProvider()));
    outputChannel.appendLine('Ifa-Lang extension activated');
}
function registerCommands(context) {
    // Run current file
    context.subscriptions.push(vscode.commands.registerCommand('ifa.run', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'ifa') {
            vscode.window.showErrorMessage('Open an .ifa file first');
            return;
        }
        await editor.document.save();
        const filePath = editor.document.fileName;
        const ifaPath = vscode_1.workspace.getConfiguration('ifa').get('path', 'ifa');
        const terminal = vscode.window.createTerminal('Ifa');
        terminal.show();
        terminal.sendText(`${ifaPath} run "${filePath}"`);
    }));
    // Run in WASM sandbox
    context.subscriptions.push(vscode.commands.registerCommand('ifa.runSandboxWasm', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'ifa') {
            vscode.window.showErrorMessage('Open an .ifa file first');
            return;
        }
        await editor.document.save();
        const filePath = editor.document.fileName;
        const ifaPath = vscode_1.workspace.getConfiguration('ifa').get('path', 'ifa');
        const terminal = vscode.window.createTerminal('Ifa (WASM Sandbox)');
        terminal.show();
        terminal.sendText(`${ifaPath} run "${filePath}" --sandbox=wasm`);
    }));
    // Run in native sandbox
    context.subscriptions.push(vscode.commands.registerCommand('ifa.runSandboxNative', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'ifa') {
            vscode.window.showErrorMessage('Open an .ifa file first');
            return;
        }
        await editor.document.save();
        const filePath = editor.document.fileName;
        const ifaPath = vscode_1.workspace.getConfiguration('ifa').get('path', 'ifa');
        const terminal = vscode.window.createTerminal('Ifa (Native Sandbox)');
        terminal.show();
        terminal.sendText(`${ifaPath} run "${filePath}" --sandbox=native`);
    }));
    // Open REPL
    context.subscriptions.push(vscode.commands.registerCommand('ifa.repl', () => {
        const ifaPath = vscode_1.workspace.getConfiguration('ifa').get('path', 'ifa');
        const terminal = vscode.window.createTerminal('Ifa REPL');
        terminal.show();
        terminal.sendText(`${ifaPath} repl`);
    }));
    // Format document (manual command, kept for reference)
    context.subscriptions.push(vscode.commands.registerCommand('ifa.format', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'ifa')
            return;
        await editor.document.save();
        const filePath = editor.document.fileName;
        const ifaPath = vscode_1.workspace.getConfiguration('ifa').get('path', 'ifa');
        try {
            (0, child_process_1.execSync)(`${ifaPath} fmt "${filePath}" --unstable`, { timeout: 10000 });
            vscode.window.showInformationMessage('Syntactic harmony restored!');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Format failed: ${error.stderr?.toString() || error.message}`);
        }
    }));
    // Restart language server
    context.subscriptions.push(vscode.commands.registerCommand('ifa.restartServer', async () => {
        if (client) {
            await client.stop();
        }
        startLanguageServer(context);
        vscode.window.showInformationMessage('Ifa language server restarted');
    }));
}
function registerFormattingProvider(context) {
    context.subscriptions.push(vscode.languages.registerDocumentFormattingEditProvider('ifa', {
        async provideDocumentFormattingEdits(document) {
            const ifaPath = vscode_1.workspace.getConfiguration('ifa').get('path', 'ifa');
            // Write unsaved changes to disk so the CLI formatter can read them
            await document.save();
            const filePath = document.fileName;
            try {
                (0, child_process_1.execSync)(`${ifaPath} fmt "${filePath}" --unstable`, {
                    timeout: 15000,
                    cwd: path.dirname(filePath),
                    encoding: 'utf-8'
                });
                // Re-read the formatted file from disk
                const uri = vscode.Uri.file(filePath);
                const contentBuffer = await vscode.workspace.fs.readFile(uri);
                const formattedText = Buffer.from(contentBuffer).toString('utf-8');
                const fullRange = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
                return [vscode.TextEdit.replace(fullRange, formattedText)];
            }
            catch (error) {
                const msg = error.stderr?.toString() || error.message;
                outputChannel.appendLine(`Format error: ${msg}`);
                vscode.window.showErrorMessage(`Ifa format failed: ${msg}`);
                return [];
            }
        }
    }));
}
function startLanguageServer(context) {
    const config = vscode_1.workspace.getConfiguration('ifa');
    const serverPath = config.get('languageServer.path');
    // Find ifa executable
    const ifaPath = serverPath || findIfaExecutable();
    if (!ifaPath) {
        outputChannel.appendLine('Ifa language server not found - syntax highlighting only');
        return;
    }
    try {
        const traceLevel = config.get('trace.server', 'off');
        const serverOptions = {
            run: {
                command: ifaPath,
                args: ['lsp'],
                transport: node_1.TransportKind.stdio
            },
            debug: {
                command: ifaPath,
                args: ['lsp', '--debug'],
                transport: node_1.TransportKind.stdio
            }
        };
        const clientOptions = {
            documentSelector: [
                { scheme: 'file', language: 'ifa' }
            ],
            synchronize: {
                fileEvents: vscode_1.workspace.createFileSystemWatcher('**/*.ifa')
            },
            outputChannel: outputChannel,
            traceOutputChannel: outputChannel
        };
        client = new node_1.LanguageClient('ifa-lang', 'Ifa Language Server', serverOptions, clientOptions);
        // Set trace level from configuration
        client.setTrace(traceLevel);
        client.start().then(() => {
            outputChannel.appendLine('Ifa language server started');
        }).catch((error) => {
            outputChannel.appendLine(`Language server failed: ${error}`);
            vscode.window.showWarningMessage('Ifa language server not available. Syntax highlighting only.');
        });
        context.subscriptions.push(client);
    }
    catch (error) {
        outputChannel.appendLine(`Error starting language server: ${error}`);
    }
}
function findIfaExecutable() {
    // Check common locations for ifa binary
    const possiblePaths = [
        'ifa', // In PATH
        'C:\\Program Files\\ifa-lang\\bin\\ifa.exe',
        'C:\\ifa-lang\\bin\\ifa.exe',
        '/usr/local/bin/ifa',
        '/usr/bin/ifa',
        path.join(process.env.HOME || '', '.ifa', 'bin', 'ifa')
    ];
    // Return first path that might work (actual check happens at runtime)
    // In practice, the 'ifa' in PATH is the most common case
    return 'ifa';
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
class IfaDebugConfigurationProvider {
    resolveDebugConfiguration(folder, debugConfiguration) {
        // If no program is specified, default to the active editor
        if (!debugConfiguration.program) {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'ifa') {
                debugConfiguration.program = editor.document.fileName;
            }
        }
        return debugConfiguration;
    }
}
class IfaDebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(session, executable) {
        const config = vscode_1.workspace.getConfiguration('ifa');
        const ifaPath = config.get('path', 'ifa');
        // Use the program path from launch config if available
        const program = session.configuration.program;
        const args = ['debug'];
        if (program) {
            args.push('--file', program);
        }
        return new vscode.DebugAdapterExecutable(ifaPath, args);
    }
}
//# sourceMappingURL=extension.js.map