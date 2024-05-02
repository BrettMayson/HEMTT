const vscode = acquireVsCodeApi();

let settings = {};

(function () {
  function getSettings() {
    const element = document.getElementById('image-preview-settings');
    if (element) {
      const data = element.getAttribute('data-settings');
      if (data) {
        return JSON.parse(data);
      }
    }

    throw new Error(`Could not load settings`);
  }
  settings = getSettings();

  /**
 * @param {Uint8Array} initialContent
 * @return {Promise<HTMLImageElement>}
 */
  async function loadImageFromData(initialContent) {
    const raw = new Blob([initialContent], { 'type': 'image/png' });
    // Convert to png
    const ab = await raw.arrayBuffer();
    const b = new Uint8Array(ab);
    const result = new ImageResult(b);
    const arr = new Uint8Array(app.memory.buffer, result.data_ptr(), result.data_len());
    const blob = new Blob([arr], { type: 'image/png' });

    return URL.createObjectURL(blob);
  }

  init(WASM_URL).then(a => {
    app = a;
    return fetch(settings.src);
  }).then(response => response.blob()).then(blob => blob.arrayBuffer()).then(ab => loadImageFromData(ab)).then(url => init_image(url));

}());
