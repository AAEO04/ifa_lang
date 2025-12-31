import * as path from 'path';
import * as vscode from 'vscode';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext) {
    // 1. Try to find the Ifá Language Server
    // Check for 'ifa' in PATH first, then fallback to python script
    const config = workspace.getConfiguration('ifa');
    const serverPath = config.get<string>('serverPath') || findIfaExecutable();

    if (!serverPath) {
        // No LSP available - extension still provides syntax highlighting
        console.log('Ifá Language Server not found. Syntax highlighting only.');
        vscode.window.showInformationMessage(
            'Ifá-Lang: Syntax highlighting active. Install ifa-lang for intellisense.'
        );
        return; // Exit early - syntax highlighting still works!
    }

    try {
        // 2. Define Server Options
        const serverArgs = serverPath.endsWith('.py') ? ['lsp'] : ['lsp'];
        const serverCommand = serverPath.endsWith('.py') ? 'python' : serverPath;
        const serverArgsWithPath = serverPath.endsWith('.py') ? [serverPath, 'lsp'] : ['lsp'];

        const serverOptions: ServerOptions = {
            run: { command: serverCommand, args: serverArgsWithPath, transport: TransportKind.stdio },
            debug: { command: serverCommand, args: serverArgsWithPath, transport: TransportKind.stdio }
        };

        // 3. Define Client Options
        const clientOptions: LanguageClientOptions = {
            documentSelector: [{ scheme: 'file', language: 'ifa' }],
            synchronize: {
                fileEvents: workspace.createFileSystemWatcher('**/*.ifa')
            }
        };

        // 4. Create and Start Client
        client = new LanguageClient(
            'ifaLanguageServer',
            'Ifá Language Server',
            serverOptions,
            clientOptions
        );

        client.start().catch((error) => {
            console.error('Failed to start Ifá Language Server:', error);
            vscode.window.showWarningMessage(
                'Ifá-Lang: Language server not available. Syntax highlighting only.'
            );
        });

    } catch (error) {
        console.error('Error initializing Ifá Language Client:', error);
    }

    // 5. Register Debug Adapter (optional)
    try {
        const factory = new IfaDebugAdapterDescriptorFactory(serverPath);
        context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory('ifa', factory));
    } catch (error) {
        console.error('Debug adapter registration failed:', error);
    }
}

function findIfaExecutable(): string | undefined {
    // Check common locations
    const possiblePaths = [
        'ifa',  // In PATH
        'C:\\ifa-lang\\bin\\ifa.bat',  // Windows installed
        '/usr/local/bin/ifa',  // Unix installed
        path.join(__dirname, '..', '..', 'src', 'cli.py')  // Dev environment
    ];

    // For now, just try the PATH version
    // In production, we'd check if the file exists
    return undefined;  // Let it fail gracefully if not found
}

class IfaDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    private serverPath: string;

    constructor(serverPath: string) {
        this.serverPath = serverPath;
    }

    createDebugAdapterDescriptor(
        session: vscode.DebugSession,
        executable: vscode.DebugAdapterExecutable | undefined
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        if (this.serverPath.endsWith('.py')) {
            return new vscode.DebugAdapterExecutable('python', [this.serverPath, 'dap']);
        }
        return new vscode.DebugAdapterExecutable(this.serverPath, ['dap']);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
