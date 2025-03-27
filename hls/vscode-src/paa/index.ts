import * as vscode from "vscode";
import { LanguageClient } from 'vscode-languageclient/node';
import { PaaViewerProvider } from './viewer';

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
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
