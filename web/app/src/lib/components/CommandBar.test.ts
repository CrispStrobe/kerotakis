/**
 * The command bar's completion popup, RENDERED — and what that can and
 * cannot prove.
 *
 * `svelte/server` produces markup and never fires a handler, so this file
 * can only assert what the bar is BEFORE anybody touches it. That is
 * still worth asserting, because it is where two regressions would live:
 * a popup that opens by itself (a wall of every verb the moment the
 * console is opened) and an input that lost the ARIA a combobox needs to
 * be announced at all. Everything that happens after a keystroke —
 * arrows, Escape, taking a suggestion — is in
 * `tools/test-ux-quality.mjs`, in a real browser, because here it would
 * pass whether the handlers existed or not.
 */
import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import CommandBar from "./CommandBar.svelte";
import type { CompletionSources } from "../completions";

const sources: CompletionSources = {
  grammar: [
    { verb: "add", example: "add v1 water 100mL", typed: "gib v1 Wasser 100mL" },
    { verb: "heat", example: "heat v1 10kJ", typed: "erhitze v1 10kJ" },
  ],
  vessels: [{ id: 0, label: "beaker" }],
  shelf: [{ key: "water", name: "water", formula: "H2O" }],
  translate: (text) => text,
};

const html = (props: Record<string, unknown> = {}) =>
  render(CommandBar, {
    props: { onsubmit: () => {}, busy: false, completionSources: sources, ...props },
  }).body;

describe("the command bar at rest", () => {
  it("is a combobox a screen reader can announce", () => {
    const body = html();
    expect(body).toContain('role="combobox"');
    expect(body).toContain('aria-autocomplete="list"');
    expect(body).toContain('aria-controls="kero-completions"');
  });

  it("does not open its popup until somebody touches it", () => {
    // The bar is a bar. A list of every verb on arrival is a wall of text
    // nobody asked for, and it would cover the feed it sits over.
    const body = html();
    expect(body).toContain('aria-expanded="false"');
    expect(body).not.toContain('role="listbox"');
  });

  it("claims no active option while it is closed", () => {
    // `aria-activedescendant` pointing at an id that is not in the
    // document is how a combobox goes silent: the reader is told the
    // focus is somewhere the page does not have.
    expect(html()).not.toContain("aria-activedescendant");
  });

  it("is still a bar with no completion sources at all", () => {
    // An older engine, or a caller with nothing to offer. The popup never
    // opens; nothing else about the bar changes.
    const body = html({ completionSources: undefined });
    expect(body).toContain('role="combobox"');
    expect(body).not.toContain('role="listbox"');
  });
});
