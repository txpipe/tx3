import * as vscode from "vscode";
import * as path from "path";
import {
  LanguageClient,
  LanguageClientOptions,
  MessageStrategy,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

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
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
