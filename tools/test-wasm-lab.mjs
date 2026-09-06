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
lab.setRegister("lv1");

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

const freezer = new Lab();
freezer.runScript("add v1 water 100mL\ncool v1 40kJ\n");
const inspected = JSON.parse(freezer.inspect(0)).rendered.join("\n");
check("inspect renders the frozen liquid inventory", /water\s+Liquid/.test(inspected), inspected);
check("inspect renders the frozen solid inventory", /water\s+Solid/.test(inspected), inspected);
check(
    "inspect prose does not leak the machine state contract",
    !/\"contents\"|\"thermal_mode\"|^\s*\{/.test(inspected),
    inspected,
);

// Boundary state is core physics, so it works even without the aqueous side module.
const closed = new Lab();
const closedRun = JSON.parse(closed.runScript("seal v1 500mL\nheat v1 10J\n"));
const closedVessel = closedRun.bench.vessels[0];
check("a finite headspace survives the Wasm JSON boundary", closedVessel.headspace.boundary === "sealed");
check(
    "heating trapped gas raises its pressure",
    closedVessel.pressure > 101325,
    `${closedVessel.pressure.toFixed(0)} Pa`,
);
const energyControlled = new Lab();
const energyRegulated = JSON.parse(
    energyControlled.runScript("regulate v1 101.325kPa 500mL\nheat v1 10J\n"),
);
const energyRegulatedVessel = energyRegulated.bench.vessels[0];
check(
    "rigid trapped gas warms more than the same pressure-controlled gas",
    closedVessel.temperature > energyRegulatedVessel.temperature,
    `sealed=${closedVessel.temperature.toFixed(6)} K, regulated=${energyRegulatedVessel.temperature.toFixed(6)} K`,
);
const controlled = new Lab();
const regulated = JSON.parse(
    controlled.runScript("regulate v1 1.5bar 250mL\nheat v1 10J\n"),
);
const regulatedVessel = regulated.bench.vessels[0];
check(
    "a pressure controller crosses the Wasm boundary",
    regulatedVessel.headspace.boundary === "pressure_controlled",
);
check(
    "the controller holds its target pressure",
    regulatedVessel.pressure === 150000,
);
const swept = JSON.parse(controlled.runScript("sweep v1 90kPa\n"));
const sweptVessel = swept.bench.vessels[0];
check(
    "a swept boundary crosses the Wasm boundary",
    sweptVessel.headspace.boundary === "swept",
);
check(
    "the sweep owns no gas inventory",
    !sweptVessel.contents.some((portion) => portion.phase === "gas"),
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

    const lab4 = new Lab();
    lab4.loadResults(readFileSync(cachePath));
    const limewater = readFileSync(new URL("../lessons/limewater.lab", import.meta.url), "utf8");
    const limewaterRun = JSON.parse(lab4.runScript(limewater));
    const limewaterEvents = limewaterRun.steps.flatMap((s) => s.events);
    check(
        "a gas dose transfers inward from cached equilibrium",
        limewaterEvents.some((e) => e.event === "gas_absorbed" && e.species === "CO2"),
    );
    check(
        "limewater clouds and clears again in cached WebAssembly",
        limewaterEvents.some((e) => e.event === "precipitated" && e.species === "CaCO3") &&
            limewaterEvents.some((e) => e.event === "dissolved" && e.species === "CaCO3"),
    );

    const lab5 = new Lab();
    lab5.loadResults(readFileSync(cachePath));
    const r1 = JSON.parse(lab5.r1Acceptance());
    check("the cached R1 report uses schema 1", r1.schema === 1);
    check(
        "all five R1 scenarios replay in cache-only WebAssembly",
        r1.cases.length === 5 && r1.cases.every((test) => test.passed),
        JSON.stringify(r1.cases.filter((test) => !test.passed)),
    );
}

// --- the grammar in the learner's own language (I18N) --------------------
// The alias layer lives in the engine and its tables are compiled into the
// wasm binary, so the browser must accept exactly what the native bench
// accepts — and hand back exactly the same canonical line to log. A
// German session that replayed as German text would be a session no
// English bench could open.
{
    const german = new Lab();
    german.setLocale("de");
    const typed = JSON.parse(german.runScript("zugeben v1 Wasser 100mL\nmessen v1 waage"));
    check(
        "a German command line runs in WebAssembly",
        typed.steps.length === 2,
        JSON.stringify(typed.steps.map((s) => s.rendered)),
    );
    check(
        "and is logged as the canonical English",
        typed.steps.map((s) => s.canonical).join(" | ") === "add v1 water 100mL | measure v1 balance",
        JSON.stringify(typed.steps.map((s) => s.canonical)),
    );
    const parsed = JSON.parse(german.parse("erhitzen v1 10kJ auf kerze"));
    check(
        "the live validator canonicalises too",
        parsed.ok === true && parsed.canonical === "heat v1 10kJ on candle",
        JSON.stringify(parsed),
    );
    const refused = JSON.parse(german.parse("blubbern v1"));
    check(
        "and an unknown verb is refused in German, naming the German verbs",
        refused.ok === false && /unbekannter Befehl/.test(refused.error) &&
            /zugeben \(add\)/.test(refused.error),
        JSON.stringify(refused),
    );
    const hints = JSON.parse(german.grammar());
    check(
        "the verb inventory offers a German line to type",
        hints.find((row) => row.verb === "add")?.typed === "zugeben v1 Wasser 100mL",
        JSON.stringify(hints.slice(0, 3)),
    );

    // English is untouched by any of it: the canonical script is the same
    // script in every language, which is what makes a lesson portable.
    const english = new Lab();
    const plain = JSON.parse(english.runScript("add v1 water 100mL"));
    check(
        "an English session logs the line it was given",
        plain.steps[0].canonical === "add v1 water 100mL",
        JSON.stringify(plain.steps[0].canonical),
    );
    check(
        "and an English bench still refuses a German verb it was never told about",
        JSON.parse(english.parse("zugeben v1 Wasser 100mL")).ok === false,
    );
}

console.log("\nThe bench runs in WebAssembly.");
