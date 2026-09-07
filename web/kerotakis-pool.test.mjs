import test from "node:test";
import assert from "node:assert/strict";
import { PhreeqcPool, openLab } from "./kerotakis.mjs";

function nativeDouble() {
    const loaded = [];
    const runs = [];
    let nextId = 0;
    return {
        loaded, runs,
        module: {
            HEAPU8: new Uint8Array(8),
            cwrap(name) {
                return (...args) => {
                    if (name === "CreateIPhreeqc") return nextId++;
                    if (name === "LoadDatabaseString") loaded.push(args);
                    if (name === "RunString") runs.push(args);
                    return 0;
                };
            },
        },
    };
}

test("prepared databases and pinned definitions use isolated engine instances", async () => {
    const native = nativeDouble();
    const prepared = [];
    const pool = await PhreeqcPool.create(
        async () => native.module,
        async () => { throw new Error("must not load raw upstream database"); },
        (tag) => { prepared.push(tag); return `prepared:${tag}`; },
    );
    assert.equal(prepared.length, 3);
    assert.equal(native.loaded.length, 6);
    const ordinary = "SOLUTION 1\nEND\n";
    const pinned = "SOLUTION_SPECIES\nFe+2 = Fe+3 + e-\n log_k 50\nEND\n";
    pool.solve("wateq4f", ordinary);
    pool.solve("wateq4f", pinned);
    pool.solve("wateq4f", ordinary);
    const computations = native.runs.filter(([, input]) => !input.startsWith("DELETE"));
    assert.equal(computations.length, 3);
    assert.equal(computations[0][0], computations[2][0]);
    assert.notEqual(computations[0][0], computations[1][0]);
});

test("openLab obtains the namespace from the Rust host", async () => {
    const native = nativeDouble();
    class Lab {
        aqueousDatabase(tag) { return `rust:${tag}`; }
        setSolver(callback) { this.solve = callback; }
    }
    const lab = await openLab(Lab, { createIPhreeqc: async () => native.module });
    assert.ok(native.loaded.every(([, text]) => text.startsWith("rust:")));
    assert.deepEqual(JSON.parse(lab.solve("wateq4f", "SOLUTION 1\nEND")), {
        selected: [], report: "",
    });
});
