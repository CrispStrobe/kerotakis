import { afterEach, describe, expect, it } from "vitest";
import { render } from "svelte/server";
import LatestResultCard from "./LatestResultCard.svelte";
import { i18n } from "../i18n.svelte";
import type { ResultSummary } from "../resultSummary";

/**
 * What the card shows, and what it stopped showing.
 *
 * Owner, from the German deploy: "it makes no sense to have the img save
 * for each measurement explicitly. we can have it generally at will and
 * save much space. also it shows redundantly the values."
 */
const precipitation: ResultSummary = {
  kind: "precipitation",
  reactionClass: "precipitation",
  vessel: 0,
  equation: "Ag⁺ + Cl⁻ → AgCl",
  reactants: ["Ag⁺", "Cl⁻"],
  observation: "a white solid forms",
  quantities: [{ label: "amount", value: 0.01, unit: "mol" }],
  provenance: "phreeqc · PHREEQC · wateq4f.dat",
};

const draw = (result: ResultSummary = precipitation): string =>
  render(LatestResultCard, { props: { result, onclose: () => {} } }).body;

afterEach(() => i18n.setLocale("en"));

describe("the result card's chrome", () => {
  it("hangs the export off one icon instead of two buttons in the body", () => {
    const card = draw();
    expect(card).toContain('class="icon-export');
    expect(card).toContain('aria-expanded="false"');
    // Closed, the menu is not in the card at all — so the two labels that
    // used to sit under every result are gone from the default view.
    expect(card).not.toContain("save SVG");
    expect(card).not.toContain("save PNG");
    expect(card).not.toContain("share-actions");
  });

  it("keeps the provenance sentence off the card and on the claim it backs", () => {
    const card = draw();
    expect(card).toContain('title="phreeqc · PHREEQC · wateq4f.dat"');
    expect(card).not.toContain('class="provenance');
  });

  it("still shows what the step actually computed", () => {
    const card = draw();
    for (const expected of ["Ag⁺ + Cl⁻ → AgCl", "a white solid forms", "0.01", "mol"]) {
      expect(card).toContain(expected);
    }
  });

  it("names the export in German", () => {
    i18n.setLocale("de");
    expect(draw()).toContain("Ergebniskarte exportieren");
  });
});
