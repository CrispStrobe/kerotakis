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
  alignSay,
  benchOccupied,
  canUseFreshVessels,
  freshVesselScript,
  highestVesselNumber,
  loadRunMode,
  runCatalogEntry,
  runGate,
  runnableLines,
  scriptVesselNumbers,
  type BenchVesselLike,
  type RunnerBench,
  type RunnerFeedLine,
  type StepReport,
  type StepVerdict,
  type RunStep,
} from "./catalogRunner";
import { catalogEntries } from "./catalogEntry";

const ENTRY = {
  id: "neutralisation",
  progress: "starter" as const,
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
  /** What a real bench writes down: the command, then what it produced. */
  feed: RunnerFeedLine[] = [];

  async submit(line: string): Promise<boolean> {
    this.submitted.push(line);
    this.feed.push({ kind: "command", text: line });
    if (line === this.refuse) return false;
    this.feed.push({ kind: "line", text: `did ${line}` });
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

/**
 * Step mode: the same walk, paced by the learner instead of by a timer.
 *
 * "Run it" and "step through it" have to be the SAME run — same door, same
 * checks, same recording — or the slow one becomes a second, less honest
 * implementation of the experiment. So these assert the difference is only
 * WHO says "next", and that stopping half way is not a completion.
 */
describe("walking a script one step at a time", () => {
  /** A gate that answers from a queue, and remembers what it was shown. */
  function gate(answers: StepVerdict[]) {
    const seen: StepReport[] = [];
    const submittedAt: number[] = [];
    const bench = new FakeBench();
    const onstepdone = async (report: StepReport): Promise<StepVerdict> => {
      seen.push(report);
      submittedAt.push(bench.submitted.length);
      return answers[seen.length - 1] ?? "next";
    };
    return { bench, seen, submittedAt, onstepdone };
  }

  it("runs exactly one line per answer, and nothing while it waits", async () => {
    const { bench, seen, submittedAt, onstepdone } = gate(["next", "next"]);
    await runCatalogEntry(bench, ENTRY, { pause: instantly, onstepdone });

    // Gated twice for a three-line script: the last line needs no consent,
    // because there is nothing after it to consent to.
    expect(seen.map((s) => s.index)).toEqual([0, 1]);
    // At each gate exactly index+1 lines had gone to the bench — one line
    // per "next", never two.
    expect(submittedAt).toEqual([1, 2]);
    expect(bench.submitted).toEqual([
      "add v1 water 1000mL",
      "add v1 HCl 0.01mol",
      "add v1 NaOH 0.01mol",
    ]);
  });

  it("shows the learner what the line produced, not the echo of the line", async () => {
    const { seen, bench, onstepdone } = gate(["next", "next"]);
    await runCatalogEntry(bench, ENTRY, { pause: instantly, onstepdone });
    expect(seen[0]?.produced).toEqual([{ kind: "line", text: "did add v1 water 1000mL" }]);
    // The command echo is the line the strip already names; repeating it
    // under "what happened" is noise, not evidence.
    expect(seen.flatMap((s) => s.produced).some((line) => line.kind === "command")).toBe(false);
    expect(seen[1]?.produced).toEqual([{ kind: "line", text: "did add v1 HCl 0.01mol" }]);
    expect(seen.map((s) => s.more)).toEqual([true, true]);
  });

  it("finishes the rest automatically when the learner stops asking", async () => {
    const { bench, seen, onstepdone } = gate(["rest"]);
    const pause = vi.fn(async () => {});
    const outcome = await runCatalogEntry(bench, { ...ENTRY, id: "rest" }, { pause, onstepdone });

    // One gate, then the runner carried on by itself to the end.
    expect(seen).toHaveLength(1);
    expect(bench.submitted).toHaveLength(3);
    expect(outcome.halted).toBe(false);
    // The handover step is not paced — the learner already watched it — so
    // only the gap between the last two lines is paced.
    expect(pause).toHaveBeenCalledTimes(1);
  });

  it("records progress for a stepped run exactly as for an automatic one", async () => {
    const { bench, onstepdone } = gate(["next", "next"]);
    bench.events = ["neutralised"];
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly, onstepdone });
    expect(outcome.result.allOk).toBe(true);
    expect(outcome.recorded).toBe(true);
    expect(bench.marked).toEqual(["neutralisation"]);
  });

  it("records nothing when the learner stops half way, even if the checks pass", async () => {
    // The trap this closes: the expectations are checked against the bench
    // whatever happened, and a bench can satisfy them after one line. That
    // is a green CHECK, not a finished experiment, and crediting it would
    // hand out the entry for work that was abandoned.
    const { bench, onstepdone } = gate(["stop"]);
    bench.events = ["neutralised"];
    const outcome = await runCatalogEntry(bench, ENTRY, { pause: instantly, onstepdone });

    expect(bench.submitted).toEqual(["add v1 water 1000mL"]);
    expect(outcome.halted).toBe(true);
    expect(outcome.result.allOk).toBe(true);
    expect(outcome.recorded).toBe(false);
    expect(bench.marked).toEqual([]);
  });

  /**
   * The prose that paces the walk.
   *
   * A sentence per step is the one thing the strip could not say: the feed
   * reports what a line DID, and the learner pacing the run also needs to
   * know what to WATCH for. The risk it introduces is worse than absence,
   * which is why these tests are about alignment and not about presence —
   * a sentence attached to the wrong line narrates the fizz while the
   * water is still being measured, and reads as confident and true.
   */
  const SAY = ["fill the beaker", "the acid goes in", "and the base neutralises it"];

  it("announces each line with its own sentence, in order", async () => {
    const { bench, seen, onstepdone } = gate(["next", "next"]);
    const announced: (string | null)[] = [];
    await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      onstepdone,
      say: SAY,
      onstep: (step) => announced.push(step.say),
    });
    expect(announced).toEqual(SAY);
    // The report a learner reads carries the same sentence the step was
    // announced with: one step, one claim, whichever half shows it.
    expect(seen.map((s) => s.say)).toEqual(["fill the beaker", "the acid goes in"]);
  });

  it("degrades to exactly today's run when an entry ships no prose", async () => {
    const { bench, seen, onstepdone } = gate(["next", "next"]);
    const announced: (string | null)[] = [];
    await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      onstepdone,
      onstep: (step) => announced.push(step.say),
    });
    expect(announced).toEqual([null, null, null]);
    expect(seen.map((s) => s.say)).toEqual([null, null]);
    expect(bench.submitted).toHaveLength(3);
  });

  it("ignores prose that does not match the script rather than shifting it", async () => {
    const { bench, onstepdone } = gate(["next", "next"]);
    const announced: (string | null)[] = [];
    await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      onstepdone,
      say: ["only one sentence"],
      onstep: (step) => announced.push(step.say),
    });
    expect(announced).toEqual([null, null, null]);
    expect(bench.submitted).toHaveLength(3);
  });

  it("keeps every sentence on its own line when the run moves to fresh glassware", async () => {
    // "Fresh vessels" prepends one `new` per vessel the script names. The
    // authored prose knows nothing about that prelude, so without the
    // offset every sentence would land one step late — the defect that
    // makes wrong prose worse than none.
    const bench = new FakeBench();
    bench.scene = { vessels: [{ id: 0, liquid: {} }] };
    const announced: { line: string; say: string | null }[] = [];
    await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      decision: "fresh",
      say: SAY,
      onstep: (step) => announced.push({ line: step.line, say: step.say }),
    });
    expect(announced[0]).toEqual({ line: "new", say: null });
    expect(announced.slice(1).map((s) => s.say)).toEqual(SAY);
    expect(announced.slice(1).map((s) => s.line)).toEqual([
      "add v2 water 1000mL",
      "add v2 HCl 0.01mol",
      "add v2 NaOH 0.01mol",
    ]);
  });

  it("credits nothing to an automatic run the learner stopped either", async () => {
    const bench = new FakeBench();
    bench.events = ["neutralised"];
    let stop = false;
    const outcome = await runCatalogEntry(bench, ENTRY, {
      pause: instantly,
      onstep: () => (stop = true),
      stopped: () => stop,
    });
    expect(outcome.halted).toBe(true);
    expect(outcome.recorded).toBe(false);
  });

  it("defaults to the automatic pace where no choice is stored", () => {
    // Node has no `window`; a private-mode browser has one whose storage
    // throws. Both are "no preference", which is not an error.
    expect(loadRunMode()).toBe("auto");
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

/**
 * One runner, reached the same way from either corpus.
 *
 * The guided half of the catalogue used to offer a different meaning of
 * "run this" — a card that launched a lesson where the other tier launched
 * the bench. Now the view model resolves the door, and the point of this
 * test is that what comes out of that resolution is the SAME script object
 * the codex card would have run, walked by the same function, recorded
 * against the same id. If those ever diverge, two runners are back.
 */
describe("both corpora reach the runner through one door", () => {
  const script = {
    id: "vinegar-and-baking-soda",
    progress: "starter" as const,
    setup: { script: "add v1 white_vinegar_5_percent 50mL\nadd v1 baking_soda 5g\n" },
    expect: { events: ["gas_evolved"] },
    registers: {},
  };
  const guided = {
    id: "K01", title: "Volcano", phenomenon: "Soap traps gas",
    title_de: "Vulkan", phenomenon_de: "Seife fängt Gas",
    status: "computed" as const, progress: "starter" as const, topics: ["gases"],
    ingredients: ["baking_soda"], apparatus: ["beaker"],
    safety: "home" as const, codex: ["vinegar-and-baking-soda"],
  };

  it("runs the guided card's own script, and records the codex id", async () => {
    const [card] = catalogEntries([script], [guided], {
      locale: "en", translate: (value) => value, completed: new Set(),
    }).filter((entry) => entry.source === "guided");
    expect(card?.run.kind).toBe("script");
    const target = card?.run.kind === "script" ? card.run.entry : null;
    expect(target).toBe(script);

    const bench = new FakeBench();
    bench.events = ["gas_evolved"];
    const outcome = await runCatalogEntry(bench, target!, { pause: async () => {} });
    expect(bench.submitted).toEqual(runnableLines(script.setup.script));
    expect(outcome.result.allOk).toBe(true);
    // Progress is the codex id, not the guided task's — one record.
    expect(bench.marked).toEqual(["vinegar-and-baking-soda"]);
  });
});

/**
 * The alignment rule on its own.
 *
 * `alignSay` is what stands between "one sentence per step" and "a
 * sentence about the previous step, stated with confidence". It is
 * exported so the rule can be pinned here rather than inferred from a
 * whole run.
 */
describe("aligning authored prose to the lines that will actually run", () => {
  const script = "# a note\nadd v1 water 100mL\n\nadd v1 HCl 1mmol\n";

  it("is silence for every line when nothing was authored", () => {
    expect(alignSay(undefined, script, script)).toEqual([null, null]);
  });

  it("pairs each sentence with its own line when the script is unchanged", () => {
    expect(alignSay(["a", "b"], script, script)).toEqual(["a", "b"]);
  });

  it("refuses a mismatched array outright, rather than covering the lines it can", () => {
    expect(alignSay(["a"], script, script)).toEqual([null, null]);
    expect(alignSay(["a", "b", "c"], script, script)).toEqual([null, null]);
  });

  it("narrates a prepended prelude as silence and keeps the body aligned", () => {
    const decided = freshVesselScript(script, 3);
    expect(alignSay(["a", "b"], script, decided)).toEqual([null, "a", "b"]);
  });
});
