import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { WssViewerProvider } from "./wssViewer";

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioWav', async (uri: vscode.Uri) => {
    await conversion(uri.toString(), "wav", client, channel);
  }));
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioOgg', async (uri: vscode.Uri) => {
    await conversion(uri.toString(), "ogg", client, channel);
  }));
  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioWss', async (uri: vscode.Uri) => {
    await conversion(uri.toString(), "wss", client, channel);
  }));

  const wssViewerProvider = new WssViewerProvider(context.extensionUri, client);
  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      WssViewerProvider.viewType,
      wssViewerProvider,
      {
        supportsMultipleEditorsPerDocument: false,
        webviewOptions: { retainContextWhenHidden: true } // Keeps audio playing when tab is not focused
      }
    )
  );
  // Make sure to add the provider to subscriptions for proper disposal
  context.subscriptions.push(wssViewerProvider);
}

export interface WssConversion {
  path: string,
  channels: number,
  sampleRate: number,
  compression: string,
}

async function conversion(url: string, to: string, client: LanguageClient, channel: vscode.OutputChannel) {
  const conv: WssConversion | undefined = await client.sendRequest("hemtt/convertAudio", {
    url,
    to,
  });
  if (!conv) {
    vscode.window.showErrorMessage("Failed to convert audio file");
    return;
  }
  // open the path returned
  vscode.commands.executeCommand("vscode.open", vscode.Uri.parse(conv.path));
}
