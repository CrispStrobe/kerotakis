/**
 * A lesson must not be graded on the previous lesson's chemistry.
 *
 * The owner's transcript: after "Trocken, dann nass: Brausen" left citric
 * acid and baking soda in `v1`, the electrode lesson's own first liquid
 * step dissolved them and then reported that vessel's pH — 3.12 — and a
 * driving force of +0.62 V as its own findings. Nothing in the app looked
 * wrong; the numbers were simply about a different experiment.
 *
 * Every `.lab` in `lessons/` names its glassware absolutely and allocates
 * the rest with `new`, so all of them are written against the bench the
 * engine hands over: one empty vessel. These tests use the owner's exact
 * pair of real lesson files, because a synthetic script cannot show that
 * the CONTENT and the guard agree about what "fresh" means.
 *
 * The fake engine below is a bench, not a chemistry model: it tracks which
 * vessel exists and what went into it, which is all that "is this vessel
 * holding another lesson's reagents?" needs to be answerable.
 */
import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { EngineHost, Scene, SceneVessel, ScriptResult } from "./host/EngineHost";
import { Session } from "./session.svelte";
import { benchDiffersFromFresh, benchOccupied } from "./catalogRunner";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../../../..");
const lesson = (file: string) => readFileSync(join(ROOT, "lessons", file), "utf8");

/** One vessel, as much of it as "what is in this?" requires. */
interface FakeVessel {
  label: string;
  liquid: boolean;
  solids: string[];
}

function sceneVessel(vessel: FakeVessel, id: number): SceneVessel {
  return {
    id,
    label: vessel.label,
    liquid: vessel.liquid
      ? { volume_l: 0.1, srgb: [0, 0, 0], colour_word: "clear", cloudiness: 0, path_length_cm: 1 }
      : null,
    solids: vessel.solids.map((species) => ({
      species,
      name: species,
      moles: 0.01,
      srgb: [0, 0, 0],
      colour_word: "white",
      metallic: false,
      settled_fraction: 1,
    })),
    bubbling: false,
    boundary: "open",
    temperature_k: 298,
    pressure_pa: 101325,
    elapsed_s: 0,
    mass_g: vessel.liquid || vessel.solids.length > 0 ? 100 : 0,
    words: vessel.label,
    badges: [],
  };
}

/**
 * An engine that models the bench the lesson scripts address.
 *
 * `new` allocates the next vessel NUMBER — which is the whole reason a
 * leftover empty beaker is not harmless — and `add vN` records what landed
 * where. `reset` restores exactly `Bench::new()`: one empty beaker.
 */
class BenchHost implements EngineHost {
  vessels: FakeVessel[] = [{ label: "beaker", liquid: false, solids: [] }];
  private scenes = 0;

  private fresh() {
    this.vessels = [{ label: "beaker", liquid: false, solids: [] }];
  }

  private currentScene(): Scene {
    this.scenes += 1;
    return { scene: this.scenes, vessels: this.vessels.map(sceneVessel) };
  }

  /** What one vessel received, by the names the scripts use. */
  contents(number: number): string[] {
    const vessel = this.vessels[number - 1];
    if (!vessel) return [];
    return [...(vessel.liquid ? ["water"] : []), ...vessel.solids].sort();
  }

  private apply(line: string): boolean {
    const bare = line.trim();
    const isNew = /^new\b/i.exec(bare);
    if (isNew) {
      const label = bare.slice(3).trim() || "beaker";
      this.vessels.push({ label, liquid: false, solids: [] });
      return true;
    }
    const add = /^add\s+v(\d+)\s+(\S+)/i.exec(bare);
    if (add) {
      const vessel = this.vessels[Number(add[1]) - 1];
      if (!vessel) return false;
      // Anything with a volume behaves as the liquid for this purpose;
      // the lessons in play only ever pour water.
      if (/^water$/i.test(add[2]!)) vessel.liquid = true;
      else vessel.solids.push(add[2]!);
      return true;
    }
    // Every other verb the two lessons use observes rather than changes.
    return true;
  }

