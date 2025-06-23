import * as vscode from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import * as audio from "./audio";
import * as p3d from "./p3d";
import * as paa from "./paa";
import * as preprocessor from "./preprocessor";
import * as rpt from "./rpt";

import { getPortPromise } from "portfinder";

let client: LanguageClient;
export let channel: vscode.OutputChannel = vscode.window.createOutputChannel("HEMTT");

export async function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(channel);
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

  audio.init(client, channel, context);
  p3d.init(client, channel, context);
  paa.init(client, channel, context);
  preprocessor.init(client, channel, context);
  rpt.init(client, channel, context);
  channel.appendLine("HEMTT initialized");
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
