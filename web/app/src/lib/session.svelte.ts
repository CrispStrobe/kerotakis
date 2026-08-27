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
import { scriptKit } from "./codex";
import { schedule, type Playback } from "./replay";
import { effectFromEvent, vesselOf, type Effect } from "./magnitudes";
import { t } from "./i18n.svelte";
import { missionTitle } from "./storyProgress";
import { reagentAccess } from "./catalogProgress";
import { persistStockUsed, restoreStockUsed, stockRemaining, suppliedSpecies } from "./storyStock";
import type { LabMode } from "./worldState";
import {
  outcomeComplete,
  outcomeMissionContract,
  secureOutcomeEvidence,
  type OutcomeMissionContract,
} from "./outcomeMission";

export type FeedEntry = {
  kind: "command" | "line" | "error" | "refusal" | "note" | "user-note" | "hazard" | "chart";
  text: string;
  /** ISO timestamp for learner-authored journal notes. */
  createdAt?: string;
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
  /** Reactive-group hazard labels (CAP-11); empty when none apply. */
  hazards?: string[];
  /** False = nobody has assessed this species — say so, don't imply safe. */
  hazard_assessed?: boolean;
  /** Density in g/mL (engine registry) — the fluid overlay's buoyancy. */
  density?: number;
};

export type MissionDebrief = {
  id: string;
  evidence: string[];
  firstCompletion: boolean;
  completedTotal: number;
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
/** Learner progress: ids of codex entries whose run checked out. Kept
 * apart from the bench save — clearing the bench must not unlearn. */
const DONE_KEY = "kero.codex.done.v1";
/** Stable lesson ids completed in Story. ModeStorage keeps this out of Sandbox. */
const MISSION_DONE_KEY = "kero.missions.done.v1";

export class Session {
  register = $state<string>("lv1");
  scene = $state<Scene | null>(null);
  feed = $state<FeedEntry[]>([]);
  busy = $state(false);
  engineReady = $state(false);
  canSolve = $state(false);
  /** Engine identity from hello (GUI-001): "0.0.1 @ abc1234" or null. */
  engineIdentity = $state<string | null>(null);
  /** Codex entries this learner has run to a green check (GUI-053). */
  completedExperiments = $state<ReadonlySet<string>>(new Set());
  /** Guided missions completed through their final engine-backed command. */
  completedMissions = $state<ReadonlySet<string>>(new Set());
  /** Result card held after the lesson overlay closes. Chemistry keeps running. */
  missionDebrief = $state<MissionDebrief | null>(null);

  /** GUI-064: a titration being replayed at bench pace — the engine
   * already finished; this is the reveal. `delivered` climbs per curve
   * point; components (burette meniscus, drips) read it live. */
  titrationPlayback = $state<{ vessel: number; delivered: number; total: number } | null>(null);
  private playback: Playback | null = null;

  /** Push one transient effect — the same channel recordEffect uses.
   * Paced operational effects carry a moderate fixed magnitude: their
   * intensity is pacing, not an event amount (GUI-059's scaling reads
   * amounts where they exist). */
  private pushEffect(vessel: number, kind: string, magnitude = 0.6): void {
    const now = Date.now();
    const list = (this.vesselEffects[vessel] ?? []).filter((e) => now - e.at < (e.durationMs ?? 4000));
    const effect = { kind, at: now, magnitude };
    list.push(effect);
    this.vesselEffects = { ...this.vesselEffects, [vessel]: list };
    this.expireEffect(vessel, effect);
  }

  /** Removing an effect is itself reactive. CSS animations therefore stop
   * when their physical playback window ends, without waiting for another command. */
  private expireEffect(vessel: number, effect: Effect): void {
    setTimeout(() => {
      const current = this.vesselEffects[vessel] ?? [];
      if (!current.includes(effect)) return;
      this.vesselEffects = {
        ...this.vesselEffects,
        [vessel]: current.filter((candidate) => candidate !== effect),
      };
    }, (effect.durationMs ?? 4000) + 50);
  }

  /** GUI-064b: pace any multi-step operation's visible effects through
   * the one scheduler (clamped, cancellable, reduced-motion honest). */
  private playOperation(count: number, msPerTick: number, onTick: (i: number) => void): void {
    this.playback?.cancel();
    const reduced =
      typeof matchMedia !== "undefined" &&
      matchMedia("(prefers-reduced-motion: reduce)").matches;
    this.playback = schedule(count, msPerTick, onTick, () => {}, {
      reducedMotion: reduced,
    });
  }

  private startTitrationPlayback(vessel: number, curve: [number, number][]): void {
    this.playback?.cancel();
    const total = curve[curve.length - 1]?.[0] ?? 0;
    this.titrationPlayback = { vessel, delivered: 0, total };
    const reduced =
      typeof matchMedia !== "undefined" &&
      matchMedia("(prefers-reduced-motion: reduce)").matches;
    this.playback = schedule(
      curve.length,
      450,
      (i) => {
        this.titrationPlayback = { vessel, delivered: curve[i]?.[0] ?? 0, total };
        // Each increment drips — the same typed-effect channel as ever.
        this.pushEffect(vessel, "drip", 1);
      },
      () => {
        // Hold the final reading a beat, then clear the overlay state.
        setTimeout(() => (this.titrationPlayback = null), 1200);
      },
      { reducedMotion: reduced },
    );
  }

  /**
   * Bench snapshots keyed by log position: undo/scrub restores in O(1)
   * instead of replaying. A missing key falls back to replay; a key must
   * therefore NEVER outlive the prefix it was taken after — truncation
   * and clear drop the affected entries. Not reactive, not persisted.
   */
  private snapshots = new Map<number, string>();
  private static readonly SNAPSHOT_CAP = 40;

  private async takeSnapshot(position: number): Promise<void> {
    try {
      this.snapshots.set(position, await this.host.snapshot());
      // Evict oldest-inserted beyond the cap; replay covers the rest.
      while (this.snapshots.size > Session.SNAPSHOT_CAP) {
        const oldest = this.snapshots.keys().next().value as number;
        this.snapshots.delete(oldest);
      }
    } catch {
      // An engine without snapshots simply keeps the replay path.
    }
  }
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
  /** An open-ended mission is evaluated from typed engine events instead of
   * advancing through the source .lab commands. The .lab remains its source
   * for narration and core kit; the contract adds alternative valid routes. */
  missionOutcome = $state<{ contract: OutcomeMissionContract; secured: string[] } | null>(null);
  /** Log position after the lesson's last own step — the point "return
   * to the script" rewinds to. Free commands past it are the deviation. */
  private lessonBaseline = $state(0);
  /** Feed boundary for the active mission's engine-backed evidence ledger. */
  private lessonFeedStart = $state(0);
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
  vesselEffects = $state<Record<number, Effect[]>>({});
  /** Story-only material dispenses. The engine owns vessel amounts; this
   * ledger owns only what remains on the physical supply shelf. */
  storyStockUsed = $state<Record<string, number>>({});

  constructor(
    private host: EngineHost,
    private storage: StorageLike | null = defaultStorage(),
    private mode: LabMode = "sandbox",
  ) {
    if (mode === "story") this.storyStockUsed = restoreStockUsed(storage);
  }

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
          ? t("The bench is live: states nobody pre-computed are solved.")
          : t("The bench answers from shipped results only — the live aqueous engine is not attached."),
      });
      // A silently degraded engine hides real failures (salts that never
      // dissolve, colours that never appear). Say WHY, loudly.
      if (!this.canSolve && hello.aqueous_note) {
        this.feed.push({
          kind: "error",
          text: t("the aqueous engine failed to attach: {reason}", { reason: hello.aqueous_note }),
        });
      }
      this.restoreProgress();
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
        // These three were silently stripped by this re-map once —
        // hazard chips and fluid buoyancy read them from the shelf.
        hazards: s.hazards ?? [],
        hazard_assessed: s.hazard_assessed,
        density: s.density,
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
        notes?: { text: string; createdAt: string }[];
        /** v2: the engine snapshot at `position` — one restore() call
         * instead of a replay. Absent in v1 saves; replay covers those. */
        snapshot?: string;
      };
      if (!Array.isArray(saved.log)) return;
      const notes = Array.isArray(saved.notes)
        ? saved.notes.filter((note) => typeof note?.text === "string" && typeof note?.createdAt === "string")
        : [];
      if (saved.log.length === 0 && notes.length === 0) return;
      if (saved.register && saved.register !== this.register) {
        await this.host.setRegister(saved.register);
        this.register = saved.register;
      }
      const position = Math.max(0, Math.min(saved.log.length, saved.position ?? saved.log.length));
      let how = t("replayed");
      if (position > 0) {
        // Instant path first; replay stays the fallback AND the
        // definition of correctness (a snapshot restore must be
        // indistinguishable — pinned at the protocol level).
        let instant = false;
        if (saved.snapshot) {
          try {
            await this.host.restore(saved.snapshot);
            instant = true;
            how = t("restored instantly");
            this.snapshots.set(position, saved.snapshot);
          } catch {
            // Stale/incompatible token (engine upgraded): replay.
          }
        }
        if (!instant) {
          await this.host.runScript(saved.log.slice(0, position).join("\n"));
        }
      }
      this.commandLog = saved.log;
      this.position = position;
      this.feed.push({
        kind: "note",
        text: t("restored your last session: {count} step(s) {how}", { count: position, how }),
      });
      this.feed.push(...notes.map((note) => ({ kind: "user-note" as const, ...note })));
    } catch {
      // A corrupt save must never wedge the bench: drop it and start clean.
      this.storage?.removeItem(SAVE_KEY);
      this.feed.push({ kind: "note", text: t("could not restore the last session — starting fresh") });
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
          notes: this.feed
            .filter((entry) => entry.kind === "user-note")
            .map(({ text, createdAt }) => ({ text, createdAt: createdAt ?? new Date().toISOString() })),
          snapshot: this.snapshots.get(this.position),
        }),
      );
    } catch {
      // Storage full or blocked: the session still works, it just won't survive a reload.
    }
  }

  /** Add a learner-authored observation without pretending it came from the engine. */
  addUserNote(text: string): void {
    const trimmed = text.trim();
    if (!trimmed) return;
    this.feed.push({ kind: "user-note", text: trimmed, createdAt: new Date().toISOString() });
    this.persist();
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
      this.snapshots.clear();
      this.storage?.removeItem(SAVE_KEY);
      this.scene = await this.host.scene();
      this.inspector = null;
      this.feed.push({ kind: "note", text: t("the bench is empty again") });
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
      const supplied = suppliedSpecies(trimmed);
      const stockItem = supplied ? this.shelf.find((item) => item.key === supplied) : undefined;
      const missionSupply = Boolean(supplied && this.lesson?.kit.includes(supplied));
      if (this.mode === "story" && stockItem) {
        const access = reagentAccess("story", this.completedMissions.size, stockItem, missionSupply);
        if (!access.available) {
          this.feed.push({ kind: "refusal", text: t("That material is not yet available. Accept an investigation that supplies it or complete more missions.") });
          return false;
        }
        if (!missionSupply && stockRemaining(stockItem, this.storyStockUsed) <= 0) {
          this.feed.push({ kind: "refusal", text: t("That bottle is empty. Mission kits still supply required materials, and the stockroom refills after a new discovery.") });
          return false;
        }
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
          if (event?.event === "distilled") {
            // GUI-064b: distillation paced — steam leaves the source,
            // drips land in the receiver, over the operation's own
            // duration. Totals are the engine's; only pacing is ours.
            const from = Number(event.from ?? 0);
            const to = Number(event.to ?? 0);
            this.playOperation(24, 110, (i) => {
              this.pushEffect(from, "evaporate");
              if (i % 2 === 0) this.pushEffect(to, "drip");
            });
          }
          if (event?.event === "transported") {
            // A transport train: the flow visibly walks the chain, one
            // vessel per tick, for the engine's own step count (capped
            // by the scheduler's clamp).
            const chain = (Array.isArray(event.chain) ? event.chain : []) as number[];
            const steps = Math.max(chain.length, Math.min(Number(event.steps ?? 8), 40));
            if (chain.length > 0) {
              this.playOperation(steps, 150, (i) => {
                this.pushEffect(chain[i % chain.length]!, "swirl");
              });
            }
          }
          if (
            event?.event === "titrated" &&
            Array.isArray((event as { curve?: unknown }).curve) &&
            ((event as { curve: unknown[] }).curve.length > 1)
          ) {
            this.startTitrationPlayback(
              Number(event.vessel ?? 0),
              (event as { curve: [number, number][] }).curve,
            );
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
              text: String(event.reason ?? t("the bench refused this operation")),
            });
          }
        }
        if (this.missionOutcome) {
          const secured = secureOutcomeEvidence(
            this.missionOutcome.contract,
            this.missionOutcome.secured,
            step.events as Array<Record<string, unknown>>,
          );
          this.missionOutcome = { ...this.missionOutcome, secured };
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
      if (result.scene) {
        this.scene = result.scene;
        if (
          result.scene.vessels.length > 0 &&
          !result.scene.vessels.some((vessel) => vessel.id === this.selected)
        ) {
          this.selected = result.scene.vessels[0]?.id ?? 0;
          this.inspector = null;
        }
      }
      // Register lines are session state, not chemistry; everything else
      // that the engine accepted becomes part of the replayable script.
      // A command issued mid-history truncates the undone future first.
      if (this.position < this.commandLog.length) {
        this.commandLog = this.commandLog.slice(0, this.position);
        for (const k of [...this.snapshots.keys()]) {
          if (k > this.position) this.snapshots.delete(k);
        }
      }
      this.commandLog.push(trimmed);
      this.position = this.commandLog.length;
      if (this.mode === "story" && stockItem && !missionSupply) {
        this.storyStockUsed = {
          ...this.storyStockUsed,
          [stockItem.key]: (this.storyStockUsed[stockItem.key] ?? 0) + 1,
        };
        persistStockUsed(this.storage, this.storyStockUsed);
      }
      await this.takeSnapshot(this.position);
      this.persist();
      // The inspected vessel's detail is stale after any step.
      if (this.inspector) await this.inspect(this.inspector.vessel);
      this.finishOutcomeMissionIfComplete();
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
    this.feed.push({ kind: "note", text: t("running {name} on this bench", { name }) });
    let lineno = 0;
    for (const raw of text.split("\n")) {
      lineno += 1;
      const line = raw.trim();
      if (!line || line.startsWith("#")) continue;
      if (!(await this.submit(line))) {
        this.feed.push({
          kind: "note",
          text: t("stopped at {name}:{line} — the rest of the file did not run", { name, line: lineno }),
        });
        return;
      }
    }
    this.feed.push({ kind: "note", text: t("{name} finished", { name }) });
  }

  /** Map a typed event onto a transient canvas effect for its vessel:
   * kero1's magnitude pipeline (GUI-059) carries the intensity, with
   * #48's instrument probes grafted in (measured events are readings,
   * not amounts — they get a fixed moderate magnitude). */
  private recordEffect(event: Record<string, unknown>): void {
    let effect = effectFromEvent(event);
    if (!effect && event?.event === "measured") {
      const inst = String(event.instrument ?? "");
      const kind =
        inst === "thermometer" ? "thermometer" : inst === "ph_meter" ? "ph_probe" : null;
      if (kind) effect = { kind, at: Date.now(), magnitude: 0.6 };
    }
    if (!effect) return;
    const vessel = vesselOf(event);
    const now = Date.now();
    const list = (this.vesselEffects[vessel] ?? []).filter((e) => now - e.at < (e.durationMs ?? 4000));
    list.push(effect);
    this.vesselEffects = { ...this.vesselEffects, [vessel]: list };
    this.expireEffect(vessel, effect);
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
      this.feed.push({ kind: "note", text: t("speaking at {level}", { level }) });
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
      // O(1) path first: a snapshot taken at this position restores in
      // one call. Replay stays as the fallback and the semantics — a
      // restore must be indistinguishable from replaying the prefix.
      const snap = this.snapshots.get(target);
      if (snap !== undefined) {
        await this.host.restore(snap);
        this.scene = await this.host.scene();
      } else {
        await this.host.reset();
        if (prefix.length > 0) {
          const replay = await this.host.runScript(prefix.join("\n"));
          if (replay.scene) this.scene = replay.scene;
        } else {
          this.scene = await this.host.scene();
        }
      }
      const was = this.position;
      this.position = target;
      this.persist();
      this.feed.push({
        kind: "note",
        text:
          target < was
            ? t("stepped back to {position} of {total}", { position: target, total: this.commandLog.length })
            : t("stepped forward to {position} of {total}", { position: target, total: this.commandLog.length }),
      });
      if (this.inspector) await this.inspect(this.inspector.vessel);
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: t("replay failed, the bench may be out of sync — {reason}", {
          reason: e instanceof Error ? e.message : String(e),
        }),
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
    this.missionDebrief = null;
    const contract = outcomeMissionContract(name);
    this.missionOutcome = contract ? { contract, secured: [] } : null;
    this.lesson = {
      lesson: parseLesson(name, text),
      cursor: 0,
      kit: [...new Set([...scriptKit(text), ...(contract?.extraKit ?? [])])],
    };
    this.lessonBaseline = this.position;
    this.feed.push({ kind: "note", text: t("lesson started: {name}", { name: t(missionTitle(name)) }) });
    this.lessonFeedStart = this.feed.length;
    this.advanceLessonNotes();
  }

  /** Mission-only results, excluding narration and the commands themselves. */
  get lessonEvidence(): string[] {
    if (!this.lesson) return [];
    return this.feed
      .slice(this.lessonFeedStart)
      .filter((entry) => entry.kind === "line" || entry.kind === "hazard" || entry.kind === "chart")
      .map((entry) => entry.text)
      .slice(-6);
  }

  /** Surface consecutive narration, stopping at the next command. */
  private advanceLessonNotes(): void {
    if (!this.lesson) return;
    const { lesson } = this.lesson;
    while (this.lesson.cursor < lesson.steps.length) {
      const step = lesson.steps[this.lesson.cursor]!;
      if (step.kind !== "note") break;
      this.feed.push({ kind: "note", text: t(step.text) });
      this.lesson.cursor += 1;
    }
    if (this.lesson.cursor >= lesson.steps.length) {
      this.finishMission();
    }
  }

  private finishMission(): void {
    if (!this.lesson) return;
    const name = this.lesson.lesson.name;
    this.feed.push({ kind: "note", text: t("lesson finished: {name}", { name: t(missionTitle(name)) }) });
    const firstCompletion = !this.completedMissions.has(name);
    const evidence = this.feed
      .slice(this.lessonFeedStart)
      .filter((entry) => entry.kind === "line" || entry.kind === "hazard" || entry.kind === "chart")
      .map((entry) => entry.text)
      .slice(-6);
    this.markMissionDone(name);
    this.missionDebrief = {
      id: name,
      evidence,
      firstCompletion,
      completedTotal: this.completedMissions.size,
    };
    this.lesson = null;
    this.missionOutcome = null;
  }

  private finishOutcomeMissionIfComplete(): void {
    if (!this.missionOutcome || !this.lesson) return;
    if (outcomeComplete(this.missionOutcome.contract, this.missionOutcome.secured)) {
      this.finishMission();
    }
  }

  /** The lesson's next command, shown before it runs. */
  get lessonNextCommand(): string | null {
    if (!this.lesson || this.missionOutcome) return null;
    const step = this.lesson.lesson.steps[this.lesson.cursor];
    return step?.kind === "command" ? step.line : null;
  }

  /** Run the lesson's next command. Deviation is allowed at any time —
   * free commands do not move the lesson cursor. */
  async lessonNext(): Promise<void> {
    const line = this.lessonNextCommand;
    if (!line || !this.lesson) return;
    if (!(await this.submit(line))) return;
    this.lessonBaseline = this.position;
    this.lesson.cursor += 1;
    this.advanceLessonNotes();
  }

  /** How far the learner has wandered off the script, in commands. */
  get lessonDeviation(): number {
    if (!this.lesson || this.missionOutcome) return 0;
    return Math.max(0, this.position - this.lessonBaseline);
  }

  /**
   * Rewind the deviation: the bench returns to the exact state after the
   * lesson's last own step (snapshot-fast where cached). The wandering
   * stays in the log's undone future until the next command truncates
   * it — return is an undo, not an erasure.
   */
  async lessonReturn(): Promise<void> {
    if (!this.lesson || this.lessonDeviation === 0) return;
    await this.jumpTo(this.lessonBaseline);
    this.feed.push({ kind: "note", text: t("back on the script.") });
  }

  exitLesson(): void {
    if (!this.lesson) return;
    this.feed.push({ kind: "note", text: t("lesson left: {name}", { name: t(missionTitle(this.lesson.lesson.name)) }) });
    this.lesson = null;
    this.missionOutcome = null;
  }

  closeMissionDebrief(): void {
    this.missionDebrief = null;
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

  /** Record a codex entry whose bench run agreed with its claims. */
  markExperimentDone(id: string): void {
    if (this.completedExperiments.has(id)) return;
    const next = new Set(this.completedExperiments);
    next.add(id);
    this.completedExperiments = next;
    try {
      this.storage?.setItem(DONE_KEY, JSON.stringify([...next]));
    } catch {
      // Progress persistence is a convenience, never a requirement.
    }
  }

  markMissionDone(id: string): void {
    if (this.completedMissions.has(id)) return;
    const next = new Set(this.completedMissions);
    next.add(id);
    this.completedMissions = next;
    if (this.mode === "story") {
      this.storyStockUsed = {};
      persistStockUsed(this.storage, this.storyStockUsed);
    }
    try {
      this.storage?.setItem(MISSION_DONE_KEY, JSON.stringify([...next]));
    } catch {
      // Story progress remains valid for this visit without persistence.
    }
  }

  /** Load learner progress; called from connect, harmless without storage. */
  restoreProgress(): void {
    this.completedExperiments = this.restoreIds(DONE_KEY);
    this.completedMissions = this.restoreIds(MISSION_DONE_KEY);
  }

  private restoreIds(key: string): ReadonlySet<string> {
    try {
      const raw = this.storage?.getItem(key);
      if (!raw) return new Set();
      const ids = JSON.parse(raw) as unknown;
      return Array.isArray(ids)
        ? new Set(ids.filter((id): id is string => typeof id === "string"))
        : new Set();
    } catch {
      // One corrupt progress blob reads as empty without hiding the other.
      return new Set();
    }
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
