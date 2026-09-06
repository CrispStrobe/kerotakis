import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { assemblyAttention, assemblyFor, drawnOnStage } from "./apparatusAssembly";

const APPARATUS_TOOLS = [
  "bunsen", "stir", "heat", "cool", "centrifuge", "electrolyse",
  "irradiate", "regulate", "sweep", "grind", "dilute", "evaporate",
];

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

/**
 * GUI-472: the assembly is an annotation ON the vessel, so the coordinates
 * have to be inside the vessel's viewBox and the annotation has to be
 * offered only where there is a drawing under it.
 */
describe("the assembly is anchored to the vessel it is built around", () => {
  it.each(APPARATUS_TOOLS)("%s puts every part inside the vessel's 0 0 100 140 viewBox", (tool) => {
    for (const item of assemblyFor(tool, {}).parts) {
      expect(item.at[0], `${item.id} x`).toBeGreaterThanOrEqual(0);
      expect(item.at[0], `${item.id} x`).toBeLessThanOrEqual(100);
      expect(item.at[1], `${item.id} y`).toBeGreaterThanOrEqual(0);
      expect(item.at[1], `${item.id} y`).toBeLessThanOrEqual(140);
    }
  });

  it("gives every edge two endpoints it can actually draw between", () => {
    for (const tool of [...APPARATUS_TOOLS, "unheard-of"]) {
      const assembly = assemblyFor(tool, {});
      const ids = new Set(assembly.parts.map((item) => item.id));
      for (const [from, to] of assembly.edges) {
        expect(ids.has(from), `${tool}: ${from}`).toBe(true);
        expect(ids.has(to), `${tool}: ${to}`).toBe(true);
      }
    }
  });

  it("separates parts that share a place, so two markers are two dots", () => {
    // Every marker is r=3.4 in the same viewBox; two parts drawn on top of
    // one another read as one piece and the assembly loses a part silently.
    for (const tool of APPARATUS_TOOLS) {
      const seen = new Set<string>();
      for (const item of assemblyFor(tool, {}).parts) {
        const key = `${item.at[0]},${item.at[1]}`;
        expect(seen.has(key), `${tool}: ${item.id} sits on another part`).toBe(false);
        seen.add(key);
      }
    }
  });

  /**
   * The claim `drawnOnStage` makes is about ANOTHER file, so read that
   * file. A tool added to `DeployedApparatus` and forgotten here loses its
   * annotation; a tool listed here that it never draws puts markers over a
   * bare beaker. Both are silent, and both are one grep away.
   */
  it("claims a stage drawing exactly where DeployedApparatus has one", () => {
    const source = readFileSync(
      join(import.meta.dirname, "components/DeployedApparatus.svelte"),
      "utf8",
    );
    const drawn = new Set(
      [...source.matchAll(/tool === "([a-z]+)"/g)].map((match) => match[1]!),
    );
    // The burette is not an apparatus operator and has no assembly.
    drawn.delete("burette");
    for (const tool of APPARATUS_TOOLS) {
      expect(drawnOnStage(tool), `${tool}`).toBe(drawn.has(tool));
    }
    // The centrifuge is the one that has to fall back to words.
    expect(drawnOnStage("centrifuge")).toBe(false);
  });
});

describe("attention is reported in words as well as in colour", () => {
  it("names the part that needs it", () => {
    const unbalanced = assemblyFor("centrifuge", { sampleMass: 5, counterbalance: 4.8 });
    expect(assemblyAttention(unbalanced).map((item) => item.label))
      .toEqual(["counterbalance tube"]);
    expect(assemblyAttention(assemblyFor("centrifuge", { sampleMass: 5, counterbalance: 5 })))
      .toEqual([]);
  });

  it("asks for the solid a mortar has not been given", () => {
    expect(assemblyAttention(assemblyFor("grind", {})).map((item) => item.id)).toEqual(["solid"]);
    expect(assemblyAttention(assemblyFor("grind", { species: "nacl" }))).toEqual([]);
  });

  it("says nothing about a setup with nothing wrong with it", () => {
    for (const tool of ["bunsen", "regulate", "sweep", "heat", "cool", "stir"]) {
      expect(assemblyAttention(assemblyFor(tool, {})), tool).toEqual([]);
    }
  });
});
