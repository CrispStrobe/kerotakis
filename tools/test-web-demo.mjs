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
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
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

// The app phones its readiness home (POST /selftest) — a worker-driven
// page cannot be probed by dumping the DOM at a fixed instant, because
// headless virtual time does not advance dedicated workers.
let reportSelftest;
const selftestReport = new Promise((r) => (reportSelftest = r));

const server = createServer(async (req, res) => {
    const path = decodeURIComponent(req.url.split("?")[0].split("#")[0]);
    if (req.method === "POST" && path === "/selftest") {
        let body = "";
        req.on("data", (d) => (body += d));
        req.on("end", () => {
            try { reportSelftest(JSON.parse(body)); } catch { reportSelftest({ ready: false, error: "unparseable report" }); }
            res.writeHead(204).end();
        });
        return;
    }
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
    "reset",
    "add v1 water 100mL",
    "cool v1 40kJ",
    "inspect v1",
].join(";");
const url = `http://127.0.0.1:${PORT}/index.html?r1=1#run=${encodeURIComponent(SCRIPT)}`;

// A persistent profile carries the service-worker registration between
// the two renders: first load online (the worker installs and precaches),
// second load with the server gone — offline-first, proven rather than
// claimed.
const profile = mkdtempSync(join(tmpdir(), "kero-profile-"));

async function render(bin, target = url) {
    return new Promise((res, rej) => {
        const p = spawn(bin, [
            "--headless=new", "--disable-gpu", "--no-sandbox",
            "--no-first-run", "--no-default-browser-check",
            `--user-data-dir=${profile}`,
            "--virtual-time-budget=120000", "--dump-dom", target,
        ]);
        let dom = "";
        let done = false;
        // `--dump-dom` writes the document and can then linger: with a
        // persistent profile, the service-worker process keeps new-headless
        // alive past the dump. The dump ending is the completion signal;
        // the process close is not to be waited for.
        const finish = () => {
            if (done) return;
            done = true;
            clearTimeout(guard);
            try { p.kill("SIGKILL"); } catch { /* already gone */ }
            res(dom);
        };
        const guard = setTimeout(finish, 240000);
        p.stdout.on("data", (d) => (dom += d));
        p.stdout.on("end", finish);
        p.on("close", finish);
        p.on("error", (e) => {
            if (done) return;
            done = true;
            clearTimeout(guard);
            rej(e);
        });
    });
}

let chrome = null;
let dom = null;
for (const bin of CHROME_CANDIDATES) {
    try {
        dom = await render(bin);
        chrome = bin;
        break;
    } catch {
        /* try the next one */
    }
}

if (dom === null) {
    server.close();
    rmSync(profile, { recursive: true, force: true });
    console.error("no Chrome found; set CHROME_PATH");
    process.exit(2);
}

// The bench app is a different page with a different failure surface:
// its worker must download the engine and attach the solver before the
// scene renders. It runs in REAL time (virtual time starves workers) and
// reports its own readiness via POST /selftest.
const appProc = spawn(chrome, [
    "--headless=new", "--disable-gpu", "--no-sandbox",
    "--no-first-run", "--no-default-browser-check",
    `--user-data-dir=${profile}-app`,
    `http://127.0.0.1:${PORT}/app/index.html?selftest=foam`,
]);
const appReport = await Promise.race([
    selftestReport,
    new Promise((r) => setTimeout(() => r({ ready: false, error: "no selftest report within 90s" }), 90000)),
]);
try { appProc.kill("SIGKILL"); } catch { /* already gone */ }
// Chrome takes a beat to actually die; profile removal is best-effort —
// a leftover tmp profile is disposable, an ENOTEMPTY crash is not.
await new Promise((r) => setTimeout(r, 1500));
try { rmSync(`${profile}-app`, { recursive: true, force: true }); } catch { /* still dying */ }

// Second render: the server is gone, the cache is all there is.
server.close();
const offlineDom = await render(chrome);
try { rmSync(profile, { recursive: true, force: true }); } catch { /* best-effort */ }

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

// --- The bench app (web/app) -------------------------------------------
// Both of the app's shipped regressions — the init race that stranded
// "warming up…", and the silent solver-attach failure that turned every
// experiment white — are visible in this one report.
check("the app reached ready (scene arrived)", appReport.ready === true);
check("the app rendered a bench with a vessel", (appReport.vessels ?? 0) >= 1);
check("the app's aqueous engine attached (can_solve)", appReport.can_solve === true);
check("the browser computed foam in both dose vessels", appReport.foam_vessels === 2);
check("the high-dose vessel visibly overflowed", appReport.overflow_vessels >= 1);
check("the browser rendered both foam columns", appReport.rendered_foam === 2);
check("the browser rendered the overflow outside the glass", appReport.rendered_overflow >= 1);
check("the higher KI dose produced more foam", appReport.dose_ordered === true);
check(
    "KI did not fall through to an unsupported-contact warning",
    appReport.unsupported_ki_warning === false,
);
check("the foam browser scenario completed", appReport.scenario_error == null);
check(
    "no engine-loading failure surfaced in the app",
    appReport.error == null,
);
if (appReport.error) console.error(`  app error: ${appReport.error}`);
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
check("inspect shows the remaining liquid water", /water\s+Liquid/.test(transcript));
check("inspect shows the frozen solid water", /water\s+Solid/.test(transcript));
check(
    "inspect is human text, not the state JSON",
    !/\"contents\"|\"thermal_mode\"/.test(transcript),
);
const R1_CASES = [
    "limewater",
    "carbonated_bottle",
    "surface_release",
    "softener_breakthrough",
    "partial_freezing",
];
check(
    "all five R1 scenarios pass in the live page",
    R1_CASES.every((id) => transcript.includes(`R1 ${id}: PASS`)),
);

// Offline-first is the premise, so it gets its own assertions: with the
// server dead, the service worker's cache must still boot the page, start
// the engine, and solve the same experiment.
const offStatus = /<span id="status" class="(\w+)">/.exec(offlineDom);
const offM = /<div id="transcript">([\s\S]*?)<\/div>\s*<aside/.exec(offlineDom);
const offTranscript = offM ? offM[1].replace(/<[^>]+>/g, "\n") : "";
check("offline: the page still boots from the worker's cache", offM !== null);
check("offline: the engine still reports live", offStatus?.[1] === "live");
check("offline: the precipitate still forms", /silver chloride precipitated/.test(offTranscript));
check(
    "offline: all five R1 scenarios pass",
    R1_CASES.every((id) => offTranscript.includes(`R1 ${id}: PASS`)),
);

process.exit(failures === 0 ? 0 : 1);
