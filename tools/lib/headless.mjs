/**
 * Drive a real headless Chrome over the DevTools protocol, with no
 * dependencies: Node's global `WebSocket` speaks CDP.
 *
 * Shared by `tools/test-pwa.mjs` and `tools/gen-pwa-screenshots.mjs`,
 * which need the same two things — the payload served from the SUBPATH it
 * is really deployed under, and a page to talk to.
 *
 * The subpath is not incidental. Kerotakis is served from
 * `crispstrobe.github.io/kerotakis/`, and a payload served at the root
 * hides every relative-URL defect in the manifest and the service worker.
 */

import { createServer } from "node:http";
import { createReadStream, readdirSync } from "node:fs";
import { stat, mkdtemp, rm } from "node:fs/promises";
import { spawn } from "node:child_process";
import { join, extname, normalize } from "node:path";
import { tmpdir } from "node:os";

/** The deploy prefix these harnesses reproduce. */
export const PREFIX = "/kerotakis";

/**
 * Chrome-for-Testing builds Playwright has already downloaded, newest first.
 *
 * Worth looking for because `/usr/bin/chromium` on Ubuntu is a snap
 * wrapper, and a snap refuses to launch outside a session snapd has
 * tagged: `<cgroup> is not a snap cgroup for tag snap.chromium.chromium`,
 * exit code 1. That happens in exactly the places this harness is used —
 * a detached `screen`, an ssh command with no login session, an agent's
 * shell — so "chromium is installed" and "chromium will run" are
 * different facts on the same machine.
 */
function playwrightChromes() {
  const roots = [
    process.env.PLAYWRIGHT_BROWSERS_PATH,
    join(process.env.HOME ?? "", ".cache/ms-playwright"),
    "/mnt/volume1/opt/ms-playwright",
  ].filter(Boolean);
  const found = [];
  for (const root of roots) {
    let entries = [];
    try {
      entries = readdirSync(root);
    } catch {
      continue;
    }
    // Prefer the full browser over the headless shell: this harness passes
    // `--headless=new`, which the shell does not need and does not want.
    for (const dir of entries.filter((d) => d.startsWith("chromium-")).sort().reverse()) {
      found.push(join(root, dir, "chrome-linux64/chrome"), join(root, dir, "chrome-linux/chrome"));
    }
    for (const dir of entries.filter((d) => d.startsWith("chromium_headless_shell-")).sort().reverse()) {
      found.push(join(root, dir, "chrome-headless-shell-linux64/chrome-headless-shell"));
    }
  }
  return found;
}

const CHROME_CANDIDATES = [
  process.env.CHROME_PATH,
  process.env.CHROME,
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/usr/bin/google-chrome",
  ...playwrightChromes(),
  "/usr/bin/chromium",
  "/usr/bin/chromium-browser",
].filter(Boolean);

const MIME = {
  ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
  ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm",
  ".svg": "image/svg+xml", ".png": "image/png", ".dat": "text/plain",
  ".webmanifest": "application/manifest+json", ".lab": "text/plain",
  ".postcard": "application/octet-stream", ".pack": "application/octet-stream",
};

/** Serve `dir` under PREFIX on an ephemeral port. */
export function serve(dir, { handleRequest } = {}) {
  const server = createServer(async (req, res) => {
    if (handleRequest && await handleRequest(req, res)) return;
    let path = decodeURIComponent(new URL(req.url, "http://x").pathname);
    if (!path.startsWith(PREFIX)) {
      res.writeHead(404).end("outside the deploy prefix");
      return;
    }
    path = path.slice(PREFIX.length) || "/";
    let file = join(dir, normalize(path));
    try {
      if ((await stat(file)).isDirectory()) file = join(file, "index.html");
      const info = await stat(file);
      res.writeHead(200, {
        "content-type": MIME[extname(file)] ?? "application/octet-stream",
        "content-length": info.size,
      });
      createReadStream(file).pipe(res);
    } catch {
      res.writeHead(404).end("not found");
    }
  });
  return new Promise((resolve) =>
    server.listen(0, "127.0.0.1", () =>
      resolve({ server, origin: `http://127.0.0.1:${server.address().port}${PREFIX}` }),
    ),
  );
}

/** One CDP connection, with the flat-session multiplexing CDP wants. */
export class Cdp {
  #ws;
  #next = 1;
  #pending = new Map();
  #listeners = [];

  static async attach(url) {
    const cdp = new Cdp();
    cdp.#ws = new WebSocket(url);
    await new Promise((res, rej) => {
      cdp.#ws.onopen = res;
      cdp.#ws.onerror = () => rej(new Error(`cannot reach ${url}`));
    });
    cdp.#ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      if (msg.id && cdp.#pending.has(msg.id)) {
        const { resolve, reject } = cdp.#pending.get(msg.id);
        cdp.#pending.delete(msg.id);
        msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
      } else if (msg.method) {
        for (const l of cdp.#listeners) l(msg);
      }
    };
    return cdp;
  }

  send(method, params = {}, sessionId) {
    const id = this.#next++;
    this.#ws.send(JSON.stringify({ id, method, params, sessionId }));
    return new Promise((resolve, reject) => this.#pending.set(id, { resolve, reject }));
  }

  on(fn) {
    this.#listeners.push(fn);
  }

  close() {
    this.#ws.close();
  }
}

