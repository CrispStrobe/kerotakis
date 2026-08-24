/**
 * Session state: a projection of the engine, plus interaction state.
 *
 * The engine is the only holder of chemistry state (PROTOCOL.md non-goal
 * list); this class holds what the UI needs to paint — the latest scene,
 * the feed, the register — and routes every user intention through the
 * EngineHost as a command line. Every GUI gesture ends up here as the
 * operator text it compiled to, which is what makes the feed a lab
 * notebook, a screen-reader surface, and a scripting teacher at once.
 *
 * Undo is replay (ROADMAP-GUI.md interaction principle 2): the engine is
 * deterministic and `reset` + `runScript` reproduce any prefix of the
 * command log exactly, so the session never snapshots chemistry itself.
 */

import type { EngineHost, ParticleCensus, Scene } from "./host/EngineHost";
import { EngineError } from "./host/EngineHost";
import { isChartSpec, type ChartSpec } from "./chart";
import { type Lesson, parseLesson } from "./lesson";

export type FeedEntry = {
  kind: "command" | "line" | "error" | "refusal" | "note" | "hazard" | "chart";
  text: string;
  /** Hazard entries: the engine's severity, for the card's chip. */
  severity?: string;
  /** Chart entries: the Chart JSON v1 spec to render. */
  chart?: ChartSpec;
};

export type ShelfItem = {
  key: string;
  name: string;
  formula: string;
  phase: string;
  /** Reflective colour of the substance itself, when curated. */
  srgb?: [number, number, number] | null;
  /** Computed transmitted tint of a 0.1 M solution through 1 cm. */
  solution_srgb?: [number, number, number] | null;
  /** Characteristic flame colour word, when curated. */
  flame?: string | null;
  /** Curated appearance word ("white", "colourless", …). */
  appearance?: string | null;
};

export const REGISTERS = [
  { level: "lv1", label: "Look" },
  { level: "lv2", label: "Measure" },
  { level: "lv3", label: "Model" },
] as const;

/** The slice of Web Storage the session needs — injectable for tests,
 * absent (null) where storage is unavailable. */
export interface StorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

const SAVE_KEY = "kero.session.v1";

export class Session {
  register = $state<string>("lv1");
  scene = $state<Scene | null>(null);
  feed = $state<FeedEntry[]>([]);
  busy = $state(false);
  engineReady = $state(false);
  canSolve = $state(false);
  /** Engine identity from hello (GUI-001): "0.0.1 @ abc1234" or null. */
  engineIdentity = $state<string | null>(null);
  /** Successful chemistry commands, in order — the session's .lab script. */
  commandLog = $state<string[]>([]);
  /**
   * How many of those commands are applied to the bench right now. Undo,
   * redo, and the timeline scrubber all move this one cursor; a new
   * command while the cursor sits mid-history truncates the future.
   */
  position = $state(0);
  /** The lesson being walked, if any. `kit` is the reagent set the lesson
   * itself uses — "what the teacher put out for you" — derived from its
   * own command lines, so it can never drift from the lesson. */
  lesson = $state<{ lesson: Lesson; cursor: number; kit: string[] } | null>(null);
  /** The registry, for the shelf. */
  shelf = $state<ShelfItem[]>([]);
  /** Curated reaction names the `react` verb accepts (from the grammar). */
  reactOptions = $state<string[]>([]);
  /** While set, submit() records event keys (tag and tag:species) here —
   * the experiment checker compares them against codex claims. */
  private eventCollector: string[] | null = null;
  /** Vessel the user last selected (0-based id), target of shelf adds. */
  selected = $state<number>(0);
  /** Open inspector content, if any. */
  inspector = $state<{
    vessel: number;
    lines: string[];
    particles?: ParticleCensus;
  } | null>(null);
  /** The most recent balanced equation the engine rendered (GUI-025) —
   * the strip pins it beside the bench at lv2+. */
  lastEquation = $state<string | null>(null);
  /**
   * Transient visual effects per vessel (GUI-026), derived STRICTLY from
   * typed events — an effect never fires without a computed event behind
   * it. Entries age out; the canvas animates what is younger than its
   * animation.
   */
  vesselEffects = $state<Record<number, { kind: string; at: number }[]>>({});

