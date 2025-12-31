"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = require("path");
const vscode = require("vscode");
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    // 1. Try to find the Ifá Language Server
    // Check for 'ifa' in PATH first, then fallback to python script
    const config = vscode_1.workspace.getConfiguration('ifa');
    const serverPath = config.get('serverPath') || findIfaExecutable();
    if (!serverPath) {
        // No LSP available - extension still provides syntax highlighting
        console.log('Ifá Language Server not found. Syntax highlighting only.');
        vscode.window.showInformationMessage('Ifá-Lang: Syntax highlighting active. Install ifa-lang for intellisense.');
        return; // Exit early - syntax highlighting still works!
    }
    try {
        // 2. Define Server Options
        const serverArgs = serverPath.endsWith('.py') ? ['lsp'] : ['lsp'];
        const serverCommand = serverPath.endsWith('.py') ? 'python' : serverPath;
        const serverArgsWithPath = serverPath.endsWith('.py') ? [serverPath, 'lsp'] : ['lsp'];
        const serverOptions = {
            run: { command: serverCommand, args: serverArgsWithPath, transport: node_1.TransportKind.stdio },
            debug: { command: serverCommand, args: serverArgsWithPath, transport: node_1.TransportKind.stdio }
        };
        // 3. Define Client Options
        const clientOptions = {
            documentSelector: [{ scheme: 'file', language: 'ifa' }],
            synchronize: {
                fileEvents: vscode_1.workspace.createFileSystemWatcher('**/*.ifa')
            }
        };
        // 4. Create and Start Client
        client = new node_1.LanguageClient('ifaLanguageServer', 'Ifá Language Server', serverOptions, clientOptions);
        client.start().catch((error) => {
            console.error('Failed to start Ifá Language Server:', error);
            vscode.window.showWarningMessage('Ifá-Lang: Language server not available. Syntax highlighting only.');
        });
    }
    catch (error) {
        console.error('Error initializing Ifá Language Client:', error);
    }
    // 5. Register Debug Adapter (optional)
    try {
        const factory = new IfaDebugAdapterDescriptorFactory(serverPath);
        context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory('ifa', factory));
    }
    catch (error) {
        console.error('Debug adapter registration failed:', error);
    }
}
function findIfaExecutable() {
    // Check common locations
    const possiblePaths = [
        'ifa', // In PATH
        'C:\\ifa-lang\\bin\\ifa.bat', // Windows installed
        '/usr/local/bin/ifa', // Unix installed
        path.join(__dirname, '..', '..', 'src', 'cli.py') // Dev environment
    ];
    // For now, just try the PATH version
    // In production, we'd check if the file exists
    return undefined; // Let it fail gracefully if not found
}
class IfaDebugAdapterDescriptorFactory {
    constructor(serverPath) {
        this.serverPath = serverPath;
    }
    createDebugAdapterDescriptor(session, executable) {
        if (this.serverPath.endsWith('.py')) {
            return new vscode.DebugAdapterExecutable('python', [this.serverPath, 'dap']);
        }
        return new vscode.DebugAdapterExecutable(this.serverPath, ['dap']);
    }
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map