import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.path_copy', async (uri?: vscode.Uri) => {
    const targetUri = uri ?? vscode.window.activeTextEditor?.document.uri;
    if (!targetUri) {
      return;
    }
    let conv = await client.sendRequest<string>('hemtt/path_copy', { url: targetUri.toString() });
    if (!conv) {
      vscode.window.showErrorMessage("Failed to get path");
      return;
    }
    await vscode.env.clipboard.writeText(conv);
    channel.appendLine(`Copied path: ${conv}`);
  }));
}
