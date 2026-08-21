// P0 gate, web half: run the AgCl end-to-end case through the Emscripten
// build of IPhreeqc under Node (same wasm as the browser), string-in /
// value-out, no filesystem.
//
// Usage: node tools/test-iphreeqc-wasm.mjs <build-dir> <path-to-wateq4f.dat>

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const [buildDir, dbPath] = process.argv.slice(2);
if (!buildDir || !dbPath) {
    console.error("usage: node test-iphreeqc-wasm.mjs <build-dir> <wateq4f.dat>");
    process.exit(2);
}

const createIPhreeqc = (await import(resolve(buildDir, "iphreeqc.mjs"))).default;
const mod = await createIPhreeqc();

const call = (name, ret, args) => mod.cwrap(name, ret, args);
const CreateIPhreeqc = call("CreateIPhreeqc", "number", []);
const LoadDatabaseString = call("LoadDatabaseString", "number", ["number", "string"]);
const RunString = call("RunString", "number", ["number", "string"]);
const GetErrorString = call("GetErrorString", "string", ["number"]);
const GetSpeciesDeltaH = call("GetSpeciesDeltaH", "number", ["number", "string", "number"]);
const SetOutputFileOn = call("SetOutputFileOn", "number", ["number", "number"]);
const SetSelectedOutputStringOn = call("SetSelectedOutputStringOn", "number", ["number", "number"]);
const LineCount = call("GetSelectedOutputStringLineCount", "number", ["number"]);
const Line = call("GetSelectedOutputStringLine", "string", ["number", "number"]);

const id = CreateIPhreeqc();
if (id < 0) throw new Error("CreateIPhreeqc failed");
SetOutputFileOn(id, 0);

const db = readFileSync(dbPath, "utf8");
if (LoadDatabaseString(id, db) !== 0) {
    throw new Error("database load failed: " + GetErrorString(id));
}
// Loading a database resets the selected-output string flag — enable AFTER.
SetSelectedOutputStringOn(id, 1);

const input = `
SOLUTION 1
    units     mol/kgw
    temp      25
    pH        7  charge
    Na        0.01
    Cl        0.01
    Ag        0.01
    N(5)      0.01
EQUILIBRIUM_PHASES 1
    Cerargyrite 0 0
SELECTED_OUTPUT
    -reset    false
    -si       Cerargyrite
    -equilibrium_phases Cerargyrite
END
`;
if (RunString(id, input) !== 0) {
    throw new Error("run failed: " + GetErrorString(id));
}

const rows = [];
for (let n = 0; n < LineCount(id); n++) {
    rows.push(Line(id, n).split("\t").map((s) => s.trim()));
}
const header = rows[0];
const last = rows[rows.length - 1];
const value = (col) => parseFloat(last[header.indexOf(col)]);

const si = value("si_Cerargyrite");
const precipitated = value("Cerargyrite");
console.log(`si_Cerargyrite = ${si}, precipitated = ${precipitated} mol`);

if (Math.abs(si) > 0.01) throw new Error(`SI should be ~0 at equilibrium, got ${si}`);
if (!(precipitated > 0.0099 && precipitated <= 0.01)) {
    throw new Error(`expected ~0.01 mol AgCl, got ${precipitated}`);
}

const deltaHPtr = mod._malloc(8);
try {
    const status = GetSpeciesDeltaH(id, "OH-", deltaHPtr);
    const deltaH = new DataView(mod.HEAPU8.buffer).getFloat64(deltaHPtr, true);
    if (status !== 0 || !Number.isFinite(deltaH) || !(deltaH > 50 && deltaH < 65)) {
        throw new Error(`native OH- reaction enthalpy failed: status ${status}, value ${deltaH}`);
    }
} finally {
    mod._free(deltaHPtr);
}

const basicStatus = RunString(
    id,
    `SOLUTION 2\n    pH 7\nSELECTED_OUTPUT\n    -reset false\nUSER_PUNCH\n10 PUNCH 12345\nEND\n`,
);
const basicError = GetErrorString(id);
if (process.env.IPHREEQC_BASIC_MODE === "my-basic") {
    const basicLines = [];
    for (let n = 0; n < LineCount(id); n++) basicLines.push(Line(id, n));
    if (basicStatus !== 0 || !basicLines.some((line) => line.includes("12345"))) {
        throw new Error(`MY-BASIC USER_PUNCH failed: ${basicError}; ${basicLines.join(" | ")}`);
    }
} else if (basicStatus === 0 || !basicError.includes("PHREEQC BASIC capability is disabled")) {
    throw new Error(`USER_PUNCH was not rejected by the no-BASIC module: ${basicError}`);
}

console.log(
    `OK: PHREEQC computes AgCl precipitation, exposes native thermochemistry, and ${
        process.env.IPHREEQC_BASIC_MODE === "my-basic" ? "executes MY-BASIC" : "rejects BASIC"
    } in WebAssembly without filesystem access.`,
);
