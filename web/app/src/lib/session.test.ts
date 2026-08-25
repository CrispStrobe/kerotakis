import { describe, expect, it } from "vitest";
import type { EngineHost, Scene, ScriptResult } from "./host/EngineHost";
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
  async grammar(): Promise<{ verb: string; example: string; options?: string[] }[]> {
    this.calls.push("grammar");
    return [];
  }
  async relations() {
    return [];
  }
  /** Set to true to model an engine predating snapshots. */
  noSnapshots = false;
  async loadPack() {
    return { added: 0, skipped: 0, loaded_total: 0 };
  }
  async snapshot(): Promise<string> {
    if (this.noSnapshots) throw new Error("no snapshot support");
    this.calls.push("snapshot");
    return `snap@${this.calls.filter((c) => c === "snapshot").length}`;
  }
  async restore(snapshot: string): Promise<void> {
    this.calls.push(`restore:${snapshot}`);
  }
  async calc() {
    return { ok: false as const, error: "not in the fake" };
  }
  async parse(line: string) {
    this.calls.push(`parse:${line}`);
    return line.startsWith("boom")
      ? { ok: false, error: "no such verb" }
      : { ok: true };
  }
  async runScript(script: string): Promise<ScriptResult> {
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

  it("undo/redo/scrub restore snapshots in O(1), with replay as fallback", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    await s.submit("add v1 water 100mL");
    await s.submit("add v1 NaCl 1g");
    host.calls.length = 0;

    // Each submit snapshotted its position, so the cursor RESTORES.
    await s.undo();
    expect(host.calls).toEqual(["restore:snap@1", "scene"]);
    // The log survives; only the cursor moved.
    expect(s.commandLog).toHaveLength(2);
    expect(s.position).toBe(1);

    await s.redo();
    expect(s.position).toBe(2);
    expect(host.calls.slice(2)).toEqual(["restore:snap@2", "scene"]);

    await s.jumpTo(0);
    expect(s.position).toBe(0);
    // No snapshot at position 0: reset then a plain scene fetch.
    expect(host.calls.slice(4)).toEqual(["reset", "scene"]);

    await s.jumpTo(0); // no-op at the same position
    expect(host.calls).toHaveLength(6);
  });

  it("an engine without snapshots keeps the replay path", async () => {
    const host = new FakeHost();
    host.noSnapshots = true;
    const s = new Session(host);
    await s.submit("add v1 water 100mL");
    await s.submit("add v1 NaCl 1g");
    host.calls.length = 0;

    await s.undo();
    expect(host.calls).toEqual(["reset", "run:add v1 water 100mL"]);
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

  it("a lesson's kit is exactly the species its own commands use", () => {
    const s = new Session(new FakeHost());
    s.startLesson(
      "titration",
      "# neutralise it\nadd v1 water 100mL\nadd v1 HCl 0.1mol\ntitrate v1 NaOH 1M 1mL until ph 7\n",
    );
    expect([...s.lesson!.kit].sort()).toEqual(["HCl", "NaOH", "water"]);
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

  it("lesson deviation counts free commands; return rewinds them", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    s.startLesson("salt", "# Salt in water\nadd v1 water 100mL\nadd v1 NaCl 1g\n");
    await s.lessonNext();
    expect(s.lessonDeviation).toBe(0);

    await s.submit("measure v1 ph");
    await s.submit("wait 30s");
    expect(s.lessonDeviation).toBe(2);

    await s.lessonReturn();
    expect(s.lessonDeviation).toBe(0);
    // The bench rewound; the lesson cursor did not move.
    expect(s.position).toBe(1);
    expect(s.lessonNextCommand).toBe("add v1 NaCl 1g");
    // And the lesson continues cleanly from there.
    await s.lessonNext();
    expect(s.commandLog).toEqual(["add v1 water 100mL", "add v1 NaCl 1g"]);
  });

  it("typed events become transient vessel effects, and only typed events", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [
        {
          operator: {},
          events: [
            { event: "precipitated", vessel: 0, species: "AgCl", moles: 0.01 },
            { event: "electrolysed", vessel: 1, species: "Cu", coulombs: 900 },
            { event: "gas_evolved", vessel: 0, species: "CO2", moles: 0.002 },
            // titrated no longer maps to an instant effect — the GUI-064
            // playback paces its drips (see its own test).
            { event: "mixed", vessel: 1 },
            { event: "solution_characterized", vessel: 0, ph: 7 },
          ],
          rendered: ["It went cloudy!"],
        },
      ],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("add v1 AgNO3 1.7g");
    expect(s.vesselEffects[0]?.map((e) => e.kind)).toEqual(["precipitate", "vent"]);
    expect(s.vesselEffects[1]?.map((e) => e.kind)).toEqual(["electrolyse", "swirl"]);
  });

  it("a titrated event starts the paced playback (GUI-064)", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [
        {
          operator: {},
          events: [
            {
              event: "titrated",
              vessel: 0,
              titrant: "NaOH",
              concentration: 0.1,
              steps: 3,
              total_volume: 0.003,
              final_ph: 7.1,
              curve: [
                [0, 2.9],
                [1, 3.4],
                [2, 5.0],
                [3, 7.1],
              ],
            },
          ],
          rendered: [],
        },
      ],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("titrate v1 NaOH 0.1M 1mL until ph 7");
    expect(s.titrationPlayback).not.toBeNull();
    expect(s.titrationPlayback!.total).toBe(3);
    expect(s.titrationPlayback!.vessel).toBe(0);
  });

  it("the latest rendered equation is pinned for the strip", async () => {
    const host = new FakeHost();
    host.runScript = async (script: string) => ({
      steps: [
        {
          operator: {},
          events: [],
          rendered: [
            "The silver and the chloride find each other.",
            "Ag+ + Cl- → AgCl",
          ],
        },
      ],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("add v1 AgNO3 1.7g");
    expect(s.lastEquation).toBe("Ag+ + Cl- → AgCl");
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
    // v2 saves carry the engine snapshot: restored in ONE call, no replay.
    expect(host2.calls).toContain("register:lv2");
    expect(host2.calls.some((c) => c.startsWith("restore:snap@"))).toBe(true);
    expect(host2.calls.some((c) => c.startsWith("run:"))).toBe(false);
    expect(s2.feed.some((f) => f.text.includes("instantly"))).toBe(true);
  });

  it("a v1 save (no snapshot) still restores, by replay", async () => {
    const storage = new FakeStorage();
    storage.setItem(
      "kero.session.v1",
      JSON.stringify({ log: ["add v1 water 100mL"], position: 1, register: "lv1" }),
    );
    const host = new FakeHost();
    const s = new Session(host, storage);
    await s.connect();
    expect(host.calls).toContain("run:add v1 water 100mL");
    expect(s.position).toBe(1);
  });

  it("a stale snapshot token falls back to replay, not a broken bench", async () => {
    const storage = new FakeStorage();
    storage.setItem(
      "kero.session.v1",
      JSON.stringify({
        log: ["add v1 water 100mL"],
        position: 1,
        register: "lv1",
        snapshot: "snap@from-an-older-engine",
      }),
    );
    const host = new FakeHost();
    host.restore = async () => {
      throw new Error("the snapshot did not parse");
    };
    const s = new Session(host, storage);
    await s.connect();
    expect(host.calls).toContain("run:add v1 water 100mL");
    expect(s.position).toBe(1);
    expect(s.feed.some((f) => f.text.includes("replayed"))).toBe(true);
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

  it("parse validates without executing; register lines are session grammar", async () => {
    const host = new FakeHost();
    const s = new Session(host);
    expect(await s.parse("boom v9")).toEqual({ ok: false, error: "no such verb" });
    expect(await s.parse("register lv3")).toEqual({ ok: true });
    expect(await s.parse("   ")).toEqual({ ok: true });
    // Only the real command reached the engine.
    expect(host.calls).toEqual(["parse:boom v9"]);
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
