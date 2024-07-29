"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = require("path");
const vscode = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function selectTextAfterCursor() {
    const editor = vscode.window.activeTextEditor;
    if (editor) {
        const selection = editor.selection;
        const newSelection = new vscode.Selection(selection.start, selection.end.with(selection.end.line, selection.end.character + 1));
        editor.selection = newSelection;
    }
}
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
    let disposable = vscode.commands.registerCommand('extension.selectTextAfterCursor', selectTextAfterCursor);
    context.subscriptions.push(disposable);
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