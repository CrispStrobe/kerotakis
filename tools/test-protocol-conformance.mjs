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

const [outDir, resultsPath] = process.argv.slice(2);
if (!outDir) {
    console.error("usage: node test-protocol-conformance.mjs <bindgen-out-dir> [results.postcard]");
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
        // Layers (GUI-058): bottom-first phase split; volumes must sum
        // to the liquid, or the drawn split lies about the computed one.
        if (v.layers !== undefined) {
            checks++;
            if (!Array.isArray(v.layers) || v.layers.length === 0) {
                fail(context, "layers present but not a non-empty array");
            } else {
                for (const l of v.layers) {
                    for (const key of ["species", "name", "volume_l", "srgb", "colour_word"]) {
                        if (l[key] === undefined) fail(context, `layer.${key} missing`);
                    }
                }
                const sum = v.layers.reduce((s, l) => s + l.volume_l, 0);
                if (v.liquid && Math.abs(sum - v.liquid.volume_l) > 1e-9) {
                    fail(context, `layer volumes ${sum} != liquid ${v.liquid.volume_l}`);
                }
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

// --- snapshot / restore (O(1) undo) --------------------------------------
// A snapshot taken mid-session, restored after further work, must give
// back the exact state it captured — restore is replay's equal, or undo
// silently lies.
{
    const lab = new Lab();
    lab.runScript("new\nadd v1 water 100mL\nadd v1 NaCl 1g");
    const snap = lab.snapshot();
    const stateAt = lab.state();
    lab.runScript("add v1 KMnO4 1pinch\nnew flask");
    checks++;
    if (lab.state() === stateAt) fail("snapshot", "further work did not change state (test is vacuous)");
    lab.restore(snap);
    checks++;
    if (lab.state() !== stateAt) fail("restore", "restored state differs from the captured one");
    checks++;
    let refused = false;
    try { lab.restore("{ not json"); } catch { refused = true; }
    if (!refused) fail("restore", "a garbage snapshot must refuse, not corrupt the bench");
    // And the bench still works after the refusal.
    lab.runScript("add v1 water 10mL");
    console.log("snapshot/restore: round-trip exact, garbage refused");
}

// --- hello carries the pack inventory (WEB-003) --------------------------
{
    const lab = new Lab();
    const meta = JSON.parse(lab.meta());
    checks++;
    if (!Array.isArray(meta.packs) || meta.packs.length < 5) {
        fail("packs", `hello meta must list the pack inventory, got ${JSON.stringify(meta.packs)}`);
    } else {
        for (const p of meta.packs) {
            checks++;
            if (typeof p.pack_id !== "string" || typeof p.licence !== "string" || p.licence.length === 0
                || typeof p.required !== "boolean") {
                fail("packs", `malformed manifest: ${JSON.stringify(p)}`);
            }
        }
        checks++;
        if (!meta.packs.some((p) => p.required)) {
            fail("packs", "no pack is marked required — core-aqueous must be");
        }
        console.log(`packs: ${meta.packs.length} in the inventory, licences declared`);
    }
}

// --- load_pack (DATA-010): the species-breadth unlock --------------------
// A pack built here in node (magic + version + sha256 + JSON payload,
// same format as kero pack export) must add its novel species to the
// shelf AND to real chemistry; a flipped byte must refuse by hash; and
// built-ins must never be shadowed.
{
    const crypto = await import("node:crypto");
    const source = JSON.parse(readFileSync("data/registry/registry-source-v1.json", "utf8"));
    const doc = Object.fromEntries(Object.entries(source).map(([k, v]) =>
        [k, k === "sources" ? v : Array.isArray(v) ? [] : v]));
    const cloneIn = (fromId, toId) => {
        for (const section of ["identities", "compositions", "phase_thermodynamics", "optical", "model_parameters"]) {
            for (const rec of source[section]) {
                const hit = rec.species_id === fromId
                    || (section === "identities" && rec.id === fromId)
                    || (rec.subject?.kind === "species" && rec.subject?.id === fromId);
                if (!hit) continue;
                const c = JSON.parse(JSON.stringify(rec));
                if (section === "identities") { c.id = toId; c.name = `conformance double of ${fromId}`; }
                if (c.species_id !== undefined) c.species_id = toId;
                if (c.subject?.id === fromId) c.subject.id = toId;
                doc[section].push(c);
            }
        }
    };
    cloneIn("water", "conformium");
    cloneIn("betanin", "conformanin"); // a dye: its SPECTRUM must load too
    const payload = Buffer.from(JSON.stringify(doc));
    const pack = Buffer.concat([
        Buffer.from("KREG"),
        Buffer.from(Uint32Array.of(1).buffer),
        crypto.createHash("sha256").update(payload).digest(),
        payload,
    ]);
    const lab = new Lab();
    const r = JSON.parse(lab.loadPack(new Uint8Array(pack)));
    checks++;
    if (r.added !== 2) fail("load_pack", `expected 2 added, got ${JSON.stringify(r)}`);
    checks++;
    if (!JSON.parse(lab.species()).some((s) => s.key === "conformium")) {
        fail("load_pack", "loaded species missing from the shelf");
    }
    checks++;
    try {
        lab.runScript("new\nadd v1 conformium 1g");
    } catch (e) {
        fail("load_pack", `loaded species unusable in chemistry: ${e.message}`);
    }
    // DATA-011: the loaded dye's spectrum colours a solution — pack
    // species get Beer–Lambert colour exactly like built-ins.
    checks++;
    const dyeRun = JSON.parse(lab.runScript("new flask\nadd v3 water 100mL\nadd v3 conformanin 1pinch"));
    const dyed = dyeRun.scene.vessels.find((v) => v.liquid && v.liquid.colour_word !== "colourless");
    if (!dyed) fail("load_pack", "pack dye did not colour its solution (spectrum not loaded)");
    else console.log(`load_pack: pack dye colours its solution ${dyed.liquid.colour_word}`);
    checks++;
    const corrupt = Buffer.from(pack);
    corrupt[60] ^= 0xff;
    let refused = false;
    try { lab.loadPack(new Uint8Array(corrupt)); } catch { refused = true; }
    if (!refused) fail("load_pack", "corrupt pack must refuse by hash");
    console.log("load_pack: novel species to shelf + chemistry; corruption refused");
}

// --- relations / calc (GUI-027, GUI-087, GUI-096) ------------------------
// The catalogue rows must be form-buildable AND self-explaining: what the
// relation answers, where it stops holding, and who published it — in every
// language the engine ships, because the drawer shows all three before
// anything is computed. An evaluation must come back with value, unit,
// provenance, and all three registers — or an honest refusal.
{
    const lab = new Lab();
    const relations = JSON.parse(lab.relations());
    checks++;
    if (!Array.isArray(relations) || relations.length === 0) {
        fail("relations", "catalogue empty or not an array");
    }
    // The prose fields carry one `_<locale>` sibling per shipped language,
    // the unsuffixed field being English. Derived from the row rather than
    // listed here, so adding a language extends the check by itself.
    //
    // Completeness is demanded per locale, unlike the .toml catalogues where
    // a half-finished translation is the intended shipping state: these are
    // required struct fields on `RelationInfo`, so "translated for purpose
    // but not for source" is not a translation in progress, it is a row
    // somebody forgot to finish.
    const shippedLocales = [...new Set(
        relations.flatMap((r) => Object.keys(r))
                 .map((k) => /^purpose_(.+)$/.exec(k)?.[1])
                 .filter(Boolean),
    )];
    checks++;
    if (shippedLocales.length === 0) {
        fail("relations", "no translated catalogue at all — expected at least purpose_de");
    }
    for (const r of relations) {
        checks++;
        if (typeof r.name !== "string" || typeof r.equation !== "string" || typeof r.args !== "string") {
            fail("relations", `malformed row: ${JSON.stringify(r)}`);
        }
        // A formula with no stated validity range teaches a learner to apply
        // it outside that range, so an empty string is a failure here rather
        // than a cosmetic gap.
        for (const field of ["purpose", "validity", "source"]) {
            for (const key of [field, ...shippedLocales.map((l) => `${field}_${l}`)]) {
                checks++;
                if (typeof r[key] !== "string" || r[key].trim() === "") {
                    fail("relations", `${r.name}: ${key} missing or empty`);
                }
            }
        }
    }
    const good = JSON.parse(lab.calc("henderson-hasselbalch", JSON.stringify(["pKa=4.76", "cA=0.1", "cB=0.1"])));
    checks++;
    if (good.ok !== true || typeof good.value !== "number" || typeof good.unit !== "string"
        || typeof good.provenance !== "string"
        || typeof good.lv1 !== "string" || typeof good.lv2 !== "string" || typeof good.lv3 !== "string") {
        fail("calc", `evaluation missing fields: ${JSON.stringify(good)}`);
    }
    checks++;
    if (good.ok === true && Math.abs(good.value - 4.76) > 1e-9) {
        fail("calc", `equimolar buffer should sit at its pKa; got ${good.value}`);
    }
    // GUI-096: the catalogue's citation and the computed result's provenance
    // are one string, not two — a drawer that credits Henderson while the
    // answer credits somebody else is worse than one that credits nobody.
    checks++;
    const hh = relations.find((r) => r.name === "henderson-hasselbalch");
    if (good.ok === true && hh && !good.provenance.includes(hh.source)) {
        fail("relations", `catalogue cites ${JSON.stringify(hh.source)}, `
            + `the result cites ${JSON.stringify(good.provenance)}`);
    }
    const bad = JSON.parse(lab.calc("no-such-relation", JSON.stringify([])));
    checks++;
    if (bad.ok !== false || typeof bad.error !== "string") {
        fail("calc", `unknown relation must refuse with an error: ${JSON.stringify(bad)}`);
    }
    console.log(`relations: ${relations.length} in the catalogue, documented in `
        + `${["en", ...shippedLocales].join("/")}, evaluation + refusal conform`);
}

// --- balance (GUI-095) ---------------------------------------------------
// The balancing exercise is generated, not authored, so the command has to
// carry enough for a client to mark answers the solver never returned. Two
// claims are pinned: the reported matrix is the one the reported answer is
// the null space of (otherwise a client marks against a different reaction
// than the engine solved), and a balanced equation sets the same question
// as its bare skeleton (otherwise the codex's own coefficients leak the
// answer into the question).
{
    const lab = new Lab();
    const report = JSON.parse(lab.balance("Mg + O2 -> MgO"));
    checks++;
    if (report.ok !== true) fail("balance", `refused a balanceable skeleton: ${JSON.stringify(report)}`);
    for (const [field, kind] of [["species", "array"], ["elements", "array"],
        ["matrix", "array"], ["coefficients", "array"], ["basis", "array"],
        ["reactants", "number"], ["reversible", "boolean"]]) {
        checks++;
        const ok = kind === "array" ? Array.isArray(report[field]) : typeof report[field] === kind;
        if (!ok) fail("balance", `${field} missing or not ${kind}: ${JSON.stringify(report)}`);
    }
    checks++;
    if (report.species.join(" ") !== "Mg O2 MgO" || report.reactants !== 2) {
        fail("balance", `species/reactants wrong: ${JSON.stringify(report)}`);
    }
    checks++;
    if (report.coefficients.join(",") !== "2,1,2") {
        fail("balance", `2 Mg + O2 -> 2 MgO expected; got ${report.coefficients.join(",")}`);
    }
    checks++;
    if (report.elements.at(-1) !== "charge") {
        fail("balance", `charge must be the last matrix row: ${JSON.stringify(report.elements)}`);
    }
    // The invariant the marking rests on: matrix · coefficients = 0.
    const annihilates = (matrix, vector) => matrix.every(
        (row) => Math.abs(row.reduce((sum, count, i) => sum + count * vector[i], 0)) < 1e-9,
    );
    checks++;
    if (!annihilates(report.matrix, report.coefficients)) {
        fail("balance", "the reported matrix does not annihilate the reported answer");
    }
    // A correct multiple must still balance — that is the whole lesson.
    checks++;
    if (!annihilates(report.matrix, report.coefficients.map((c) => c * 3))) {
        fail("balance", "a multiple of the answer must still balance");
    }
    checks++;
    const alreadyBalanced = JSON.parse(lab.balance("2 Mg + O₂ → 2 MgO"));
    if (alreadyBalanced.ok !== true
        || alreadyBalanced.coefficients.join(",") !== report.coefficients.join(",")
        || alreadyBalanced.species.length !== report.species.length) {
        fail("balance", `a balanced equation must set the same question as its skeleton: `
            + `${JSON.stringify(alreadyBalanced)}`);
    }
    // Underdetermined: C + O2 -> CO + CO2 admits two independent reactions.
    const family = JSON.parse(lab.balance("C + O2 -> CO + CO2"));
    checks++;
    if (family.ok !== true || family.basis.length === 0) {
        fail("balance", `an underdetermined skeleton must report its basis: ${JSON.stringify(family)}`);
    }
    checks++;
    if (family.ok === true && !family.basis.every((v) => annihilates(family.matrix, v))) {
        fail("balance", "every basis vector must lie in the reported null space");
    }
    // Prose in an equation field is refused rather than balanced.
    const refused = JSON.parse(lab.balance("CH₃COOH / CH₃COO⁻ buffer"));
    checks++;
    if (refused.ok !== false || typeof refused.error !== "string") {
        fail("balance", `prose must refuse with an error: ${JSON.stringify(refused)}`);
    }
    console.log(`balance: ${report.species.length}-species skeleton solved, `
        + `matrix annihilates it, family reported, prose refused`);
}

// --- The chart contract on the wire (GUI-021/CAP-12) ---------------------
// A titration must EARN its chart: the titrate step carries a charts
// array in the CAP-3 shape — axes with labels, a line series of [x,y]
// pairs, and a provenance line. Absence here means the emitter regressed.
// Computing pH per increment needs solver states: the pre-warmed results
// supply them (CI passes lessons.postcard). Without them the check is
// SKIPPED AND SAYS SO — never silently green.
if (!resultsPath) {
    console.log("charts: SKIPPED (no results.postcard argument — pH per increment needs pre-warmed states)");
} else {
    const lab = new Lab();
    lab.loadResults(readFileSync(resultsPath));
    // The exact pre-warmed lesson, so every increment's state is cached.
    const doc = JSON.parse(lab.runScript(readFileSync("lessons/titration.lab", "utf8")));
    const withCharts = doc.steps.filter((s) => Array.isArray(s.charts) && s.charts.length > 0);
    checks++;
    if (withCharts.length === 0) {
        fail("charts", "the titrate step carried no chart");
    } else {
        const chart = withCharts.at(-1).charts[0];
        checks++;
        if (typeof chart.title !== "string" || typeof chart.provenance !== "string"
            || typeof chart.x?.label !== "string" || typeof chart.y?.label !== "string") {
            fail("charts", `chart missing contract fields: ${JSON.stringify(chart).slice(0, 200)}`);
        }
        const series = chart.series?.[0];
        checks++;
        if (series?.kind !== "line" || !Array.isArray(series.points) || series.points.length < 2
            || series.points.some((pt) => !Array.isArray(pt) || pt.length !== 2)) {
            fail("charts", "titration series is not a line of [x,y] pairs");
        }
        console.log(`charts: titration curve on the wire (${series?.points?.length ?? 0} points)`);
    }
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
