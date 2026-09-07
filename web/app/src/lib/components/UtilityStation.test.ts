import { afterEach, describe, expect, it } from "vitest";
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import UtilityStation from "./UtilityStation.svelte";
import { i18n } from "../i18n.svelte";
import { isDisposable, wasteStationAction } from "../wasteStation";

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
 * control has. What it must NOT be is a one-press destructive control.
 *
 * It does empty a single vessel now. `discard vN` is a real engine verb
 * (kerotakis-core::ops::Operator::Discard), so the station offers the
 * selected vessel to the bench's waste container whenever that vessel is
 * holding something, and keeps its older bench-clearing meaning when it is
 * not. `wasteStation.ts` owns that decision; the station's wording and the
 * caller's command both read it, so they cannot disagree.
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

describe("what the station's confirmed press means", () => {
  it("submits discard for the selected vessel, addressed from one", () => {
    expect(wasteStationAction({ id: 0, mass_g: 12.3 })).toEqual({
      kind: "discard",
      command: "discard v1",
    });
    expect(wasteStationAction({ id: 2, mass_g: 0.004 })).toEqual({
      kind: "discard",
      command: "discard v3",
    });
  });

  it("falls back to clearing the bench when there is no vessel to empty", () => {
    // No selection at all — a bench whose scene has not arrived, or a
    // selection pointing at a vessel that is no longer there.
    expect(wasteStationAction(null)).toEqual({ kind: "clear" });
    expect(wasteStationAction(undefined)).toEqual({ kind: "clear" });
    // A vessel standing there empty is not a disposal either: there is
    // nothing for the container to take, and the press keeps the meaning
    // the station has always had.
    expect(wasteStationAction({ id: 0, mass_g: 0 })).toEqual({ kind: "clear" });
  });

  it("does not call floating-point dust a vessel worth emptying", () => {
    expect(isDisposable({ id: 0, mass_g: 1e-12 })).toBe(false);
    expect(isDisposable({ id: 0, mass_g: 1e-6 })).toBe(true);
  });
});

describe("the station once it has a vessel to empty", () => {
  it("names the vessel it would pour out, rather than the whole bench", () => {
    const rendered = text(body({ vessel: 1, disposable: true }));
    expect(rendered).toContain("Pour the contents of vessel v2");
    // The bench-wide policy sentence belongs to the fallback. Leaving it
    // standing over a press that empties one vessel would describe a
    // different operation from the one about to happen.
    expect(rendered).not.toContain("Chemical contents are never discarded silently");
  });

  it("stays pressable for a full vessel even when the bench has nothing to clear", () => {
    // `clearable` asks whether the BENCH has anything to clear. A restored
    // lab with a full vessel and an empty command log had a disposal
    // control that was greyed out over a vessel nobody could empty.
    expect(body({ disposable: true, clearable: false })).not.toContain("disabled");
    expect(body({ disposable: false, clearable: false })).toContain("disabled");
  });

  it("says the disposal offer in German too", () => {
    i18n.setLocale("de");
    const rendered = text(body({ vessel: 0, disposable: true }));
    expect(rendered).toContain("Abfallbehälter");
    expect(rendered).not.toContain("waste container");
  });

  it("shows what the container took, and stays quiet before it has taken anything", () => {
    const receipt = "The waste container took 12.30 g from v1.";
    expect(text(body({ receipt }))).toContain(receipt);
    expect(text(body())).not.toContain("waste container took");
  });
});
