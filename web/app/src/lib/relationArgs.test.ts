import { describe, expect, it } from "vitest";
import { buildArgs, parseArgSpec } from "./relationArgs";

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
