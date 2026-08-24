import { describe, expect, it } from "vitest";
import type { EngineHost, Scene } from "./host/EngineHost";
import { Session, type StorageLike } from "./session.svelte";

class FakeStorage implements StorageLike {
  map = new Map<string, string>();
  getItem(k: string) {
    return this.map.get(k) ?? null;
  }
  setItem(k: string, v: string) {
    this.map.set(k, v);
  }
  removeItem(k: string) {
    this.map.delete(k);
  }
}

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

  it("undo/redo/scrub are one cursor over a replayed log", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.submit("add v1 water 100mL");
    await s.submit("add v1 NaCl 1g");
    host.calls.length = 0;

    await s.undo();
    expect(host.calls).toEqual(["reset", "run:add v1 water 100mL"]);
    // The log survives; only the cursor moved.
    expect(s.commandLog).toHaveLength(2);
    expect(s.position).toBe(1);

    await s.redo();
    expect(s.position).toBe(2);
    expect(host.calls.slice(2)).toEqual(["reset", "run:add v1 water 100mL\nadd v1 NaCl 1g"]);

    await s.jumpTo(0);
    expect(s.position).toBe(0);
    // Empty prefix: reset then a plain scene fetch, no replay.
    expect(host.calls.slice(4)).toEqual(["reset", "scene"]);

    await s.jumpTo(0); // no-op at the same position
    expect(host.calls).toHaveLength(6);
  });

  it("a new command mid-history truncates the undone future", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.submit("add v1 water 100mL");
    await s.submit("add v1 NaCl 1g");
    await s.undo();
    await s.submit("add v1 KMnO4 1pinch");
    expect(s.commandLog).toEqual(["add v1 water 100mL", "add v1 KMnO4 1pinch"]);
    expect(s.position).toBe(2);
  });

  it("walks a lesson: narration to the feed, commands one Next at a time", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    s.startLesson("salt", "# Salt in water\nadd v1 water 100mL\n# Now the salt\nadd v1 NaCl 1g\n");
    // Leading narration surfaced immediately, cursor sits on the command.
    expect(s.feed.some((f) => f.text === "Salt in water")).toBe(true);
    expect(s.lessonNextCommand).toBe("add v1 water 100mL");

    await s.lessonNext();
    expect(s.commandLog).toEqual(["add v1 water 100mL"]);
    expect(s.feed.some((f) => f.text === "Now the salt")).toBe(true);
    expect(s.lessonNextCommand).toBe("add v1 NaCl 1g");

    // Deviation: a free command does not move the lesson cursor.
    await s.submit("measure v1 ph");
    expect(s.lessonNextCommand).toBe("add v1 NaCl 1g");

    await s.lessonNext();
    // Lesson exhausted: closed, with a finishing note.
    expect(s.lesson).toBeNull();
    expect(s.feed.at(-1)!.text).toContain("lesson finished");
  });

  it("hazard events become cards; a veto reads as a refusal", async () => {
    const host = new FakeHost();
    host.runScript = async (script: string) => ({
      steps: [
        {
          operator: {},
          events: [
            {
              event: "hazard_warning",
              severity: "danger",
              hazard: "chloramine gas",
              real_world: "bleach and ammonia make a gas that hurts to breathe",
            },
            { event: "safety_veto", reason: "this the bench will not do" },
          ],
          rendered: [`did: ${script}`],
        },
      ],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("add v1 NaOCl 10mL");
    const hazard = s.feed.find((f) => f.kind === "hazard");
    expect(hazard?.severity).toBe("danger");
    expect(hazard?.text).toContain("chloramine");
    expect(s.feed.some((f) => f.kind === "refusal" && f.text.includes("will not do"))).toBe(
      true,
    );
  });

  it("a step carrying charts renders them into the feed", async () => {
    const host = new FakeHost();
    const base = host.runScript.bind(host);
    host.runScript = async (script: string) => {
      const result = await base(script);
      (result.steps[0] as Record<string, unknown>).charts = [
        {
          title: "titration",
          x: { label: "volume", unit: "mL" },
          y: { label: "pH" },
          series: [{ kind: "line", name: "pH", points: [[0, 1]] }],
          provenance: "PHREEQC",
        },
      ];
      return result;
    };
    const s = new Session(host);
    await s.submit("titrate v1 NaOH 0.1M");
    const chart = s.feed.find((f) => f.kind === "chart");
    expect(chart?.chart?.title).toBe("titration");
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

  it("autosaves, and a reloaded session replays back to the same bench", async () => {
    const storage = new FakeStorage();
    const host = new FakeHost();
    const s1 = new Session(host, storage);
    await s1.submit("add v1 water 100mL");
    await s1.submit("register lv2");
    await s1.submit("add v1 NaCl 1g");
    await s1.undo();

    // "Reload": a fresh session over the same storage.
    const host2 = new FakeHost();
    const s2 = new Session(host2, storage);
    await s2.connect();
    expect(s2.commandLog).toEqual(["add v1 water 100mL", "add v1 NaCl 1g"]);
    expect(s2.position).toBe(1);
    expect(s2.register).toBe("lv2");
    // Restored by replaying exactly the applied prefix.
    expect(host2.calls).toContain("register:lv2");
    expect(host2.calls).toContain("run:add v1 water 100mL");
    expect(s2.feed.some((f) => f.text.includes("restored"))).toBe(true);
  });

  it("a corrupt save is dropped, never wedging the bench", async () => {
    const storage = new FakeStorage();
    storage.setItem("kero.session.v1", "{not json");
    const s = new Session(new FakeHost(), storage);
    await s.connect();
    expect(s.commandLog).toEqual([]);
    expect(storage.getItem("kero.session.v1")).toBeNull();
  });

  it("clear empties the bench and the save; jumpTo(0) keeps the future", async () => {
    const storage = new FakeStorage();
    const host = new FakeHost();
    const s = new Session(host, storage);
    await s.submit("add v1 water 100mL");
    await s.jumpTo(0);
    expect(s.commandLog).toHaveLength(1); // redo still possible
    await s.clear();
    expect(s.commandLog).toEqual([]);
    expect(storage.getItem("kero.session.v1")).toBeNull();
    expect(host.calls.at(-2)).toBe("reset");
  });

  it("importLab composes onto the bench, skips comments, stops at a bad line", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.importLab("demo.lab", "# a demo\nadd v1 water 100mL\n\nadd v1 NaCl 1g\n");
    expect(s.commandLog).toEqual(["add v1 water 100mL", "add v1 NaCl 1g"]);
    expect(s.feed.at(-1)!.text).toContain("demo.lab finished");

    // A rejected line stops the walk and names the line number.
    host.runScript = async () => {
      throw new Error("no");
    };
    await s.importLab("bad.lab", "# comment\nboom v9\nadd v1 water 1mL\n");
    expect(s.commandLog).toHaveLength(2);
    expect(s.feed.at(-1)!.text).toContain("bad.lab:2");
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
