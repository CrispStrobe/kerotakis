/**
 * The catalogue runner: what "run it on the bench" is allowed to mean.
 *
 * The bug this file exists to prevent is not a wrong answer — the checker
 * was always honest — but an invisible one: a script fired at the bench in
 * a single tick, behind a modal, so a learner saw a verdict and no
 * experiment. So the assertions here are about the WALK: one `submit` per
 * line, in order, each one announced before it goes, with a pause between
 * steps that the caller controls.
 */
import { describe, expect, it, vi } from "vitest";
import {
  benchOccupied,
  canUseFreshVessels,
  freshVesselScript,
  highestVesselNumber,
  runCatalogEntry,
  runGate,
  runnableLines,
  scriptVesselNumbers,
  type BenchVesselLike,
  type RunnerBench,
  type RunStep,
} from "./catalogRunner";

const ENTRY = {
  id: "neutralisation",
  setup: { script: "# a comment\nadd v1 water 1000mL\n\nadd v1 HCl 0.01mol\nadd v1 NaOH 0.01mol\n" },
  expect: { events: ["neutralised"], ph: { min: 6.9, max: 7.1 } },
};

/** A bench that records everything asked of it and plays back a scene. */
class FakeBench implements RunnerBench {
  submitted: string[] = [];
  cleared = 0;
  marked: string[] = [];
  capturing: string[] | null = null;
  events: string[] = [];
  ph: number[] = [7.0];
  refuse: string | null = null;
  scene: { vessels?: BenchVesselLike[] } | null = { vessels: [] };
  completedExperiments: ReadonlySet<string> = new Set<string>();

  async submit(line: string): Promise<boolean> {
    this.submitted.push(line);
    if (line === this.refuse) return false;
    this.capturing?.push(...this.events);
    return true;
  }
  async clear(): Promise<void> {
    this.cleared += 1;
    this.scene = { vessels: [] };
  }
  beginEventCapture(): void {
    this.capturing = [];
  }
  endEventCapture(): string[] {
    const seen = this.capturing ?? [];
    this.capturing = null;
    return seen;
  }
  finalStateForCheck() {
    return { phValues: this.ph, temperaturesC: [] };
  }
  markExperimentDone(id: string): void {
    this.marked.push(id);
    this.completedExperiments = new Set([...this.completedExperiments, id]);
  }
}

const instantly = async () => {};

describe("what the runner will submit", () => {
  it("skips comments and blank lines, keeping order", () => {
    expect(runnableLines(ENTRY.setup.script)).toEqual([
      "add v1 water 1000mL",
      "add v1 HCl 0.01mol",
      "add v1 NaOH 0.01mol",
    ]);
  });
});

describe("running an entry on the visible bench", () => {
  it("walks the script one submit at a time, announcing each line first", async () => {
    const bench = new FakeBench();
    bench.events = ["neutralised"];
    const announced: RunStep[] = [];
    const seenAtAnnounce: number[] = [];

    const outcome = await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      onstep: (step) => {
        announced.push(step);
        // The panel names the line it is ABOUT to run: at announce time
        // that line has not been submitted yet. This is what lets the dock
        // read "step 2 of 3: add v1 HCl" while the bench does it.
        seenAtAnnounce.push(bench.submitted.length);
      },
    });

    expect(bench.submitted).toEqual([
      "add v1 water 1000mL",
      "add v1 HCl 0.01mol",
      "add v1 NaOH 0.01mol",
    ]);
    expect(announced.map((s) => s.line)).toEqual(bench.submitted);
    expect(announced.map((s) => s.index)).toEqual([0, 1, 2]);
    expect(announced.every((s) => s.total === 3)).toBe(true);
    expect(seenAtAnnounce).toEqual([0, 1, 2]);
    expect(outcome.refusedAt).toBeNull();
  });

  it("pauses between steps but not after the last one", async () => {
    const bench = new FakeBench();
    const pause = vi.fn(async () => {});
    await runCatalogEntry(bench, ENTRY, { pause, paceMs: 42 });
    expect(pause).toHaveBeenCalledTimes(2);
    expect(pause).toHaveBeenCalledWith(42);
  });

  it("stops at a refused line and names it, leaving the rest unrun", async () => {
    const bench = new FakeBench();
    bench.refuse = "add v1 HCl 0.01mol";
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    expect(bench.submitted).toEqual(["add v1 water 1000mL", "add v1 HCl 0.01mol"]);
    expect(outcome.refusedAt).toBe(1);
    expect(outcome.ran.at(-1)).toBe("add v1 HCl 0.01mol");
  });

  it("obeys a learner who stops the run", async () => {
    const bench = new FakeBench();
    let stop = false;
    await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      onstep: () => (stop = true),
      stopped: () => stop,
    });
    expect(bench.submitted).toEqual(["add v1 water 1000mL"]);
  });
});