  constructor(
    private host: EngineHost,
    private storage: StorageLike | null = defaultStorage(),
  ) {}

  async connect(): Promise<void> {
    try {
      const hello = await this.host.hello();
      this.engineReady = true;
      this.canSolve = hello.can_solve ?? false;
      if (hello.engine_version) {
        this.engineIdentity = hello.git_rev
          ? `${hello.engine_version} @ ${hello.git_rev}`
          : hello.engine_version;
      }
      this.feed.push({
        kind: "note",
        text: this.canSolve
          ? "The bench is live: states nobody pre-computed are solved."
          : "The bench answers from shipped results only — the live aqueous engine is not attached.",
      });
      // A silently degraded engine hides real failures (salts that never
      // dissolve, colours that never appear). Say WHY, loudly.
      if (!this.canSolve && hello.aqueous_note) {
        this.feed.push({
          kind: "error",
          text: `the aqueous engine failed to attach: ${hello.aqueous_note}`,
        });
      }
      await this.restore();
      // One patient retry: a slow engine download must degrade to a wait,
      // never to a bench that stays "warming up" forever.
      try {
        this.scene = await this.host.scene();
      } catch {
        await new Promise((r) => setTimeout(r, 2000));
        this.scene = await this.host.scene();
      }
      const species = (await this.host.species()) as ShelfItem[];
      this.shelf = species.map((s) => ({
        key: s.key,
        name: s.name,
        formula: s.formula,
        phase: String(s.phase ?? ""),
        srgb: s.srgb ?? null,
        solution_srgb: s.solution_srgb ?? null,
        flame: s.flame ?? null,
        appearance: s.appearance ?? null,
      }));
      try {
        const grammar = (await this.host.grammar()) as {
          verb: string;
          options?: string[];
        }[];
        this.reactOptions = grammar.find((g) => g.verb === "react")?.options ?? [];
      } catch {
        // An older host without grammar still runs; the picker just hides.
      }
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
    }
  }

  /** Rebuild the bench from the autosaved log — a reloaded tab comes back
   * exactly where it was, by replay, never by trusting a cached scene. */
  private async restore(): Promise<void> {
    const raw = this.storage?.getItem(SAVE_KEY);
    if (!raw) return;
    try {
      const saved = JSON.parse(raw) as {
        log: string[];
        position: number;
        register: string;
      };
      if (!Array.isArray(saved.log) || saved.log.length === 0) return;
      if (saved.register && saved.register !== this.register) {
        await this.host.setRegister(saved.register);
        this.register = saved.register;
      }
      const position = Math.max(0, Math.min(saved.log.length, saved.position ?? saved.log.length));
      if (position > 0) {
        await this.host.runScript(saved.log.slice(0, position).join("\n"));
      }
      this.commandLog = saved.log;
      this.position = position;
      this.feed.push({
        kind: "note",
        text: `restored your last session: ${position} step(s) replayed`,
      });
    } catch {
      // A corrupt save must never wedge the bench: drop it and start clean.
      this.storage?.removeItem(SAVE_KEY);
      this.feed.push({ kind: "note", text: "could not restore the last session — starting fresh" });
    }
  }

  private persist(): void {
    try {
      this.storage?.setItem(
        SAVE_KEY,
        JSON.stringify({
          log: this.commandLog,
          position: this.position,
          register: this.register,
        }),
      );
    } catch {
      // Storage full or blocked: the session still works, it just won't survive a reload.
    }
  }

  /** Empty the bench and forget the session — distinct from jumpTo(0),
   * which keeps the future for redo. */
  async clear(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    try {
      await this.host.reset();
      this.commandLog = [];
      this.position = 0;
      this.storage?.removeItem(SAVE_KEY);
      this.scene = await this.host.scene();
      this.inspector = null;
      this.feed.push({ kind: "note", text: "the bench is empty again" });
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
    } finally {
      this.busy = false;
    }
  }

