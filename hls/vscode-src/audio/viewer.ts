import * as vscode from 'vscode';
import { LanguageClient } from "vscode-languageclient/node";
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { channel } from '../extension';
import { WssConversion } from '.';

export class WssViewerProvider implements vscode.CustomReadonlyEditorProvider {
  public static readonly viewType = 'hemtt.wssViewer';
  private tempDir: string;
  private tempFiles: Map<string, WssConversion> = new Map();
  
  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly client: LanguageClient
  ) {
    this.tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'hemtt-wss-'));
    channel.appendLine(`Temporary directory created: ${this.tempDir}`);
  }

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
    // Set up webview properties
    const webview = webviewPanel.webview;
    webview.options = {
      enableScripts: true,
      localResourceRoots: [
        this.extensionUri,
        vscode.Uri.file(this.tempDir)
      ]
    };

    // Convert WSS to WAV
    const wavPath = await this.convertToWav(document.uri);
    if (!wavPath) {
      webview.html = this.getErrorHtml();
      return;
    }

    // Set the webview's HTML content
    let wssInfo = `Channels: ${wavPath.channels}<br/>Sample Rate: ${wavPath.sampleRate}<br/>Compression: ${wavPath.compression}`;
    webview.html = this.getHtml(webview, wavPath.path, wssInfo);
  }

  private async convertToWav(wssUri: vscode.Uri): Promise<WssConversion | undefined> {
    try {
      // If we've already converted this file, return the cached path
      if (this.tempFiles.has(wssUri.toString())) {
        return this.tempFiles.get(wssUri.toString());
      }
      
      // Create a unique name for the temp file
      const fileName = path.basename(wssUri.fsPath, '.wss');
      const tempWavPath = path.join(this.tempDir, `${fileName}-${Date.now()}.wav`);
      
      // Call your existing audio conversion function
      channel.appendLine(`Converting WSS file to WAV: ${wssUri.toString()} -> ${tempWavPath}`);
      const result: WssConversion | undefined = await this.client.sendRequest("hemtt/audio/convert", {
        url: wssUri.toString(),
        to: "wav",
        out: tempWavPath
      });
      
      if (!result) {
        vscode.window.showErrorMessage("Failed to convert WSS file to WAV");
        return undefined;
      }
      
      // Save the mapping for cleanup later
      this.tempFiles.set(wssUri.toString(), result);
      return result;
    } catch (error) {
      console.error(error);
      return undefined;
    }
  }

  private getHtml(webview: vscode.Webview, wavPath: string, wssInfo: string): string {
    // Create a URI that the webview can use to access the temporary WAV file
    const wavUri = webview.asWebviewUri(vscode.Uri.file(wavPath));
    
    return `
      <!DOCTYPE html>
      <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>WSS Audio Viewer</title>
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
          audio {
            width: 80%;
            margin: 20px 0;
          }
          .info {
            margin-top: 20px;
            font-family: var(--vscode-editor-font-family);
            font-size: var(--vscode-editor-font-size);
          }
        </style>
      </head>
      <body>
        <h2>HEMTT Audio Player</h2>
        <audio controls src="${wavUri}" autoplay></audio>
        <div class="info">
          <p>
            ${wssInfo}
          </p>
        </div>
        <div class="info">
          Playing converted WAV file from WSS source
        </div>
      </body>
      </html>
    `;
  }

  private getErrorHtml(): string {
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
        <h3>Error: Failed to convert WSS audio file</h3>
      </body>
      </html>
    `;
  }

  // Clean up temp files when the extension is deactivated
  public dispose(): void {
    try {
      // Delete all temporary files
      for (const filePath of this.tempFiles.values()) {
        if (fs.existsSync(filePath.path)) {
          fs.unlinkSync(filePath.path);
        }
      }
      
      // Remove the temporary directory
      if (fs.existsSync(this.tempDir)) {
        fs.rmdirSync(this.tempDir);
      }
    } catch (error) {
      console.error('Error cleaning up temporary files:', error);
    }
  }
}
