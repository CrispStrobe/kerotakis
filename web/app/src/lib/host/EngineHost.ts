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
  solids: SceneSolid[];
  bubbling: boolean;
  /** Flattened Headspace tag: open | sealed | pressure_controlled | swept. */
  boundary: string;
  temperature_k: number;
  pressure_pa: number;
  elapsed_s: number;
  /** The lv1 observation sentence — also the vessel's accessible name. */
  words: string;
  badges: SceneBadge[];
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
  srgb: [number, number, number];
  colour_word: string;
  metallic: boolean;
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

/** What `step` returns; `run_script` returns one entry per line plus scene. */
export interface StepResult {
  events: unknown[];
  rendered: string[];
  scene?: Scene;
}

export interface ScriptResult {
  steps: { operator: unknown; events: unknown[]; rendered: string[] }[];
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
  hello(): Promise<{ protocol: number; can_solve?: boolean }>;
  step(operatorJson: string): Promise<StepResult>;
  runScript(script: string): Promise<ScriptResult>;
  /** Validate one line without executing it (GUI-005). */
  parse(line: string): Promise<{ ok: boolean; operator?: unknown; error?: string }>;
  setRegister(level: string): Promise<void>;
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
