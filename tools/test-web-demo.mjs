// Drive the actual page in a real browser, headlessly.
//
// The Node bridge test proves the two wasm modules can talk. It cannot
// prove the *page* works, and the difference is not academic: the first
// browser run failed on something Node does not reproduce at all — Chrome
// refuses `TextDecoder.decode()` on a view into a resizable ArrayBuffer,
// which is exactly what Emscripten's growable heap hands it, so every
// string coming back from the engine threw. Node's ArrayBuffers are not
// resizable, so CI was green while the demo was broken.
//
// Usage: node tools/test-web-demo.mjs <site-dir> [port]

import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, resolve } from "node:path";

const [siteDir, portArg] = process.argv.slice(2);
if (!siteDir) {
    console.error("usage: node test-web-demo.mjs <site-dir> [port]");
    process.exit(2);
}
const PORT = Number(portArg ?? 8731);
const ROOT = resolve(siteDir);

const TYPES = {
    ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
    ".wasm": "application/wasm", ".dat": "text/plain", ".ts": "text/plain",
};

const server = createServer(async (req, res) => {
    const path = decodeURIComponent(req.url.split("?")[0].split("#")[0]);
    try {
        const body = await readFile(join(ROOT, path === "/" ? "index.html" : path));
        res.writeHead(200, { "content-type": TYPES[extname(path)] ?? "application/octet-stream" });
        res.end(body);
    } catch {
        res.writeHead(404).end("not found");
    }
});
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const CHROME_CANDIDATES = [
    process.env.CHROME_PATH,
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "google-chrome", "chromium", "chromium-browser",
].filter(Boolean);

const SCRIPT = [
    "add v1 water 200mL",
    "add v1 NaCl 0.1mol",
    "add v1 AgNO3 0.01mol",
    "look v1",
    "particles v1",
].join(";");
const url = `http://127.0.0.1:${PORT}/index.html#run=${encodeURIComponent(SCRIPT)}`;

async function render(bin) {
    return new Promise((res, rej) => {
        const p = spawn(bin, [
            "--headless=new", "--disable-gpu", "--no-sandbox",
            "--virtual-time-budget=120000", "--dump-dom", url,
        ]);
        let dom = "";
        p.stdout.on("data", (d) => (dom += d));
        p.on("error", rej);
        p.on("close", () => res(dom));
    });
}

let dom = null;
for (const bin of CHROME_CANDIDATES) {
    try {
        dom = await render(bin);
        break;
    } catch {
        /* try the next one */
    }
}
server.close();

if (dom === null) {
    console.error("no Chrome found; set CHROME_PATH");
    process.exit(2);
}

const m = /<div id="transcript">([\s\S]*?)<\/div>\s*<aside/.exec(dom);
const transcript = m
    ? m[1].replace(/<[^>]+>/g, "\n").replace(/&gt;/g, ">").replace(/&lt;/g, "<").replace(/&amp;/g, "&")
    : "";
console.log(transcript.split("\n").filter((l) => l.trim()).join("\n"));

let failures = 0;
const check = (name, cond) => {
    console.log(`${cond ? "ok  " : "FAIL"} ${name}`);
    if (!cond) failures++;
};

const status = /<span id="status" class="(\w+)">([^<]*)</.exec(dom);
check("the page reports a live engine", status?.[1] === "live");
check("nothing threw across the bridge", !/threw|did not start/.test(transcript));
check("no solver failed", !/solver '[^']*' failed/.test(transcript));
// The marquee result: silver and chloride find each other.
check("the precipitate formed", /silver chloride precipitated/.test(transcript));
check("the solution was characterised", /pH \d/.test(transcript));
// The particle view must be drawn from solved speciation, not the inventory.
check("particles came from the speciation", /Na\+\s+\(positive ion\)/.test(transcript));
check(
    "and the too-dilute complex is named rather than dropped",
    /present below one glyph/.test(transcript),
);

process.exit(failures === 0 ? 0 : 1);
