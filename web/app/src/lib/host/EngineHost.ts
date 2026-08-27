/**
 * The EngineHost protocol, client side (PROTOCOL.md, GUI-001).
 *
 * The UI talks only to this interface. Two hosts implement it: the web
 * WorkerHost (kerotakis-wasm in a module worker) and, later, the native
 * TauriHost. The UI must not be able to tell them apart.
 *
 * On types: per PROTOCOL.md, Rust serde is the source of truth and these
 * are CONSUMER types — they name only the fields this client reads, and
 * every consumer must tolerate fields it does not know. Do not grow them
 * into hand-maintained mirrors of the Rust enums; when a full typing is
 * wanted, generate it from the Rust types instead.
 */

/** One request envelope. `cmd` values are the WEB-002 WorkerCommand tags. */
export interface EngineRequest {
  id: number;
  cmd: string;
  [field: string]: unknown;
}

/** Terminal + streaming response envelopes (WEB-002 WorkerResponse tags). */
export type EngineResponse =
  | { id: number; type: "done"; result_json: string }
  | { id: number; type: "progress"; fraction: number; message: string }
  | { id: number; type: "error"; message: string; kind?: string }
  | { id: number; type: "cancelled" };

/** Scene JSON v1 — the render model (kerotakis-core/src/scene.rs). */
export interface Scene {
  scene: number;
  vessels: SceneVessel[];
}

export interface SceneVessel {
  id: number;
  label: string;
  liquid: SceneLiquid | null;
  /** Liquid layers, bottom first; absent/single for a mixed solution. */
  layers?: SceneLayer[];
  solids: SceneSolid[];
  bubbling: boolean;
  foam?: SceneFoam | null;
  /** Flattened Headspace tag: open | sealed | pressure_controlled | swept. */
  boundary: string;
  temperature_k: number;
  pressure_pa: number;
  elapsed_s: number;
  mass_g: number;
  /** The lv1 observation sentence — also the vessel's accessible name. */
  words: string;
  badges: SceneBadge[];
}

export interface SceneFoam {
  trapped_gas_liters: number;
  volume_liters: number;
  height_cm: number;
  overflow_liters: number;
}

/** One visible liquid layer, bottom first (GUI-058) — the engine's
 * computed phase split, e.g. hexane floating on water. */
export interface SceneLayer {
  species: string;
  name: string;
  volume_l: number;
  srgb: [number, number, number];
  colour_word: string;
}

export interface SceneLiquid {
  volume_l: number;
  srgb: [number, number, number];
  colour_word: string;
  cloudiness: number;
  path_length_cm: number;
}

export interface SceneSolid {
  species: string;
  name: string;
  moles: number;
  /** Pure-solid volume from engine registry mass and density. */
  volume_l?: number;
  srgb: [number, number, number];
  colour_word: string;
  metallic: boolean;
  settled_fraction: number;
}

export interface SceneBadge {
  key: string;
  value: number;
  confidence: string;
}

/** The submicroscopic census (kerotakis-core/src/particles.rs) — drawn at
 * solved ratios; `source` says whether speciation or inventory backs it. */
export interface ParticleCensus {
  populations: {
    label: string;
    kind: string;
    drawn: number;
    amount: number;
  }[];
  per_glyph: number;
  too_rare: [string, number][];
  source: string;
}

/** One engine-evaluated quest output: a nudge spoken, a claim
 * satisfied, or the quest completed — register texts spelled out. */
export interface QuestOutput {
  kind: "nudge" | "claim_satisfied" | "completed";
  quest: string;
  say?: { lv1: string; lv2: string; lv3: string };
  title?: { lv1: string; lv2: string; lv3: string };
}

/** What `step` returns; `run_script` returns one entry per line plus scene. */
export interface StepResult {
  events: unknown[];
  rendered: string[];
  quest?: QuestOutput[];
  scene?: Scene;
}

export interface ScriptResult {
  steps: { operator: unknown; events: unknown[]; rendered: string[]; quest?: QuestOutput[] }[];
  scene?: Scene;
}

/** A structured engine failure. `refused` is a result, not a fault. */
export class EngineError extends Error {
  constructor(
    message: string,
    public readonly kind: "parse" | "refused" | "engine" | "internal" = "internal",
  ) {
    super(message);
    this.name = "EngineError";
  }
}

/**
 * What the UI programs against. Methods mirror PROTOCOL.md's command table;
 * everything is asynchronous so worker and native transports are
 * indistinguishable from here.
 */