  /** Run one command line — from the command bar or compiled from a
   * gesture. Returns whether the engine accepted it. */
  async submit(line: string): Promise<boolean> {
    const trimmed = line.trim();
    if (!trimmed || this.busy) return false;
    this.busy = true;
    this.feed.push({ kind: "command", text: trimmed });
    try {
      if (trimmed.startsWith("register ")) {
        return await this.applyRegister(trimmed.slice("register ".length).trim());
      }
      const result = await this.host.runScript(trimmed);
      for (const step of result.steps) {
        // Hazard events become cards, from the typed event itself — the
        // warning always precedes the chemistry, and the chemistry then
        // runs and shows why (the engine's "hazards teach" rule).
        for (const event of step.events as Array<Record<string, unknown>>) {
          this.recordEffect(event);
          if (this.eventCollector) {
            const tag = String(event?.event ?? "");
            if (tag) {
              this.eventCollector.push(tag);
              if (typeof event.species === "string") {
                this.eventCollector.push(`${tag}:${event.species}`);
              }
            }
          }
          if (event?.event === "hazard_warning") {
            this.feed.push({
              kind: "hazard",
              severity: String(event.severity ?? ""),
              text: `${event.hazard} — ${event.real_world}`,
            });
          } else if (event?.event === "safety_veto") {
            this.feed.push({
              kind: "refusal",
              text: String(event.reason ?? "the bench refused this operation"),
            });
          }
        }
        for (const rendered of step.rendered) {
          this.feed.push({ kind: "line", text: rendered });
          // The engine writes balanced equations with a real arrow; the
          // latest one is the reaction the bench is showing right now.
          const eq = rendered.match(/\S[^.:]*(?:→|⇌)[^.]*/);
          if (eq) this.lastEquation = eq[0].trim();
        }
        // Charts (the CAP-3 contract, kerotakis-core::chart): rendered
        // inline the moment a step object carries them.
        const charts = (step as { charts?: ChartSpec[] }).charts;
        for (const chart of charts ?? []) {
          if (isChartSpec(chart)) {
            this.feed.push({ kind: "chart", text: chart.title, chart });
          }
        }
      }
      if (result.scene) this.scene = result.scene;
      // Register lines are session state, not chemistry; everything else
      // that the engine accepted becomes part of the replayable script.
      // A command issued mid-history truncates the undone future first.
      if (this.position < this.commandLog.length) {
        this.commandLog = this.commandLog.slice(0, this.position);
      }
      this.commandLog.push(trimmed);
      this.position = this.commandLog.length;
      this.persist();
      // The inspected vessel's detail is stale after any step.
      if (this.inspector) await this.inspect(this.inspector.vessel);
      return true;
    } catch (e) {
      const refusal = e instanceof EngineError && e.kind === "refused";
      this.feed.push({
        kind: refusal ? "refusal" : "error",
        text: e instanceof Error ? e.message : String(e),
      });
      return false;
    } finally {
      this.busy = false;
    }
  }

  /**
   * Run a .lab file line by line onto the CURRENT bench — import composes
   * rather than destroys (clear first for a fresh start). Comments and
   * blanks are skipped; the walk stops at the first line the engine
   * rejects, naming it. Every accepted line enters the log, so an import
   * is undoable like anything else.
   */
  async importLab(name: string, text: string): Promise<void> {
    this.feed.push({ kind: "note", text: `running ${name} on this bench` });
    let lineno = 0;
    for (const raw of text.split("\n")) {
      lineno += 1;
      const line = raw.trim();
      if (!line || line.startsWith("#")) continue;
      if (!(await this.submit(line))) {
        this.feed.push({
          kind: "note",
          text: `stopped at ${name}:${lineno} — the rest of the file did not run`,
        });
        return;
      }
    }
    this.feed.push({ kind: "note", text: `${name} finished` });
  }

