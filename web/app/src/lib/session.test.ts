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
  async setLocale(code: string) {
    // Recorded, so a test can assert the session tells the ENGINE which
    // language to render in — separately from the interface's own locale,
    // because the engine composes prose the shell never sees.
    this.calls.push(`locale:${code}`);
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
  it("persists learner-authored journal notes independently of chemistry commands", async () => {
    const storage = new FakeStorage();
    const first = new Session(new FakeHost(), storage);
    first.addUserNote("  A slow white precipitate.  ");

    const second = new Session(new FakeHost(), storage);
    await second.connect();
    expect(second.feed).toEqual(expect.arrayContaining([
      expect.objectContaining({ kind: "user-note", text: "A slow white precipitate." }),
    ]));
  });

  it("edits and removes learner notes without touching chemistry history", async () => {
    const storage = new FakeStorage();
    const session = new Session(new FakeHost(), storage);
    session.addUserNote("first wording");
    const createdAt = session.feed.find((entry) => entry.kind === "user-note")?.createdAt;
    expect(createdAt).toBeTruthy();

    session.editUserNote(createdAt!, "better wording");
    expect(session.feed).toContainEqual(expect.objectContaining({ kind: "user-note", text: "better wording", createdAt }));
    expect(session.commandLog).toEqual([]);

    session.removeUserNote(createdAt!);
    expect(session.feed.some((entry) => entry.kind === "user-note")).toBe(false);
    const restored = new Session(new FakeHost(), storage);
    await restored.connect();
    expect(restored.feed.some((entry) => entry.kind === "user-note")).toBe(false);
  });

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

  it("consumes finite Story stock transactionally while mission supplies prevent deadlocks", async () => {
    const host = new FakeHost();
    const storage = new FakeStorage();
    const s = new Session(host, storage, "story");
    await s.connect();
    for (let i = 0; i < 10; i += 1) expect(await s.submit("add v1 NaCl 1g")).toBe(true);
    expect(s.storyStockUsed.NaCl).toBe(10);
    expect(await s.submit("add v1 NaCl 1g")).toBe(false);
    expect(s.feed.at(-1)?.kind).toBe("refusal");

    // Bench undo is not a magical bottle refill, but an accepted mission
    // supplies its own kit and a first discovery replenishes the cabinet.
    await s.undo();
    expect(await s.submit("add v1 NaCl 1g")).toBe(false);
    s.startLesson("resupply", "add v1 NaCl 1g\nmeasure v1 balance");
    await s.lessonNext();
    expect(s.storyStockUsed.NaCl).toBe(10);
    await s.lessonNext();
    expect(s.completedMissions.has("resupply")).toBe(true);
    expect(s.storyStockUsed).toEqual({});
    expect(await s.submit("add v1 NaCl 1g")).toBe(true);

    const restored = new Session(new FakeHost(), storage, "story");
    expect(restored.storyStockUsed).toEqual({ NaCl: 1 });
  });

  it("does not consume Story stock when the engine rejects a dispense", async () => {
    const host = new FakeHost();
    const s = new Session(host, new FakeStorage(), "story");
    await s.connect();
    host.runScript = async () => { throw new Error("rejected by model"); };
    expect(await s.submit("add v1 NaCl 1g")).toBe(false);
    expect(s.storyStockUsed).toEqual({});
  });

  it("does not let typed commands bypass Story access gates", async () => {
    const host = new FakeHost();
    host.species = async () => [{ key: "HCl", name: "hydrochloric acid", formula: "HCl", phase: "liquid", hazards: ["corrosive"], hazard_assessed: true }];
    const s = new Session(host, new FakeStorage(), "story");
    await s.connect();
    expect(await s.submit("add v1 HCl 10mL")).toBe(false);
    expect(host.calls).not.toContain("run:add v1 HCl 10mL");
    s.startLesson("acid-kit", "add v1 HCl 10mL\nmeasure v1 ph");
    await s.lessonNext();
    expect(s.commandLog).toContain("add v1 HCl 10mL");
    expect(s.storyStockUsed).toEqual({});
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
    expect(s.completedMissions.has("salt")).toBe(true);
    expect(s.missionDebrief).toMatchObject({ id: "salt", firstCompletion: true, completedTotal: 1 });
    expect(s.missionDebrief?.evidence).toContain("did: add v1 NaCl 1g");
  });

  it("completes the chloride lead from typed solver evidence rather than its script", async () => {
    const host = new FakeHost();
    host.runScript = async (line) => ({
      steps: [{
        operator: {},
        events: line.includes("AgNO3")
          ? [{ event: "precipitated", vessel: 0, species: "AgCl", moles: 0.0099 }]
          : line.includes("AgCl")
            ? [{ event: "added", vessel: 0, species: "AgCl", moles: 0.01 }]
            : [],
        rendered: [`did: ${line}`],
      }],
      scene: { scene: 1, vessels: [] },
    });
    const s = new Session(host, new FakeStorage(), "story");
    s.startLesson("silver-and-salt", "# Find chloride\nadd v1 water 100mL\nadd v1 NaCl 0.01mol\nadd v1 AgNO3 0.01mol\n");

    expect(s.lessonNextCommand).toBeNull();
    expect(s.lesson?.kit).toEqual(expect.arrayContaining(["water", "NaCl", "AgNO3", "KCl"]));
    await s.submit("add v1 AgCl 0.01mol");
    expect(s.completedMissions.has("silver-and-salt")).toBe(false);
    await s.submit("add v1 KCl 0.01mol");
    expect(s.completedMissions.has("silver-and-salt")).toBe(false);
    await s.submit("add v1 AgNO3 0.01mol");

    expect(s.completedMissions.has("silver-and-salt")).toBe(true);
    expect(s.lesson).toBeNull();
    expect(s.missionOutcome).toBeNull();
    expect(s.missionDebrief).toMatchObject({ id: "silver-and-salt", firstCompletion: true });
  });

  it("persists mission completion but does not complete an exited lesson", async () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    };
    const first = new Session(new FakeHost(), storage);
    first.startLesson("left-early", "add v1 water 1mL");
    first.exitLesson();
    expect(first.completedMissions.size).toBe(0);

    first.startLesson("finished", "add v1 water 1mL");
    await first.lessonNext();
    const restored = new Session(new FakeHost(), storage);
    restored.restoreProgress();
    expect(restored.completedMissions.has("finished")).toBe(true);
    expect(restored.completedMissions.has("left-early")).toBe(false);
  });

  it("does not advance or award a mission when the engine rejects its step", async () => {
    const host = new FakeHost();
    host.runScript = async () => { throw new Error("rejected by model"); };
    const s = new Session(host);
    s.startLesson("must-work", "add v1 water 1mL");
    await s.lessonNext();
    expect(s.lessonNextCommand).toBe("add v1 water 1mL");
    expect(s.completedMissions.has("must-work")).toBe(false);
    expect(s.missionDebrief).toBeNull();
  });

  it("keeps only engine-backed mission evidence and dismisses the debrief independently", async () => {
    const s = new Session(new FakeHost());
    s.startLesson("evidence", "# narration\nadd v1 water 1mL\n");
    expect(s.lessonEvidence).toEqual([]);
    await s.lessonNext();
    expect(s.missionDebrief?.evidence).toEqual(["did: add v1 water 1mL"]);
    s.closeMissionDebrief();
    expect(s.missionDebrief).toBeNull();
    expect(s.commandLog).toEqual(["add v1 water 1mL"]);
  });

  it("restores mission progress even when the codex progress blob is corrupt", () => {
    const storage = new FakeStorage();
    storage.setItem("kero.codex.done.v1", "not-json");
    storage.setItem("kero.missions.done.v1", '["silver-and-salt"]');
    const s = new Session(new FakeHost(), storage);
    s.restoreProgress();
    expect(s.completedExperiments.size).toBe(0);
    expect(s.completedMissions.has("silver-and-salt")).toBe(true);
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

  it("colours a transfer from the computed pre-transfer source liquid", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{
        operator: {},
        events: [{ event: "transferred", from: 0, to: 1, fraction: 0.5 }],
        rendered: [],
      }],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = {
      scene: 1,
      vessels: [{
        id: 0,
        liquid: { volume_l: 0.1, srgb: [93, 42, 181], colour_word: "violet", cloudiness: 0, path_length_cm: 2 },
      } as Scene["vessels"][number]],
    };
    await s.submit("pour v1 into v2 50%");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      kind: "pour",
      fluidColour: "rgb(93 42 181)",
    });
  });

  it("retains the engine-scene solids and their colours on filter paper", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{ event: "filtered", from: 0, to: 1 }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = {
      scene: 1,
      vessels: [{
        id: 0,
        liquid: { volume_l: 0.08, srgb: [210, 232, 248], colour_word: "pale blue", cloudiness: .3, path_length_cm: 2 },
        solids: [{ species: "CuO", name: "copper(II) oxide", moles: .025, srgb: [35, 31, 28], colour_word: "black", metallic: false, settled_fraction: .8 }],
      } as Scene["vessels"][number]],
    };
    await s.submit("filter v1 into v2");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      operation: "filter",
      fluidColour: "rgb(210 232 248)",
      magnitude: .25,
      filterResidue: [{ species: "CuO", name: "copper(II) oxide", moles: .025, colour: "rgb(35 31 28)" }],
    });
  });

  it("drains the engine-selected lower layer with its own computed colour", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{ event: "drained", from: 0, to: 1, solvent: "water", moles: .2 }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = {
      scene: 1,
      vessels: [{
        id: 0,
        liquid: { volume_l: .1, srgb: [120, 100, 140], colour_word: "mixed", cloudiness: 0, path_length_cm: 2 },
        layers: [
          { species: "water", name: "water", volume_l: .06, srgb: [50, 110, 220], colour_word: "blue" },
          { species: "hexane", name: "hexane", volume_l: .04, srgb: [240, 210, 60], colour_word: "yellow" },
        ],
        solids: [],
      } as Scene["vessels"][number]],
    };
    await s.submit("drain v1 into v2");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      operation: "drain",
      fluidColour: "rgb(50 110 220)",
      drain: {
        solvent: "water",
        moles: .2,
        lowerColour: "rgb(50 110 220)",
        upperColour: "rgb(240 210 60)",
      },
    });
  });

  it("animates only engine-classified magnetic solids with scene amounts and colours", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{ event: "magnet_separated", from: 0, to: 1, attracted: ["Fe"], remained: ["S"] }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = {
      scene: 1,
      vessels: [{
        id: 0,
        liquid: null,
        solids: [
          { species: "Fe", name: "iron", moles: .04, srgb: [82, 86, 91], colour_word: "grey", metallic: true, settled_fraction: 1 },
          { species: "S", name: "sulfur", moles: .08, srgb: [240, 205, 40], colour_word: "yellow", metallic: false, settled_fraction: 1 },
        ],
      } as Scene["vessels"][number]],
    };
    await s.submit("magnet v1 v2");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      operation: "magnet",
      magnetic: {
        attractedSpecies: ["Fe"],
        remainedSpecies: ["S"],
        attracted: [{ species: "Fe", name: "iron", moles: .04, colour: "rgb(82 86 91)" }],
      },
    });
    expect(s.vesselEffects[0]?.[0]?.magnitude).toBeCloseTo(.4);
  });

  it("colours computed gravity-settling populations from the pre-wait scene", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{
        event: "gravity_settled",
        vessel: 0,
        seconds: 5,
        separations: [{ species: "SiO2", particle_diameter_um: 70, terminal_speed_m_s: .004, distance_m: .02, separated_fraction: .5, direction: "settles" }],
      }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = {
      scene: 1,
      vessels: [{
        id: 0,
        liquid: { volume_l: .1, srgb: [245, 245, 245], colour_word: "colourless", cloudiness: .5, path_length_cm: 2 },
        solids: [{ species: "SiO2", name: "silica", moles: .03, srgb: [226, 219, 194], colour_word: "sand", metallic: false, settled_fraction: .1 }],
      } as Scene["vessels"][number]],
    };
    await s.submit("wait 5s");
    expect(s.vesselEffects[0]?.[0]?.settling?.populations[0]).toMatchObject({
      species: "SiO2",
      colour: "rgb(226 219 194)",
      separatedFraction: .5,
    });
  });

  it("colours centrifuge separation results from the pre-run solid inventory", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{
        event: "centrifuged", vessel: 0, rpm: 3000, seconds: 10, rotor_radius_m: .08,
        rcf: 805, sample_mass_g: 2, counterbalance_g: 2, imbalance_g: 0,
        fluid_density_kg_m3: 998, dynamic_viscosity_pa_s: .001, state_coupled: false,
        separations: [{ species: "CuO", particle_diameter_um: 40, particle_size_assumed: false, particle_density_kg_m3: 6310, terminal_speed_m_s: .01, distance_m: .03, separated_fraction: .75, direction: "outward" }],
      }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = { scene: 1, vessels: [{
      id: 0, liquid: null,
      solids: [{ species: "CuO", name: "copper(II) oxide", moles: .02, srgb: [38, 32, 28], colour_word: "black", metallic: false, settled_fraction: .1 }],
    } as Scene["vessels"][number]] };
    await s.submit("centrifuge v1 3000rpm 10s 8cm 2g");
    expect(s.vesselEffects[0]?.[0]?.centrifuge?.populations[0]).toMatchObject({
      species: "CuO", colour: "rgb(38 32 28)", separatedFraction: .75,
    });
  });

  it("shows computed stirring resuspension using the pre-stir non-metal solids", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{
        event: "stirred", vessel: 0, rpm: 700, seconds: 8, bar_length_m: .025,
        tip_speed_m_s: .916, resuspended_fraction: .72, rate_coupled: false,
      }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    s.scene = { scene: 1, vessels: [{
      id: 0, liquid: null,
      solids: [
        { species: "SiO2", name: "silica", moles: .02, srgb: [220, 210, 185], colour_word: "sand", metallic: false, settled_fraction: .8 },
        { species: "Fe", name: "iron", moles: .01, srgb: [80, 84, 88], colour_word: "grey", metallic: true, settled_fraction: .9 },
      ],
    } as Scene["vessels"][number]] };
    await s.submit("stir v1 700rpm 8s");
    expect(s.vesselEffects[0]?.[0]?.stir).toMatchObject({
      rpm: 700, tipSpeedMS: .916, resuspendedFraction: .72, rateCoupled: false,
      solids: [{ species: "SiO2", name: "silica", colour: "rgb(220 210 185)" }],
    });
  });

  it("surfaces an engine-confirmed gas test as a physical vessel effect", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{
        event: "gas_tested", vessel: 0, test: "damp_litmus", positive: true,
        notes: "red litmus turns blue",
      }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("test v1 litmus");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      kind: "gas_test",
      gasTest: { test: "damp_litmus", positive: true, notes: "red litmus turns blue" },
    });
  });

  it("surfaces a safe waft result without presenting raw prose as the interaction", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{ operator: {}, events: [{
        event: "smelled", vessel: 0, notes: [["NH3", "sharp, pungent"]],
      }], rendered: [] }],
      scene: { scene: 2, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("smell v1");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      kind: "waft",
      waft: { notes: [{ species: "NH3", description: "sharp, pungent" }] },
    });
  });

  it("measured events surface as instrument effects (GUI-062)", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [
        {
          operator: {},
          events: [
            { event: "measured", vessel: 0, instrument: "thermometer", value: 25.0, unit: "°C" },
            { event: "measured", vessel: 1, instrument: "ph_meter", value: 4.2, unit: "pH" },
            { event: "measured", vessel: 0, instrument: "balance", value: 12.3, unit: "g" },
            { event: "measured", vessel: 1, instrument: "pressure_gauge", value: 152.4, unit: "kPa" },
            { event: "measured", vessel: 0, instrument: "volume_meter", value: 480, unit: "mL" },
            { event: "measured", vessel: 1, instrument: "conductivity_meter", value: 12840, unit: "µS/cm" },
            { event: "measured", vessel: 0, instrument: "spectrophotometer", value: 0.742, unit: "absorbance at 525 nm" },
            { event: "measured", vessel: 1, instrument: "calorimeter", value: -2.81, unit: "kJ" },
            { event: "measured", vessel: 0, instrument: "geiger_counter", value: 250000, unit: "Bq" },
          ],
          rendered: [],
        },
      ],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("measure v1 thermometer");
    expect(s.vesselEffects[0]?.map((e) => e.kind)).toEqual(["thermometer", "balance", "volume_meter", "uvvis", "geiger_counter"]);
    expect(s.vesselEffects[0]?.[0]).toMatchObject({ reading: 25, unit: "°C" });
    expect(s.vesselEffects[0]?.[1]).toMatchObject({ reading: 12.3, unit: "g" });
    expect(s.vesselEffects[0]?.[2]).toMatchObject({ reading: 480, unit: "mL" });
    expect(s.vesselEffects[0]?.[3]).toMatchObject({ reading: 0.742, unit: "absorbance at 525 nm" });
    expect(s.vesselEffects[0]?.[4]).toMatchObject({ reading: 250000, unit: "Bq" });
    expect(s.vesselEffects[0]?.[4]?.magnitude).toBeCloseTo(Math.log10(250000) / 8);
    expect(s.vesselEffects[1]?.map((e) => e.kind)).toEqual(["ph_probe", "pressure_gauge", "conductivity_meter", "calorimeter"]);
    expect(s.vesselEffects[1]?.[0]).toMatchObject({ reading: 4.2, unit: "pH" });
    expect(s.vesselEffects[1]?.[1]).toMatchObject({ reading: 152.4, unit: "kPa" });
    expect(s.vesselEffects[1]?.[2]).toMatchObject({ reading: 12840, unit: "µS/cm" });
    expect(s.vesselEffects[1]?.[3]).toMatchObject({ reading: -2.81, unit: "kJ" });
  });

  it("keeps computed chromatography peaks for physical playback", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{
        operator: {},
        events: [{
          event: "chromatographed",
          vessel: 0,
          plates: 1600,
          void_time_s: 42,
          peaks: [
            { species: "acetone", retention_time_s: 61, width_s: 6.1, relative_area: 0.4, partition_k: 0.2 },
            { species: "ethanol", retention_time_s: 118, width_s: 11.8, relative_area: 1, partition_k: 1.8 },
          ],
          outside_method: ["Na+"],
        }],
        rendered: [],
      }],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("chromatograph v1");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      kind: "chromatograph",
      voidTimeS: 42,
      plates: 1600,
      outsideMethod: ["Na+"],
      bands: [
        { species: "acetone", retentionTimeS: 61, widthS: 6.1, relativeArea: 0.4, partitionK: 0.2 },
        { species: "ethanol", retentionTimeS: 118, widthS: 11.8, relativeArea: 1, partitionK: 1.8 },
      ],
    });
  });

  it("keeps the computed appearance for magnified inspection", async () => {
    const host = new FakeHost();
    host.runScript = async () => ({
      steps: [{
        operator: {},
        events: [{
          event: "observed",
          vessel: 0,
          appearance: {
            liquid: { r: 122, g: 188, b: 241, strength: 0 },
            cloudiness: 0.42,
            deposit: ["AgCl", { r: 244, g: 244, b: 238, strength: 1 }],
            bubbling: true,
            words: "a cloudy blue liquid",
          },
        }],
        rendered: [],
      }],
      scene: { scene: 1, vessels: [] } as Scene,
    });
    const s = new Session(host);
    await s.submit("measure v1 eyes");
    expect(s.vesselEffects[0]?.[0]).toMatchObject({
      kind: "inspect",
      appearance: {
        liquidRgb: [122, 188, 241],
        cloudiness: 0.42,
        deposit: { species: "AgCl", rgb: [244, 244, 238] },
        bubbling: true,
      },
    });
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
