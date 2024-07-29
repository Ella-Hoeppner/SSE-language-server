import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
  Executable
} from 'vscode-languageclient/node';

let client: LanguageClient;

function selectTextAfterCursor() {
  const editor = vscode.window.activeTextEditor;
  if (editor) {
      const selection = editor.selection;
      const newSelection = new vscode.Selection(selection.start, selection.end.with(selection.end.line, selection.end.character + 1));
      editor.selection = newSelection;
  }
}

export function activate(context: vscode.ExtensionContext) {
  const serverPath = path.join(__dirname, '..', '..', '..', 'target', 'debug', 'sse_lsp');

  const runOptions: Executable = { command: serverPath, transport: TransportKind.stdio };
  const debugOptions: Executable = { command: serverPath, transport: TransportKind.stdio, args: ['--nolazy', '--inspect=6009'] };

  const serverOptions: ServerOptions = {
    run: runOptions,
    debug: debugOptions
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'sse' }],
  };

  let disposable = vscode.commands.registerCommand('extension.selectTextAfterCursor', selectTextAfterCursor);
  context.subscriptions.push(disposable);

  client = new LanguageClient(
    'sseLanguageServer',
    'SSE Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}