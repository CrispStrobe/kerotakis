// Track A meets Track B: a real aqueous solver in a browser runtime.
//
// The two halves have been built and tested separately since P0 — the Rust
// bench compiles to wasm32-unknown-unknown, and IPhreeqc compiles under
// Emscripten and answers the AgCl case. What was never tested is the pair
// working *together*, which is the only configuration a school will ever
// see.
//
// The decisive check is not "does it return a number". It is that the
// browser returns the *same* number as the desktop build for a state
// nobody pre-computed — because the whole design rests on there being one
// engine reached by one path, rather than a web version that could quietly
// drift from the one the codex was linted against.
//
// Usage:
//   node tools/test-wasm-bridge.mjs <bindgen-out-dir> <iphreeqc-build-dir> [expected.json]

import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { openLab } from "../web/kerotakis.mjs";

const [outDir, engineDir, expectedPath] = process.argv.slice(2);
if (!outDir || !engineDir) {
    console.error(
        "usage: node test-wasm-bridge.mjs <bindgen-out-dir> <iphreeqc-build-dir> [expected.json]",
    );
    process.exit(2);
}

const require = createRequire(import.meta.url);
const { Lab } = require(resolve(outDir, "kerotakis_wasm.js"));
const createIPhreeqc = (await import(resolve(engineDir, "iphreeqc.mjs"))).default;

let failures = 0;
const check = (name, cond, detail = "") => {
    if (cond) {
        console.log(`ok   ${name}`);
    } else {
        console.error(`FAIL ${name}${detail ? ": " + detail : ""}`);
        failures++;
    }
};

const loadDatabase = async (file) =>
    readFileSync(resolve("vendor/iphreeqc/database", file), "latin1");

// --- Without a solver: honest, and not a laboratory. -------------------
const bare = new Lab();
check("a bare bench admits it cannot solve", bare.canSolve() === false);

// A state nobody pre-warmed must be refused rather than guessed at.
const bareOut = JSON.parse(
    bare.runScript("add v1 water 137mL\nadd v1 NaCl 0.037mol\n"),
);
const bareRendered = bareOut.rendered.join(" ");
check(
    "and reports a miss rather than inventing an answer",
    /not in the shipped results|could not|cache-only/i.test(bareRendered),
    bareRendered.slice(0, 200),
);

// --- With the engine wired in. ----------------------------------------
const lab = await openLab(Lab, { createIPhreeqc, loadDatabase });
check("with IPhreeqc attached, the bench can solve", lab.canSolve() === true);

// Deliberately odd quantities: nothing in any lesson or codex entry uses
// these, so a cache cannot answer and the engine must.
const SCRIPT = "add v1 water 137mL\nadd v1 NaCl 0.037mol\n";
const out = JSON.parse(lab.runScript(SCRIPT));
const vessel = out.bench.vessels[0];
check(
    "a state nobody pre-computed is now solved",
    vessel.solution != null,
    JSON.stringify(out.rendered).slice(0, 300),
);

if (vessel.solution) {
    const { ph, ionic_strength } = vessel.solution;
    console.log(`     browser: pH ${ph.toFixed(4)}, I = ${ionic_strength.toFixed(6)} mol/kgw`);
    check("the pH is a real number", Number.isFinite(ph) && ph > 0 && ph < 14);

    if (expectedPath) {
        // The desktop build's answer for the identical script, produced by
        // `kero run --json`. Same engine, same routing, same cache logic;
        // any disagreement means the web has drifted.
        const expected = JSON.parse(readFileSync(expectedPath, "utf8"));
        const dph = Math.abs(ph - expected.ph);
        const dmu = Math.abs(ionic_strength - expected.ionic_strength);
        console.log(
            `     desktop: pH ${expected.ph.toFixed(4)}, I = ${expected.ionic_strength.toFixed(6)} mol/kgw`,
        );
        check("browser and desktop agree on pH", dph < 1e-6, `Δ = ${dph}`);
        check("browser and desktop agree on ionic strength", dmu < 1e-9, `Δ = ${dmu}`);
    }
}

// The species distribution has to survive the trip too: it is parsed out of
// the run report, which crosses the bridge as a string.
if (vessel.solution) {
    check(
        "the species distribution came across",
        vessel.solution.species.length > 2,
        `${vessel.solution.species.length} species`,
    );
}

process.exit(failures === 0 ? 0 : 1);