export interface EngineHost {
  /** Protocol + engine identity; answerable before any pack loads. */
  hello(): Promise<{
    protocol: number;
    can_solve?: boolean;
    engine_loaded?: boolean;
    load_failure?: string | null;
    aqueous_note?: string | null;
    engine_version?: string;
    git_rev?: string | null;
    registers?: string[];
    /** WEB-003 pack inventory; empty content_hash = built in, not yet
     * independently deliverable. */
    packs?: {
      pack_id: string;
      version: string;
      content_hash: string;
      licence: string;
      required: boolean;
    }[];
  }>;
  step(operatorJson: string): Promise<StepResult>;
  runScript(script: string): Promise<ScriptResult>;
  /** Validate one line without executing it (GUI-005). */
  parse(line: string): Promise<{ ok: boolean; operator?: unknown; error?: string }>;
  /** The verb inventory with canonical examples (GUI-029). */
  grammar(): Promise<{ verb: string; example: string; options?: string[] }[]>;
  /** The named-relations catalogue (CAP-5). */
  relations(): Promise<
    {
      name: string;
      equation: string;
      args: string;
      /** What question it answers, and where it stops holding (GUI-087). */
      purpose?: string;
      purpose_de?: string;
      validity?: string;
      validity_de?: string;
    }[]
  >;
  /** Evaluate a named relation; the result explains itself per register. */
  calc(
    name: string,
    args: string[],
  ): Promise<
    | { ok: true; value: number; unit: string; provenance: string; lv1: string; lv2: string; lv3: string }
    | { ok: false; error: string }
  >;
  /** Start/stop the engine-evaluated quest (GUI-066); outputs arrive on
   * step results as `quest: QuestOutput[]`. */
  questStart(specJson: string): Promise<void>;
  questStop(): Promise<void>;
  questAnswer(alias: string, guess: string): Promise<QuestOutput[]>;
  /** DATA-010: load a species pack. Honest counts back; built-ins are
   * never shadowed. */
  loadPack(bytes: Uint8Array): Promise<{ added: number; skipped: number; loaded_total: number }>;
  /** The bench as an opaque restorable token (O(1) undo/scrub). */
  snapshot(): Promise<string>;
  /** Replace the bench with a `snapshot()` token; session state survives. */
  restore(snapshot: string): Promise<void>;
  setRegister(level: string): Promise<void>;
  /** The language the ENGINE renders its own prose in (I18N-5).
   *
   * Separate from the interface's locale by necessity: the engine composes
   * the vessel summary and the journal itself, out of fragments, so no
   * amount of translating in the shell can reach them.
   *
   * Cannot fail. An unknown tag falls back to English inside the engine,
   * so there is no error for a host to handle and no reason to make
   * callers handle one. */
  setLocale(code: string): Promise<void>;
  scene(): Promise<Scene>;
  state(): Promise<unknown>;
  species(): Promise<unknown[]>;
  inspect(vessel: number): Promise<{ rendered: string[] }>;
  particles(vessel: number): Promise<{ census?: ParticleCensus; rendered: string[] }>;
  reset(): Promise<void>;
  dispose(): void;
}

/**
 * Correlates request envelopes with their terminal responses over any
 * postMessage-shaped channel. This is the piece both hosts share and the
 * piece worth unit-testing without a real worker.
 */
export interface MessagePortLike {
  postMessage(data: unknown): void;
  set onmessage(handler: ((ev: { data: unknown }) => void) | null);
}

export class RequestChannel {
  private nextId = 1;
  /** Set once the transport is gone; every later request fails fast. */
  private dead: string | null = null;
  private pending = new Map<
    number,
    {
      resolve: (json: string) => void;
      reject: (err: Error) => void;
      onProgress?: (fraction: number, message: string) => void;
    }
  >();

  constructor(private port: MessagePortLike) {
    port.onmessage = (ev) => this.receive(ev.data);
  }

  request(
    cmd: string,
    fields: Record<string, unknown> = {},
    onProgress?: (fraction: number, message: string) => void,
  ): Promise<string> {
    if (this.dead) return Promise.reject(new EngineError(this.dead, "engine"));
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject, onProgress });
      this.port.postMessage({ id, cmd, ...fields } satisfies EngineRequest);
    });
  }

  private receive(data: unknown) {
    const msg = data as EngineResponse;
    if (typeof msg !== "object" || msg === null || typeof msg.id !== "number") return;
    const entry = this.pending.get(msg.id);
    if (!entry) return;
    switch (msg.type) {
      case "progress":
        entry.onProgress?.(msg.fraction, msg.message);
        return; // not terminal
      case "done":
        this.pending.delete(msg.id);
        entry.resolve(msg.result_json);
        return;
      case "error":
        this.pending.delete(msg.id);
        entry.reject(
          new EngineError(msg.message, (msg.kind as EngineError["kind"]) ?? "internal"),
        );
        return;
      case "cancelled":
        this.pending.delete(msg.id);
        entry.reject(new EngineError("cancelled", "internal"));
        return;
    }
  }

  /** The transport is gone: fail everything in flight AND everything that
   * comes later, with the same honest reason. */
  abandon(reason: string) {
    this.dead = reason;
    for (const [, entry] of this.pending) {
      entry.reject(new EngineError(reason, "engine"));
    }
    this.pending.clear();
  }
}
