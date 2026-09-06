import { describe, expect, it } from "vitest";
import type { FeedEntry } from "./session.svelte";
import { JOURNAL_WINDOW, displayText, entryVessel, journalEntries, statusIcon } from "./journalFeed";

/**
 * The feed a returning learner actually has.
 *
 * `restoreProgress` replays the saved script through the engine WITHOUT
 * writing its lines to the journal, so a bench that comes back from a save
 * carries exactly these two notes and nothing else until the next command.
 * That is the state a header rebuild once blanked, and it is the state
 * every rule below is measured against.
 */
const restoredSession: FeedEntry[] = [
  { kind: "note", status: "bench-live", text: "The bench is live: states nobody pre-computed are solved." },
  { kind: "note", status: "restored", text: "restored your last session: 4 step(s) restored instantly" },
];

const workingSession: FeedEntry[] = [
  ...restoredSession,
  { kind: "command", text: "add v1 water 100mL" },
  { kind: "line", text: "v1: You add 100 mL of water." },
  { kind: "command", text: "measure v2 thermometer" },
  { kind: "line", text: "v2: Thermometer: 21.81 °C" },
  { kind: "user-note", text: "smells of nothing", createdAt: "2026-09-06T09:00:00.000Z" },
  { kind: "hazard", text: "wear goggles", severity: "caution" },
];

describe("what the journal shows", () => {
  it("keeps a restored session's status notes in the log", () => {
    // The regression this file exists for: the header started drawing these
    // two as icons and the filter dropped them from the list, so the only
    // entries a just-loaded bench has vanished and the logbook opened blank.
    expect(journalEntries(restoredSession, { showTrace: false })).toEqual(restoredSession);
    expect(journalEntries(restoredSession, { showTrace: true })).toEqual(restoredSession);
  });

  it("never empties a non-empty feed, under either toggle", () => {
    for (const showTrace of [false, true]) {
      expect(journalEntries(workingSession, { showTrace }).length).toBeGreaterThan(0);
    }
  });

  it("hides only the typed commands, and only in the observations view", () => {
    const observations = journalEntries(workingSession, { showTrace: false });
    expect(observations.some((entry) => entry.kind === "command")).toBe(false);
    expect(observations.length).toBe(workingSession.length - 2);
    expect(journalEntries(workingSession, { showTrace: true })).toEqual(workingSession);
  });

  it("shows every vessel's lines, whichever vessel is selected", () => {
    // There is no scope any more. The rule is pinned rather than merely
    // absent, because the filter that hid other vessels was removed from a
    // journal whose whole job is to be the record of the WHOLE bench.
    const shown = journalEntries(workingSession, { showTrace: true });
    expect(shown.filter((entry) => entryVessel(entry) === 0).length).toBeGreaterThan(0);
    expect(shown.filter((entry) => entryVessel(entry) === 1).length).toBeGreaterThan(0);
  });

  it("keeps an entry of every kind the session can push", () => {
    const kinds: FeedEntry["kind"][] = [
      "command", "line", "error", "refusal", "note", "user-note", "hazard", "chart", "nudge", "claim",
    ];
    const feed: FeedEntry[] = kinds.map((kind) => ({ kind, text: `a ${kind}` }));
    const shown = journalEntries(feed, { showTrace: true }).map((entry) => entry.kind);
    expect(shown).toEqual(kinds);
  });

  it("windows only what is longer than the window", () => {
    const long: FeedEntry[] = Array.from({ length: JOURNAL_WINDOW + 10 }, (_, index) => ({
      kind: "line", text: `line ${index}`,
    }));
    expect(journalEntries(long, { showTrace: false }).length).toBe(JOURNAL_WINDOW + 10);
  });
});

describe("how a line names its vessel", () => {
  it("reads the vessel from a command and from an engine line", () => {
    expect(entryVessel({ kind: "command", text: "heat v3 10kJ" })).toBe(2);
    expect(entryVessel({ kind: "line", text: "v2: it warms" })).toBe(1);
    expect(entryVessel({ kind: "line", text: "it warms" })).toBeNull();
    expect(entryVessel({ kind: "user-note", text: "v1 looks cloudy" })).toBeNull();
  });

  it("strips the prefix the chip already carries", () => {
    expect(displayText({ kind: "line", text: "v2: Thermometer: 21.81 °C" })).toBe("Thermometer: 21.81 °C");
    expect(displayText({ kind: "command", text: "measure v2 thermometer" })).toBe("measure v2 thermometer");
  });
});

describe("the session's own bookkeeping", () => {
  it("marks the four status notes and nothing else", () => {
    expect(statusIcon({ kind: "note", status: "bench-live", text: "" })).toBe("◉");
    expect(statusIcon({ kind: "note", status: "bench-shipped", text: "" })).toBe("◌");
    expect(statusIcon({ kind: "note", status: "restored", text: "" })).toBe("⟳");
    expect(statusIcon({ kind: "note", status: "restore-failed", text: "" })).toBe("⚠");
    expect(statusIcon({ kind: "note", text: "the bench is empty again" })).toBeUndefined();
  });
});