describe("expectations are checked against the scene that resulted", () => {
  it("agrees when the real run emitted the event and left the real pH", async () => {
    const bench = new FakeBench();
    bench.events = ["neutralised"];
    bench.ph = [7.0];
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    expect(outcome.observed).toContain("neutralised");
    expect(outcome.result.allOk).toBe(true);
    expect(outcome.result.ph?.value).toBe(7.0);
  });

  it("disagrees on the bench's own numbers, not on a recomputation", async () => {
    const bench = new FakeBench();
    bench.events = ["neutralised"];
    bench.ph = [2.4];
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    expect(outcome.result.allOk).toBe(false);
    expect(outcome.result.ph).toEqual({ range: { min: 6.9, max: 7.1 }, value: 2.4, ok: false });
  });
});

describe("progress is recorded once per completed entry", () => {
  it("records the first green run and nothing on the replay", async () => {
    const bench = new FakeBench();
    bench.events = ["neutralised"];
    const first = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    const second = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    expect(first.recorded).toBe(true);
    expect(second.recorded).toBe(false);
    expect(bench.marked).toEqual(["neutralisation"]);
  });

  it("records nothing when the chemistry disagreed", async () => {
    const bench = new FakeBench();
    bench.ph = [2.4];
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    expect(outcome.recorded).toBe(false);
    expect(bench.marked).toEqual([]);
  });

  it("records nothing when the script did not finish, even if the checks pass", async () => {
    const bench = new FakeBench();
    bench.events = ["neutralised"];
    bench.refuse = "add v1 NaOH 0.01mol";
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly });
    expect(outcome.result.allOk).toBe(true);
    expect(outcome.recorded).toBe(false);
    expect(bench.marked).toEqual([]);
  });
});

describe("a bench with work on it is asked about, never wiped", () => {
  const withWork = { vessels: [{ id: 0, liquid: { volume_ml: 50 }, solids: [] }] };
  const withSolids = { vessels: [{ id: 0, liquid: null, solids: [{ species: "NaCl" }] }] };
  const bare = { vessels: [{ id: 0, liquid: null, solids: [] }] };

  it("reads contents, not vessel count", () => {
    expect(benchOccupied(withWork)).toBe(true);
    expect(benchOccupied(withSolids)).toBe(true);
    expect(benchOccupied(bare)).toBe(false);
    expect(benchOccupied({ vessels: [] })).toBe(false);
    expect(benchOccupied(null)).toBe(false);
  });

  it("asks once, then honours whatever the learner answered", () => {
    expect(runGate(withWork, null)).toBe("ask");
    expect(runGate(withWork, "clear")).toBe("ready");
    expect(runGate(withWork, "fresh")).toBe("ready");
    expect(runGate(withWork, "keep")).toBe("ready");
    expect(runGate(bare, null)).toBe("ready");
  });

  it("clears only when the learner chose to", async () => {
    const bench = new FakeBench();
    bench.scene = withWork;
    await runCatalogEntry(bench, ENTRY, { pause: instantly, decision: "keep" });
    expect(bench.cleared).toBe(0);
    await runCatalogEntry(bench, ENTRY, { pause: instantly, decision: "clear" });
    expect(bench.cleared).toBe(1);
  });
});

describe("running beside the learner's work", () => {
  it("counts vessels the way the grammar spells them", () => {
    expect(highestVesselNumber({ vessels: [{ id: 0 }, { id: 1 }] })).toBe(2);
    expect(highestVesselNumber({ vessels: [] })).toBe(0);
    expect(scriptVesselNumbers("add v1 water 1L\ntransport v1 v2")).toEqual([1, 2]);
  });

  it("shifts the script onto glassware it asks for first", () => {
    expect(freshVesselScript("add v1 water 1L\ntransport v1 v2", 2)).toBe(
      "new\nnew\nadd v3 water 1L\ntransport v3 v4",
    );
    // Nothing to move past: an empty bench runs the script as written.
    expect(freshVesselScript("add v1 water 1L", 0)).toBe("add v1 water 1L");
  });

  it("declines the offer for a script that allocates its own glassware", () => {
    expect(canUseFreshVessels("add v1 water 1L")).toBe(true);
    expect(canUseFreshVessels("new flask\nadd v2 water 1L")).toBe(false);
    expect(canUseFreshVessels("# nothing here")).toBe(false);
  });

  it("submits the shifted script when the learner keeps their work", async () => {
    const bench = new FakeBench();
    bench.scene = { vessels: [{ id: 0, liquid: { volume_ml: 10 }, solids: [] }] };
    await runCatalogEntry(bench, ENTRY, { pause: instantly, decision: "fresh" });
    expect(bench.cleared).toBe(0);
    expect(bench.submitted).toEqual([
      "new",
      "add v2 water 1000mL",
      "add v2 HCl 0.01mol",
      "add v2 NaOH 0.01mol",
    ]);
  });
});
