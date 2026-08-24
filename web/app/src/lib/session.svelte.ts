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

import type { EngineHost, Scene } from "./host/EngineHost";
import { EngineError } from "./host/EngineHost";
import { type Lesson, parseLesson } from "./lesson";

export type FeedEntry = {
  kind: "command" | "line" | "error" | "refusal" | "note";
  text: string;
};

export type ShelfItem = {
  key: string;
  name: string;
  formula: string;
  phase: string;
};

export const REGISTERS = [
  { level: "lv1", label: "Look" },
  { level: "lv2", label: "Measure" },
  { level: "lv3", label: "Model" },
] as const;

export class Session {
  register = $state<string>("lv1");
  scene = $state<Scene | null>(null);
  feed = $state<FeedEntry[]>([]);
  busy = $state(false);
  engineReady = $state(false);
  canSolve = $state(false);
  /** Successful chemistry commands, in order — the session's .lab script. */
  commandLog = $state<string[]>([]);
  /**
   * How many of those commands are applied to the bench right now. Undo,
   * redo, and the timeline scrubber all move this one cursor; a new
   * command while the cursor sits mid-history truncates the future.
   */
  position = $state(0);
  /** The lesson being walked, if any. */
  lesson = $state<{ lesson: Lesson; cursor: number } | null>(null);
  /** The registry, for the shelf. */
  shelf = $state<ShelfItem[]>([]);
  /** Vessel the user last selected (0-based id), target of shelf adds. */
  selected = $state<number>(0);
  /** Open inspector content, if any. */
  inspector = $state<{ vessel: number; lines: string[] } | null>(null);

  constructor(private host: EngineHost) {}

  async connect(): Promise<void> {
    try {
      const hello = await this.host.hello();
      this.engineReady = true;
      this.canSolve = hello.can_solve ?? false;
      this.feed.push({
        kind: "note",
        text: this.canSolve
          ? "The bench is live: states nobody pre-computed are solved."
          : "The bench answers from shipped results only — the live aqueous engine is not attached.",
      });
      this.scene = await this.host.scene();
      const species = (await this.host.species()) as ShelfItem[];
      this.shelf = species.map((s) => ({
        key: s.key,
        name: s.name,
        formula: s.formula,
        phase: String(s.phase ?? ""),
      }));
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
    }
  }

  /** Run one command line — from the command bar or compiled from a gesture. */
  async submit(line: string): Promise<void> {
    const trimmed = line.trim();
    if (!trimmed || this.busy) return;
    this.busy = true;
    this.feed.push({ kind: "command", text: trimmed });
    try {
      if (trimmed.startsWith("register ")) {
        await this.applyRegister(trimmed.slice("register ".length).trim());
        return;
      }
      const result = await this.host.runScript(trimmed);
      for (const step of result.steps) {
        for (const rendered of step.rendered) {
          this.feed.push({ kind: "line", text: rendered });
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
      // The inspected vessel's detail is stale after any step.
      if (this.inspector) await this.inspect(this.inspector.vessel);
    } catch (e) {
      const refusal = e instanceof EngineError && e.kind === "refused";
      this.feed.push({
        kind: refusal ? "refusal" : "error",
        text: e instanceof Error ? e.message : String(e),
      });
    } finally {
      this.busy = false;
    }
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

  private async applyRegister(level: string): Promise<void> {
    try {
      await this.host.setRegister(level);
      this.register = level;
      this.feed.push({ kind: "note", text: `speaking at ${level}` });
      if (this.inspector) await this.inspect(this.inspector.vessel);
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
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
    this.lesson = { lesson: parseLesson(name, text), cursor: 0 };
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

  /** Append the submicroscopic view to the open inspector. */
  async particles(): Promise<void> {
    if (!this.inspector) return;
    try {
      const p = await this.host.particles(this.inspector.vessel);
      this.inspector = {
        vessel: this.inspector.vessel,
        lines: [...this.inspector.lines, "", ...p.rendered],
      };
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

  /** The session as a .lab script — every session is one. */
  exportLab(): string {
    return this.commandLog.join("\n") + "\n";
  }
}
