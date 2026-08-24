// OPT-11 baseline: how many times does the wasm bench cross into the
// JS-hosted IPhreeqc engine, per lesson and per step — and how much JSON
// crosses with it?
//
// The roadmap's claim ("one vessel equilibration can make hundreds of
// engine calls") deserves numbers before surgery. This instrument wraps
// the solver hook with a counter and replays the lesson corpus through
// the same two-wasm pairing the browser runs; its output is the Baselines
// row OPT-11's acceptance asks for, before and after.
//
// Usage:
//   node tools/measure-wasm-calls.mjs <bindgen-out-dir> <iphreeqc-build-dir> [lesson.lab ...]
// With no lessons given, the whole lessons/ corpus runs.

import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { PhreeqcPool } from "../web/kerotakis.mjs";

const [outDir, engineDir, ...lessonArgs] = process.argv.slice(2);
if (!outDir || !engineDir) {
    console.error(
        "usage: node measure-wasm-calls.mjs <bindgen-out-dir> <iphreeqc-build-dir> [lesson.lab ...]",
    );
    process.exit(2);
}

const require = createRequire(import.meta.url);
const { Lab } = require(resolve(outDir, "kerotakis_wasm.js"));

// emsdk 3.1.25's node glue reaches for __dirname and require, which do not
// exist in ES-module scope (fixed in later emsdks). Shim them in a patched
// copy, keeping __dirname pointing at the ORIGINAL dir so the glue finds
// its .wasm beside the real module.
async function loadIPhreeqc(dir) {
    const src = readFileSync(resolve(dir, "iphreeqc.mjs"), "utf8");
    const shim =
        `import { createRequire as __cr } from "node:module";\n` +
        `const require = __cr(import.meta.url);\n` +
        `const __dirname = ${JSON.stringify(resolve(dir))};\n` +
        `const __filename = ${JSON.stringify(resolve(dir, "iphreeqc.mjs"))};\n`;
    const patched = resolve(dir, "iphreeqc.node-shim.mjs");
    const { writeFileSync } = await import("node:fs");
    writeFileSync(patched, shim + src);
    const factory = (await import("file://" + patched)).default;
    // The glue's own path arithmetic yields a `file:` URL it then hands to
    // fs. Sidestep file resolution entirely: hand it the wasm bytes.
    const wasmBinary = readFileSync(resolve(dir, "iphreeqc.wasm"));
    return (opts = {}) => factory({ wasmBinary, ...opts });
}
const createIPhreeqc = await loadIPhreeqc(engineDir);
const loadDatabase = async (file) =>
    readFileSync(resolve("vendor/iphreeqc/database", file), "latin1");

const pool = await PhreeqcPool.create(createIPhreeqc, loadDatabase);

// The counting wrapper: every crossing, its payload sizes, its wall time.
const stats = { calls: 0, ms: 0, bytesIn: 0, bytesOut: 0 };
const countingSolve = (dbTag, input) => {
    const t0 = performance.now();
    const out = pool.solve(dbTag, input);
    stats.ms += performance.now() - t0;
    stats.calls += 1;
    stats.bytesIn += input.length;
    stats.bytesOut += out.length;
    return out;
};
const snapshot = () => ({ ...stats });
const delta = (a, b) => ({
    calls: b.calls - a.calls,
    ms: b.ms - a.ms,
    bytesIn: b.bytesIn - a.bytesIn,
    bytesOut: b.bytesOut - a.bytesOut,
});

const lessons =
    lessonArgs.length > 0
        ? lessonArgs
        : readdirSync("lessons")
              .filter((f) => f.endsWith(".lab"))
              .sort()
              .map((f) => resolve("lessons", f));

const fmtKB = (b) => (b / 1024).toFixed(1).padStart(8);
const grand = snapshot();
console.log("lesson                        calls      ms    in-KB   out-KB  costliest step");

for (const path of lessons) {
    const name = path.split("/").pop().replace(/\.lab$/, "");
    const lab = new Lab();
    lab.setSolver(countingSolve);
    const before = snapshot();
    let costliest = { calls: -1, line: "" };
    for (const raw of readFileSync(path, "utf8").split("\n")) {
        const line = raw.trim();
        if (!line || line.startsWith("#")) continue;
        const stepBefore = snapshot();
        try {
            lab.runScript(line);
        } catch (e) {
            console.error(`  ${name}: '${line}' failed: ${e.message ?? e}`);
            continue;
        }
        const d = delta(stepBefore, snapshot());
        if (d.calls > costliest.calls) costliest = { calls: d.calls, line };
    }
    const d = delta(before, snapshot());
    console.log(
        `${name.padEnd(26)} ${String(d.calls).padStart(7)} ${d.ms.toFixed(0).padStart(7)} ` +
            `${fmtKB(d.bytesIn)} ${fmtKB(d.bytesOut)}  ${costliest.calls}× '${costliest.line}'`,
    );
}

const total = delta(grand, snapshot());
console.log(
    `\nTOTAL: ${total.calls} engine crossings, ${total.ms.toFixed(0)} ms in the hook, ` +
        `${(total.bytesIn / 1024).toFixed(0)} KB in / ${(total.bytesOut / 1024).toFixed(0)} KB out`,
);
