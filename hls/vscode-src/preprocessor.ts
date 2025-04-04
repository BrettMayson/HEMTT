import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
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
        const text: string | undefined = await client.sendRequest("hemtt/sqf/compiled", {
          url: uri.toString()
        });
        if (!text) {
          vscode.window.showErrorMessage("Failed to get compiled text.");
          throw new Error("Failed to get compiled text.");
        }
        return text;
      } catch (e) {
        channel.appendLine("sendRequest: hemtt/sqf/compiled: " + uri.toString() + " failed");
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
