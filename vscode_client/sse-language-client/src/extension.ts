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

async function selectTextAfterCursor() {
  const editor = vscode.window.activeTextEditor;
  if (editor) {
    vscode.Selection
    const selection = editor.selection;

    try {
      const result = await vscode.commands.executeCommand(
        'expandSelection',
        [{
          textDocument: { uri: editor.document.uri.toString() },
          position: {
            line: selection.start.line,
            character: selection.start.character
          },
        },
        {
          textDocument: { uri: editor.document.uri.toString() },
          position: {
            line: selection.end.line,
            character: selection.end.character
          },
        }]
      ) as [number, number, number, number] | undefined;

      if (result !== undefined) {
        console.error('result was defined');
        editor.selection = new vscode.Selection(
          new vscode.Position(result[0], result[1]),
          new vscode.Position(result[2], result[3]),
        );
      } else {
        console.error('result was undefined');
      }
    } catch (error) {
      console.error('Error calling custom LSP command:', error);
    }
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