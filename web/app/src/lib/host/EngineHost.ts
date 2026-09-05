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
  /** BRD-002: the finite bottles on the shelf. Absent — and absent per
   * key — means an unlimited supply, never an empty one. */
  stock?: SceneStockBottle[];
}

/** What is left in one shelf bottle, in the unit the `add` grammar takes
 * ("mol", "g" or "mL"). */
export interface SceneStockBottle {
  key: string;
  remaining: number;
  unit: string;
}

export interface SceneVessel {
  id: number;
  label: string;
  liquid: SceneLiquid | null;
  /** Liquid layers, bottom first; absent/single for a mixed solution. */
  layers?: SceneLayer[];
  solids: SceneSolid[];
  /** Coherent named objects positioned using whole-object bulk density. */
  bulk_objects?: SceneBulkObject[];
  /** Persistent, source-backed protective films on coherent objects. */
  coatings?: SceneCoating[];
  /** Prepared coherent objects whose ingredients remain object-owned. */
  material_objects?: SceneMaterialObject[];
  /** Conserved hard-water/fatty-soap aggregate. */
  soap_scum?: SceneSoapScum | null;
  /** Reviewed lemon-juice mark carried by a paper substrate. */
  lemon_paper_mark?: SceneLemonPaperMark | null;
  /** Current borate-crosslinked polymer projection; not a rheology model. */
  gel?: SceneGel | null;
  bubbling: boolean;
  foam?: SceneFoam | null;
  surface_particles?: SceneSurfaceParticles | null;
  surface_colours?: SceneSurfaceColour[];
  emulsion?: SceneEmulsion | null;
  curds?: SceneCurds | null;
  swelling?: SceneSwelling | null;
  chemiluminescence?: SceneChemiluminescence | null;
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

export interface SceneMaterialObject {
  material: string;
  recipe_id: string;
  mass_g: number;
  exchanged_water_moles: number;
  browned_fraction: number;
}

export interface SceneSoapScum {
  aggregate_mass_g: number;
  divalent_ion_moles: number;
}

export interface SceneGel {
  polymer: string;
  crosslinker: string;
  gelled_fraction: number;
  polymer_grams: number;
  crosslinker_moles: number;
}

export interface SceneLemonPaperMark {
  dry: boolean;
  browned_fraction: number;
}

export interface SceneFoam {
  trapped_gas_liters: number;
  volume_liters: number;
  height_cm: number;
  overflow_liters: number;
  /** Additive scene-v1 fields; absent when talking to an older host. */
  srgb?: [number, number, number];
  colour_word?: string;
}

export interface SceneSurfaceParticles {
  material: string;
  coverage_fraction: number;
  cleared_fraction: number;
}

export interface SceneSurfaceColour {
  material: string;
  srgb: [number, number, number];
  spread_fraction: number;
  relative_amount: number;
}

export interface SceneEmulsion {
  material: string;
  dispersed_volume_l: number;
  dispersed_fraction: number;
  half_life_seconds: number;
}

export interface SceneCurds {
  material: string;
  formed_fraction: number;
  separation_progress: number;
  solids_mass_g: number;
  srgb: [number, number, number];
}

export interface SceneSwelling {
  dry_polymer_g: number;
  retained_water_g: number;
  swelling_ratio_g_per_g: number;
  capacity_g_per_g: number;
  saturated: boolean;
}

export interface SceneChemiluminescence {
  relative_intensity: number;
  half_life_s: number;
  elapsed_s: number;
  temperature_k: number;
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
  represented_by_bulk_object?: boolean;
}

export interface SceneBulkObject {
  material: string;
  recipe_id: string;
  amount_g: number;
  bulk_density_g_per_ml: number;
  position: "floating" | "sunk" | "dry";
  srgb: [number, number, number];
}

export interface SceneCoating {
  kind: "paint" | "passive_film";
  recipe_id: string;
  host_species: string;
  words: string;
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
/**
 * GUI-095: the balancing exercise — the question, and nothing that answers
 * it (`kerotakis-core::stoich::balance_exercise`).
 *
 * This interface used to be `BalanceReport`, and it carried the solver's
 * `coefficients` and the composition `matrix`. Both are answers: the
 * coefficients are the answer written down, and the matrix is the answer
 * one null space away. A browser is a place where anyone can open the
 * network pane, so neither crosses the wire any more. What is left is what
 * drawing the question needs, plus two facts *about* the answer that give
 * nothing away — `trivial` (every coefficient is 1, so there is nothing to
 * work out and the drill should prefer another) and `family` (several
 * answers are right, which the learner is entitled to know up front).
 *
 * Marking is `balanceMark`; the answer is `balanceReveal`, and asking for
 * it is the learner's decision to make.
 */
export interface BalanceExercise {
  ok: true;
  species: string[];
  reactants: number;
  reversible: boolean;
  trivial: boolean;
  family: boolean;
  /** The question as one line, every coefficient stripped. */
  skeleton: string;
}

/** A skeleton that could not be balanced says why rather than guessing. */
export type BalanceExerciseReply = BalanceExercise | { ok: false; error: string };

/** What a marked answer turned out to be — the engine's verdict, not the
 * client's. Spelled as `kerotakis_core::stoich::Verdict`. */
export type BalanceVerdict = "correct" | "multiple" | "unbalanced" | "incomplete";

/** One row of the composition matrix that does not cancel; `amount` is the
 * signed surplus on the left. */
export interface BalanceMiss {
  element: string;
  amount: number;
}

/** The verdict on one answer, with the detail that makes it teachable. */
export interface BalanceMark {
  ok: true;
  verdict: BalanceVerdict;
  /** What does not cancel, worst first. Empty unless `unbalanced`. */
  misses: BalanceMiss[];
  /** The shared factor, when the answer is a correct multiple. */
  factor: number;
  /** True when the skeleton admits more than one independent reaction. */
  family: boolean;
}

export type BalanceMarkReply = BalanceMark | { ok: false; error: string };

/** The answer, written out — the one reply that gives it up. */
export type BalanceAnswerReply = { ok: true; equation: string } | { ok: false; error: string };

/** WORLD-007: why an answer was not accepted — a stable tag with its
 * parameters, so a German client says it in German. A wrong guess is
 * spoken guidance, never a block, so it arrives as a RESULT rather than
 * as an error carrying an English sentence. */
export type AnswerRefusal =
  | { refused: "wrong_guess"; alias: string; guess: string }
  | { refused: "unknown_alias"; alias: string };

export interface QuestAnswerResult {
  outputs: QuestOutput[];
  refusal?: AnswerRefusal;
}

export interface QuestOutput {
  /** `constraint_violated` (WORLD-004) is said, never blocking: a mission
   * says "not like that" without refusing to let it happen, because a lab
   * that prevents the mistake cannot teach it. */
  kind: "nudge" | "claim_satisfied" | "completed" | "constraint_violated";
  quest: string;
  /** The claim's stable id, on `claim_satisfied`. Optional because an
   * engine built before this field existed does not send it. */
  claim?: string;
  say?: { lv1: string; lv2: string; lv3: string };
  title?: { lv1: string; lv2: string; lv3: string };
}

/** What `step` returns; `run_script` returns one entry per line plus scene. */
export interface StepResult {
  events: unknown[];
  rendered: string[];
  /** GUI-092: the net ionic equations the step earned, validated on
   * arrival by `ionic.ts` rather than trusted here. */
  ionic?: unknown[];
  quest?: QuestOutput[];
  scene?: Scene;
}

export interface ScriptResult {
  steps: {
    operator: unknown;
    events: unknown[];
    rendered: string[];
    ionic?: unknown[];
    quest?: QuestOutput[];
  }[];
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
/** WORLD-003 — one joined answer to "what can this learner reach, and why".
 *
 * Reasons are stable tags with parameters, never prose: the client writes the
 * sentence in its own language from `reason` and `minimum_completed`.
 */
export type CatalogReason =
  | { reason: "sandbox" }
  | { reason: "earned"; minimum_completed: number }
  | { reason: "awarded" }
  | { reason: "loaned" }
  | { reason: "locked"; minimum_completed: number };

export type CatalogItem = {
  /** A verb (`filter`), an instrument (`measure:ph`), or a species key. */
  id: string;
  kind: "reagent" | "apparatus" | "instrument";
  minimum_completed: number;
  available: boolean;
  reason: CatalogReason;
};

export type CatalogRequest = {
  mode?: "story" | "sandbox";
  completed?: number;
  /** Ids permanently granted by closed cases. */
  awarded?: string[];
  /** Ids the active mission supplies for its own duration. */
  mission_kit?: string[];
};

export type CatalogResponse = {
  mode: "story" | "sandbox";
  completed: number;
  items: CatalogItem[];
  packs: string[];
};

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
  /** WORLD-003: the runtime catalog — availability and its stable reason. */
  catalog(request: CatalogRequest): Promise<CatalogResponse>;

