import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  STOCKS,
  concentrationOptions,
  defaultConcentration,
  normaliseConcentration,
  solutionCommand,
} from "./solutions";

describe("stock solution commands", () => {
  it("defaults the shelf bottle to the common stock below lv3", () => {
    expect(defaultConcentration("lv1", "HCl")).toBe("37%");
    expect(defaultConcentration("lv2", "NaOH")).toBe("1M");
    expect(concentrationOptions("lv2", "HCl").map((o) => o.token)).not.toContain("");
    expect(concentrationOptions("lv3", "HCl").map((o) => o.token)).toContain("");
    expect(concentrationOptions("lv1", "NaCl")).toEqual([]);
  });

  it("emits the grammar the engine parses", () => {
    expect(solutionCommand(0, "HCl", "50mL", "37%")).toBe("add v1 HCl 50mL 37%");
    expect(solutionCommand(0, "HCl", "50mL", "37 % (w/w)")).toBe("add v1 HCl 50mL 37%");
    expect(solutionCommand(1, "NaOH", "100mL", "1 M")).toBe("add v2 NaOH 100mL 1M");
    expect(solutionCommand(0, "HCl", "0.1mol", "1M")).toBe("add v1 HCl 0.1mol as 1M");
    expect(solutionCommand(0, "H2O2", "20g", "30%w/w")).toBe("add v1 H2O2 20g 30%");
    expect(solutionCommand(0, "HCl", "1g", "")).toBe("add v1 HCl 1g");
  });

  it("refuses what is not a concentration", () => {
    for (const bad of ["120%", "0%", "-5%", "0M", "-1M", "1m", "abc", "%"]) {
      expect(normaliseConcentration(bad)).toBeNull();
      expect(() => solutionCommand(0, "HCl", "10mL", bad)).toThrow();
    }
  });

  it("mirrors every stock row in the engine's data file", () => {
    const toml = readFileSync(
      fileURLToPath(new URL("../../../../data/solutions/stock-solutions.toml", import.meta.url)),
      "utf8",
    );
    const rows = [...toml.matchAll(/\[\[stock\]\]\n([\s\S]*?)(?=\n\[\[|\s*$)/g)].map((m) => {
      const body = m[1]!;
      const get = (k: string) => new RegExp(`^${k} = "?([^"\\n]+)"?`, "m").exec(body)?.[1];
      const percent = get("percent");
      return {
        key: get("key"),
        species: get("species"),
        token: percent ? `${Number(percent)}%` : `${Number(get("molar"))}M`,
        common: get("common") === "true",
      };
    });
    const mirrored = Object.entries(STOCKS).flatMap(([species, options]) =>
      options.map((o) => ({ key: o.key, species, token: o.token, common: !!o.common })),
    );
    expect(mirrored.sort((a, b) => a.key!.localeCompare(b.key!))).toEqual(
      rows.sort((a, b) => a.key!.localeCompare(b.key!)),
    );
  });
});
