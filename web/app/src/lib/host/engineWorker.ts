/**
 * The engine worker: kerotakis-wasm behind the WEB-002 envelope.
 *
 * This is the seed of GUI-004's one-worker engine. Today it loads the bench
 * wasm and the pre-warmed results; attaching the Emscripten IPhreeQC module
 * *in this same worker* (replacing the main-thread wiring in the legacy
 * web/kerotakis.mjs) is the consolidation step — the synchronous solver
 * hook then never crosses to the UI thread.
 *
 * Honesty rule: with no engine loaded, every chemistry command answers
 * with an error naming the missing piece — never a canned result.
 */

type Lab = {
  step(operatorJson: string): string;
  runScript(text: string): string;
  setRegister(level: string): void;
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

const PROTOCOL = 1;

function done(id: number, resultJson: string) {
  postMessage({ id, type: "done", result_json: resultJson });
}

function fail(id: number, message: string, kind = "internal") {
  postMessage({ id, type: "error", message, kind });
}

async function init(engineBase: string) {
  try {
    const mod = await import(/* @vite-ignore */ new URL("kerotakis_wasm.js", engineBase).href);
    await mod.default();
    lab = new mod.Lab() as Lab;
    try {
      const res = await fetch(new URL("results.postcard", engineBase).href);
      if (res.ok) lab.loadResults(new Uint8Array(await res.arrayBuffer()));
    } catch {
      // Pre-warmed results are an accelerant, not a requirement.
    }
  } catch (e) {
    loadFailure = e instanceof Error ? e.message : String(e);
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
