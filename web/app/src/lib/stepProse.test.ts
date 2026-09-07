/**
 * Per-step prose, walked over the content that actually ships.
 *
 * Two halves. The first pins the RULE — what parses, what a mismatched
 * length does, which language a reader gets — on fixtures. The second
 * walks the real export against the real catalogue, because the failure
 * this field can cause is not a crash: prose that is one line out of step
 * still renders, still reads as authoritative, and describes the wrong
 * moment of the experiment. Only a walk over the shipped rows can catch
 * that, and only against the same `runnableLines` the runner uses.
 */
import { describe, expect, it } from "vitest";
import { NO_STEP_PROSE, parseStepProse, sayForScript } from "./stepProse";
import { runnableLines, runCatalogEntry, type RunnerBench, type RunnerFeedLine } from "./catalogRunner";
import { catalogEntries } from "./catalogEntry";
import { parseCodexIndex, type CodexEntry } from "./codex";
import { type KidsExperiment } from "./kidsCatalog";
import codexExportJson from "../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";
import kidsCatalogJson from "../../../../data/kids/experiments-v1.json?raw";
import stepProseJson from "../../../../data/steps/step-prose-v1.json?raw";
import stepProseDeJson from "../../../../data/steps/step-prose-de-v1.json?raw";

/**
 * The two authored files, merged the way `tools/step-prose.py` merges them
 * for the payload.
 *
 * The merge is one line because the sidecar is positional by design, and
 * the generator's own shape is pinned by `tools/tests/test_step_prose.py`.
 * What this file is for is the other end: that the shape a build produces
 * is the shape the app can read, over the rows that actually ship.
 */
const english = JSON.parse(stepProseJson) as { scripts: Record<string, string[]> };
const german = JSON.parse(stepProseDeJson) as { scripts: Record<string, string[]> };
const shipped = {
  schema: 1,
  scripts: Object.fromEntries(Object.entries(english.scripts).map(([id, say]) => [
    id,
    { say, say_de: german.scripts[id] },
  ])),
};

const codex = parseCodexIndex(JSON.parse(codexExportJson));
// The authored source, not the export: German is merged in at build time,
// and the derivations this walk exercises never read it.
const guided = (JSON.parse(kidsCatalogJson) as { experiments: KidsExperiment[] }).experiments;

describe("what counts as prose for a step", () => {
  const row = { schema: 1, scripts: { entry: { say: ["one", "two"], say_de: ["eins", "zwei"] } } };

  it("keeps a row whose translation is the same length as its English", () => {
    const prose = parseStepProse(row);
    expect(prose.get("entry")?.say).toEqual(["one", "two"]);
    expect(prose.get("entry")?.de).toEqual(["eins", "zwei"]);
  });

  it("drops a translation that cannot be positional, keeping the English", () => {
    const prose = parseStepProse({ schema: 1, scripts: { entry: { say: ["one", "two"], say_de: ["eins"] } } });
    expect(prose.get("entry")?.say).toEqual(["one", "two"]);
    expect(prose.get("entry")?.de).toBeUndefined();
  });

  it("is empty for a payload built before the field existed", () => {
    expect(parseStepProse(null).size).toBe(0);
    expect(parseStepProse({ schema: 1 }).size).toBe(0);
    expect(parseStepProse({ schema: 2, scripts: row.scripts }).size).toBe(0);
  });

  it("reads a reader's own language, and falls back to English per entry", () => {
    const prose = parseStepProse(row);
    const script = "add v1 water 100mL\nadd v1 HCl 1mmol\n";
    expect(sayForScript(prose, "entry", script, "de")).toEqual(["eins", "zwei"]);
    expect(sayForScript(prose, "entry", script, "en")).toEqual(["one", "two"]);
    expect(sayForScript(prose, "entry", script, "fr")).toEqual(["one", "two"]);
  });

  it("says nothing at all when the script it claims to pace has changed", () => {
    const prose = parseStepProse(row);
    expect(sayForScript(prose, "entry", "add v1 water 100mL\n", "en")).toBeNull();
    expect(sayForScript(prose, "absent", "add v1 water 100mL\n", "en")).toBeNull();
    expect(sayForScript(NO_STEP_PROSE, "entry", "add v1 water 100mL\n", "en")).toBeNull();
  });
});

