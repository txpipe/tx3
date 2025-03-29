import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;
let previewPanel: vscode.WebviewPanel | null = null;

function getServerPath(context: vscode.ExtensionContext) {
  if (context.extensionMode === vscode.ExtensionMode.Development) {
    return {
      command: "cargo",
      args: ["run", "--bin", "tx3-lsp", "--"],
    };
  }

  // Get the path to the Rust LSP server binary
  // This assumes the binary is in the extension's root directory
  switch (process.platform) {
    case "win32":
      return {
        command: context.asAbsolutePath("tx3-lsp.exe"),
        args: [],
      };
    default:
      return {
        command: context.asAbsolutePath("tx3-lsp"),
        args: [],
      };
  }
}

export function activate(context: vscode.ExtensionContext) {
  const serverConfig = getServerPath(context);

  // The server options. We launch the Rust binary directly
  const serverOptions: ServerOptions = {
    run: {
      command: serverConfig.command,
      args: serverConfig.args,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverConfig.command,
      args: serverConfig.args, // TODO: Add --debug
      transport: TransportKind.stdio,
    },
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for Tx3 documents
    documentSelector: [{ scheme: "file", language: "tx3" }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contain in the workspace
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.tx3"),
    },
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "tx3",
    "Tx3 Language Server",
    serverOptions,
    clientOptions
  );

  // Start the client. This will also launch the server
  client.start();

  // Start commands subscriptions
  context.subscriptions.push(vscode.commands.registerCommand("tx3.startServer", () => client.start()));
  context.subscriptions.push(vscode.commands.registerCommand("tx3.startPreview", () => previewCommandHandler(context)));
}

const previewCommandHandler = (context: vscode.ExtensionContext) => {
  previewPanel = vscode.window.createWebviewPanel(
    'tx3Preview',
    'Tx3 Preview',
    vscode.ViewColumn.Two,
    {
      enableScripts: true,
      enableForms: true
    }
  );

  previewPanel.onDidDispose(
    () => previewPanel = null,
    null,
    context.subscriptions
  );

  updatePreviewPanel();

  vscode.commands.executeCommand<vscode.DocumentSymbol[]>("vscode.executeDocumentSymbolProvider", vscode.window.activeTextEditor?.document.uri)
    .then(symbols => {
      const data = { parties: [], transactions: [] } as any;
      for (const symbol of symbols) {
        // TODO: We need a better way to identify the symbols, probably using a tag for the symbol type (e.g. party, tx, parameter)
        // TODO: We also need a way to identify the parameter type, probably using a tag as well
        if (symbol.kind === vscode.SymbolKind.Object) {
          data.parties.push({ name: symbol.name });
        }
        if (symbol.kind === vscode.SymbolKind.Method) {
          const parameters = [] as any;
          for (const children of symbol.children) {
            if (children.kind === vscode.SymbolKind.Field) {
              parameters.push({ name: children.name });
            }
          }
          data.transactions.push({
            name: symbol.name,
            parameters
          });
        }
      }
      updatePreviewPanel(JSON.stringify(data, null, 2));
    });
}

const updatePreviewPanel = (content: string = "") => {
  previewPanel!!.webview.html = `
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Tx3 preview</title>
      </head>
      <body>
        <h1>Tx3 preview</h1>
        <pre>${content}</pre>
      </body>
    </html>
  `;
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
