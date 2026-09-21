/**
 * The complete ionic equation, RENDERED (GUI-092).
 *
 * `ionic.test.ts` pins the contract; this pins the markup that obeys it,
 * because the whole feature is a visual one — the spectators are the
 * terms the reader has to *see* cancel. A suite that only checked the
 * data would have passed on a line that drew every term the same.
 *
 * Svelte's server renderer needs no DOM and fires no handler, which is
 * right here: nothing on this line is pressable.
 */
import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import CompleteIonicLine from "./CompleteIonicLine.svelte";
import type { CompleteIonic } from "../ionic";

/** Barium chloride met by sodium sulfate: two of each spectator. */
const bariumSulfate: CompleteIonic = {
  reactants: [
    { species: "Ba+2", label: "Ba²⁺", coefficient: 1, charge: 2, phase: "aqueous" },
    { species: "SO4-2", label: "SO₄²⁻", coefficient: 1, charge: -2, phase: "aqueous" },
    {
      species: "Na+", label: "Na⁺", coefficient: 2, charge: 1,
      phase: "aqueous", spectator: true,
    },
    {
      species: "Cl-", label: "Cl⁻", coefficient: 2, charge: -1,
      phase: "aqueous", spectator: true,
    },
  ],
  products: [
    { species: "BaSO4", label: "BaSO₄", coefficient: 1, charge: 0, phase: "solid" },
    {
      species: "Na+", label: "Na⁺", coefficient: 2, charge: 1,
      phase: "aqueous", spectator: true,
    },
    {
      species: "Cl-", label: "Cl⁻", coefficient: 2, charge: -1,
      phase: "aqueous", spectator: true,
    },
  ],
  equation:
    "Ba²⁺(aq) + SO₄²⁻(aq) + 2 Na⁺(aq) + 2 Cl⁻(aq) → " +
    "BaSO₄(s) + 2 Na⁺(aq) + 2 Cl⁻(aq)",
};

function body(complete: CompleteIonic): string {
  return render(CompleteIonicLine, {
    props: { complete, label: "complete ionic" },
  }).body;
}

/**
 * Everything between `<s>` and `</s>`, in order.
 *
 * Deliberately not `<s[^>]*>`, which also matches `<span`, and the
 * comment strip is for Svelte's own SSR anchors.
 */
function struck(html: string): string[] {
  return [...html.matchAll(/<s(?:\s[^>]*)?>([\s\S]*?)<\/s>/g)].map((m) =>
    (m[1] ?? "").replace(/<!--[\s\S]*?-->/g, "").trim(),
  );
}

describe("the complete ionic line", () => {
  it("strikes the spectators through and leaves the participants standing", () => {
    const html = body(bariumSulfate);
    // Both sides, so four struck terms: the cancellation is the thing
    // being shown, and a strike on one side only shows nothing.
    expect(struck(html)).toEqual(["2 Na⁺(aq)", "2 Cl⁻(aq)", "2 Na⁺(aq)", "2 Cl⁻(aq)"]);
    // The participants are drawn, and drawn plain.
    expect(html).toContain("Ba²⁺(aq)");
    expect(html).toContain("BaSO₄(s)");
    expect(struck(html)).not.toContain("Ba²⁺(aq)");
  });

  it("writes the solved coefficient, not a 1", () => {
    // The bug this feature fixes, seen at the only place a reader meets
    // it: every spectator used to carry `coefficient: 1`, and against a
    // 1:1 salt that is invisible.
    const html = body(bariumSulfate);
    expect(html.match(/2 Na⁺\(aq\)/g)).toHaveLength(2);
    expect(html.match(/2 Cl⁻\(aq\)/g)).toHaveLength(2);
  });

  it("draws one arrow and the label it was handed", () => {
    const html = body(bariumSulfate);
    expect(html.match(/→/g)).toHaveLength(1);
    expect(html).toContain("complete ionic");
  });

  it("makes no chemical decision of its own", () => {
    // The strike-through is presentation of a flag the ENGINE set. Handed
    // the same equation with the flags cleared, the component draws it
    // with nothing struck rather than working out which terms repeat.
    const unflagged: CompleteIonic = {
      ...bariumSulfate,
      reactants: bariumSulfate.reactants.map((t) => ({ ...t, spectator: false })),
      products: bariumSulfate.products.map((t) => ({ ...t, spectator: false })),
    };
    expect(struck(body(unflagged))).toEqual([]);
    expect(body(unflagged)).toContain("2 Na⁺(aq)");
  });
});
