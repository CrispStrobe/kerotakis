/**
 * Session state: a projection of the engine, plus interaction state.
 *
 * The engine is the only holder of chemistry state (PROTOCOL.md non-goal
 * list); this class holds what the UI needs to paint — the latest scene,
 * the feed, the register — and routes every user intention through the
 * EngineHost as a command line. Every GUI gesture ends up here as the
 * operator text it compiled to, which is what makes the feed a lab
 * notebook, a screen-reader surface, and a scripting teacher at once.
 */

import type { EngineHost, Scene } from "./host/EngineHost";
import { EngineError } from "./host/EngineHost";

export type FeedEntry = {
  kind: "command" | "line" | "error" | "refusal" | "note";
  text: string;
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
        await this.setRegister(trimmed.slice("register ".length).trim());
        return;
      }
      const result = await this.host.runScript(trimmed);
      for (const step of result.steps) {
        for (const rendered of step.rendered) {
          this.feed.push({ kind: "line", text: rendered });
        }
      }
      if (result.scene) this.scene = result.scene;
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
    try {
      await this.host.setRegister(level);
      this.register = level;
      this.feed.push({ kind: "note", text: `register ${level}` });
    } catch (e) {
      this.feed.push({
        kind: "error",
        text: e instanceof Error ? e.message : String(e),
      });
    } finally {
      this.busy = false;
    }
  }
}
