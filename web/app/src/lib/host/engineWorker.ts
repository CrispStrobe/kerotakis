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
  parse(line: string): string;
  grammar(): string;
  meta(): string;
  questStart(specJson: string): void;
  questStop(): void;
  questAnswer(alias: string, guess: string): string;
  snapshot(): string;
  restore(snapshot: string): void;
  relations(): string;
  calc(name: string, argsJson: string): string;
  balanceExercise(equation: string): string;
  balanceMark(equation: string, answerJson: string): string;
  balanceReveal(equation: string): string;
  catalog(requestJson: string): string;
  setRegister(level: string): void;
  setLocale(code: string): void;
  setSolver(hook: (dbTag: string, input: string) => string): void;
  scene(): string;
  state(): string;
  species(): string;
  element_coverage(): string;
  inspect(vessel: number): string;
  particles(vessel: number): string;
  reset(): void;
  canSolve(): boolean;
  loadResults(bytes: Uint8Array): number;
  loadPack(bytes: Uint8Array): string;
};

let lab: Lab | null = null;
let loadFailure: string | null = null;
/** Why the live aqueous engine is absent, when it is. */
let aqueousNote: string | null = null;
/**
 * Loading the engine takes real seconds on a school line (the bench wasm
 * alone is ~4 MB). Every command that arrives before init settles WAITS
 * for it instead of failing — the race where the UI's first scene request
 * outruns the download must not strand the bench on "warming up".
 */
let initPromise: Promise<void> | null = null;

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
    // The Emscripten glue mis-detects module workers and resolves its
    // .wasm relative to the WORKER bundle's URL — fetch the bytes
    // ourselves and hand them over, so no path arithmetic can go wrong.
    const wasmRes = await fetch(new URL("iphreeqc.wasm", base).href);
    if (!wasmRes.ok) throw new Error(`fetching iphreeqc.wasm: HTTP ${wasmRes.status}`);
    const wasmBinary = new Uint8Array(await wasmRes.arrayBuffer());
    const factory = (opts: object = {}) =>
      iph.default({
        wasmBinary,
        locateFile: (p: string) => new URL(p, base).href,
        ...opts,
      });
    const pool = await PhreeqcPool.create(factory, async (file: string) => {
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
    initPromise = init(String(msg.engine_base ?? "./engine/"));
    await initPromise;
    done(id, "{}");
    return;
  }
  // Everything else waits for the engine load to settle first.
  if (initPromise) await initPromise;
  if (cmd === "hello") {
    // Engine identity rides along once the engine exists (GUI-001).
    let meta: Record<string, unknown> = {};
    try {
      meta = lab ? (JSON.parse(lab.meta()) as Record<string, unknown>) : {};
    } catch {
      // An engine without meta() is an older build; hello stays honest.
    }
    done(
      id,
      JSON.stringify({
        protocol: PROTOCOL,
        can_solve: lab?.canSolve() ?? false,
        engine_loaded: lab !== null,
        load_failure: loadFailure,
        aqueous_note: aqueousNote,
        ...meta,
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
      case "parse":
        done(id, lab.parse(String(msg.line)));
        break;
      case "grammar":
        done(id, lab.grammar());
        break;
      case "catalog":
        done(id, lab.catalog(JSON.stringify(msg.request ?? {})));
        break;
      case "relations":
        done(id, lab.relations());
        break;
      case "balance_exercise":
        done(id, lab.balanceExercise(String(msg.equation)));
        break;
      case "balance_mark":
        done(id, lab.balanceMark(String(msg.equation), String(msg.answer)));
        break;
      case "balance_reveal":
        done(id, lab.balanceReveal(String(msg.equation)));
        break;
      case "quest_start":
        lab.questStart(String(msg.spec_json));
        done(id, "{}");
        break;
      case "quest_stop":
        lab.questStop();
        done(id, "{}");
        break;
      case "quest_answer":
        done(id, lab.questAnswer(String(msg.alias), String(msg.guess)));
        break;
      case "load_pack":
        done(id, lab.loadPack(new Uint8Array(msg.bytes as ArrayBuffer)));
        break;
      case "snapshot":
        done(id, JSON.stringify({ snapshot: lab.snapshot() }));
        break;
      case "restore":
        lab.restore(String(msg.snapshot));
        done(id, "{}");
        break;
      case "calc":
        done(id, lab.calc(String(msg.name), JSON.stringify(msg.args ?? [])));
        break;
      case "set_register":
        lab.setRegister(String(msg.level));
        done(id, "{}");
        break;
      case "set_locale":
        lab.setLocale(String(msg.code));
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
      case "element_coverage":
        done(id, lab.element_coverage());
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
