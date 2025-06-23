import { existsSync, lstatSync } from "fs";
import { readdir } from "fs/promises";
import path = require("path");
import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { LogFileViewer } from "./viewer";

export function init(client: LanguageClient, channel: vscode.OutputChannel, context: vscode.ExtensionContext) {
  LogFileViewer.setupRptStatusBarVisibility();
  context.subscriptions.push(vscode.commands.registerCommand("hemtt.openLastRPT", async () => {
    let rptPath = "";
    switch (process.platform) {
      case "win32":
        if (process.env.LOCALAPPDATA) {
          rptPath = path.join(process.env.LOCALAPPDATA, "Arma 3");
        } else {
          return vscode.window.showErrorMessage("Could not find local app data path.");
        }
        break;
      case "linux":
        rptPath = linuxRptPath();
        break;
      case "darwin":
        rptPath = path.join(process.env.HOME || "", "Library", "Application Support", "Steam", "steamapps", "common", "Arma 3");
        break;
    }
    if (rptPath === "") {
      return vscode.window.showErrorMessage("Could not determine Arma 3 rpt path for your platform.");
    }
    if (!existsSync(rptPath)) {
      return vscode.window.showErrorMessage(`Could not find Arma 3 rpt path: ${rptPath}`);
    }
    const rptFiles = await readdir(rptPath);
    const rptFile = rptFiles
      .filter(file => file.endsWith(".rpt"))
      .map(file => {
        return {
          path: path.join(rptPath, file),
          mtime: lstatSync(path.join(rptPath, file)).mtime,
        }
      })
      .sort((a, b) => b.mtime.getTime() - a.mtime.getTime())[0];
    if (!rptFile) {
      return vscode.window.showErrorMessage("No rpt files found in Arma 3 directory.");
    }
    if (!existsSync(rptFile.path)) {
      return vscode.window.showErrorMessage(`Could not find rpt file: ${rptFile.path}`);
    }
    LogFileViewer.watch(rptFile.path);
  }));
}

function linuxRptPath(): string {
  if (process.env.HOME) {
    let rptPath = path.join(process.env.HOME, ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/107410/pfx/drive_c/users/steamuser/AppData/Local/Arma 3/");
    if (existsSync(rptPath)) {
      return rptPath;
    }
    rptPath = path.join(process.env.HOME, "/.steam/steam/steamapps/compatdata/107410/pfx/drive_c/users/steamuser/AppData/Local/Arma 3/");
    if (existsSync(rptPath)) {
      return rptPath;
    }
  }
  return "";
}
