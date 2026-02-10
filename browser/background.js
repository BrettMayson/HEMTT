import init, { FromPaaResult } from './hemtt_paa.js';

async function run() {
    // Firefox linter has trouble with 'import', so import directly here
    let app = await init();
    chrome.runtime.onMessage.addListener(function(request, sender, sendResponse) {
        if (request.contentScriptQuery == "fetch_blob") {
            let b = new Uint8Array(request.ab);
            let result = new FromPaaResult(b);
            let arr = new Uint8Array(app.memory.buffer, result.data_ptr(), result.data_len());
            let blob = new Blob([arr], { type: 'image/png' });
            let reader = new FileReader();
            reader.readAsDataURL(blob);
            reader.onload = function () {
                sendResponse(reader.result);
            };
            return true;
        }
    });
}
run();
