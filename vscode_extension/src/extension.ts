import * as path from 'path';
import * as vscode from 'vscode';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';
import { execSync } from 'child_process';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export function activate(context: ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Ifa-Lang');
    outputChannel.appendLine('Ifa-Lang extension activating...');

    const config = workspace.getConfiguration('ifa');
    const lspEnabled = config.get<boolean>('languageServer.enable', true);

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
    context.subscriptions.push(
        vscode.debug.registerDebugConfigurationProvider('ifa', new IfaDebugConfigurationProvider())
    );

    outputChannel.appendLine('Ifa-Lang extension activated');
}

function registerCommands(context: ExtensionContext) {
    // Run current file
    context.subscriptions.push(
        vscode.commands.registerCommand('ifa.run', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'ifa') {
                vscode.window.showErrorMessage('Open an .ifa file first');
                return;
            }

            await editor.document.save();
            const filePath = editor.document.fileName;
            const ifaPath = workspace.getConfiguration('ifa').get<string>('path', 'ifa');

            const terminal = vscode.window.createTerminal('Ifa');
            terminal.show();
            terminal.sendText(`${ifaPath} run "${filePath}"`);
        })
    );

    // Run in WASM sandbox
    context.subscriptions.push(
        vscode.commands.registerCommand('ifa.runSandboxWasm', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'ifa') {
                vscode.window.showErrorMessage('Open an .ifa file first');
                return;
            }

            await editor.document.save();
            const filePath = editor.document.fileName;
            const ifaPath = workspace.getConfiguration('ifa').get<string>('path', 'ifa');

            const terminal = vscode.window.createTerminal('Ifa (WASM Sandbox)');
            terminal.show();
            terminal.sendText(`${ifaPath} run "${filePath}" --sandbox=wasm`);
        })
    );

    // Run in native sandbox
    context.subscriptions.push(
        vscode.commands.registerCommand('ifa.runSandboxNative', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'ifa') {
                vscode.window.showErrorMessage('Open an .ifa file first');
                return;
            }

            await editor.document.save();
            const filePath = editor.document.fileName;
            const ifaPath = workspace.getConfiguration('ifa').get<string>('path', 'ifa');

            const terminal = vscode.window.createTerminal('Ifa (Native Sandbox)');
            terminal.show();
            terminal.sendText(`${ifaPath} run "${filePath}" --sandbox=native`);
        })
    );

    // Open REPL
    context.subscriptions.push(
        vscode.commands.registerCommand('ifa.repl', () => {
            const ifaPath = workspace.getConfiguration('ifa').get<string>('path', 'ifa');
            const terminal = vscode.window.createTerminal('Ifa REPL');
            terminal.show();
            terminal.sendText(`${ifaPath} repl`);
        })
    );

    // Format document (manual command, kept for reference)
    context.subscriptions.push(
        vscode.commands.registerCommand('ifa.format', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'ifa') return;

            await editor.document.save();
            const filePath = editor.document.fileName;
            const ifaPath = workspace.getConfiguration('ifa').get<string>('path', 'ifa');

            try {
                execSync(`${ifaPath} fmt "${filePath}" --unstable`, { timeout: 10000 });
                vscode.window.showInformationMessage('Syntactic harmony restored!');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Format failed: ${error.stderr?.toString() || error.message}`);
            }
        })
    );

    // Restart language server
    context.subscriptions.push(
        vscode.commands.registerCommand('ifa.restartServer', async () => {
            if (client) {
                await client.stop();
            }
            startLanguageServer(context);
            vscode.window.showInformationMessage('Ifa language server restarted');
        })
    );
}

function registerFormattingProvider(context: ExtensionContext) {
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider('ifa', {
            async provideDocumentFormattingEdits(document: vscode.TextDocument): Promise<vscode.TextEdit[]> {
                const ifaPath = workspace.getConfiguration('ifa').get<string>('path', 'ifa');

                // Write unsaved changes to disk so the CLI formatter can read them
                await document.save();

                const filePath = document.fileName;
                try {
                    execSync(`${ifaPath} fmt "${filePath}" --unstable`, {
                        timeout: 15000,
                        cwd: path.dirname(filePath),
                        encoding: 'utf-8'
                    });

                    // Re-read the formatted file from disk
                    const uri = vscode.Uri.file(filePath);
                    const contentBuffer = await vscode.workspace.fs.readFile(uri);
                    const formattedText = Buffer.from(contentBuffer).toString('utf-8');

                    const fullRange = new vscode.Range(
                        document.positionAt(0),
                        document.positionAt(document.getText().length)
                    );

                    return [vscode.TextEdit.replace(fullRange, formattedText)];
                } catch (error: any) {
                    const msg = error.stderr?.toString() || error.message;
                    outputChannel.appendLine(`Format error: ${msg}`);
                    vscode.window.showErrorMessage(`Ifa format failed: ${msg}`);
                    return [];
                }
            }
        })
    );
}

function startLanguageServer(context: ExtensionContext) {
    const config = workspace.getConfiguration('ifa');
    const serverPath = config.get<string>('languageServer.path');

    // Find ifa executable
    const ifaPath = serverPath || findIfaExecutable();

    if (!ifaPath) {
        outputChannel.appendLine('Ifa language server not found - syntax highlighting only');
        return;
    }

    try {
        const traceLevel = config.get<string>('trace.server', 'off');

        const serverOptions: ServerOptions = {
            run: {
                command: ifaPath,
                args: ['lsp'],
                transport: TransportKind.stdio
            },
            debug: {
                command: ifaPath,
                args: ['lsp', '--debug'],
                transport: TransportKind.stdio
            }
        };

        const clientOptions: LanguageClientOptions = {
            documentSelector: [
                { scheme: 'file', language: 'ifa' }
            ],
            synchronize: {
                fileEvents: workspace.createFileSystemWatcher('**/*.ifa')
            },
            outputChannel: outputChannel,
            traceOutputChannel: outputChannel
        };

        client = new LanguageClient(
            'ifa-lang',
            'Ifa Language Server',
            serverOptions,
            clientOptions
        );

        // Set trace level from configuration
        client.setTrace(traceLevel as any);

        client.start().then(() => {
            outputChannel.appendLine('Ifa language server started');
        }).catch((error) => {
            outputChannel.appendLine(`Language server failed: ${error}`);
            vscode.window.showWarningMessage(
                'Ifa language server not available. Syntax highlighting only.'
            );
        });

        context.subscriptions.push(client);

    } catch (error) {
        outputChannel.appendLine(`Error starting language server: ${error}`);
    }
}

function findIfaExecutable(): string | undefined {
    // Check common locations for ifa binary
    const possiblePaths = [
        'ifa',  // In PATH
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

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

class IfaDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
    resolveDebugConfiguration(folder: vscode.WorkspaceFolder | undefined, debugConfiguration: vscode.DebugConfiguration): vscode.ProviderResult<vscode.DebugConfiguration> {
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

class IfaDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(session: vscode.DebugSession, executable: vscode.DebugAdapterExecutable | undefined): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        const config = workspace.getConfiguration('ifa');
        const ifaPath = config.get<string>('path', 'ifa');

        // Use the program path from launch config if available
        const program = session.configuration.program;
        const args = ['debug'];
        if (program) {
            args.push('--file', program);
        }

        return new vscode.DebugAdapterExecutable(ifaPath, args);
    }
}
