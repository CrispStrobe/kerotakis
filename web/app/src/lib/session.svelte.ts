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
import { latestNetIonic, spectatorPhrase, type NetIonic } from "./ionic";
import { type Lesson, parseLesson } from "./lesson";
import { scriptKit } from "./codex";
import { schedule, type Playback } from "./replay";
import type { AnswerRefusal, QuestOutput } from "./host/EngineHost";
import { effectFromEvent, vesselOf, type Effect } from "./magnitudes";
import { i18n, t } from "./i18n.svelte";
import { missionTitle } from "./storyProgress";
import { caseAwardedTools, contaminatedSampleComplete } from "./storyChapter";
import { access as catalogAccess, catalogMap, type CatalogMap } from "./catalogProgress";
import { persistStockUsed, restoreStockUsed, stockRemaining, suppliedSpecies, STORY_STOCK_KEY } from "./storyStock";
import type { LabMode } from "./worldState";
import { parseElementCoverage, type ElementCoverageReport } from "./elements";
import {
  completedRoute,
  outcomeComplete,
  outcomeMissionContract,
  routeProgress,
  secureOutcomeEvidence,
  type OutcomeMissionContract,
} from "./outcomeMission";
import { summarizeResult, type ResultSummary } from "./resultSummary";
import { incidentNotebookEvidence } from "./incidents";

export type FeedEntry = {
  kind: "command" | "line" | "error" | "refusal" | "note" | "user-note" | "hazard" | "chart" | "nudge" | "claim";
  text: string;
  /** ISO timestamp for learner-authored journal notes. */
  createdAt?: string;
  /** Hazard entries: the engine's severity, for the card's chip. */
  severity?: string;
  hazardText?: string;
  realWorld?: string;
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
  /** A versioned named mixture/object rather than a pure species. */
  material?: boolean;
  /**
   * GUI-093 shelf-role inputs. All additive: an older engine build omits
   * them and `reagentRoles.ts` falls back to what `hazards` still says.
   */
  /** Unflattened `kerotakis_safety::groups` rows ("acid_strong", …). */
  reactive_groups?: string[];
  /** Element counts from the engine's formula parser. */
  elements?: Record<string, number>;
  /** Net charge from the same parse; 0 for a neutral species. */
  charge?: number;
  /** In `kerotakis_core::indicator::INDICATORS`. */
  indicator?: boolean;
  /** A solvent the engine models solutions in (water, or an organic one). */
  solvent?: boolean;
  /** Materials only: the registry keys of what the mixture is made of. */
  components?: string[];
  /** Biochemical discovery metadata supplied by the engine. */
  enzyme_family?:
    | "lactase"
    | "protease"
    | "lipase"
    | "catalase"
    | "pepsin"
    | "bromelain";
  protein?: boolean;
  capability?: "modeled_reaction" | "modeled_activity" | "modeled_observation" | "identity_only";
};

