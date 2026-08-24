// GUI-001: the EngineHost protocol conformance suite, wasm-host half.
//
// crates/kerotakis-cli/tests/protocol_conformance.rs pins the step/event/
// scene shapes through the CLI's --json stream; this runs the SAME
// structural assertions against the wasm Lab — the two hosts must expose
// one shape, and a drift between them fails here before a client sees it.
//
// Usage: node tools/test-protocol-conformance.mjs <bindgen-out-dir>
// (nodejs-target wasm-bindgen output; runs the whole lessons/ corpus,
// shipped-results bench — shapes do not depend on the live engine.)

import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";

const [outDir] = process.argv.slice(2);
if (!outDir) {
    console.error("usage: node test-protocol-conformance.mjs <bindgen-out-dir>");
    process.exit(2);
}
const require = createRequire(import.meta.url);
const { Lab } = require(resolve(outDir, "kerotakis_wasm.js"));

let checks = 0;
let failures = 0;
const fail = (context, what) => {
    console.error(`FAIL ${context}: ${what}`);
    failures++;
};

const VESSEL_KEYS = [
    "id", "label", "solids", "bubbling", "boundary",
    "temperature_k", "pressure_pa", "elapsed_s", "words", "badges",
];
const LIQUID_KEYS = ["volume_l", "srgb", "colour_word", "cloudiness", "path_length_cm"];
const SOLID_KEYS = ["species", "name", "moles", "srgb", "colour_word", "metallic"];

function assertScene(scene, context) {
    checks++;
    if (scene?.scene !== 1) return fail(context, "scene version is not 1");
    if (!Array.isArray(scene.vessels)) return fail(context, "scene.vessels not an array");
    for (const v of scene.vessels) {
        for (const key of VESSEL_KEYS) {
            if (v[key] === undefined) fail(context, `vessel.${key} missing`);
        }
        if (typeof v.words !== "string" || v.words.length === 0) {
            fail(context, "vessel.words empty");
        }
        if (v.liquid) {
            for (const key of LIQUID_KEYS) {
                if (v.liquid[key] === undefined) fail(context, `liquid.${key} missing`);
            }
            if (!Array.isArray(v.liquid.srgb) || v.liquid.srgb.length !== 3) {
                fail(context, "liquid.srgb not [r,g,b]");
            }
        }
        for (const s of v.solids ?? []) {
            for (const key of SOLID_KEYS) {
                if (s[key] === undefined) fail(context, `solid.${key} missing`);
            }
        }
        for (const b of v.badges ?? []) {
            for (const key of ["key", "value", "confidence"]) {
                if (b[key] === undefined) fail(context, `badge.${key} missing`);
            }
        }
    }
}

const lessons = readdirSync("lessons")
    .filter((f) => f.endsWith(".lab"))
    .sort();

for (const file of lessons) {
    const lab = new Lab();
    const text = readFileSync(resolve("lessons", file), "utf8");
    let lineno = 0;
    for (const raw of text.split("\n")) {
        lineno += 1;
        const line = raw.trim();
        if (!line || line.startsWith("#")) continue;
        const context = `${file}:${lineno}`;
        let doc;
        try {
            doc = JSON.parse(lab.runScript(line));
        } catch (e) {
            // A refusal or an engineless miss is a legitimate outcome; shape
            // conformance only asserts on what the host DOES return.
            continue;
        }
        checks++;
        if (!Array.isArray(doc.steps)) {
            fail(context, "runScript result lacks steps[]");
            continue;
        }
        for (const step of doc.steps) {
            checks++;
            if (typeof step.operator?.op !== "string") fail(context, "operator untagged");
            if (!Array.isArray(step.events)) fail(context, "events not an array");
            for (const event of step.events) {
                if (typeof event.event !== "string") {
                    fail(context, `event without tag: ${JSON.stringify(event)}`);
                }
            }
            if (!Array.isArray(step.rendered)) fail(context, "rendered not an array");
        }
        if (doc.scene !== undefined) assertScene(doc.scene, context);
    }
    // The standalone scene call conforms too.
    assertScene(JSON.parse(lab.scene()), `${file}: standalone scene`);
    // And parse never mutates: state before === after.
    const before = lab.state();
    lab.parse("add v1 water 100mL");
    lab.parse("utter nonsense &&&");
    checks++;
    if (lab.state() !== before) fail(file, "parse mutated the bench");
}

// --- The sandbox-completeness invariant (GUI-029) -----------------------
// Every grammar verb must have an affordance-manifest entry; a verb the
// parser gains without a GUI decision fails here. Planned rows are
// reported, not failed — the invariant is tracked until GUI-033 flips
// them to real components.
{
    const lab = new Lab();
    const grammar = JSON.parse(lab.grammar());
    const manifest = JSON.parse(
        readFileSync(resolve("web/app/src/lib/affordances.json"), "utf8"),
    );
    let planned = 0;
    for (const { verb, example } of grammar) {
        checks++;
        const entry = manifest[verb];
        if (entry === undefined) {
            fail("affordances", `grammar verb '${verb}' (${example}) has no manifest entry`);
        } else if (String(entry).startsWith("planned:")) {
            planned++;
        }
    }
    for (const key of Object.keys(manifest)) {
        if (key.startsWith("_")) continue;
        checks++;
        if (!grammar.some((g) => g.verb === key)) {
            fail("affordances", `manifest names '${key}', which the grammar does not have`);
        }
    }
    console.log(
        `affordances: ${grammar.length} verbs, ${grammar.length - planned} with GUI form, ${planned} planned (GUI-033)`,
    );
}

if (failures > 0) {
    console.error(`\n${failures} conformance failure(s) in ${checks} checks`);
    process.exit(1);
}
console.log(`protocol conformance (wasm host): ${checks} checks over ${lessons.length} lessons, all clean`);
