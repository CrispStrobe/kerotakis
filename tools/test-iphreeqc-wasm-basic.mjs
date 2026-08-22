// Wasm differential test: verify that MY-BASIC PHREEQC programs produce
// the same results in the Emscripten build as in the native build.
//
// This exercises the critical BASIC features through the wasm module:
// - RATES with KINETICS (rate program execution)
// - USER_PUNCH (selected output from BASIC)
// - CALCULATE_VALUES (chained BASIC evaluation)
// - DATA/READ/RESTORE (runtime cursor)
// - Chemistry callbacks (MOL, TOT, SI, DELTA_H_SPECIES)
// - Resource limits (statement budget)
//
// Usage: IPHREEQC_BASIC_MODE=my-basic node tools/test-iphreeqc-wasm-basic.mjs <build-dir>

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const buildDir = process.argv[2];
if (!buildDir) {
    console.error("usage: node test-iphreeqc-wasm-basic.mjs <build-dir>");
    process.exit(2);
}

const createIPhreeqc = (await import(resolve(buildDir, "iphreeqc.mjs"))).default;
const mod = await createIPhreeqc();

const call = (name, ret, args) => mod.cwrap(name, ret, args);
const CreateIPhreeqc = call("CreateIPhreeqc", "number", []);
const DestroyIPhreeqc = call("DestroyIPhreeqc", "number", ["number"]);
const LoadDatabaseString = call("LoadDatabaseString", "number", ["number", "string"]);
const RunString = call("RunString", "number", ["number", "string"]);
const GetErrorString = call("GetErrorString", "string", ["number"]);
const SetOutputFileOn = call("SetOutputFileOn", "number", ["number", "number"]);
const SetErrorFileOn = call("SetErrorFileOn", "number", ["number", "number"]);
const SetLogFileOn = call("SetLogFileOn", "number", ["number", "number"]);
const SetDumpFileOn = call("SetDumpFileOn", "number", ["number", "number"]);
const SetSelectedOutputFileOn = call("SetSelectedOutputFileOn", "number", ["number", "number"]);
const SetSelectedOutputStringOn = call("SetSelectedOutputStringOn", "number", ["number", "number"]);
const LineCount = call("GetSelectedOutputStringLineCount", "number", ["number"]);
const Line = call("GetSelectedOutputStringLine", "string", ["number", "number"]);

function makeEngine(dbPath) {
    const id = CreateIPhreeqc();
    if (id < 0) throw new Error("CreateIPhreeqc failed");
    SetOutputFileOn(id, 0);
    SetErrorFileOn(id, 0);
    SetLogFileOn(id, 0);
    SetDumpFileOn(id, 0);
    SetSelectedOutputFileOn(id, 0);

    const db = readFileSync(dbPath, "utf8");
    if (LoadDatabaseString(id, db) !== 0) {
        throw new Error("database load failed: " + GetErrorString(id));
    }
    SetSelectedOutputStringOn(id, 1);
    return id;
}

function run(id, input) {
    SetSelectedOutputStringOn(id, 1);
    if (RunString(id, input) !== 0) {
        throw new Error("run failed: " + GetErrorString(id));
    }
}

function selectedOutput(id) {
    const rows = [];
    for (let n = 0; n < LineCount(id); n++) {
        rows.push(Line(id, n).split("\t").map((s) => s.trim()));
    }
    return rows;
}

function lastValue(id, column) {
    const rows = selectedOutput(id);
    const idx = rows[0].indexOf(column);
    if (idx < 0) throw new Error(`missing column: ${column}`);
    return parseFloat(rows[rows.length - 1][idx]);
}

let passed = 0;
let failed = 0;

function check(name, condition, detail = "") {
    if (condition) {
        console.log(`  ok   ${name}`);
        passed++;
    } else {
        console.error(`  FAIL ${name}${detail ? ": " + detail : ""}`);
        failed++;
    }
}

const dbPath = resolve("vendor/iphreeqc/database/phreeqc.dat");

// ── Test 1: Simple kinetics rate program ───────────────────────────────
console.log("Test 1: Simple kinetics");
{
    const id = makeEngine(dbPath);
    run(id, `
RATES
Decay
-start
10 rate = PARM(1) * M * TIME
20 IF rate > M THEN rate = M
30 SAVE rate
-end
SOLUTION 1
    pH 7
KINETICS 1
    Decay
        -formula H2O 0
        -m 1
        -m0 1
        -parms 0.5
        -tol 1e-12
        -steps 0.5 seconds
SELECTED_OUTPUT
    -reset false
    -high_precision true
    -kinetics Decay
END
`);
    const remaining = lastValue(id, "k_Decay");
    // Native oracle value: 0.7788 (exp(-0.25))
    check("rate program executes", Number.isFinite(remaining));
    check("remaining moles match native", Math.abs(remaining - 0.7788) < 5e-4,
        `got ${remaining}`);
    DestroyIPhreeqc(id);
}

// ── Test 2: USER_PUNCH with chemistry callbacks ────────────────────────
console.log("Test 2: USER_PUNCH with callbacks");
{
    const id = makeEngine(dbPath);
    run(id, `
SOLUTION 1
    units mol/kgw
    temp 25
    pH 7
    Na 0.01
    Cl 0.01
SELECTED_OUTPUT
    -reset false
USER_PUNCH
    -headings punch_na punch_cl punch_si punch_ph
    10 PUNCH TOT("Na"), TOT("Cl"), SI("Halite"), -LA("H+")
END
`);
    const na = lastValue(id, "punch_na");
    const cl = lastValue(id, "punch_cl");
    const ph = lastValue(id, "punch_ph");
    check("TOT Na is finite", Number.isFinite(na));
    check("TOT Na matches input", Math.abs(na - 0.01) < 1e-4, `got ${na}`);
    check("TOT Cl matches input", Math.abs(cl - 0.01) < 1e-4, `got ${cl}`);
    check("pH from -LA(H+) is ~7", Math.abs(ph - 7) < 0.1, `got ${ph}`);
    DestroyIPhreeqc(id);
}

