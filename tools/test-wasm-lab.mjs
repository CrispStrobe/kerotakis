// The bench in a browser runtime: burn magnesium in WebAssembly, and check
// that shipped aqueous results replay while a state nobody pre-computed is
// reported as a stated miss rather than a guess.
//
// Usage: node tools/test-wasm-lab.mjs <bindgen-out-dir> [cache.postcard]

import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";

const [outDir, cachePath] = process.argv.slice(2);
if (!outDir) {
    console.error("usage: node test-wasm-lab.mjs <bindgen-out-dir> [cache.postcard]");
    process.exit(2);
}
const require = createRequire(import.meta.url);
const { Lab } = require(resolve(outDir, "kerotakis_wasm.js"));

const check = (name, cond, detail = "") => {
    if (!cond) {
        console.error(`FAIL ${name}${detail ? ": " + detail : ""}`);
        process.exit(1);
    }
    console.log(`ok   ${name}`);
};

const lab = new Lab();
lab.setRegister("child");

// The lab knows its shelf.
const species = JSON.parse(lab.species());
check("species registry is exposed", species.length > 20, `${species.length} species`);
check(
    "every species carries provenance",
    species.every((s) => typeof s.provenance === "string" && s.provenance.length > 0),
);

// Thermal chemistry is computed here, in wasm.
const add = (vessel, key, moles) =>
    JSON.parse(lab.step(JSON.stringify({ op: "add", vessel, species: key, moles })));
const ignite = (vessel) => JSON.parse(lab.step(JSON.stringify({ op: "ignite", vessel })));

add(0, "Mg", 0.0494);
const fire = ignite(0);
check(
    "magnesium ignites in WebAssembly",
    fire.events.some((e) => e.event === "ignited"),
    JSON.stringify(fire.rendered),
);
check(
    "it burns to the oxide",
    fire.events.some((e) => e.event === "precipitated" && e.species === "MgO"),
);
const vessel = fire.bench.vessels[0];
check(
    "and the flame is hot",
    vessel.temperature > 2500,
    `${vessel.temperature.toFixed(0)} K`,
);
check(
    "the child register speaks",
    fire.rendered.some((line) => line.includes("blinding white")),
    JSON.stringify(fire.rendered),
);

// Aqueous chemistry without an engine: a miss must be stated.
const lab2 = new Lab();
const salted = JSON.parse(
    lab2.runScript("add v1 water 100mL\nadd v1 NaCl 0.58g\n"),
);
check(
    "an aqueous state nobody pre-computed is an honest miss, not a guess",
    salted.steps.some((s) =>
        s.events.some(
            (e) => e.event === "solver_failed" && /not in the shipped results/.test(e.detail),
        ),
    ),
    JSON.stringify(salted.steps.at(-1).events),
);

// With the shipped results loaded, the very same lesson answers instantly.
if (cachePath) {
    const lab3 = new Lab();
    const loaded = lab3.loadResults(readFileSync(cachePath));
    check("shipped results load", loaded > 0, `${loaded} entries`);

    const lesson = readFileSync(new URL("../lessons/silver-and-salt.lab", import.meta.url), "utf8");
    const played = JSON.parse(lab3.runScript(lesson));
    const events = played.steps.flatMap((s) => s.events);
    check(
        "a whole lesson replays from shipped data, with no engine present",
        !events.some((e) => e.event === "solver_failed"),
        JSON.stringify(events.filter((e) => e.event === "solver_failed")),
    );
    check(
        "and the marquee precipitate appears",
        events.some((e) => e.event === "precipitated" && e.species === "AgCl"),
    );
    const child = played.steps.flatMap((s) => s.rendered);
    check(
        "narrated for a nine-year-old",
        child.some((l) => l.includes("cloudy")),
        JSON.stringify(child),
    );
}

console.log("\nThe bench runs in WebAssembly.");
