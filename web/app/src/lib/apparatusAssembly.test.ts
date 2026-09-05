import { describe, expect, it } from "vitest";
import { assemblyFor } from "./apparatusAssembly";

describe("physical apparatus assemblies", () => {
  it("connects both centrifuge tubes to the rotor", () => {
    const assembly = assemblyFor("centrifuge", { sampleMass: 5, counterbalance: 5 });
    expect(assembly.edges).toEqual([["sample", "rotor"], ["balance", "rotor"]]);
    expect(assembly.parts.find((part) => part.id === "balance")?.state).toBe("ready");
  });

  it("marks an unsafe counterbalance as needing attention", () => {
    expect(assemblyFor("centrifuge", { sampleMass: 5, counterbalance: 4.8 })
      .parts.find((part) => part.id === "balance")?.state).toBe("attention");
  });

  it("shows the complete carrier-gas path", () => {
    expect(assemblyFor("sweep", {}).parts.map((part) => part.id))
      .toEqual(["source", "inlet", "sample", "outlet"]);
  });

  it("shows familiar candle and balloon skins over existing operators", () => {
    expect(assemblyFor("bunsen", {}).parts.map((part) => part.id))
      .toEqual(["wax", "wick", "flame", "sample"]);
    expect(assemblyFor("regulate", {}).parts.map((part) => part.id))
      .toEqual(["bag", "seal", "sample"]);
  });
});
