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

function selectionPositions(editor: vscode.TextEditor) {
  const selection = editor.selection;
  return [{
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
  }];
}

async function expandSelection() {
  const editor = vscode.window.activeTextEditor;
  if (editor) {
    try {
      const result = await vscode.commands.executeCommand(
        'expandSelection',
        selectionPositions(editor)
      ) as [number, number, number, number] | undefined;
      if (result !== undefined) {
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

async function moveCursorLeft() {
  const editor = vscode.window.activeTextEditor;
  if (editor) {
    try {
      const result = await vscode.commands.executeCommand(
        'moveCursorLeft',
        selectionPositions(editor)
      ) as [number, number] | undefined;
      if (result !== undefined) {
        editor.selection = new vscode.Selection(
          new vscode.Position(result[0], result[1]),
          new vscode.Position(result[0], result[1]),
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

  for (let [commandName, commandHandler] of
    [['extension.moveCursorLeft', moveCursorLeft],['extension.expandSelection', expandSelection],
    ] as const) {
    context.subscriptions.push(
      vscode.commands.registerCommand(commandName, commandHandler)
    );
  }

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