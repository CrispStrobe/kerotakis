// The bridge: a real aqueous solver in the browser.
//
// Two WebAssembly modules, built by different toolchains, that have to be
// introduced to each other.
//
//   * `kerotakis_wasm` — the bench, `wasm32-unknown-unknown`, pure Rust.
//     Vessels, operators, thermal chemistry, rates, rendering.
//   * `iphreeqc.mjs`   — IPhreeqc, Emscripten, C++. The aqueous engine.
//
// Rust cannot link the second: `wasm32-unknown-unknown` has no C++ runtime.
// So the bench asks JavaScript, and JavaScript asks IPhreeqc. The call is
// synchronous in both directions once the modules are loaded, which is what
// makes it possible at all — `Lab.step` stays an ordinary function and the
// solver hook returns before it does.
//
// Everything above the hook is unchanged: database routing, the content
// cache, the temperature fixed point, the parsers. The browser therefore
// gets the same answers by the same path as the desktop build, rather than
// a second implementation that could quietly drift from it.
//
// Without this, the web bench serves only pre-warmed results. That is
// honest — a miss is reported as a miss — and it is also not a laboratory:
// "try things" degrades to "replay what we prepared" in the one channel
// schools can actually use.

const DATABASES = {
    "wateq4f": "wateq4f.dat",
    "minteq.v4": "minteq.v4.dat",
    "pitzer": "pitzer.dat",
};

/// One IPhreeqc instance per database, because loading a database resets
/// the instance — and the router picks a database per vessel state, so
/// switching on every call would be both slow and stateful in the wrong way.
class PhreeqcPool {
    constructor(mod, instances) {
        this.mod = mod;
        this.instances = instances;
    }

    static async create(createIPhreeqc, loadDatabase) {
        const mod = await createIPhreeqc();
        const c = (name, ret, args) => mod.cwrap(name, ret, args);
        // Read C strings by hand rather than letting the glue do it.
        //
        // Chrome refuses `TextDecoder.decode()` on a view into a *resizable*
        // ArrayBuffer, and a growable wasm heap is exactly that — so the
        // Emscripten helper throws on the first string that crosses the
        // bridge, which is the run report. `slice` copies out of the heap
        // first, and a copy is not resizable. Node never reproduced this,
        // so it only appeared once the page ran in a real browser.
        const str = (ptr) => {
            if (!ptr) return "";
            const heap = mod.HEAPU8;
            let end = ptr;
            while (heap[end] !== 0) end++;
            return new TextDecoder("utf-8").decode(heap.slice(ptr, end));
        };
        const api = {
            create: c("CreateIPhreeqc", "number", []),
            loadDb: c("LoadDatabaseString", "number", ["number", "string"]),
            run: c("RunString", "number", ["number", "string"]),
            errorPtr: c("GetErrorString", "number", ["number"]),
            outputFileOn: c("SetOutputFileOn", "number", ["number", "number"]),
            errorFileOn: c("SetErrorFileOn", "number", ["number", "number"]),
            logFileOn: c("SetLogFileOn", "number", ["number", "number"]),
            dumpFileOn: c("SetDumpFileOn", "number", ["number", "number"]),
            selectedFileOn: c("SetSelectedOutputFileOn", "number", ["number", "number"]),
            selectedStringOn: c("SetSelectedOutputStringOn", "number", ["number", "number"]),
            outputStringOn: c("SetOutputStringOn", "number", ["number", "number"]),
            outputStringPtr: c("GetOutputString", "number", ["number"]),
            lineCount: c("GetSelectedOutputStringLineCount", "number", ["number"]),
            error: (id) => str(api.errorPtr(id)),
            outputString: (id) => str(api.outputStringPtr(id)),
            line: (id, i) => str(api.linePtr(id, i)),
            linePtr: c("GetSelectedOutputStringLine", "number", ["number", "number"]),
        };

        const instances = {};
        for (const [tag, file] of Object.entries(DATABASES)) {
            const id = api.create();
            if (id < 0) throw new Error(`CreateIPhreeqc failed for ${tag}`);
            // No filesystem in this build; everything is strings in memory.
            api.outputFileOn(id, 0);
            api.errorFileOn(id, 0);
            api.logFileOn(id, 0);
            api.dumpFileOn(id, 0);
            api.selectedFileOn(id, 0);
            // The full report carries the species distribution and the
            // saturation indices, and the bench reads both.
            api.outputStringOn(id, 1);

            const text = await loadDatabase(file);
            if (api.loadDb(id, text) !== 0) {
                throw new Error(`loading ${file}: ${api.error(id)}`);
            }
            // Loading a database clears this flag — IPhreeqc resets it in
            // its load path — so it must be set *after* the load, and again
            // before every run for safety. Getting this wrong yields empty
            // selected output with no error at all.
            api.selectedStringOn(id, 1);
            instances[tag] = id;
        }
        const pool = new PhreeqcPool(mod, instances);
        pool.api = api;
        return pool;
    }

    /// The solver hook the Rust bench calls. Synchronous by necessity.
    solve(dbTag, input) {
        const id = this.instances[dbTag] ?? this.instances["wateq4f"];
        // Instances are pooled across vessels. Clear numbered solutions and
        // reactants in a separate run so a populated SURFACE 1 cannot leak
        // into the next cell, while the real run retains one clean selected-
        // output schema. Thermodynamic database definitions are unaffected.
        if (this.api.run(id, "DELETE\n    -all\nEND\n") !== 0) {
            throw new Error(`resetting reused IPhreeqc state: ${this.api.error(id)}`);
        }
        this.api.selectedStringOn(id, 1);
        if (this.api.run(id, input) !== 0) {
            throw new Error(this.api.error(id));
        }
        const rows = [];
        const n = this.api.lineCount(id);
        for (let i = 0; i < n; i++) {
            const line = this.api.line(id, i);
            if (line === null || line === undefined) continue;
            const cells = line.split("\t").map((s) => s.trim());
            if (cells.length > 1) rows.push(cells);
        }
        return JSON.stringify({
            selected: rows,
            report: this.api.outputString(id) ?? "",
        });
    }
}

/// Open a bench with a real aqueous engine behind it.
///
/// `opts.createIPhreeqc` is the Emscripten module factory; `opts.loadDatabase`
/// is an async `(filename) => text`; `opts.results` is optional pre-warmed
/// postcard bytes, which are still worth loading because they answer
/// instantly and keep the guided content snappy.
export async function openLab(Lab, opts) {
    const lab = new Lab();
    if (opts.results) lab.loadResults(opts.results);
    if (opts.createIPhreeqc) {
        const pool = await PhreeqcPool.create(opts.createIPhreeqc, opts.loadDatabase);
        // Bound so `this` survives the trip through Rust.
        lab.setSolver((dbTag, input) => pool.solve(dbTag, input));
    }
    return lab;
}

export { PhreeqcPool };
