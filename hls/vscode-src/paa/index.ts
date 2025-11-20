import * as vscode from "vscode";
import { LanguageClient } from 'vscode-languageclient/node';
import { PaaViewerProvider } from './viewer';
import { orActive } from "../util";

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertPaa', async (uri: vscode.Uri | undefined) => {
    orActive(uri, async (uri) => await conversion(uri.toString(), "paa", client, channel));
  }));
  ["png", "jpg", "bmp", "tga", "webp"].forEach(ext => {
    context.subscriptions.push(vscode.commands.registerCommand(`hemtt.convert${ext.charAt(0).toUpperCase() + ext.slice(1)}`, async (uri: vscode.Uri | undefined) => {
      orActive(uri, async (uri) => await conversion(uri.toString(), ext, client, channel));
    }));
  });

  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      PaaViewerProvider.viewType,
      new PaaViewerProvider(context.extensionUri, client),
      {
        webviewOptions: {
          retainContextWhenHidden: true,
        },
      }
    )
  );
}

async function conversion(url: string, to: string, client: LanguageClient, channel: vscode.OutputChannel) {
  const conv: string = await client.sendRequest("hemtt/paa/convert", {
    url,
    to,
  });
  if (!conv) {
    vscode.window.showErrorMessage("Failed to convert image");
    return;
  }
  vscode.commands.executeCommand("vscode.open", vscode.Uri.parse(conv));
}
