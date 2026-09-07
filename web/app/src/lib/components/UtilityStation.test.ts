import { afterEach, describe, expect, it } from "vitest";
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import UtilityStation from "./UtilityStation.svelte";
import { i18n } from "../i18n.svelte";

/**
 * The disposal station was a paragraph pretending to be a control.
 *
 * Two of the three stations were buttons; the third was an `<article>` that
 * explained, correctly, that chemical contents are never discarded
 * silently — and then offered no way to discard them loudly either. The
 * remove-vessel dialog's "open waste station" led straight to it, so a
 * reader with a full vessel followed a signpost to a statement.
 *
 * It is a button now, with the same ask-once shape the toolbar's empty
 * control has. What it must NOT be is a one-press destructive control, and
 * what it must not do is claim to empty a single vessel: the engine has no
 * verb that discards one vessel's contents, and the station would be lying
 * about the chemistry if it said otherwise.
 */
function body(props: Partial<ComponentProps<typeof UtilityStation>> = {}): string {
  return render(UtilityStation, {
    props: {
      vessel: 0,
      onwater: () => {},
      onequipment: () => {},
      onwaste: () => {},
      onclose: () => {},
      ...props,
    },
  }).body;
}

/** The rendered text, with the markup and Svelte's anchors taken out. */
function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
}

afterEach(() => i18n.setLocale("en"));

describe("the utility station's disposal", () => {
  it("offers disposal as a control, not as a paragraph", () => {
    const rendered = body();
    // Unterminated on purpose, as every other rendered-markup assertion in
    // this directory is: Svelte appends its scoping class to any element the
    // component styles, so the attribute is `class="station waste svelte-…"`.
    expect(rendered).toContain('class="station waste');
    expect(rendered).toContain("<button");
    // The shape the other two stations have: a button, with the same
    // affordance chevron, rather than an <article> among buttons.
    expect(rendered).not.toContain("<article");
  });

  it("keeps saying why nothing goes away by itself", () => {
    expect(text(body())).toContain("Chemical contents are never discarded silently");
  });

  it("says so in German too", () => {
    i18n.setLocale("de");
    const rendered = text(body());
    expect(rendered).toContain("Entsorgungsstation");
    expect(rendered).not.toContain("waste station");
  });

  it("is not armed until it is pressed, so the first press cannot destroy anything", () => {
    const rendered = text(body());
    expect(rendered).not.toContain("keep it");
    expect(rendered).not.toContain("clear the bench?");
  });

  it("goes quiet when there is nothing on the bench to dispose of", () => {
    expect(body({ clearable: false })).toContain("disabled");
    expect(body({ clearable: true })).not.toContain("disabled");
  });

  it("still names the vessel the other two stations act on", () => {
    expect(text(body({ vessel: 2 }))).toContain("v3");
  });
});