  async hello() {
    return { protocol: 1, can_solve: true };
  }
  async runScript(script: string): Promise<ScriptResult> {
    const lines = script.split("\n").map((line) => line.trim()).filter(Boolean);
    for (const line of lines) {
      if (!this.apply(line)) throw new Error(`no such vessel: ${line}`);
    }
    return {
      steps: lines.map((line) => ({ operator: {}, events: [], rendered: [`did: ${line}`] })),
      scene: this.currentScene(),
    };
  }
  async reset() {
    this.fresh();
  }
  async scene() {
    return this.currentScene();
  }
  async parse() {
    return { ok: true };
  }
  async step() {
    return { events: [], rendered: [] };
  }
  async grammar() {
    return [] as { verb: string; example: string; options?: string[] }[];
  }
  async relations() {
    return [];
  }
  async questStart() {}
  async questStop() {}
  async questAnswer(): Promise<import("./host/EngineHost").QuestAnswerResult> {
    return { outputs: [] };
  }
  async loadPack() {
    return { added: 0, skipped: 0, loaded_total: 0 };
  }
  async snapshot(): Promise<string> {
    throw new Error("no snapshot support in the bench fake");
  }
  async restore() {}
  async calc() {
    return { ok: false as const, error: "not in the fake" };
  }
  async balanceExercise() {
    return { ok: false as const, error: "not in the fake" };
  }
  async balanceMark() {
    return { ok: false as const, error: "not in the fake" };
  }
  async balanceReveal() {
    return { ok: false as const, error: "not in the fake" };
  }
  async setRegister() {}
  async setLocale() {}
  async state() {
    return {};
  }
  async species() {
    return [];
  }
  async elementCoverage() {
    return null;
  }
  async inspect() {
    return { vessel: 0, lines: [] } as unknown as Awaited<ReturnType<EngineHost["inspect"]>>;
  }
  async particles() {
    return null as unknown as Awaited<ReturnType<EngineHost["particles"]>>;
  }
  async catalog() {
    return null as unknown as Awaited<ReturnType<EngineHost["catalog"]>>;
  }
  dispose() {}
}

/** Walk every command a lesson owns, the way the mission panel does. */
async function playLesson(session: Session): Promise<void> {
  for (let guard = 0; guard < 200 && session.lessonNextCommand; guard += 1) {
    await session.lessonNext();
  }
}

/** Ask for a lesson, then take the gate's default answer if it appears. */
async function play(session: Session, file: string, answer: "clear" | "keep" | null = "clear") {
  session.requestLesson(file.replace(/\.lab$/, ""), lesson(file));
  if (session.lessonGate) await session.resolveLessonGate(answer);
  await playLesson(session);
}

