import { describe, expect, it } from "vitest";
import type { FeedEntry } from "./session.svelte";
import { collapseTick, normaliseNumbers, type TickRun } from "./tickCollapse";

const marker = (count: number): FeedEntry => ({ kind: "note", text: `↻ ×${count}` });

function tick(feed: FeedEntry[], run: TickRun | null, lines: FeedEntry[], command = "wait 1s") {
  const start = feed.length;
  feed.push({ kind: "command", text: command }, ...lines);
  return collapseTick(feed, start, command, run, marker);
}

const o2 = (n: string): FeedEntry => ({ kind: "line", text: `v1: O2 produced ${n} mol` });
const heat = (n: string): FeedEntry => ({ kind: "line", text: `v1: decomposition releases ${n} kJ` });
const hazard: FeedEntry = { kind: "hazard", severity: "high", text: "oxidiser — keep away", hazardText: "oxidiser" };

describe("normaliseNumbers", () => {
  it("ignores numeric drift, keeps wording", () => {
    expect(normaliseNumbers("T = 25.31 °C, 1e-3 mol")).toBe(normaliseNumbers("T = 25.4 °C, 2.5e-4 mol"));
    expect(normaliseNumbers("O2 produced")).not.toBe(normaliseNumbers("H2 produced"));
  });
});

describe("collapseTick", () => {
  it("collapses ticks that differ only slightly in numbers, keeping the latest values", () => {
    const feed: FeedEntry[] = [{ kind: "note", text: "start" }];
    let run = tick(feed, null, [o2("0.0101"), heat("1.20")]);
    run = tick(feed, run, [o2("0.0102"), heat("1.21")]);
    run = tick(feed, run, [o2("0.0103"), heat("1.22")]);
    expect(feed.map((e) => e.text)).toEqual([
      "start",
      "wait 1s",
      "v1: O2 produced 0.0103 mol",
      "v1: decomposition releases 1.22 kJ",
      "↻ ×3",
    ]);
    expect(feed.at(-1)!.repeat).toBe(3);
    expect(run!.count).toBe(3);
  });

  it("does not collapse different event kinds or wording", () => {
    const feed: FeedEntry[] = [];
    let run = tick(feed, null, [o2("0.01")]);
    run = tick(feed, run, [{ kind: "error", text: "v1: O2 produced 0.01 mol" }]);
    run = tick(feed, run, [heat("1.0")]);
    expect(feed.filter((e) => e.kind === "command")).toHaveLength(3);
    expect(feed.some((e) => e.repeat !== undefined)).toBe(false);
  });

  it("never collapses non-tick commands", () => {
    const feed: FeedEntry[] = [];
    let run = tick(feed, null, [o2("0.01")], "add v1 water 10mL");
    expect(run).toBeNull();
    run = tick(feed, run, [o2("0.01")], "add v1 water 10mL");
    expect(feed).toHaveLength(4);
  });

  it("does not merge across an interleaved entry", () => {
    const feed: FeedEntry[] = [];
    const run = tick(feed, null, [o2("0.01")]);
    feed.push({ kind: "user-note", text: "hmm" });
    tick(feed, run, [o2("0.02")]);
    expect(feed.filter((e) => e.kind === "command")).toHaveLength(2);
  });

  it("shows an identical hazard once per run of ticks", () => {
    const feed: FeedEntry[] = [];
    let run = tick(feed, null, [hazard, o2("0.01")]);
    run = tick(feed, run, [hazard, heat("1.0")]);
    run = tick(feed, run, [hazard, o2("0.03"), heat("2.0")]);
    expect(feed.filter((e) => e.kind === "hazard")).toHaveLength(1);
    expect(feed.filter((e) => e.kind === "command")).toHaveLength(3);
  });
});
