import * as fs from "node:fs";
import * as path from "node:path";
import * as https from "node:https";
import * as vscode from "vscode";

export default async function fetchHLS(
  context: vscode.ExtensionContext,
): Promise<string> {
  const platform = process.platform;
  const arch = process.arch;

  let hlsPlatform: string;

  if (platform === "linux" && arch === "x64") {
    hlsPlatform = "hls-linux-x64";
  } else if (platform === "win32" && arch === "x64") {
    hlsPlatform = "hls-windows-x64";
  } else if (platform === "darwin" && arch === "arm64") {
    hlsPlatform = "hls-macos-arm64";
  } else {
    throw new Error(`Unsupported platform: ${platform} ${arch}`);
  }

  const destination = path.join(
    context.extensionPath,
    hlsPlatform,
  );

  const versionFile = path.join(
    context.extensionPath,
    "hls-version.txt",
  );

  // The HLS version always matches the extension version.
  const version = context.extension.packageJSON.version;

  // Check the currently installed HLS version.
  let installedVersion: string | undefined;

  try {
    installedVersion = (
      await fs.promises.readFile(versionFile, "utf8")
    ).trim();
  } catch {
    // Version file doesn't exist.
  }

  // Already have the correct HLS version.
  if (
    installedVersion === version &&
    await fileExists(destination)
  ) {
    return hlsPlatform;
  }

  const url =
    `https://github.com/BrettMayson/hemtt/releases/download/v${version}/${hlsPlatform}`;

  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: `Downloading HEMTT Language Server ${version}`,
      cancellable: false,
    },
    async (progress) => {
      await download(url, destination, (percent) => {
        progress.report({
          increment: percent,
        });
      });
    },
  );

  // Make executable on Linux/macOS.
  if (platform !== "win32") {
    await fs.promises.chmod(destination, 0o755);
  }

  // Record the version we downloaded.
  await fs.promises.writeFile(
    versionFile,
    version,
    "utf8",
  );

  return hlsPlatform;
}

async function fileExists(file: string): Promise<boolean> {
  try {
    await fs.promises.access(file, fs.constants.F_OK);
    return true;
  } catch {
    return false;
  }
}

function download(
  url: string,
  destination: string,
  onProgress: (percent: number) => void,
): Promise<void> {
  return new Promise((resolve, reject) => {
    https.get(url, (response) => {
      // Follow GitHub redirects.
      if (
        response.statusCode &&
        response.statusCode >= 300 &&
        response.statusCode < 400 &&
        response.headers.location
      ) {
        response.resume();

        download(
          response.headers.location,
          destination,
          onProgress,
        )
          .then(resolve)
          .catch(reject);

        return;
      }

      if (response.statusCode !== 200) {
        response.resume();

        reject(
          new Error(
            `Failed to download HLS: HTTP ${response.statusCode}`,
          ),
        );

        return;
      }

      const totalBytes = Number(
        response.headers["content-length"] ?? 0,
      );

      let downloadedBytes = 0;
      let lastPercent = 0;

      const file = fs.createWriteStream(destination);

      response.on("data", (chunk: Buffer) => {
        downloadedBytes += chunk.length;

        if (totalBytes > 0) {
          const percent =
            (downloadedBytes / totalBytes) * 100;

          const increment = percent - lastPercent;
          lastPercent = percent;

          onProgress(increment);
        }
      });

      response.pipe(file);

      file.on("finish", () => {
        file.close();
        resolve();
      });

      file.on("error", (error) => {
        file.destroy();
        reject(error);
      });

      response.on("error", (error) => {
        file.destroy();
        reject(error);
      });
    }).on("error", reject);
  });
}
