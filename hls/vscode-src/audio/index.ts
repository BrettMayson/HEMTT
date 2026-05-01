import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { WssViewerProvider } from "./viewer";
import { orActive } from "../util";

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {

  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioWav', async (uri: vscode.Uri | undefined, multiselection: Array<vscode.Uri> | undefined) => {
    orActive(uri, multiselection, async (uri) => await conversion(uri.toString(), "wav", client, channel));
  }));

  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioOgg', async (uri: vscode.Uri | undefined, multiselection: Array<vscode.Uri> | undefined) => {
    orActive(uri, multiselection, async (uri) => await conversion(uri.toString(), "ogg", client, channel));
  }));

  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioWssNone', async (uri: vscode.Uri | undefined, multiselection: Array<vscode.Uri> | undefined) => {
    orActive(uri, multiselection, async (uri) => await conversion(uri.toString(), "wss", client, channel, Compression.None));
  }));

  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioWssNibble', async (uri: vscode.Uri | undefined, multiselection: Array<vscode.Uri> | undefined) => {
    orActive(uri, multiselection, async (uri) => await conversion(uri.toString(), "wss", client, channel, Compression.Nibble));
  }));

  context.subscriptions.push(vscode.commands.registerCommand('hemtt.convertAudioWssByte', async (uri: vscode.Uri | undefined, multiselection: Array<vscode.Uri> | undefined) => {
    orActive(uri, multiselection, async (uri) => await conversion(uri.toString(), "wss", client, channel, Compression.Byte));
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
  context.subscriptions.push(wssViewerProvider);
}

export interface WssConversion {
  path: string,
  channels: number,
  sampleRate: number,
  compression: string,
}

export interface AudioConversionResult {
  conversionDate: Date,
  conversion: WssConversion
}

export enum Compression {
  None = 0,
  Nibble = 4,
  Byte = 8,
}

async function conversion(url: string, to: string, client: LanguageClient, channel: vscode.OutputChannel, compression?: Compression) {
  const conv: WssConversion | undefined = await client.sendRequest("hemtt/audio/convert", {
    url,
    to,
    compression,
  });
  if (!conv) {
    vscode.window.showErrorMessage("Failed to convert audio file");
    return;
  }
  vscode.commands.executeCommand("vscode.open", vscode.Uri.parse(conv.path));
}