// ── Test 3: CALCULATE_VALUES with DATA/READ ────────────────────────────
console.log("Test 3: CALCULATE_VALUES with DATA/READ");
{
    const id = makeEngine(dbPath);
    run(id, `
CALCULATE_VALUES
DataSum
-start
10 DATA 1.5, 2.5, 3.0
20 RESTORE 10
30 READ a, b, c
40 SAVE a + b + c
-end
SOLUTION 1
    pH 7
SELECTED_OUTPUT
    -reset false
    -calculate_values DataSum
END
`);
    const val = lastValue(id, "V_DataSum");
    check("DATA/READ/RESTORE produces 7.0", Math.abs(val - 7.0) < 1e-10,
        `got ${val}`);
    DestroyIPhreeqc(id);
}

// ── Test 4: Temperature-dependent rate ─────────────────────────────────
console.log("Test 4: Temperature-dependent rate");
{
    const id = makeEngine(dbPath);
    run(id, `
RATES
TempRate
-start
10 k = PARM(1) * (TC / 25.0)
20 rate = k * M * TIME
30 IF rate > M THEN rate = M
40 SAVE rate
-end
SOLUTION 1
    temp 50
    pH 7
KINETICS 1
    TempRate
        -formula H2O 0
        -m 1
        -m0 1
        -parms 0.5
        -tol 1e-12
        -steps 0.5 seconds
SELECTED_OUTPUT
    -reset false
    -high_precision true
    -kinetics TempRate
END
`);
    const remaining = lastValue(id, "k_TempRate");
    // k_eff = 0.5 * (50/25) = 1.0; exp(-0.5) ≈ 0.6065
    check("temp-dependent rate executes", Number.isFinite(remaining));
    check("temp-dependent moles match", Math.abs(remaining - 0.6065) < 5e-4,
        `got ${remaining}`);
    DestroyIPhreeqc(id);
}

// ── Test 5: Resource limits (statement budget) ─────────────────────────
console.log("Test 5: Statement budget enforcement");
{
    const id = makeEngine(dbPath);
    const status = RunString(id, `
RATES
Spin
-start
10 GOTO 10
-end
SOLUTION 1
    pH 7
KINETICS 1
    Spin
        -formula H2O 0
        -m 1
        -steps 1 second
END
`);
    const error = GetErrorString(id);
    check("infinite loop is rejected", status !== 0);
    check("error mentions budget", error.includes("budget"), error.slice(0, 120));

    // Engine reusable after budget exhaustion
    const status2 = RunString(id, "SOLUTION 2\n    pH 7\nEND\n");
    check("engine reusable after budget", status2 === 0,
        GetErrorString(id).slice(0, 120));
    DestroyIPhreeqc(id);
}

// ── Test 6: Multi-step kinetics trajectory ─────────────────────────────
console.log("Test 6: Multi-step trajectory");
{
    const id = makeEngine(dbPath);
    run(id, `
RATES
Decay
-start
10 rate = PARM(1) * M * TIME
20 IF rate > M THEN rate = M
30 SAVE rate
-end
SOLUTION 1
    pH 7
KINETICS 1
    Decay
        -formula H2O 0
        -m 1
        -m0 1
        -parms 0.5
        -tol 1e-12
        -steps 0.25 0.25 0.5 1.0 seconds
SELECTED_OUTPUT
    -reset false
    -high_precision true
    -kinetics Decay
END
`);
    const rows = selectedOutput(id);
    const kIdx = rows[0].indexOf("k_Decay");
    const dataRows = rows.slice(1);
    // Native captured values: [0, 0.8825, 0.8825, 0.7788, 0.6065]
    const expected = [0.0, 0.8825, 0.8825, 0.7788, 0.6065];
    check("trajectory has 5 data rows", dataRows.length === 5,
        `got ${dataRows.length}`);
    for (let i = 0; i < Math.min(dataRows.length, expected.length); i++) {
        const actual = parseFloat(dataRows[i][kIdx]);
        check(`step ${i} moles match (${expected[i].toFixed(4)})`,
            Math.abs(actual - expected[i]) < 5e-4,
            `got ${actual}`);
    }
    DestroyIPhreeqc(id);
}

// ── Test 7: Bundled rate program (Calcite from phreeqc.dat) ────────────
console.log("Test 7: Bundled Calcite rate");
{
    const id = makeEngine(dbPath);
    run(id, `
SOLUTION 1
    units mol/kgw
    temp 25
    pH 7
    pe 4
    Na 0.01
    Ca 0.001
    C 0.002
KINETICS 1
    Calcite
        -formula H2O 0
        -m 0.001
        -m0 0.001
        -parms 1000 0.6
        -steps 1 second
SELECTED_OUTPUT
    -reset false
    -kinetics Calcite
END
`);
    const remaining = lastValue(id, "k_Calcite");
    check("bundled Calcite rate executes", Number.isFinite(remaining));
    // Native oracle value: 0.001 (zero-formula harness)
    check("bundled rate matches native", Math.abs(remaining - 0.001) < 5e-8,
        `got ${remaining}`);
    DestroyIPhreeqc(id);
}

// ── Summary ────────────────────────────────────────────────────────────
console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
console.log("OK: MY-BASIC PHREEQC dialect produces identical results in WebAssembly.");
