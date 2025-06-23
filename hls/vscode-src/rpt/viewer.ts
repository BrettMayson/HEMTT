import * as vscode from 'vscode';
import * as fs from 'fs';

export class LogFileViewer {
  private static readonly scheme = 'logfile';
  private static provider: LogFileContentProvider | undefined;
  private static autoScroll = true;
  private static statusBarItem: vscode.StatusBarItem | undefined;
  private static disposables: Map<string, vscode.Disposable> = new Map();

  public static async watch(filePath: string) {
    const uri = vscode.Uri.from({ scheme: this.scheme, path: filePath });

    if (!this.provider) {
      this.provider = new LogFileContentProvider();
      vscode.workspace.registerTextDocumentContentProvider(this.scheme, this.provider);
    }

    this.showAutoScrollStatusBar();

    const doc = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(doc, { preview: false });
    vscode.languages.setTextDocumentLanguage(doc, 'log');

    let timeout: NodeJS.Timeout | undefined;
    const watcher = fs.watch(filePath, {}, () => {
      if (timeout) clearTimeout(timeout);
      timeout = setTimeout(() => {
        this.provider?.update(uri);
      }, 100);
    });

    this.disposables.set(uri.toString(), vscode.workspace.onDidCloseTextDocument(closedDoc => {
      if (closedDoc.uri.toString() === uri.toString()) {
        watcher.close();
        this.disposables.get(uri.toString())?.dispose();
        this.disposables.delete(uri.toString());
      }
    }));

    if (this.autoScroll) {
      scrollToBottom(vscode.window.activeTextEditor!);
    }
  }

  private static showAutoScrollStatusBar() {
    if (!this.statusBarItem) {
      this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
      this.statusBarItem.command = 'logfile.toggleAutoScroll';
      vscode.commands.registerCommand('logfile.toggleAutoScroll', () => {
        this.autoScroll = !this.autoScroll;
        this.updateStatusBarText();
        if (this.autoScroll) {
          scrollToBottom(vscode.window.activeTextEditor!);
        }
      });
    }
    this.updateStatusBarText();
  }

  private static updateStatusBarText() {
    if (this.statusBarItem) {
      this.statusBarItem.text = `$(arrow-down) Auto Scroll: ${this.autoScroll ? 'On' : 'Off'}`;
      this.statusBarItem.tooltip = 'Toggle auto scroll to bottom on log update';
    }
  }

  public static isAutoScrollEnabled() {
    return this.autoScroll;
  }

  public static setupRptStatusBarVisibility() {
    const updateStatusBarVisibility = () => {
      const hasRpt = vscode.window.visibleTextEditors.some(
        editor => editor.document.uri.fsPath.endsWith('.rpt')
      );
      if (hasRpt) {
        this.statusBarItem?.show();
      } else {
        this.statusBarItem?.hide();
      }
    };

    vscode.window.onDidChangeVisibleTextEditors(updateStatusBarVisibility);
    vscode.workspace.onDidCloseTextDocument(updateStatusBarVisibility);
    vscode.workspace.onDidOpenTextDocument(updateStatusBarVisibility);
    updateStatusBarVisibility();
  }
}

function scrollToBottom(editor: vscode.TextEditor) {
  const lastLine = editor.document.lineCount - 1;
  const lastLineRange = editor.document.lineAt(lastLine).range;
  editor.revealRange(lastLineRange, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
}

class LogFileContentProvider implements vscode.TextDocumentContentProvider {
  private _onDidChange = new vscode.EventEmitter<vscode.Uri>();
  public onDidChange = this._onDidChange.event;
  private filePositions: Map<string, number> = new Map();
  private fileContents: Map<string, string> = new Map();

  provideTextDocumentContent(uri: vscode.Uri): Thenable<string> {
    const filePath = uri.path;
    return new Promise((resolve) => {
      const lastPos = this.filePositions.get(filePath) ?? 0;
      let start = lastPos;
      let data = '';
      let bytesRead = 0;
      const stream = fs.createReadStream(filePath, { encoding: 'utf8', start });
      stream.on('data', chunk => {
        data += chunk;
        bytesRead += Buffer.byteLength(chunk, 'utf8');
      });
      stream.on('end', () => {
        const prev = start === 0 ? '' : (this.fileContents.get(filePath) ?? '');
        const content = prev + data;
        this.fileContents.set(filePath, content);
        this.filePositions.set(filePath, start + bytesRead);
        resolve(content);
      });
      stream.on('error', error => {
        this.filePositions.set(filePath, 0);
        this.fileContents.set(filePath, '');
        const errorMessage = error instanceof Error ? error.message : String(error);
        resolve(`Failed to read file: ${errorMessage}`);
      });
    });
  }

  update(uri: vscode.Uri) {
    this._onDidChange.fire(uri);
    setTimeout(() => {
      if (LogFileViewer.isAutoScrollEnabled()) {
        const editor = vscode.window.visibleTextEditors.find(e => e.document.uri.toString() === uri.toString());
        if (editor) {
          scrollToBottom(editor);
        }
      }
    }, 100);
  }
}
