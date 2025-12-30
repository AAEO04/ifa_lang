"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
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
    const serverOptions = {
        run: { command: serverPath, args: serverArgs, transport: node_1.TransportKind.stdio },
        debug: { command: serverPath, args: serverArgs, transport: node_1.TransportKind.stdio }
    };
    // 3. Define Client Options
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'ifa' }],
        synchronize: {
            // Notify server of file changes
            fileEvents: vscode_1.workspace.createFileSystemWatcher('**/*.ifa')
        }
    };
    // 4. Create and Start Client
    client = new node_1.LanguageClient('ifaLanguageServer', 'Ifá Language Server', serverOptions, clientOptions);
    client.start();
    // 5. Register Debug Adapter
    const factory = new IfaDebugAdapterDescriptorFactory();
    context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory('ifa', factory));
}
class IfaDebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(session, executable) {
        // Use the same 'ifa' executable (or python script in dev) as CLI
        const command = "python";
        const args = ["c:\\Users\\allio\\Desktop\\ifa_lang\\src\\cli.py", "dap"];
        return new vscode.DebugAdapterExecutable(command, args);
    }
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map