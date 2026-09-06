import { describe, expect, it } from "vitest";
import { DOCK_INSTRUMENTS, vesselQuickActions } from "./directActions";
import { INSTRUMENTS } from "./instruments";
import {
  DEFAULT_RECENT_INSTRUMENTS,
  QUICK_ACCESS_SIZE,
  loadRecentInstruments,
  quickAccess,
  quickAccessCandidates,
  quickAccessRow,
  rememberInstrument,
  saveRecentInstruments,
} from "./instrumentRecents";

const KNOWN = INSTRUMENTS.map((item) => item.token);

describe("the quick-access row", () => {
  it("seeds with four measurements that actually exist", () => {
    // A default naming a retired token would render a row of nothing on a
    // first visit — the one visit where the row has to work.
    expect(QUICK_ACCESS_SIZE).toBe(4);
    expect(DEFAULT_RECENT_INSTRUMENTS.length).toBe(QUICK_ACCESS_SIZE);
    for (const token of DEFAULT_RECENT_INSTRUMENTS) expect(KNOWN).toContain(token);
    expect(quickAccess([], KNOWN)).toEqual([...DEFAULT_RECENT_INSTRUMENTS]);
  });

  it("never shows more than the row can hold without scrolling", () => {
    // Twelve in a non-wrapping row is what this replaces; a row that
    // scrolls again would be the same bug — and the row also carries the
    // cupboard door, which is the thing it exists to lead to.
    expect(quickAccess(KNOWN, KNOWN).length).toBe(QUICK_ACCESS_SIZE);
  });

  it("never offers what the vessel dock already carries", () => {
    // GUI-103, decisions 1 and 2: the dock's look / thermometer / pH are
    // fixed landmarks on every vessel, so a strip that also held them spent
    // three of its slots on buttons that never move — and the learner saw
    // the same reading twice on one screen.
    for (const dock of DOCK_INSTRUMENTS) expect(KNOWN).toContain(dock);
    expect(quickAccessCandidates(KNOWN)).toEqual(KNOWN.filter((token) => !DOCK_INSTRUMENTS.includes(token)));
    for (const dock of DOCK_INSTRUMENTS) {
      expect(DEFAULT_RECENT_INSTRUMENTS).not.toContain(dock);
      expect(quickAccess([dock], KNOWN)).not.toContain(dock);
      expect(quickAccessRow([dock], KNOWN)).not.toContain(dock);
      // Measuring one from the cupboard neither enters the row nor evicts
      // anything already in it.
      expect(rememberInstrument(["geiger"], dock, KNOWN)).toEqual(["geiger"]);
    }
  });

  it("names the same three the dock actually renders", () => {
    // Two hand-written lists, pinned to each other: the dock builds
    // `measure v1 <token>` lines, and this is the exclusion that reads them.
    const measured = vesselQuickActions(0, "open")
      .map((action) => /^measure v1 (\S+)$/.exec(action.line)?.[1])
      .filter((token): token is string => token !== undefined);
    expect([...measured].sort()).toEqual([...DOCK_INSTRUMENTS].sort());
  });

  it("pads a short history from the seed, so one measurement is still a full row", () => {
    const row = quickAccess(["geiger"], KNOWN);
    expect(row[0]).toBe("geiger");
    expect(row.length).toBe(QUICK_ACCESS_SIZE);
    expect(new Set(row).size).toBe(QUICK_ACCESS_SIZE);
  });

  it("drops what it cannot render rather than offering a dead button", () => {
    // `ph` is not unknown — it is the dock's, which is the same answer here.
    expect(quickAccess(["not-an-instrument", "ph", "ph"], KNOWN)).toEqual([...DEFAULT_RECENT_INSTRUMENTS]);
    expect(quickAccess(["not-an-instrument", "geiger", "geiger"], KNOWN).slice(0, 2)).toEqual(["geiger", "balance"]);
  });

  it("moves a re-used instrument to the front instead of adding it twice", () => {
    const first = rememberInstrument([], "geiger", KNOWN);
    const second = rememberInstrument(first, "uvvis", KNOWN);
    expect(second.slice(0, 2)).toEqual(["uvvis", "geiger"]);
    const third = rememberInstrument(second, "geiger", KNOWN);
    expect(third.slice(0, 2)).toEqual(["geiger", "uvvis"]);
    expect(new Set(third).size).toBe(third.length);
  });

  it("caps the stored order and ignores a token it does not know", () => {
    let row: string[] = [];
    for (const token of KNOWN) row = rememberInstrument(row, token, KNOWN);
    expect(row.length).toBe(QUICK_ACCESS_SIZE);
    expect(rememberInstrument(row, "not-an-instrument", KNOWN)).toEqual(row);
  });
});

describe("persisting the row", () => {
  it("reads back what it wrote", () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => void store.set(key, value),
    };
    saveRecentInstruments(storage, "k", ["ph", "geiger"]);
    expect(loadRecentInstruments(storage, "k")).toEqual(["ph", "geiger"]);
  });

  it("treats unreadable storage as no history rather than as a crash", () => {
    // A private window, cleared site data, or a hand-edited value: the row
    // falls back to the seed, which is a working row.
    expect(loadRecentInstruments(null, "k")).toEqual([]);
    expect(loadRecentInstruments({ getItem: () => "not json" }, "k")).toEqual([]);
    expect(loadRecentInstruments({ getItem: () => '{"ph":1}' }, "k")).toEqual([]);
    expect(loadRecentInstruments({ getItem: () => '["ph", 7, null]' }, "k")).toEqual(["ph"]);
    expect(loadRecentInstruments({ getItem: () => { throw new Error("blocked"); } }, "k")).toEqual([]);
    expect(() => saveRecentInstruments({ setItem: () => { throw new Error("full"); } }, "k", ["ph"])).not.toThrow();
    expect(() => saveRecentInstruments(null, "k", ["ph"])).not.toThrow();
  });
});

describe("the drawn row", () => {
  it("keeps the catalogue's order, so a tap never moves the button under the finger", () => {
    // Membership is by recency; position is not. A row that re-sorted on
    // every tap would slide the next instrument out from under the finger
    // that had just used this one.
    const before = quickAccessRow([], KNOWN);
    const after = quickAccessRow(rememberInstrument([], "ph", KNOWN), KNOWN);
    expect(after).toEqual(before);
    expect(before).toEqual(KNOWN.filter((token) => before.includes(token)));
  });

  it("moves only when something enters or leaves the row", () => {
    const row = quickAccessRow(rememberInstrument([], "geiger", KNOWN), KNOWN);
    expect(row).toContain("geiger");
    expect(row.length).toBe(QUICK_ACCESS_SIZE);
    expect(row).toEqual(KNOWN.filter((token) => row.includes(token)));
  });
});
