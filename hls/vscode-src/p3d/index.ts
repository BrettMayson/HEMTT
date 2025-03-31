import * as vscode from "vscode";
import { LanguageClient } from 'vscode-languageclient/node';
import { P3dViewerProvider } from './viewer';

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      P3dViewerProvider.viewType,
      new P3dViewerProvider(context.extensionUri, client),
      {
        webviewOptions: {
          retainContextWhenHidden: true,
        },
      }
    )
  );
}