export type MissionDebrief = {
  id: string;
  evidence: string[];
  firstCompletion: boolean;
  completedTotal: number;
  /** Which of several valid solutions this learner actually took. Null where
   * the mission offers only one, so the debrief stays quiet about a choice
   * that was never offered. */
  route: string | null;
  /** The instrument a closed case just earned, on the one run that closed
   * it. Null otherwise — including on every later replay, because the award
   * is derived from the leads and was already earned. */
  caseAward: string | null;
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
  /** Optional batched write: several keys promoted as ONE save, so a
   * multi-part outcome cannot be half-recorded. Storage that cannot do it
   * (a plain Web Storage, a test double) simply omits it and the session
   * falls back to sequential writes. */
  setItems?(changes: Readonly<Record<string, string>>): void;
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
  /** Exact command currently waiting on the engine. Presentation may use it
   * to energize only the matching apparatus; `busy` is deliberately broader. */
  activeCommand = $state<string | null>(null);
  engineReady = $state(false);
  /** The engine would not take a language, so its prose is English.
   *
   * Observable rather than merely logged: the native app refused
   * `set_locale` for months and the only evidence was that the journal
   * read English while the buttons around it read German. A test can
   * assert on this; nobody was ever going to notice the absence. */
  localeRefused = $state(false);
  canSolve = $state(false);
  /** Engine identity from hello (GUI-001): "0.0.1 @ abc1234" or null. */
  engineIdentity = $state<string | null>(null);
  /** Codex entries this learner has run to a green check (GUI-053). */
  completedExperiments = $state<ReadonlySet<string>>(new Set());
  /** Guided missions completed through their final engine-backed command. */
  completedMissions = $state<ReadonlySet<string>>(new Set());
  /** Result card held after the lesson overlay closes. Chemistry keeps running. */
  missionDebrief = $state<MissionDebrief | null>(null);
  /** WORLD-006: the last commit that could not be written, held absolute so
   * retrying it is indistinguishable from having written it the first time. */
  private pendingWrite = $state<Record<string, string> | null>(null);

  /**
   * WORLD-003: what this learner can reach, as the ENGINE answers it.
   *
   * Fetched rather than computed, and refreshed whenever an input to the
   * answer changes — the mode, the completed count, a closed case's award,
   * or the active mission's kit. Empty until the engine replies, which
   * callers must treat as "not yet known" rather than as a refusal.
   */
  catalog = $state<CatalogMap>(new Map());

  /** GUI-066: the running quest, engine-evaluated. Claims progress for
   * the panel; nudges arrive as feed cards. */
  quest = $state<{
    id: string;
    title: Record<string, string>;
    goal: Record<string, string>;
    claims: { id: string; title: Record<string, string>; satisfied: boolean }[];
    unknowns: string[];
    complete: boolean;
  } | null>(null);

  /** Begin a quest from its exported spec (the panel fetched it). */
  async startQuest(spec: {
    id: string;
    title: Record<string, string>;
    goal: Record<string, string>;
    claims: { id: string; title: Record<string, string> }[];
    unknowns?: Record<string, string>;
  }): Promise<void> {
    await this.host.questStart(JSON.stringify(spec));
    this.quest = {
      id: spec.id,
      title: spec.title,
      goal: spec.goal,
      claims: spec.claims.map((c) => ({ ...c, satisfied: false })),
      unknowns: Object.keys(spec.unknowns ?? {}),
      complete: false,
    };
    this.feed.push({
      kind: "note",
      text: t("quest started: {title}", { title: spec.title[this.register] ?? spec.id }),
    });
  }

  async stopQuest(): Promise<void> {
    await this.host.questStop();
    this.quest = null;
  }

  /** Name a sealed unknown; the engine answers, never blocks. */
  async answerUnknown(alias: string, guess: string): Promise<void> {
    try {
      const { outputs, refusal } = await this.host.questAnswer(alias, guess);
      this.applyQuestOutputs(outputs);
      if (refusal !== undefined) {
        // Rendered HERE from the engine's stable id, so a German session
        // reads German. It arrives as a note, not an error: the engine's
        // whole contract for a wrong guess is that it is spoken, never a
        // block, and styling it as a failure said the opposite.
        this.feed.push({ kind: "note", text: this.refusalText(refusal, guess) });
      } else if (outputs.length === 0) {
        this.feed.push({ kind: "note", text: t('"{guess}" — not it yet; look again.', { guess }) });
      }
    } catch (e) {
      this.feed.push({ kind: "error", text: e instanceof Error ? e.message : String(e) });
    }
  }

  /** One place the engine's refusal ids become sentences, in the learner's
   * own language. An id the client does not know yet still says something
   * useful rather than nothing. */
  private refusalText(refusal: AnswerRefusal, guess: string): string {
    if (refusal.refused === "wrong_guess") {
      return t('"{guess}" is not what {alias} hides — the measurements you have already made settle it; look at what they say', {
        guess,
        alias: refusal.alias,
      });
    }
    if (refusal.refused === "unknown_alias") {
      return t('nothing in this investigation seals an unknown called "{alias}"', { alias: refusal.alias });
    }
    return t('"{guess}" — not it yet; look again.', { guess });
  }

  private applyQuestOutputs(outputs: QuestOutput[]): void {
    for (const o of outputs) {
      const text = (o.say ?? o.title)?.[this.register as "lv1" | "lv2" | "lv3"] ?? "";
      if (o.kind === "nudge") {
        this.feed.push({ kind: "nudge", text });
      } else if (o.kind === "claim_satisfied") {
        this.feed.push({ kind: "claim", text });
        if (this.quest) {
          // By id. Comparing titles was recognising a claim by its sentence
          // while its id sat unused in both directions — and two claims
          // sharing a title would have satisfied the wrong one. Prose stays
          // only as the fallback for an engine older than the `claim` field.
          const c = this.quest.claims.find((cl) =>
            o.claim !== undefined
              ? cl.id === o.claim
              : cl.title.lv1 === o.title?.lv1 || cl.title.lv2 === o.title?.lv2,
          );
          if (c) c.satisfied = true;
        }
      } else if (o.kind === "constraint_violated") {
        // Spoken, never blocking (WORLD-004). It reads as a nudge because
        // that is what it is: the mission noticing, not the bench refusing.
        this.feed.push({ kind: "nudge", text });
      } else if (o.kind === "completed") {
        this.feed.push({ kind: "claim", text: t("quest complete: {title}", { title: text }) });
        if (this.quest) this.quest.complete = true;
      }
    }
  }

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
  /** Core-generated coverage; null only for an older/degraded host. */
  elementCoverage = $state<ElementCoverageReport | null>(null);
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
   * Every distinct balanced equation this session's own reactions produced,
   * newest first (GUI-095).
   *
   * The balancing drill prefers these over the catalogue's: an equation the
   * learner just made happen on the bench is a better question than one out
   * of a book they have not opened. Bounded, because it is practice
   * material rather than a record — the feed and the notebook are the
   * record, and they keep everything.
   */
  benchEquations = $state<string[]>([]);
  /** The same reaction as an ionic equation, derived by the engine from
   * the solved speciation (GUI-092). Null is the common case and an
   * honest one: no speciation, no ionic claim. */
  lastIonic = $state<NetIonic | null>(null);
  /** The ions the solver left out of the reaction, as one phrase — the
   * half of the lesson the equation itself cannot show. */
  get lastSpectators(): string | null {
    return this.lastIonic ? spectatorPhrase(this.lastIonic) : null;
  }
  /** Compact evidence digest for the latest accepted operation. */
  latestResult = $state<ResultSummary | null>(null);
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

  /** Drops the language subscription if the session is reconnected. */
  private stopWatchingLocale: (() => void) | null = null;

  constructor(
    private host: EngineHost,
    private storage: StorageLike | null = defaultStorage(),
    private mode: LabMode = "sandbox",
  ) {
    if (mode === "story") this.storyStockUsed = restoreStockUsed(storage);
  }

  async connect(): Promise<void> {
    try {
      // The locale goes FIRST, before hello and therefore before anything
      // that waits on hello. The engine renders each line as the operation
      // runs, so a locale arriving second leaves the opening lines in
      // English under an otherwise German session. The worker log said it
      // plainly: `run_script` at id 3, `set_locale` at id 4.
      //
      // Awaiting it inside connect after hello was not enough — something
      // reaches the engine without waiting for connect to finish — so it
      // moves ahead of the one call everything does wait for.
      try {
        await this.host.setLocale(i18n.locale);
      } catch (e) {
        // English is a working fallback; a dead session is not — so this
        // still does not throw. But it must not be silent either: this
        // exact catch hid the native app refusing `set_locale` for as long
        // as the engine has had a catalogue, and the iPad rendered English
        // prose under German buttons with nothing anywhere reporting it.
        console.error("the engine refused the locale; rendering English", e);
        this.localeRefused = true;
      }
      const hello = await this.host.hello();
      this.engineReady = true;
      // Match the engine to the interface immediately, and follow it
      // afterwards. Done here rather than in the constructor because it
      // needs a live host; done before the first prose is rendered so the
      // opening lines are not English in a German session.
      // AWAITED, not fired and forgotten. The engine renders each line as
      // the command runs, so a locale still in flight means the opening
      // lines come out English while everything after them is German —
      // which is exactly what the browser gate caught: a German vessel
      // summary above an English journal entry.
      //
      // It still cannot throw: applyEngineLocale swallows its own
      // failures, because a host that cannot switch language should keep
      // rendering English rather than take the session down.
      this.stopWatchingLocale?.();
      this.stopWatchingLocale = i18n.onChange((code) => void this.applyEngineLocale(code));
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
      // Progress is restored, so the catalog request can be asked truthfully.
      // Awaited: the cabinet must not render a moment of "nothing available"
      // before the engine has said what is.
      await this.refreshCatalog();
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
        material: s.material === true,
      }));
      try {
        this.elementCoverage = parseElementCoverage(await this.host.elementCoverage());
      } catch {
        // Host upgrades are rolling; formula-derived shelf coverage remains
        // an honest fallback until the new endpoint is available.
        this.elementCoverage = null;
      }
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

  /** Notes are identified by their creation time, which is stable in saves and exports. */
  editUserNote(createdAt: string, text: string): void {
    const trimmed = text.trim();
    if (!trimmed) return;
    this.feed = this.feed.map((entry) =>
      entry.kind === "user-note" && entry.createdAt === createdAt
        ? { ...entry, text: trimmed }
        : entry,
    );
    this.persist();
  }

  removeUserNote(createdAt: string): void {
    this.feed = this.feed.filter((entry) =>
      entry.kind !== "user-note" || entry.createdAt !== createdAt,
    );
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
      this.latestResult = null;
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
    this.activeCommand = trimmed;
    this.feed.push({ kind: "command", text: trimmed });
    try {
      if (trimmed.startsWith("register ")) {
        return await this.applyRegister(trimmed.slice("register ".length).trim());
      }
      const supplied = suppliedSpecies(trimmed);
      const stockItem = supplied ? this.shelf.find((item) => item.key === supplied) : undefined;
      const missionSupply = Boolean(supplied && this.lesson?.kit.includes(supplied));
      if (this.mode === "story" && stockItem) {
        // The engine decides what progress has earned; the client asks and
        // obeys. Two deliberate exceptions to obeying:
        //
        // A catalog that has not arrived says NOTHING, and saying nothing
        // must not read as a refusal — a dropped round trip should not lock
        // a learner out of their own shelf.
        //
        // And a mission's own kit is loaned by the client that assembled it,
        // so it is honoured without waiting for the engine to agree. The
        // engine says `loaned` for the same materials when asked; making the
        // gate wait for that round trip would refuse the first tap of a
        // mission the learner has only just accepted.
        const access = this.catalogAccess(stockItem.key);
        if (access !== null && !access.available && !missionSupply) {
          this.feed.push({ kind: "refusal", text: t("That material is not yet available. Accept an investigation that supplies it or complete more missions.") });
          return false;
        }
        if (!missionSupply && stockRemaining(stockItem, this.storyStockUsed) <= 0) {
          this.feed.push({ kind: "refusal", text: t("That bottle is empty. Mission kits still supply required materials, and the stockroom refills after a new discovery.") });
          return false;
        }
      }
      const beforeScene = this.scene;
      const result = await this.host.runScript(trimmed);
      for (const step of result.steps) {
        // Hazard events become cards, from the typed event itself — the
        // warning always precedes the chemistry, and the chemistry then
        // runs and shows why (the engine's "hazards teach" rule).
        for (const event of step.events as Array<Record<string, unknown>>) {
          this.recordEffect(event);
          const incidentEvidence = incidentNotebookEvidence(event);
          if (incidentEvidence) this.feed.push({ kind: "note", text: incidentEvidence });
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
          if (event?.event === "hazard_warning" || event?.event === "spill_hazard") {
            const hazardText = String(event.hazard ?? "");
            const realWorld = String(event.real_world ?? "");
            this.feed.push({
              kind: "hazard",
              severity: String(event.severity ?? ""),
              text: `${hazardText} — ${realWorld}`,
              hazardText,
              realWorld,
            });
          } else if (event?.event === "safety_veto") {
            this.feed.push({
              kind: "refusal",
              text: String(event.reason ?? t("the bench refused this operation")),
            });
          }
        }
        const questOutputs = (step as { quest?: QuestOutput[] }).quest;
        if (questOutputs && questOutputs.length > 0) {
          this.applyQuestOutputs(questOutputs);
        }
        if (this.missionOutcome) {
          const secured = secureOutcomeEvidence(
            this.missionOutcome.contract,
            this.missionOutcome.secured,
            step.events as Array<Record<string, unknown>>,
          );
          this.missionOutcome = { ...this.missionOutcome, secured };
        }
        let pinnedEquation = false;
        for (const rendered of step.rendered) {
          this.feed.push({ kind: "line", text: rendered });
          // The engine writes balanced equations with a real arrow; the
          // latest one is the reaction the bench is showing right now.
          const eq = rendered.match(/\S[^.:]*(?:→|⇌)[^.]*/);
          if (eq) {
            const equation = eq[0].trim();
            this.lastEquation = equation;
            this.rememberEquation(equation);
            pinnedEquation = true;
          }
        }
        // The ionic form (GUI-092, kerotakis-core::ionic): structured, not
        // parsed back out of prose. It is pinned WITH the molecular
        // equation rather than independently — an ionic line left standing
        // under a later reaction's molecular one would be a claim about
        // chemistry that did not happen.
        const stepIonic = latestNetIonic([step]);
        if (pinnedEquation || stepIonic) this.lastIonic = stepIonic;
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
      const resultEvents = result.steps.flatMap((step) => step.events);
      const resultLines = result.steps.flatMap((step) => step.rendered);
      this.latestResult = summarizeResult(resultEvents, resultLines, beforeScene, result.scene ?? this.scene);
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
      this.activeCommand = null;
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
        inst === "thermometer"
          ? "thermometer"
          : inst === "ph_meter"
            ? "ph_probe"
            : inst === "balance"
              ? "balance"
              : inst === "pressure_gauge"
                ? "pressure_gauge"
                : inst === "volume_meter"
                  ? "volume_meter"
                  : inst === "conductivity_meter"
                    ? "conductivity_meter"
                    : inst === "spectrophotometer"
                      ? "uvvis"
                    : inst === "calorimeter"
                        ? "calorimeter"
                        : inst === "geiger_counter"
                          ? "geiger_counter"
              : null;
      if (kind) {
        const reading = Number(event.value);
        effect = {
          kind,
          at: Date.now(),
          magnitude: kind === "geiger_counter"
            ? Math.min(1, Math.max(0, Math.log10(Math.max(1, reading)) / 8))
            : 0.6,
          reading,
          unit: String(event.unit ?? ""),
        };
      }
    }
    if (!effect && event?.event === "chromatographed" && Array.isArray(event.peaks)) {
      const bands = event.peaks.flatMap((peak) => {
        if (!peak || typeof peak !== "object") return [];
        const value = peak as Record<string, unknown>;
        return [{
          species: String(value.species ?? "?"),
          retentionTimeS: Number(value.retention_time_s ?? 0),
          widthS: Number(value.width_s ?? 0),
          relativeArea: Number(value.relative_area ?? 0),
          partitionK: Number(value.partition_k ?? 0),
        }];
      });
      effect = {
        kind: "chromatograph",
        at: Date.now(),
        durationMs: 5200,
        magnitude: Math.min(1, 0.35 + bands.length * 0.13),
        bands,
        voidTimeS: Number(event.void_time_s ?? 0),
        plates: Number(event.plates ?? 0),
        outsideMethod: Array.isArray(event.outside_method) ? event.outside_method.map(String) : [],
      };
    }
    if (!effect && event?.event === "observed" && event.appearance && typeof event.appearance === "object") {
      const appearance = event.appearance as Record<string, unknown>;
      const colour = (value: unknown): [number, number, number] | undefined => {
        if (!value || typeof value !== "object") return undefined;
        const rgb = value as Record<string, unknown>;
        return [Number(rgb.r ?? 255), Number(rgb.g ?? 255), Number(rgb.b ?? 255)];
      };
      const deposit = Array.isArray(appearance.deposit) ? appearance.deposit : null;
      effect = {
        kind: "inspect",
        at: Date.now(),
        durationMs: 4500,
        magnitude: Math.max(0.3, Math.min(1, Number(appearance.cloudiness ?? 0) + (deposit ? 0.25 : 0))),
        appearance: {
          liquidRgb: colour(appearance.liquid),
          cloudiness: Number(appearance.cloudiness ?? 0),
          deposit: deposit && deposit.length >= 2 && colour(deposit[1])
            ? { species: String(deposit[0]), rgb: colour(deposit[1])! }
            : undefined,
          bubbling: Boolean(appearance.bubbling),
        },
      };
    }
    if (effect?.kind === "settle" && effect.settling) {
      const vessel = this.scene?.vessels.find((candidate) => candidate.id === Number(event.vessel ?? 0));
      effect = {
        ...effect,
        settling: {
          ...effect.settling,
          populations: effect.settling.populations.map((population) => {
            const solid = vessel?.solids.find((candidate) => candidate.species === population.species);
            return {
              ...population,
              colour: solid ? `rgb(${solid.srgb[0]} ${solid.srgb[1]} ${solid.srgb[2]})` : undefined,
            };
          }),
        },
      };
    }
    if (effect?.kind === "centrifuge" && effect.centrifuge) {
      const vessel = this.scene?.vessels.find((candidate) => candidate.id === Number(event.vessel ?? 0));
      effect = {
        ...effect,
        centrifuge: {
          ...effect.centrifuge,
          populations: effect.centrifuge.populations.map((population) => {
            const solid = vessel?.solids.find((candidate) => candidate.species === population.species);
            return {
              ...population,
              colour: solid ? `rgb(${solid.srgb[0]} ${solid.srgb[1]} ${solid.srgb[2]})` : undefined,
            };
          }),
        },
      };
    }
    if (effect?.kind === "swirl" && effect.stir) {
      const vessel = this.scene?.vessels.find((candidate) => candidate.id === Number(event.vessel ?? 0));
      const solids = (vessel?.solids ?? [])
        .filter((solid) => !solid.metallic)
        .map((solid) => ({
          species: solid.species,
          name: solid.name,
          moles: solid.moles,
          colour: `rgb(${solid.srgb[0]} ${solid.srgb[1]} ${solid.srgb[2]})`,
        }));
      effect = { ...effect, stir: { ...effect.stir, solids } };
    }
    if (effect?.kind === "evaporate" && event?.event === "evaporated") {
      const srgb = this.scene?.vessels.find((candidate) => candidate.id === Number(event.vessel ?? 0))?.liquid?.srgb;
      if (srgb) effect = { ...effect, fluidColour: `rgb(${srgb[0]} ${srgb[1]} ${srgb[2]})` };
    }
    if (
      effect?.source !== undefined &&
      effect.operation &&
      ["pour", "filter", "drain", "magnet"].includes(effect.operation)
    ) {
      const source = this.scene?.vessels.find((vessel) => vessel.id === effect!.source);
      const lowerLayer = effect.operation === "drain" ? source?.layers?.[0] : undefined;
      const upperLayer = effect.operation === "drain" && (source?.layers?.length ?? 0) > 1
        ? source?.layers?.at(-1)
        : undefined;
      const srgb = lowerLayer?.srgb ?? source?.liquid?.srgb;
      if (srgb && effect.operation !== "magnet") effect = { ...effect, fluidColour: `rgb(${srgb[0]} ${srgb[1]} ${srgb[2]})` };
      if (effect.operation === "magnet" && effect.magnetic && source) {
        const attractedKeys = new Set(effect.magnetic.attractedSpecies);
        const attracted = source.solids
          .filter((solid) => attractedKeys.has(solid.species))
          .map((solid) => ({
            species: solid.species,
            name: solid.name,
            moles: solid.moles,
            colour: `rgb(${solid.srgb[0]} ${solid.srgb[1]} ${solid.srgb[2]})`,
          }));
        const attractedMoles = attracted.reduce((sum, solid) => sum + solid.moles, 0);
        effect = {
          ...effect,
          magnitude: attractedMoles > 0
            ? Math.max(.15, Math.min(1, attractedMoles / .1))
            : effect.magnitude,
          magnetic: { ...effect.magnetic, attracted },
        };
      }
      if (effect.operation === "drain" && effect.drain) {
        effect = {
          ...effect,
          drain: {
            ...effect.drain,
            lowerColour: srgb ? `rgb(${srgb[0]} ${srgb[1]} ${srgb[2]})` : undefined,
            upperColour: upperLayer
              ? `rgb(${upperLayer.srgb[0]} ${upperLayer.srgb[1]} ${upperLayer.srgb[2]})`
              : undefined,
          },
        };
      }
      if (effect.operation === "filter" && source) {
        const filterResidue = source.solids.map((solid) => ({
          species: solid.species,
          name: solid.name,
          moles: solid.moles,
          colour: `rgb(${solid.srgb[0]} ${solid.srgb[1]} ${solid.srgb[2]})`,
        }));
        const retainedMoles = filterResidue.reduce((sum, solid) => sum + solid.moles, 0);
        effect = {
          ...effect,
          filterResidue,
          magnitude: retainedMoles > 0
            ? Math.max(0.15, Math.min(1, retainedMoles / 0.1))
            : effect.magnitude,
        };
      }
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

  /**
   * Keep the ENGINE speaking the interface's language.
   *
   * The engine composes the vessel summary and the journal itself, out of
   * fragments, so translating in the shell cannot reach them — it has to
   * be told. Subscribed rather than called from the switcher, so a future
   * caller of setLocale does not have to know to do this.
   *
   * Failures are swallowed on purpose: a host that cannot switch language
   * should keep rendering English, not break the bench. The worst case is
   * prose in the wrong language, which the learner can see and report; a
   * thrown error here would take the session down.
   */
  private async applyEngineLocale(code: string): Promise<void> {
    try {
      await this.host.setLocale(code);
      if (this.inspector) await this.inspect(this.inspector.vessel);
    } catch {
      // English is a working fallback; a dead session is not.
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
      this.latestResult = null;
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
    // The mission's kit is part of what the engine is asked, so entering a
    // mission changes the answer: its loaned apparatus appears on the wall.
    void this.refreshCatalog();
    this.advanceLessonNotes();
  }

  /**
   * How far through the mission the learner is, counted against the route
   * they are closest to finishing rather than every route at once — an
   * alternative solution they did not take must not read as work outstanding.
   */
  get missionProgress(): { done: number; total: number } | null {
    if (!this.missionOutcome) return null;
    const { done, total } = routeProgress(this.missionOutcome.contract, this.missionOutcome.secured);
    return { done, total };
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
    // Read the route BEFORE the outcome is cleared: which valid solution the
    // learner found is the most interesting thing the debrief can say.
    // Was the case still open before this mission landed? Asked BEFORE the
    // completion is recorded, because afterwards every replay looks like the
    // run that closed it.
    const caseWasOpen = !contaminatedSampleComplete(this.completedMissions);
    const outcome = this.missionOutcome;
    const route =
      outcome && outcome.contract.routes.length > 1
        ? (completedRoute(outcome.contract, outcome.secured)?.label ?? null)
        : null;
    const evidence = this.feed
      .slice(this.lessonFeedStart)
      .filter((entry) => entry.kind === "line" || entry.kind === "hazard" || entry.kind === "chart")
      .map((entry) => entry.text)
      .slice(-6);
    this.markMissionDone(name);
    const caseAward =
      caseWasOpen && contaminatedSampleComplete(this.completedMissions)
        ? (caseAwardedTools(this.completedMissions)[0] ?? null)
        : null;
    this.missionDebrief = {
      id: name,
      evidence,
      firstCompletion,
      completedTotal: this.completedMissions.size,
      route,
      caseAward,
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
    // Leaving gives the loan back; the wall must say so.
    queueMicrotask(() => void this.refreshCatalog());
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

  /**
   * Record a completed mission, and everything that completion changes,
   * as one transaction.
   *
   * The completion, the replenished stockroom, and any case the mission
   * closes are one fact about the world. Written key by key they are three,
   * and an interrupted write leaves a learner who finished a mission with a
   * spent stockroom and no completion — or a closed case whose instrument
   * never arrived. Storage that can promote a batch does so in one save.
   *
   * Nothing here grants a reward: what a closed case is worth is DERIVED
   * from the completed leads, so replaying a mission cannot claim it twice.
   */
  markMissionDone(id: string): void {
    if (this.completedMissions.has(id)) return;
    const next = new Set(this.completedMissions);
    next.add(id);
    const changes: Record<string, string> = { [MISSION_DONE_KEY]: JSON.stringify([...next]) };
    const replenish = this.mode === "story";
    if (replenish) changes[STORY_STOCK_KEY] = JSON.stringify({});
    this.commit(changes);
    this.completedMissions = next;
    if (replenish) this.storyStockUsed = {};
    void this.refreshCatalog();
  }

  /**
   * Write several keys as one save, and keep what could not be written.
   *
   * WORLD-006. Two properties make an interrupted commit survivable.
   *
   * Changes are ABSOLUTE, never deltas: each value is the complete next
   * state of its key. So replaying a commit is indistinguishable from
   * making it once, and a retry needs no reasoning about what already
   * landed — which is what "retry safely after interruption" has to mean
   * when the interruption is a write that threw halfway.
   *
   * A failed write is REMEMBERED, not swallowed. The change set is held and
   * merged into the next commit, so a learner who finishes two missions
   * through a full quota and then frees space keeps both, rather than
   * silently losing the first. Newer values win the merge, because they are
   * the later truth about the same key.
   *
   * Persistence remains a convenience: a blocked storage costs the visit
   * its record, never its session.
   */
  private commit(changes: Readonly<Record<string, string>>): void {
    const storage = this.storage;
    if (!storage) return;
    const merged = { ...(this.pendingWrite ?? {}), ...changes };
    try {
      if (typeof storage.setItems === "function") {
        storage.setItems(merged);
      } else {
        for (const [key, value] of Object.entries(merged)) storage.setItem(key, value);
      }
      this.pendingWrite = null;
    } catch {
      this.pendingWrite = merged;
    }
  }

  /**
   * Try again to write what a previous commit could not.
   *
   * Safe to call at any time and any number of times: the held change set
   * is absolute, so a redundant retry writes the same bytes. Returns true
   * when nothing is outstanding.
   */
  retryPendingWrite(): boolean {
    if (this.pendingWrite === null) return true;
    this.commit({});
    return this.pendingWrite === null;
  }

  /** True while a completed mission's record has not reached storage.
   * The UI can say so rather than letting a learner believe it is saved. */
  get progressUnsaved(): boolean {
    return this.pendingWrite !== null;
  }

  /**
   * Ask the engine what this learner can reach.
   *
   * Everything the answer depends on is in the request — mode, completed
   * count, the awards a closed case derived, and the active mission's kit —
   * so the response is complete and the client never has to reason about
   * "yes but a mission is lending it". Called whenever one of those changes.
   *
   * A failure leaves the previous answer standing rather than emptying the
   * cabinet: a dropped round trip is not the same as losing your equipment.
   */
  async refreshCatalog(): Promise<void> {
    try {
      const response = await this.host.catalog({
        mode: this.mode,
        completed: this.completedMissions.size,
        awarded: caseAwardedTools(this.completedMissions),
        mission_kit: [
          ...(this.lesson?.kit ?? []),
          ...(this.missionOutcome?.contract.extraTools ?? []),
        ],
      });
      this.catalog = catalogMap(response.items);
    } catch {
      // Keep whatever the engine last told us.
    }
  }

  /** What the engine said about one catalog id, or null before it answers. */
  catalogAccess(id: string) {
    return catalogAccess(this.catalog, id);
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
   * Keep one of the bench's own equations for the balancing drill.
   *
   * Newest first and capped: this is practice material, not a record. The
   * feed and the notebook are the record and they keep everything, so
   * dropping the twenty-first equation loses nothing a learner can go and
   * look up.
   */
  private rememberEquation(equation: string) {
    if (!/→|⇌/.test(equation)) return;
    this.benchEquations = [
      equation,
      ...this.benchEquations.filter((existing) => existing !== equation),
    ].slice(0, 20);
  }

  /** The balancing exercise for one skeleton (GUI-095) — the question,
   * with no route back to the answer. */
  async balanceExercise(equation: string) {
    try {
      return await this.host.balanceExercise(equation);
    } catch (e) {
      return { ok: false as const, error: e instanceof Error ? e.message : String(e) };
    }
  }

  /** Mark one answer engine-side (GUI-095). */
  async balanceMark(equation: string, answer: number[]) {
    try {
      return await this.host.balanceMark(equation, answer);
    } catch (e) {
      return { ok: false as const, error: e instanceof Error ? e.message : String(e) };
    }
  }

  /** The answer, written out, when the learner asks for it (GUI-095). */
  async balanceReveal(equation: string) {
    try {
      return await this.host.balanceReveal(equation);
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