describe("the bench a lesson starts on", () => {
  it("does not carry the previous lesson's reagents into the next one", async () => {
    const host = new BenchHost();
    const session = new Session(host);
    await session.connect();

    await play(session, "dry-then-wet-fizz.lab");
    // The sherbet lesson really did leave its reagents behind: without
    // that this test would pass on an engine that never ran anything.
    expect(host.contents(1)).toEqual(["baking_soda", "citric_acid"]);
    session.exitLesson();

    await play(session, "electrode.lab");

    // The electrode lesson puts water, copper sulfate and zinc into v1
    // and nothing else. Before the fix this read
    // ["CuSO4", "Zn", "baking_soda", "citric_acid", "water"], and the
    // pH and driving force it reported were that mixture's.
    expect(host.contents(1)).toEqual(["CuSO4", "Zn", "water"]);
    expect(host.contents(1)).not.toContain("citric_acid");
    expect(host.contents(1)).not.toContain("baking_soda");
  });

  it("asks before it empties anything, and starts at once on a fresh bench", async () => {
    const host = new BenchHost();
    const session = new Session(host);
    await session.connect();

    // A bench as the engine hands it over is never asked about.
    session.requestLesson("electrode", lesson("electrode.lab"));
    expect(session.lessonGate).toBeNull();
    expect(session.lesson?.lesson.name).toBe("electrode");
    await playLesson(session);
    session.exitLesson();

    // With work on it, the second lesson is held at the door instead.
    session.requestLesson("dry-then-wet-fizz", lesson("dry-then-wet-fizz.lab"));
    expect(session.lessonGate).not.toBeNull();
    expect(session.lessonGate?.occupied).toBe(true);
    expect(session.lesson).toBeNull();
    expect(host.contents(1)).toEqual(["CuSO4", "Zn", "water"]);
  });

  it("cancelling leaves the bench exactly as it was and starts nothing", async () => {
    const host = new BenchHost();
    const session = new Session(host);
    await session.connect();

    await play(session, "electrode.lab");
    session.exitLesson();
    const before = host.contents(1);
    const vesselsBefore = host.vessels.length;

    session.requestLesson("dry-then-wet-fizz", lesson("dry-then-wet-fizz.lab"));
    await session.resolveLessonGate(null);

    expect(session.lesson).toBeNull();
    expect(session.lessonGate).toBeNull();
    expect(host.contents(1)).toEqual(before);
    expect(host.vessels.length).toBe(vesselsBefore);
  });

  it("says in the feed what happened to the contents, either way", async () => {
    const cleared = new Session(new BenchHost());
    await cleared.connect();
    await play(cleared, "dry-then-wet-fizz.lab");
    cleared.exitLesson();
    cleared.requestLesson("electrode", lesson("electrode.lab"));
    const journalBefore = cleared.feed.length;
    await cleared.resolveLessonGate("clear");
    // `clear()` writes its own note, so the emptying is IN the record.
    // Nothing is allowed to leave the bench behind the learner's back.
    expect(cleared.feed.slice(0, journalBefore + 1).some(
      (entry) => entry.kind === "note" && /empty again/i.test(entry.text),
    )).toBe(true);

    const kept = new Session(new BenchHost());
    await kept.connect();
    await play(kept, "dry-then-wet-fizz.lab");
    kept.exitLesson();
    kept.requestLesson("electrode", lesson("electrode.lab"));
    await kept.resolveLessonGate("keep");
    // Keeping is allowed, and then the readings are not the lesson's
    // alone — which the feed has to say, or the numbers are unattributable.
    expect(kept.feed.some(
      (entry) => entry.kind === "note" && /was not empty/i.test(entry.text),
    )).toBe(true);
    expect(kept.lesson?.lesson.name).toBe("electrode");
  });

  it("treats a leftover empty vessel as a bench that is not fresh", async () => {
    // Unoccupied and still wrong: `new` in the next lesson would hand back
    // v3 where the script says v2, so the script's own second vessel is an
    // earlier run's glassware, of whatever type that happened to be.
    const spare = { vessels: [
      sceneVessel({ label: "beaker", liquid: false, solids: [] }, 0),
      sceneVessel({ label: "flask", liquid: false, solids: [] }, 1),
    ] };
    expect(benchOccupied(spare)).toBe(false);
    expect(benchDiffersFromFresh(spare)).toBe(true);

    const host = new BenchHost();
    host.vessels.push({ label: "flask", liquid: false, solids: [] });
    const session = new Session(host);
    await session.connect();
    session.requestLesson("electrode", lesson("electrode.lab"));
    expect(session.lessonGate).not.toBeNull();
    // The dialog says "more glassware than expected", not "still holds
    // material", because that is what is true here.
    expect(session.lessonGate?.occupied).toBe(false);
  });

  it("every lesson the app ships numbers its glassware from v1 with no gaps", () => {
    // The guard is only correct while the CONTENT agrees with it, and the
    // agreement is not "every lesson opens on v1" — `grouping-does-not-
    // change-water-heat.lab` deliberately keeps v1 empty until the end and
    // works in v2 onwards, and `repeated-liquid-extraction.lab` lets
    // `extract` allocate its own receiver. What every one of them does
    // assume is the NUMBERING: it counts from the one vessel the engine
    // hands over and leaves no gaps. A lesson naming v4 while it has only
    // ever spoken of v1 would be a lesson written to land on someone
    // else's glassware, and this precondition would be wrong for it.
    const listed = readdirSync(join(ROOT, "lessons")).filter((name) => name.endsWith(".lab"));
    expect(listed.length).toBeGreaterThan(100);
    for (const file of listed) {
      const named = new Set<number>();
      for (const raw of lesson(file).split("\n")) {
        const line = raw.trim();
        if (!line || line.startsWith("#")) continue;
        for (const match of line.matchAll(/\bv(\d+)\b/g)) named.add(Number(match[1]));
      }
      if (named.size === 0) continue; // narration-only; nothing to contaminate.
      const numbers = [...named].sort((a, b) => a - b);
      expect(numbers, `${file} does not start at v1`).toContain(1);
      expect(numbers, `${file} skips a vessel number`)
        .toEqual(numbers.map((_, index) => index + 1));
    }
  });
});