  /** Map a typed event onto a transient canvas effect for its vessel. */
  private recordEffect(event: Record<string, unknown>): void {
    const EFFECTS: Record<string, string> = {
      precipitated: "precipitate",
      dissolved: "dissolve",
      electrolysed: "electrolyse",
      plated: "plate",
      ignited: "ignite",
      evaporated: "evaporate",
      distilled: "evaporate",
    };
    const kind = EFFECTS[String(event?.event ?? "")];
    if (!kind) return;
    const vessel = Number(
      (event.vessel as number | undefined) ?? (event.from as number | undefined) ?? 0,
    );
    const now = Date.now();
    const list = (this.vesselEffects[vessel] ?? []).filter((e) => now - e.at < 4000);
    list.push({ kind, at: now });
    this.vesselEffects = { ...this.vesselEffects, [vessel]: list };
  }

  async setRegister(level: string): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.feed.push({ kind: "command", text: `register ${level}` });
    try {
      await this.applyRegister(level);
    } finally {
      this.busy = false;
    }
  }

  private async applyRegister(level: string): Promise<boolean> {
    try {
      await this.host.setRegister(level);
      this.register = level;
      this.persist();
      this.feed.push({ kind: "note", text: `speaking at ${level}` });
      if (this.inspector) await this.inspect(this.inspector.vessel);
      return true;
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
      return false;
    } finally {
      this.busy = false;
    }
  }

  /**
   * Move the timeline cursor: replay the first `to` commands onto a fresh
   * bench. Undo, redo, and the scrubber are all this one deterministic
   * operation — O(session) engine work, zero client-side chemistry.
   */
  async jumpTo(to: number): Promise<void> {
    const target = Math.max(0, Math.min(this.commandLog.length, Math.floor(to)));
    if (this.busy || target === this.position) return;
    this.busy = true;
    const prefix = this.commandLog.slice(0, target);
    try {
      await this.host.reset();
      if (prefix.length > 0) {
        const replay = await this.host.runScript(prefix.join("\n"));
        if (replay.scene) this.scene = replay.scene;
      } else {
        this.scene = await this.host.scene();
      }
      const was = this.position;
      this.position = target;
      this.persist();
      this.feed.push({
        kind: "note",
        text:
          target < was
            ? `stepped back to ${target} of ${this.commandLog.length}`
            : `stepped forward to ${target} of ${this.commandLog.length}`,
      });
      if (this.inspector) await this.inspect(this.inspector.vessel);
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: `replay failed, the bench may be out of sync — ${
          e instanceof Error ? e.message : String(e)
        }`,
      });
    } finally {
      this.busy = false;
    }
  }

  async undo(): Promise<void> {
    await this.jumpTo(this.position - 1);
  }

  async redo(): Promise<void> {
    await this.jumpTo(this.position + 1);
  }

  /** Begin walking a lesson. The bench keeps whatever is on it — a lesson
   * is an overlay on the real bench, not a sandbox swap. */
  startLesson(name: string, text: string): void {
    // The kit: every species the lesson's own commands touch.
    const kit = new Set<string>();
    for (const line of text.split("\n")) {
      const m = line.trim().match(/^(?:add|titrate|grind)\s+\S+\s+(\S+)/);
      if (m) kit.add(m[1]!);
    }
    this.lesson = { lesson: parseLesson(name, text), cursor: 0, kit: [...kit] };
    this.feed.push({ kind: "note", text: `lesson started: ${name}` });
    this.advanceLessonNotes();
  }

  /** Surface consecutive narration, stopping at the next command. */
  private advanceLessonNotes(): void {
    if (!this.lesson) return;
    const { lesson } = this.lesson;
    while (this.lesson.cursor < lesson.steps.length) {
      const step = lesson.steps[this.lesson.cursor]!;
      if (step.kind !== "note") break;
      this.feed.push({ kind: "note", text: step.text });
      this.lesson.cursor += 1;
    }
    if (this.lesson.cursor >= lesson.steps.length) {
      this.feed.push({ kind: "note", text: `lesson finished: ${lesson.name}` });
      this.lesson = null;
    }
  }

  /** The lesson's next command, shown before it runs. */
  get lessonNextCommand(): string | null {
    if (!this.lesson) return null;
    const step = this.lesson.lesson.steps[this.lesson.cursor];
    return step?.kind === "command" ? step.line : null;
  }

  /** Run the lesson's next command. Deviation is allowed at any time —
   * free commands do not move the lesson cursor. */
  async lessonNext(): Promise<void> {
    const line = this.lessonNextCommand;
    if (!line || !this.lesson) return;
    await this.submit(line);
    this.lesson.cursor += 1;
    this.advanceLessonNotes();
  }

  exitLesson(): void {
    if (!this.lesson) return;
    this.feed.push({ kind: "note", text: `lesson left: ${this.lesson.lesson.name}` });
    this.lesson = null;
  }

  /** Open (or refresh) the register-dependent detail for one vessel. */
  async inspect(vessel: number): Promise<void> {
    this.selected = vessel;
    try {
      const detail = await this.host.inspect(vessel);
      this.inspector = { vessel, lines: detail.rendered };
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
    }
  }

  /** Add the submicroscopic view to the open inspector — drawn from the
   * census when the host supplies one, with the words kept alongside. */
  async particles(): Promise<void> {
    if (!this.inspector) return;
    try {
      const p = await this.host.particles(this.inspector.vessel);
      this.inspector = {
        vessel: this.inspector.vessel,
        lines: this.inspector.lines,
        particles: p.census,
      };
      if (!p.census) {
        this.inspector.lines = [...this.inspector.lines, "", ...p.rendered];
      }
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
    }
  }

  closeInspector(): void {
    this.inspector = null;
  }

  /** The named-relations catalogue (GUI-027's toolbox drawer). */
  async relations() {
    try {
      return await this.host.relations();
    } catch {
      return [];
    }
  }

  /** Evaluate one named relation; the drawer shows the result verbatim. */
  async calc(name: string, args: string[]) {
    try {
      return await this.host.calc(name, args);
    } catch (e) {
      return { ok: false as const, error: e instanceof Error ? e.message : String(e) };
    }
  }

  /**
   * Validate a command line without executing it (GUI-005). Register
   * lines are session grammar, not engine grammar — always valid here.
   */
  async parse(line: string): Promise<{ ok: boolean; error?: string }> {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("register ") || trimmed.startsWith("#")) {
      return { ok: true };
    }
    try {
      return await this.host.parse(trimmed);
    } catch {
      // A host that cannot parse yet must not paint valid input as wrong.
      return { ok: true };
    }
  }

  /**
   * Run an experiment's setup script line by line, collecting the typed
   * event keys the engine emits — the material the codex checker compares
   * against. The run is ordinary bench work: it lands in the feed, the
   * log, and undo like anything else.
   */
  async runExperiment(text: string): Promise<string[]> {
    this.eventCollector = [];
    try {
      for (const raw of text.split("\n")) {
        const line = raw.trim();
        if (!line || line.startsWith("#")) continue;
        if (!(await this.submit(line))) break;
      }
      return this.eventCollector;
    } finally {
      this.eventCollector = null;
    }
  }

  /** Final-state numbers for the checker, from the render model. */
  finalStateForCheck(): { phValues: number[]; temperaturesC: number[] } {
    const vessels = this.scene?.vessels ?? [];
    return {
      phValues: vessels.flatMap((v) =>
        v.badges.filter((b) => b.key === "ph").map((b) => b.value),
      ),
      temperaturesC: vessels.map((v) => v.temperature_k - 273.15),
    };
  }

  /** The session as a .lab script — every session is one. */
  exportLab(): string {
    return this.commandLog.join("\n") + "\n";
  }
}

/** localStorage where the browser allows it; null where it throws
 * (private windows, blocked site data) — the session then simply does
 * not survive a reload, which is a feature loss, not a failure. */
function defaultStorage(): StorageLike | null {
  try {
    if (typeof localStorage !== "undefined") return localStorage;
  } catch {
    // fall through
  }
  return null;
}
