// Load PAAs only when visible
const intersectionObserver = new IntersectionObserver(entries => {
    for (const entry of entries.filter(entry => entry.isIntersecting)) {
        replacePaa({ block: entry.target, ...JSON.parse(entry.target.dataset.paaRs) });
    }
});

const onRender = () => {
    // Load / lazy-load PAAs
    for (const paa of findPaas()) {
        const dataset = paa.block.dataset;
        if (dataset.paaRs && JSON.parse(dataset.paaRs).href !== paa.href) {
            dataset.paaRs = JSON.stringify({ href: paa.href, class: paa.class });
            replacePaa(paa);
        } else {
            dataset.paaRs = JSON.stringify({ href: paa.href, class: paa.class });
            intersectionObserver.observe(paa.block);
        }
    }

    // Remove orphaned PAAs
    for (const paaRsEl of document.querySelectorAll(".paa-rs-spinner, .paa-rs-single, .paa-rs-list")) {
        if (!paaRsEl.previousElementSibling || !paaRsEl.previousElementSibling.dataset.paaRs) {
            paaRsEl.remove();
        }
    }
};

const findPaas = () => {
    const viewRawBlocks = Array.from(document.querySelectorAll("div a[href$='.paa'], div a[href$='.paa?raw=true']"))
        .filter(a => a.innerText === "View raw")
        .map(a => ({
            block: a.parentElement,
            href: a.href,
            class: "single",
        }));

    const binaryFileNotShownBlocks = Array.from(document.querySelectorAll("div[data-file-type='.paa'] + div .empty"))
        .map(div => ({
            block: div,
            href: div.parentElement.parentElement.querySelector("a[href$='.paa']").href,
            class: "list",
        }));

    return viewRawBlocks.concat(binaryFileNotShownBlocks);
};

const replacePaa = (paa) => {
    // Block cannot be completely removed because of React
    paa.block.style.display = "none";

    // Remove spinner or previously loaded PAA
    if (paa.block.nextElementSibling && paa.block.nextElementSibling.className.startsWith("paa-rs")) {
        paa.block.nextElementSibling.remove();
    }

    // Add spinner
    const spinner = document.createElement("i");
    spinner.classList.add("paa-rs-spinner");
    paa.block.insertAdjacentElement("afterend", spinner);

    fetch(paa.href + "?raw=true")
        .then(response => response.blob())
        .then(blob => blob.arrayBuffer()).then(ab => {
            chrome.runtime.sendMessage({contentScriptQuery: "fetch_blob", ab: Array.from(new Uint8Array(ab))}, response => {
                // Abort if spinner was removed - user might have clicked and loaded a different PAA already
                if (!document.body.contains(spinner)) {
                    return;
                }
                
                const img = document.createElement("img");
                img.classList.add(`paa-rs-${paa.class}`);
                img.src = response;
                spinner.replaceWith(img);
            });
        });
};

const addStyles = () => {
    const style = document.createElement("style");
    style.appendChild(document.createTextNode(`
        /* Spinner based on https://cssloaders.github.io/ */
        i.paa-rs-spinner {
            display: block;
            width: 58px;
            height: 58px;
            margin: 30px auto;
            border: 2px solid var(--color-fg-subtle);
            border-bottom-color: var(--color-fade-fg-30);
            border-radius: 50%;
            box-sizing: border-box;
            animation: paa-rs-rotation 1s linear infinite;
        }
        @keyframes paa-rs-rotation {
            0% { transform: rotate(0deg) }
            100% { transform: rotate(360deg) }
        }
        img.paa-rs-single {
            display: block;
            max-width: 100%;
            margin: 0 auto;
        }
        img.paa-rs-list {
            display: block;
            max-width: 600px;
            margin: 30px auto;
            background: url("https://viewscreen.githubusercontent.com/static/bg.gif") right bottom #f6f8fa;
            border: 2px solid hsla(210,18%,87%,1);
        }
    `));

    document.head.appendChild(style);
};

const init = () => {
    addStyles();
    onRender();
    
    const mutationObserver = new MutationObserver(_ => onRender());
    mutationObserver.observe(document.body, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["href"],
    });
};

init();
