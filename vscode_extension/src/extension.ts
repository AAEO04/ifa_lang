import * as path from 'path';
import * as vscode from 'vscode';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // 1. Locate the Language Server (ifa executable)
    // For development, we point to the python script directly
    // In production, this would be the 'ifa' binary
    const serverPath = "python"; // Assumes python is in PATH
    const serverArgs = [
        // Hardcoded path for dev environment - user would config this normally
        "c:\\Users\\allio\\Desktop\\ifa_lang\\src\\cli.py",
        "lsp"
    ];

    // 2. Define Server Options
    const serverOptions: ServerOptions = {
        run: { command: serverPath, args: serverArgs, transport: TransportKind.stdio },
        debug: { command: serverPath, args: serverArgs, transport: TransportKind.stdio }
    };

    // 3. Define Client Options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'ifa' }],
        synchronize: {
            // Notify server of file changes
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

    client.start();

    // 5. Register Debug Adapter
    const factory = new IfaDebugAdapterDescriptorFactory();
    context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory('ifa', factory));
}

class IfaDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(session: vscode.DebugSession, executable: vscode.DebugAdapterExecutable | undefined): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // Use the same 'ifa' executable (or python script in dev) as CLI
        const command = "python";
        const args = ["c:\\Users\\allio\\Desktop\\ifa_lang\\src\\cli.py", "dap"];
        return new vscode.DebugAdapterExecutable(command, args);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
