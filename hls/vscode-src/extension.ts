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

  // Processed view
  const processedProvider = new (class implements vscode.TextDocumentContentProvider {
    onDidChangeEmitter = new vscode.EventEmitter<vscode.Uri>();
    onDidChange = this.onDidChangeEmitter.event;
    async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
      channel.appendLine("processedProvider: " + uri.toString());
      try {
        const text: string | undefined = await client.sendRequest("hemtt/processed", {
          url: uri.toString()
        });
        if (!text) {
          vscode.window.showErrorMessage("Failed to get processed text.");
          throw new Error("Failed to get processed text.");
        }
        return text;
      } catch (e) {
        channel.appendLine("sendRequest: hemtt/processed: " + uri.toString() + " failed");
        channel.appendLine(e as any);
        throw e;
      }
    }
  })();
  vscode.workspace.registerTextDocumentContentProvider("hemttprocessed", processedProvider);
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.showProcessed', async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No editor is open.');
      return;
    }
    let uri = vscode.Uri.parse('hemttprocessed://' + editor.document.uri.path);
    channel.appendLine("showProcessed: " + uri.toString());
    let doc = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(doc, { preview: false });
  }));

  // Compiled view
  const compiledProvider = new (class implements vscode.TextDocumentContentProvider {
    onDidChangeEmitter = new vscode.EventEmitter<vscode.Uri>();
    onDidChange = this.onDidChangeEmitter.event;
    async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
      channel.appendLine("compiledProvider: " + uri.toString());
      try {
        const text: string | undefined = await client.sendRequest("hemtt/compiled", {
          url: uri.toString()
        });
        if (!text) {
          vscode.window.showErrorMessage("Failed to get compiled text.");
          throw new Error("Failed to get compiled text.");
        }
        return text;
      } catch (e) {
        channel.appendLine("sendRequest: hemtt/compiled: " + uri.toString() + " failed");
        channel.appendLine(e as any);
        throw e;
      }
    }
  })();
  vscode.workspace.registerTextDocumentContentProvider("hemttcompiled", compiledProvider);
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.showCompiled', async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showInformationMessage('No editor is open.');
      return;
    }
    let uri = vscode.Uri.parse('hemttcompiled://' + editor.document.uri.path);
    channel.appendLine("showCompiled: " + uri.toString());
    let doc = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(doc, { preview: false });
  }));

  vscode.workspace.onDidSaveTextDocument((document) => {
    let uri = vscode.Uri.parse('hemttcompiled://' + document.uri.path);
    compiledProvider.onDidChangeEmitter.fire(uri);
    uri = vscode.Uri.parse('hemttprocessed://' + document.uri.path);
    processedProvider.onDidChangeEmitter.fire(uri);
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
