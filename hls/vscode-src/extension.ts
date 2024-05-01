import * as vscode from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import * as paa from './paa';

let client: LanguageClient;

export async function activate(context: vscode.ExtensionContext) {
  paa.activate(context);
  let command = process.env.SERVER_PATH || "hemtt-language-server";
  if (process.platform === "win32") {
    command += ".exe";
  }
  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
      },
    },
    transport: {
      kind: TransportKind.socket,
      port: 9632,
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      { scheme: "file", language: "sqf" },
      { scheme: "file", language: "arma-config" },
    ],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: vscode.workspace.createFileSystemWatcher("**/.hemtt/**"),
    },
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "hemtt-language-server",
    "HEMTT Language Server",
    serverOptions,
    clientOptions
  );
  // activateInlayHints(context);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
