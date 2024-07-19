"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = require("path");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const serverPath = path.join(__dirname, '..', '..', '..', 'target', 'debug', 'sse_lsp');
    const runOptions = { command: serverPath, transport: node_1.TransportKind.stdio };
    const debugOptions = { command: serverPath, transport: node_1.TransportKind.stdio, args: ['--nolazy', '--inspect=6009'] };
    const serverOptions = {
        run: runOptions,
        debug: debugOptions
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'sse' }],
    };
    client = new node_1.LanguageClient('sseLanguageServer', 'SSE Language Server', serverOptions, clientOptions);
    client.start();
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map