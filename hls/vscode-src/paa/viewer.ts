import * as vscode from 'vscode';
import { LanguageClient } from "vscode-languageclient/node";
import { channel } from '../extension';

export class PaaViewerProvider implements vscode.CustomReadonlyEditorProvider {
  public static readonly viewType = 'hemtt.paaViewer';
  
  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly client: LanguageClient
  ) {}

  async openCustomDocument(
    uri: vscode.Uri,
    openContext: vscode.CustomDocumentOpenContext,
    token: vscode.CancellationToken
  ): Promise<vscode.CustomDocument> {
    return { uri, dispose: () => {} };
  }

  async resolveCustomEditor(
    document: vscode.CustomDocument,
    webviewPanel: vscode.WebviewPanel,
    token: vscode.CancellationToken
  ): Promise<void> {
    const webview = webviewPanel.webview;
    webview.options = {
      enableScripts: true,
      localResourceRoots: [
        this.extensionUri
      ]
    };

    try {
      channel.appendLine(`Processing PAA file: ${document.uri.toString()}`);
      const paaData = await this.client.sendRequest("hemtt/paa/json", {
        url: document.uri.toString()
      });
      
      if (!paaData) {
        webview.html = this.getErrorHtml("Failed to load PAA file");
        return;
      }

      webview.html = this.getHtml(webview, paaData);
      
      webview.onDidReceiveMessage(message => {
        switch (message.command) {
          case 'error':
            vscode.window.showErrorMessage(message.text);
            break;
        }
      });
    } catch (error) {
      console.error(error);
      webview.html = this.getErrorHtml(`Error: ${error}`);
    }
  }

  private getHtml(webview: vscode.Webview, paaData: any): string {
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'webview-dist', 'paa', 'viewer.js')
    );
    const styleUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'media', 'paa', 'styles.css')
    );
    
    const nonce = getNonce();

    return `
      <!DOCTYPE html>
      <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} data:; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
        <title>PAA Texture Viewer</title>
        <link rel="stylesheet" href="${styleUri}">
      </head>
      <body>
        <div id="imageContainer">
            <div id="imageWrapper">
                <img id="paaImage" alt="PAA Image">
            </div>
        </div>

        <div id="controls">
          <div>
            <label for="mipmapLevel">Mipmap:</label>
            <select id="mipmapLevel"></select>
          </div>
          <div>
            <label><input type="checkbox" id="channelRed" checked=true> Red</label>
            <label><input type="checkbox" id="channelGreen" checked=true> Green</label>
            <label><input type="checkbox" id="channelBlue" checked=true> Blue</label>
            <label><input type="checkbox" id="channelAlpha"> Alpha</label>
          </div>
          <div id="displayInfo">
            <div>Format: <span id="formatInfo"></span></div>
            <div>Dimensions: <span id="dimensionsInfo"></span></div>
          </div>
        </div>

        <script nonce="${nonce}" type="module">
          window.paaData = ${JSON.stringify(paaData)};
        </script>
        <script nonce="${nonce}" type="module" src="${scriptUri}"></script>
      </body>
      </html>
    `;
  }

  private getErrorHtml(errorMessage: string): string {
    return `
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="UTF-8">
        <style>
          body {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background-color: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
          }
        </style>
      </head>
      <body>
        <h3>Error: ${errorMessage}</h3>
      </body>
      </html>
    `;
  }
}

function getNonce() {
  let text = '';
  const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}
