import { afterEach, describe, expect, it } from "vitest";
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import Feed from "./Feed.svelte";
import { i18n } from "../i18n.svelte";
import type { FeedEntry } from "../session.svelte";

/**
 * The journal, RENDERED.
 *
 * `journalFeed.test.ts` pins the rule; this pins the component that obeys
 * it. Both are needed, and the regression that prompted them proves why:
 * the rule and the markup disagreed — the header drew the session's status
 * notes as icons while the list dropped every entry carrying one — and no
 * assertion anywhere in the suite ever asked what the component actually
 * put on screen. So the logbook shipped empty.
 *
 * Svelte's server renderer needs no DOM, which is the whole reason this can
 * run in the same plain-node vitest as everything else.
 */
function body(entries: FeedEntry[], props: Partial<ComponentProps<typeof Feed>> = {}): string {
  return render(Feed, { props: { entries, ...props } }).body;
}

/** The rendered text, with the markup and Svelte's anchors taken out. */
function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
}

/** What comes after the header: the log itself. */
function log(html: string): string {
  const end = html.indexOf("</header>");
  return text(end === -1 ? html : html.slice(end));
}

const restored: FeedEntry[] = [
  { kind: "note", status: "bench-live", text: "The bench is live: states nobody pre-computed are solved." },
  { kind: "note", status: "restored", text: "restored your last session: 4 step(s) restored instantly" },
];

afterEach(() => i18n.setLocale("en"));

describe("the journal on screen", () => {
  it("is not empty for a learner who has not run a command yet", () => {
    // The bug, stated as the owner met it: open the deploy, and the logbook
    // shows a row of grey glyphs with nothing under them.
    const rendered = log(body(restored));
    expect(rendered).toContain("The bench is live");
    expect(rendered).toContain("restored your last session");
  });

  it("shows the same log in German", () => {
    i18n.setLocale("de");
    expect(log(body(restored))).toContain("The bench is live");
  });

  it("draws every entry kind the bench can produce", () => {
    const feed: FeedEntry[] = [
      ...restored,
      { kind: "line", text: "v1: You add 100 mL of water." },
      { kind: "error", text: "v1: no such species" },
      { kind: "refusal", text: "v1: the bench declines" },
      { kind: "nudge", text: "try warming it" },
      { kind: "claim", text: "you found a precipitate" },
      { kind: "hazard", text: "wear goggles", severity: "caution" },
      { kind: "user-note", text: "smells of nothing", createdAt: "2026-09-06T09:00:00.000Z" },
    ];
    const rendered = log(body(feed));
    for (const expected of [
      "The bench is live", "restored your last session", "You add 100 mL of water",
      "no such species", "the bench declines", "try warming it",
      "you found a precipitate", "wear goggles", "smells of nothing",
    ]) {
      expect(rendered).toContain(expected);
    }
  });

  it("holds the typed commands back until the trace is asked for", () => {
    const feed: FeedEntry[] = [
      { kind: "command", text: "add v1 water 100mL" },
      { kind: "line", text: "v1: You add 100 mL of water." },
    ];
    const rendered = body(feed);
    expect(log(rendered)).toContain("You add 100 mL of water");
    expect(log(rendered)).not.toContain("add v1 water 100mL");
    // …and the toggle that reveals them says how many there are.
    expect(rendered).toContain('aria-label="full trace"');
  });

  it("shows every vessel whichever one is selected", () => {
    const feed: FeedEntry[] = [
      { kind: "line", text: "v1: You add 100 mL of water." },
      { kind: "line", text: "v2: Thermometer: 21.81 °C" },
    ];
    for (const selectedVessel of [0, 1, 4]) {
      const rendered = log(body(feed, { selectedVessel }));
      expect([selectedVessel, rendered.includes("You add 100 mL of water")]).toEqual([selectedVessel, true]);
      expect([selectedVessel, rendered.includes("Thermometer")]).toEqual([selectedVessel, true]);
    }
  });

  it("carries no vessel scope to filter the bench down to one vessel", () => {
    const rendered = body(restored, { selectedVessel: 0 });
    expect(rendered).not.toContain("journal-scope");
    expect(rendered).not.toContain("whole lab");
  });

  it("keeps the note composer collapsed without hiding the log behind it", () => {
    const rendered = body(restored, { onaddnote: () => {} });
    expect(rendered).toContain('aria-expanded="false"');
    expect(rendered).not.toContain("<textarea");
    expect(log(rendered)).toContain("The bench is live");
  });
});