/**
 * Launch headless Chrome and attach. Returns the connection, a page
 * session, and a `close()` that leaves nothing behind.
 */
export async function browser({ disableGpu = true, extraArgs = [], headless = true } = {}) {
  const profile = await mkdtemp(join(tmpdir(), "kero-headless-"));
  const args = [
    ...(headless ? ["--headless=new"] : []),
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "--no-first-run",
    "--no-default-browser-check",
    ...(disableGpu ? ["--disable-gpu"] : []),
    "--no-sandbox",
    "--hide-scrollbars",
    ...extraArgs,
    "about:blank",
  ];

  // This file speaks CDP over Node's GLOBAL WebSocket, which landed in
  // Node 22. On Node 20 every candidate launches perfectly and then the
  // attach throws `WebSocket is not defined`, which reads as "no usable
  // Chrome found" and sends the reader after the wrong thing entirely.
  if (typeof globalThis.WebSocket === "undefined") {
    throw new Error(
      `this harness needs Node 22's global WebSocket to speak CDP; this is ${process.version}`,
    );
  }

  let child = null;
  let lastError = null;
  /** What each candidate said when it refused, for the error at the end. */
  const attempts = new Map();
  for (const bin of CHROME_CANDIDATES) {
    // Kept out of the Promise so the catch below can quote it. A refusal
    // explains itself on stderr and then exits; the exit code alone does
    // not distinguish "no such binary" from "snapd said no".
    let stderr = "";
    try {
      child = spawn(bin, args, { stdio: ["ignore", "ignore", "pipe"] });
      // Chrome announces the debugger URL on stderr when it picks the port.
      const wsUrl = await new Promise((resolve, reject) => {
        let buf = "";
        const timer = setTimeout(
          () => reject(new Error("Chrome never announced a debugger URL")),
          30000,
        );
        // A missing binary surfaces as an asynchronous 'error' event, not a
        // throw from spawn(). Without this handler it escapes the try/catch
        // entirely and kills the run on the first candidate that does not
        // exist — which is every candidate but one, on every platform.
        child.on("error", (err) => {
          clearTimeout(timer);
          reject(err);
        });
        child.stderr.on("data", (d) => {
          buf += d;
          stderr += d;
          const m = buf.match(/ws:\/\/[^\s]+/);
          if (m) {
            clearTimeout(timer);
            resolve(m[0]);
          }
        });
        child.on("exit", (code) => {
          clearTimeout(timer);
          reject(new Error(`Chrome exited with ${code}`));
        });
      });
      const cdp = await Cdp.attach(wsUrl);
      const { targetId } = await cdp.send("Target.createTarget", { url: "about:blank" });
      const { sessionId } = await cdp.send("Target.attachToTarget", {
        targetId,
        flatten: true,
      });
      await cdp.send("Page.enable", {}, sessionId);
      await cdp.send("Runtime.enable", {}, sessionId);
      await cdp.send("Network.enable", {}, sessionId);

      const goto = async (url) => {
        const loaded = new Promise((resolve) =>
          cdp.on((msg) => {
            if (msg.method === "Page.loadEventFired" && msg.sessionId === sessionId) resolve();
          }),
        );
        await cdp.send("Page.navigate", { url }, sessionId);
        await loaded;
      };

      const evaluate = async (expression) => {
        const { result, exceptionDetails } = await cdp.send(
          "Runtime.evaluate",
          { expression, awaitPromise: true, returnByValue: true },
          sessionId,
        );
        if (exceptionDetails) {
          throw new Error(exceptionDetails.exception?.description ?? "evaluation failed");
        }
        return result.value;
      };

      return {
        cdp,
        sessionId,
        goto,
        evaluate,
        async close() {
          cdp.close();
          child.kill();
          await rm(profile, { recursive: true, force: true }).catch(() => {});
        },
      };
    } catch (err) {
      lastError = err;
      attempts.set(bin, `${err.message}${stderr ? ` — ${stderr.trim().split("\n").slice(-1)[0]}` : ""}`);
      try {
        child?.kill();
      } catch {
        /* already gone */
      }
    }
  }
  await rm(profile, { recursive: true, force: true }).catch(() => {});
  // Say what each candidate said. "Chrome exited with 1" on its own sent
  // one reader looking for a missing binary when the binary was there and
  // snapd had refused the cgroup — the answer was in Chrome's stderr and
  // this harness was throwing it away.
  const tried = CHROME_CANDIDATES.map((c) => `      ${c}: ${attempts.get(c) ?? "not tried"}`).join("\n");
  throw new Error(
    `no usable Chrome found (set CHROME_PATH): ${lastError?.message}\n    tried:\n${tried}`,
  );
}

/** Poll the page until `predicate` (a JS expression) is truthy, or give up. */
export async function waitFor(page, expression, { timeout = 30000, step = 250 } = {}) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (await page.evaluate(`Boolean(${expression})`)) return true;
    await new Promise((r) => setTimeout(r, step));
  }
  return false;
}
