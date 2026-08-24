/**
 * The engine worker: kerotakis-wasm behind the WEB-002 envelope.
 *
 * This is GUI-004's one-worker engine, client half: the bench wasm AND the
 * Emscripten IPhreeQC module load together in this worker, wired by the
 * same PhreeqcPool the legacy console page uses (one bridge, one source of
 * truth — web/kerotakis.mjs). The solver hook stays synchronous inside the
 * worker; only the UI messaging is asynchronous, which is exactly the
 * split ROADMAP-Webapp.md's delivery section calls for.
 *
 * Honesty rule: with no engine loaded, every chemistry command answers
 * with an error naming the missing piece — never a canned result. With
 * the bench loaded but no aqueous module, the bench itself says which
 * answers come from shipped results (`canSolve`).
 */

import { PhreeqcPool } from "../../../../kerotakis.mjs";

type Lab = {
  step(operatorJson: string): string;
  runScript(text: string): string;
  setRegister(level: string): void;
  setSolver(hook: (dbTag: string, input: string) => string): void;
  scene(): string;
  state(): string;
  species(): string;
  inspect(vessel: number): string;
  particles(vessel: number): string;
  reset(): void;
  canSolve(): boolean;
  loadResults(bytes: Uint8Array): number;
};

let lab: Lab | null = null;
let loadFailure: string | null = null;
/** Why the live aqueous engine is absent, when it is. */
let aqueousNote: string | null = null;

const PROTOCOL = 1;

function done(id: number, resultJson: string) {
  postMessage({ id, type: "done", result_json: resultJson });
}

function fail(id: number, message: string, kind = "internal") {
  postMessage({ id, type: "error", message, kind });
}

async function init(engineBase: string) {
  // The host sends an absolute URL (resolved against the page); the
  // self.location fallback only serves direct-worker test setups.
  const base = new URL(engineBase, self.location.href);
  try {
    const mod = await import(/* @vite-ignore */ new URL("kerotakis_wasm.js", base).href);
    await mod.default();
    lab = new mod.Lab() as Lab;
  } catch (e) {
    loadFailure = e instanceof Error ? e.message : String(e);
    return;
  }
  try {
    const res = await fetch(new URL("results.postcard", base).href);
    if (res.ok) lab.loadResults(new Uint8Array(await res.arrayBuffer()));
  } catch {
    // Pre-warmed results are an accelerant, not a requirement.
  }
  // Attach the live aqueous engine in this same worker (GUI-004/OPT-11).
  try {
    const iph = await import(/* @vite-ignore */ new URL("iphreeqc.mjs", base).href);
    const pool = await PhreeqcPool.create(iph.default, async (file: string) => {
      const res = await fetch(new URL(`db/${file}`, base).href);
      if (!res.ok) throw new Error(`fetching ${file}: HTTP ${res.status}`);
      return res.text();
    });
    lab.setSolver((dbTag: string, input: string) => pool.solve(dbTag, input));
  } catch (e) {
    // Honest degradation: the bench runs from shipped results and says so.
    aqueousNote = e instanceof Error ? e.message : String(e);
  }
}

onmessage = async (ev: MessageEvent) => {
  const msg = ev.data as { id: number; cmd: string; [k: string]: unknown };
  if (typeof msg !== "object" || msg === null || typeof msg.cmd !== "string") return;
  const { id, cmd } = msg;

  if (cmd === "init") {
    await init(String(msg.engine_base ?? "./engine/"));
    done(id, "{}");
    return;
  }
  if (cmd === "hello") {
    done(
      id,
      JSON.stringify({
        protocol: PROTOCOL,
        can_solve: lab?.canSolve() ?? false,
        engine_loaded: lab !== null,
        load_failure: loadFailure,
        aqueous_note: aqueousNote,
      }),
    );
    return;
  }
  if (!lab) {
    fail(
      id,
      loadFailure
        ? `the engine failed to load: ${loadFailure}`
        : "the engine is not loaded yet",
      "engine",
    );
    return;
  }

  try {
    switch (cmd) {
      case "step":
        done(id, lab.step(String(msg.operator_json)));
        break;
      case "run_script":
        done(id, lab.runScript(String(msg.script)));
        break;
      case "set_register":
        lab.setRegister(String(msg.level));
        done(id, "{}");
        break;
      case "scene":
        done(id, lab.scene());
        break;
      case "state":
        done(id, lab.state());
        break;
      case "species":
        done(id, lab.species());
        break;
      case "inspect":
        done(id, lab.inspect(Number(msg.vessel)));
        break;
      case "particles":
        done(id, lab.particles(Number(msg.vessel)));
        break;
      case "reset":
        lab.reset();
        done(id, "{}");
        break;
      default:
        fail(id, `unknown command '${cmd}'`, "parse");
    }
  } catch (e) {
    // The wasm bridge surfaces engine refusals and parse errors as thrown
    // JsError values; their message is the rendered explanation.
    fail(id, e instanceof Error ? e.message : String(e), "engine");
  }
};
