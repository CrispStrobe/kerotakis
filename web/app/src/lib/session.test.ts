import { describe, expect, it } from "vitest";
import type { EngineHost, Scene } from "./host/EngineHost";
import { Session } from "./session.svelte";

/** An engine that records calls and plays a deterministic bench. */
class FakeHost implements EngineHost {
  calls: string[] = [];
  private sceneCounter = 0;

  private nextScene(): Scene {
    this.sceneCounter += 1;
    return { scene: 1, vessels: [] };
  }

  async hello() {
    this.calls.push("hello");
    return { protocol: 1, can_solve: true };
  }
  async step() {
    this.calls.push("step");
    return { events: [], rendered: [] };
  }
  async runScript(script: string) {
    this.calls.push(`run:${script}`);
    return {
      steps: script.split("\n").map((line) => ({
        operator: {},
        events: [],
        rendered: [`did: ${line}`],
      })),
      scene: this.nextScene(),
    };
  }
  async setRegister(level: string) {
    this.calls.push(`register:${level}`);
  }
  async scene() {
    this.calls.push("scene");
    return this.nextScene();
  }
  async state() {
    return {};
  }
  async species() {
    this.calls.push("species");
    return [{ key: "NaCl", name: "sodium chloride", formula: "NaCl", phase: "solid" }];
  }
  async inspect(vessel: number) {
    this.calls.push(`inspect:${vessel}`);
    return { rendered: [`vessel ${vessel} detail`] };
  }
  async particles(vessel: number) {
    this.calls.push(`particles:${vessel}`);
    return { rendered: ["oOo"] };
  }
  async reset() {
    this.calls.push("reset");
  }
  dispose() {}
}

describe("Session", () => {
  it("connects, loads the shelf, and reports the honesty state", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.connect();
    expect(s.engineReady).toBe(true);
    expect(s.canSolve).toBe(true);
    expect(s.shelf).toHaveLength(1);
    expect(s.shelf[0]!.key).toBe("NaCl");
  });

  it("logs successful chemistry commands, but not register switches", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.submit("add v1 water 100mL");
    await s.submit("register lv3");
    await s.submit("add v1 NaCl 1g");
    expect(s.commandLog).toEqual(["add v1 water 100mL", "add v1 NaCl 1g"]);
    expect(s.register).toBe("lv3");
    expect(s.exportLab()).toBe("add v1 water 100mL\nadd v1 NaCl 1g\n");
  });

  it("undo replays the prefix onto a fresh bench", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.submit("add v1 water 100mL");
    await s.submit("add v1 NaCl 1g");
    host.calls.length = 0;

    await s.undo();
    expect(host.calls).toEqual(["reset", "run:add v1 water 100mL"]);
    expect(s.commandLog).toEqual(["add v1 water 100mL"]);
    expect(s.feed.at(-1)).toEqual({ kind: "note", text: "undid: add v1 NaCl 1g" });

    await s.undo();
    expect(s.commandLog).toEqual([]);
    // Nothing left: reset then a plain scene fetch, no replay.
    expect(host.calls.slice(2)).toEqual(["reset", "scene"]);

    await s.undo(); // empty log is a no-op
    expect(host.calls).toHaveLength(4);
  });

  it("a failed command is not logged and cannot be undone into", async () => {
    const host = new FakeHost();
    host.runScript = async () => {
      throw new Error("no such species");
    };
    const s = new Session(host);
    await s.submit("add v1 unobtainium 1g");
    expect(s.commandLog).toEqual([]);
    expect(s.feed.at(-1)!.kind).toBe("error");
  });

  it("inspect opens register-dependent detail and refreshes after steps", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.inspect(0);
    expect(s.inspector).toEqual({ vessel: 0, lines: ["vessel 0 detail"] });
    host.calls.length = 0;
    await s.submit("add v1 NaCl 1g");
    expect(host.calls).toContain("inspect:0");
  });
});
