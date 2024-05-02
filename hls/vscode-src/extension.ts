import * as vscode from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import * as paa from "./paa";

import { getPortPromise } from "portfinder";

let client: LanguageClient;
let channel: vscode.OutputChannel = vscode.window.createOutputChannel("HEMTT");

export async function activate(context: vscode.ExtensionContext) {
  paa.activate(context);
  let command = context.asAbsolutePath("hemtt-language-server");
  if (process.platform === "win32") {
    command += ".exe";
  }
  const port = await getPortPromise({
    port: 12000,
  });

  channel.appendLine(`Starting HEMTT Language Server on port ${port}`);
  channel.appendLine(`Using command: ${command}`);

  const run: Executable = {
    command,
    args: [port.toString()],
    options: {
      env: {
        ...process.env,
      },
    },
    transport: {
      kind: TransportKind.socket,
      port,
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
