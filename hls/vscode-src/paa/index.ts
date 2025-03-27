import * as vscode from "vscode";

// PAa
import { PaaViewerProvider } from "./paaViewer";
import { SizeStatusBarEntry } from "./sizeStatusBarEntry";
import { ZoomStatusBarEntry } from "./zoomStatusBarEntry";
import { BinarySizeStatusBarEntry } from "./binarySizeStatusBarEntry";

export function activate(context: vscode.ExtensionContext) {
  // PAA
  const sizeStatusBarEntry = new SizeStatusBarEntry();
  context.subscriptions.push(sizeStatusBarEntry);
  const binarySizeStatusBarEntry = new BinarySizeStatusBarEntry();
  context.subscriptions.push(binarySizeStatusBarEntry);
  const zoomStatusBarEntry = new ZoomStatusBarEntry();
  context.subscriptions.push(zoomStatusBarEntry);
  const previewManager = new PaaViewerProvider(
    context.extensionUri,
    sizeStatusBarEntry,
    binarySizeStatusBarEntry,
    zoomStatusBarEntry
  );
  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      PaaViewerProvider.viewType,
      previewManager,
      {
        supportsMultipleEditorsPerDocument: false,
      }
    )
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("hemtt.zoomIn", () => {
      previewManager.activePreview?.zoomIn();
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("hemtt.zoomOut", () => {
      previewManager.activePreview?.zoomOut();
    })
  );
}
