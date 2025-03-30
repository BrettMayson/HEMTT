import * as vscode from 'vscode';
import { LanguageClient } from "vscode-languageclient/node";
import { channel } from '../extension';

export class P3dViewerProvider implements vscode.CustomReadonlyEditorProvider {
  public static readonly viewType = 'hemtt.p3dViewer';
  
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
      channel.appendLine(`Processing P3D file: ${document.uri.toString()}`);
      const modelData = await this.client.sendRequest("hemtt/p3d/json", {
        url: document.uri.toString()
      });
      
      if (!modelData) {
        webview.html = this.getErrorHtml("Failed to load P3D file");
        return;
      }

      webview.html = this.getHtml(webview, modelData);

      webview.onDidReceiveMessage(message => {
        channel.appendLine(`Received message: ${JSON.stringify(message)}`);
        switch (message.command) {
          case 'requestTexture':
            this.client.sendRequest("hemtt/paa/p3d", {
              url: document.uri.toString(),
              texture: message.texture,
            }).then((response: any) => {
              if (response) {
                webview.postMessage({
                  command: 'textureResponse',
                  id: message.id,
                  data: response,
                })
              }
            });
            break;
          }
        });
      
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

  private getHtml(webview: vscode.Webview, modelData: any): string {
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'webview-dist', 'p3d', 'viewer.js')
    );
    const styleUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, 'media', 'p3d', 'styles.css')
    );
    
    const nonce = getNonce();

    return `
      <!DOCTYPE html>
      <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} data:; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
        <title>P3D Model Viewer</title>
        <link rel="stylesheet" href="${styleUri}">
      </head>
      <body>
        <div id="controls">
          <div>
            <label for="lodLevel">LOD:</label>
            <select id="lodLevel"></select>
          </div>
          <div id="displayOptions">
            <label for="material">Material:</label>
            <select id="material"></select>
            <label for="culling">Culling:</label>
            <select id="culling"></select>
          </div>
          <div>
          </div>
        </div>

        <div id="light">
          <h4>Light Position</h4>
          <div>
              <label>X</label>
              <input type="range" id="lightXSlider" min="-10" max="10" value="2" step="0.1">
          </div>
          <div>
              <label>Y</label>
              <input type="range" id="lightYSlider" min="-10" max="10" value="5" step="0.1">
          </div>
          <div>
              <label>Z</label>
              <input type="range" id="lightZSlider" min="-10" max="10" value="1" step="0.1">
          </div>
        </div>
        
        <div id="stats"></div>

        <script nonce="${nonce}" type="module">
          window.modelData = ${JSON.stringify(modelData)};
          window.vscode = acquireVsCodeApi();
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
