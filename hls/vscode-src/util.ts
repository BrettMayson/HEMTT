import * as vscode from 'vscode';

export function getNonce() {
  let text = '';
  const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}

export function orActive(uri: vscode.Uri | undefined, callback: (uri: vscode.Uri) => Promise<void> | void = () => {}) {
  if (!uri) {
    uri = (vscode.window.tabGroups.activeTabGroup?.activeTab?.input as any)?.uri;
    if (!uri) {
      vscode.window.showErrorMessage("No active file to convert");
      return;
    }
  }
  callback(uri);
  return uri;
}
