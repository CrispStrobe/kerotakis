import { afterEach, describe, expect, it } from "vitest";
import { buildArgs, parseArgSpec } from "./relationArgs";
import { hasGermanTranslation, i18n, t, tEngine } from "./i18n.svelte";

describe("relation arg specs become forms", () => {
  it("parses k=<hint> tokens with optional brackets", () => {
    const { fields, freeform } = parseArgSpec("A=<prefactor> Ea=<J/mol> T=<K> [b=<exponent>]");
    expect(freeform).toBe(false);
    expect(fields.map((f) => f.name)).toEqual(["A", "Ea", "T", "b"]);
    expect(fields[3]!.optional).toBe(true);
    expect(fields[1]!.hint).toBe("J/mol");
  });

  it("pair syntax falls back to freeform", () => {
    expect(parseArgSpec("<z>:<m> <z>:<m> ... (charge:molality pairs)").freeform).toBe(true);
  });

  it("builds k=v args, skipping empty optionals, refusing non-numbers", () => {
    const { fields } = parseArgSpec("pKa=<value> cA=<mol/L> cB=<mol/L>");
    expect(buildArgs(fields, { pKa: "4.76", cA: "0.1", cB: "0.1" })).toEqual([
      "pKa=4.76",
      "cA=0.1",
      "cB=0.1",
    ]);
    expect(buildArgs(fields, { pKa: "4.76", cA: "", cB: "0.1" })).toBeNull();
    expect(buildArgs(fields, { pKa: "abc", cA: "0.1", cB: "0.1" })).toBeNull();
  });
});

/** GUI-087/GUI-096: what the relation answers, where it holds, and where it
 *  came from all arrive ON the catalogue row from the engine — a different
 *  road from the shell dictionary, and one the drawer walks with tEngine.
 *  These pin the field NAMES the drawer reads: rename `source_de` on the
 *  Rust side and the panel silently shows English inside a German page,
 *  which no emptiness check anywhere would notice. */
describe("a relation says what it is for", () => {
  afterEach(() => {
    i18n.locale = "en";
  });

  // Shaped exactly like one `relations` row, abridged to the prose fields.
  const debyeHuckel = {
    name: "debye-huckel",
    purpose: "How far an ion's activity falls below its concentration.",
    purpose_de: "Wie weit die Aktivität eines Ions unter seiner Konzentration liegt.",
    validity: "Only in dilute solution, roughly below I = 0.01 mol/kg.",
    validity_de: "Nur in verdünnter Lösung, etwa unterhalb I = 0,01 mol/kg.",
    source: "Debye–Hückel limiting law (P. Debye and E. Hückel, 1923)",
    source_de: "Debye–Hückel-Grenzgesetz (P. Debye und E. Hückel, 1923)",
  };

  it("shows the engine's English sentences when the shell is English", () => {
    i18n.locale = "en";
    for (const field of ["purpose", "validity", "source"] as const) {
      expect(tEngine(debyeHuckel, field)).toBe(debyeHuckel[field]);
    }
  });

  it("shows the engine's German sentences when the shell is German", () => {
    i18n.locale = "de";
    expect(tEngine(debyeHuckel, "purpose")).toBe(debyeHuckel.purpose_de);
    expect(tEngine(debyeHuckel, "validity")).toBe(debyeHuckel.validity_de);
    expect(tEngine(debyeHuckel, "source")).toBe(debyeHuckel.source_de);
  });

  it("falls back per string, so an untranslated citation reads English, not blank", () => {
    i18n.locale = "de";
    const { source_de: _dropped, ...halfTranslated } = debyeHuckel;
    expect(tEngine(halfTranslated, "source")).toBe(debyeHuckel.source);
    // …and the other fields are unaffected by the one that is missing.
    expect(tEngine(halfTranslated, "validity")).toBe(debyeHuckel.validity_de);
  });

  it("labels the citation in German too", () => {
    expect(hasGermanTranslation("where it comes from")).toBe(true);
    i18n.locale = "de";
    expect(t("where it comes from")).not.toBe("where it comes from");
    // Distinct from the code-source label, which is a different sense of
    // the same English word and is already spoken for.
    expect(t("where it comes from")).not.toBe(t("source"));
  });
});
