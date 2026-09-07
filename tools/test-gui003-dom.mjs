#!/usr/bin/env node
/** GUI-003: pin semantic DOM for the five representative release lessons. */

import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { browser, serve } from "./lib/headless.mjs";

const payload = process.argv[2];
if (!payload) {
  console.error("usage: node tools/test-gui003-dom.mjs <payload-dir>");
  process.exit(2);
}

let acceptReport;
const report = new Promise((resolveReport) => (acceptReport = resolveReport));
const handleRequest = async (request, response) => {
  const path = new URL(request.url, "http://x").pathname;
  if (request.method !== "POST" || path !== "/selftest") return false;
  let body = "";
  for await (const chunk of request) body += chunk;
  try { acceptReport(JSON.parse(body)); }
  catch { acceptReport({ ready: false, error: "unparseable GUI-003 report" }); }
  response.writeHead(204).end();
  return true;
};

const { server, origin } = await serve(payload, { handleRequest });
const page = await browser();
try {
  await page.goto(`${origin}/app/?selftest=gui003`);
  const result = await Promise.race([
    report,
    new Promise((resolveReport) => setTimeout(
      () => resolveReport({ ready: false, error: "no GUI-003 report within 180s" }),
      180000,
    )),
  ]);
  if (!result.ready || result.scenario_error) {
    throw new Error(result.scenario_error ?? result.error ?? "bench did not become ready");
  }
  const current = `${JSON.stringify(result.gui003_dom, null, 2)}\n`;
  const goldenPath = resolve("tools/golden/gui003-dom-five.json");
  if (process.env.KEROTAKIS_BLESS_GOLDEN === "1") {
    await writeFile(goldenPath, current);
    console.log(`GUI-003: blessed ${goldenPath}`);
  } else {
    const expected = await readFile(goldenPath, "utf8");
    if (current.trimEnd() !== expected.trimEnd()) {
      console.error(`GUI003-DOM-ACTUAL-BEGIN\n${current}GUI003-DOM-ACTUAL-END`);
      throw new Error(
        `semantic DOM changed; inspect the five lesson scenes before running ` +
        `KEROTAKIS_BLESS_GOLDEN=1 node tools/test-gui003-dom.mjs ${payload}`,
      );
    }
    console.log("GUI-003: five semantic DOM lesson snapshots match");
  }
} finally {
  await page.close();
  server.close();
}