describe("the prose that ships", () => {
  const prose = parseStepProse(shipped);
  const byId = new Map(codex.map((entry) => [entry.id, entry]));

  it("names only entries the codex actually has", () => {
    expect(prose.size).toBeGreaterThan(0);
    expect([...prose.keys()].filter((id) => !byId.has(id))).toEqual([]);
  });

  it("has one sentence per runnable line of every script it paces", () => {
    const misaligned = [...prose.entries()]
      .map(([id, row]) => ({ id, prose: row.say.length, lines: runnableLines(byId.get(id)!.setup.script).length }))
      .filter((row) => row.prose !== row.lines);
    expect(misaligned).toEqual([]);
  });

  it("carries German for every English sentence, and never the same words", () => {
    for (const [id, row] of prose) {
      expect(row.de, `${id} has no German`).toHaveLength(row.say.length);
      expect(row.de).not.toEqual(row.say);
    }
  });

  it("survives the reader's language reaching it through the catalogue", () => {
    for (const locale of ["en", "de"]) {
      for (const [id, entry] of byId) {
        if (!prose.has(id)) continue;
        expect(sayForScript(prose, id, entry.setup.script, locale), `${id} in ${locale}`)
          .toHaveLength(runnableLines(entry.setup.script).length);
      }
    }
  });
});

/** A bench that accepts everything and writes a feed, as the real one does. */
class WalkBench implements RunnerBench {
  submitted: string[] = [];
  completedExperiments: ReadonlySet<string> = new Set<string>();
  scene = { vessels: [] };
  feed: RunnerFeedLine[] = [];
  async submit(line: string): Promise<boolean> {
    this.submitted.push(line);
    this.feed.push({ kind: "command", text: line });
    this.feed.push({ kind: "line", text: `did ${line}` });
    return true;
  }
  async clear(): Promise<void> {}
  beginEventCapture(): void {}
  endEventCapture(): string[] { return []; }
  finalStateForCheck() { return { phValues: [], temperaturesC: [] }; }
  markExperimentDone(): void {}
}

/**
 * Every guided experiment, end to end, through the same runner the panel
 * drives.
 *
 * The unified catalogue is the reason this walk exists at all: a guided
 * card whose run resolves to a codex script inherits that script's prose,
 * and the two halves of the library must not disagree about which line a
 * sentence belongs to. So the walk is over the guided sixty, resolved by
 * `catalogEntries`, run by `runCatalogEntry`, with the prose handed in
 * exactly as the panel hands it in.
 */
describe("all sixty guided experiments still run, prose or no prose", () => {
  const prose = parseStepProse(shipped);
  const entries = catalogEntries(codex, guided, {
    locale: "en",
    translate: (value: string) => value,
    completed: new Set<string>(),
  }).filter((entry) => entry.source === "guided");

  it("is the whole guided catalogue, and every card has an action", () => {
    expect(entries).toHaveLength(60);
    expect(entries.filter((entry) => entry.run.kind === "boundary" && entry.status === "computed")).toEqual([]);
  });

  it.each(entries.map((entry) => [entry.id, entry] as const))(
    "%s walks its script with the sentences it was given",
    async (_id, entry) => {
      if (entry.run.kind !== "script") return;
      const script: CodexEntry = entry.run.entry;
      const say = sayForScript(prose, script.id, script.setup.script, "de") ?? undefined;
      const bench = new WalkBench();
      const announced: (string | null)[] = [];
      const outcome = await runCatalogEntry(bench, script, {
        pause: async () => {},
        say,
        onstep: (step) => announced.push(step.say),
      });
      const lines = runnableLines(script.setup.script);
      expect(bench.submitted).toEqual(lines);
      expect(outcome.refusedAt).toBeNull();
      expect(announced).toHaveLength(lines.length);
      // Prose is optional; alignment is not. An entry with none announces
      // nothing, and an entry with some announces every line.
      expect(announced.every((line) => line === null) || announced.every((line) => line !== null)).toBe(true);
      if (say) expect(announced).toEqual(say);
    },
  );

  it("covers every guided experiment the bench can actually pace", () => {
    const runnable = entries.filter((entry) => entry.run.kind === "script");
    const withProse = runnable.filter((entry) =>
      entry.run.kind === "script" && prose.has(entry.run.entry.id));
    expect(withProse).toHaveLength(runnable.length);
  });
});

/** Both languages, in the two places the sentences reach a reader. */
describe("no learner is addressed by age or as a child", () => {
  const prose = parseStepProse(shipped);
  const forbidden = /\b(kind|kinder|kindern|kids?|child|children|alter|jahre|jahren|ages?|aged|years)\b/i;
  it.each([...prose.keys()])("%s says nothing about a reader", (id) => {
    const row = prose.get(id)!;
    for (const sentences of Object.values(row)) {
      for (const sentence of sentences) expect(sentence).not.toMatch(forbidden);
    }
  });
});