  /** The named-relations catalogue (CAP-5). */
  relations(): Promise<
    {
      name: string;
      equation: string;
      args: string;
      /** What question it answers, and where it stops holding (GUI-087),
       *  and who published it (GUI-096). Optional here because a host from
       *  an older engine build answers without them; the drawer omits the
       *  paragraph rather than rendering an empty one. */
      purpose?: string;
      purpose_de?: string;
      validity?: string;
      validity_de?: string;
      source?: string;
      source_de?: string;
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
  /** GUI-095: the balancing exercise for one skeleton — the question,
   * with no route back to the answer. */
  balanceExercise(equation: string): Promise<BalanceExerciseReply>;
  /** GUI-095: mark one answer engine-side. `answer` is one positive
   * integer per species, in the order `balanceExercise` listed them. */
  balanceMark(equation: string, answer: number[]): Promise<BalanceMarkReply>;
  /** GUI-095: the answer, written out, when the learner asks for it. */
  balanceReveal(equation: string): Promise<BalanceAnswerReply>;
  /** Start/stop the engine-evaluated quest (GUI-066); outputs arrive on
   * step results as `quest: QuestOutput[]`. */
  questStart(specJson: string): Promise<void>;
  questStop(): Promise<void>;
  questAnswer(alias: string, guess: string): Promise<QuestAnswerResult>;
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
  elementCoverage(): Promise<unknown>;
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
