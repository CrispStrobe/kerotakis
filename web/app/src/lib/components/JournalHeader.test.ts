import { afterEach, describe, expect, it } from "vitest";
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import JournalHeader from "./JournalHeader.svelte";
import { i18n } from "../i18n.svelte";

/**
 * GUI-107 — the journal's one row of chrome.
 *
 * Two rows became one, so the thing worth pinning is that nothing fell out
 * on the way: every control the two rows carried is still here, still
 * named, and still in an order a keyboard can walk.
 *
 * What this CANNOT tell you, and why a second guard lives in
 * `tools/test-ux-quality.mjs`: `svelte/server` produces markup and never
 * fires a handler, so a dead `onclick` passes every assertion below.
 */
function body(props: Partial<ComponentProps<typeof JournalHeader>> = {}): string {
  return render(JournalHeader, {
    props: {
      entryCount: 2,
      hiddenCommands: 0,
      collapsed: false,
      canCompose: true,
      oncollapse: () => {},
      ...props,
    },
  }).body;
}

/** Every `aria-label` in the row, in source order — which is tab order. */
function labels(html: string): string[] {
  return [...html.matchAll(/aria-label="([^"]*)"/g)].map((match) => match[1] ?? "");
}

afterEach(() => i18n.setLocale("en"));

describe("the journal's heading", () => {
  it("carries the identity the pane heading used to carry alone", () => {
    const rendered = body({ entryCount: 7 });
    expect(rendered).toContain("lab journal");
    expect(rendered).toContain(">7<");
    expect(rendered).toContain("notebook entries");
  });

  it("carries the controls the feed's second row used to carry", () => {
    const rendered = body();
    expect(rendered).toContain("journal-view");
    expect(labels(rendered)).toEqual([
      "journal view",
      "observations",
      "full trace",
      "add note",
      "collapse lab journal",
    ]);
  });

  it("opens on the observations, with the trace as the other choice", () => {
    // The default is not remembered, deliberately: the journal opens on the
    // journal every session, exactly as it did when the toggle was inside
    // the feed.
    const rendered = body();
    const pressed = [...rendered.matchAll(/aria-pressed="(true|false)"/g)].map((match) => match[1]);
    expect(pressed).toEqual(["true", "false"]);
    expect(body({ showTrace: true }).match(/aria-pressed="(true|false)"/g))
      .toEqual(['aria-pressed="false"', 'aria-pressed="true"']);
  });

  it("badges the trace with how many lines it would add, and only when there are some", () => {
    expect(body({ hiddenCommands: 0 })).not.toContain('class="count"');
    expect(body({ hiddenCommands: 12 })).toContain(">12<");
  });

  it("draws no note chevron when the shell passes no note handler", () => {
    // `canCompose` is false in any surface that offers the journal
    // read-only; a chevron that opens a form nobody will read is chrome for
    // its own sake, which is the thing this task removed.
    expect(labels(body({ canCompose: false }))).not.toContain("add note");
    expect(body({ canCompose: false })).not.toContain("journal-note-composer");
  });

  it("names the collapse chevron for what pressing it will do", () => {
    expect(body({ collapsed: false })).toContain("collapse lab journal");
    expect(body({ collapsed: true })).toContain("open lab journal");
    expect(body({ collapsed: true })).toContain('aria-expanded="false"');
  });

  it("names every control in German too", () => {
    i18n.setLocale("de");
    const named = labels(body());
    expect(named).toHaveLength(5);
    expect(named.every((label) => label.trim().length > 0)).toBe(true);
    // The one that would betray an untranslated row: the pane's own name.
    expect(body()).toContain("Laborbuch");
  });
});
