import { describe, expect, it } from "vitest";
import { comparisonRows } from "./comparisonResults";
import type { Scene } from "./host/EngineHost";
const scene = (temperatures: number[], masses = temperatures.map(() => 10)): Scene => ({ scene: 1, vessels: temperatures.map((temperature_k, id) => ({ id, label: `v${id + 1}`, liquid: null, solids: [], bubbling: false, boundary: "open", temperature_k, pressure_pa: 101325, elapsed_s: 2, mass_g: masses[id]!, words: "", badges: [] })) });
describe("authored comparison results", () => {
  it("evaluates an ordered multi-vessel trial", () => { expect(comparisonRows([{ kind: "increasing", samples: [{ step: "final", metric: "temperature_c", vessel: "v1" }, { step: "final", metric: "temperature_c", vessel: "v2" }] }], [{ step: "final", scene: scene([280, 300]) }])[0]?.holds).toBe(true); });
  it("compares conserved totals within tolerance", () => { expect(comparisonRows([{ kind: "conserved", tolerance: .01, samples: [{ step: "initial", metric: "mass_g" }, { step: "final", metric: "mass_g" }] }], [{ step: "initial", scene: scene([298], [20]) }, { step: "final", scene: scene([310], [20.005]) }])[0]?.holds).toBe(true); });
  it("does not invent unavailable values", () => { expect(comparisonRows([{ kind: "equal", samples: [{ step: "after:1", metric: "ph", vessel: "v1" }, { step: "final", metric: "ph", vessel: "v1" }] }], [{ step: "final", scene: scene([298]) }])[0]?.holds).toBeNull(); });
  it("follows vessel names onto fresh glassware", () => { expect(comparisonRows([{ kind: "increasing", samples: [{ step: "final", metric: "temperature_c", vessel: "v1" }, { step: "final", metric: "temperature_c", vessel: "v2" }] }], [{ step: "final", scene: scene([999, 999, 280, 300]) }], [3, 4], 2)[0]?.holds).toBe(true); });
  it("scopes fresh aggregate values to the script vessels", () => { expect(comparisonRows([{ kind: "equal", samples: [{ step: "final", metric: "mass_g" }, { step: "final", metric: "mass_g" }] }], [{ step: "final", scene: scene([298, 298, 298], [500, 10, 20]) }], [2, 3])[0]?.cells[0]?.value).toBe(30); });
  it("does not turn a missing named vessel into zero", () => { expect(comparisonRows([{ kind: "equal", samples: [{ step: "final", metric: "mass_g", vessel: "v9" }, { step: "final", metric: "mass_g", vessel: "v9" }] }], [{ step: "final", scene: scene([298]) }], [1])[0]?.holds).toBeNull(); });
  it("evaluates a two-sample ratio with absolute and relative tolerance", () => {
    expect(comparisonRows([{ kind: "ratio", ratio: 2, tolerance: 0, relative_tolerance: 0.01, samples: [{ step: "initial", metric: "mass_g" }, { step: "final", metric: "mass_g" }] }], [{ step: "initial", scene: scene([298], [10]) }, { step: "final", scene: scene([298], [20.1]) }])[0]?.holds).toBe(true);
  });
  it("applies relative tolerance to equality without changing its default", () => {
    expect(comparisonRows([{ kind: "equal", tolerance: 0, relative_tolerance: 0.01, samples: [{ step: "initial", metric: "mass_g" }, { step: "final", metric: "mass_g" }] }], [{ step: "initial", scene: scene([298], [100]) }, { step: "final", scene: scene([298], [100.5]) }])[0]?.holds).toBe(true);
  });
  it("does not invent event or species values absent from Scene v1", () => {
    const rows = comparisonRows([
      { kind: "decreasing", tolerance: 0.0001, samples: [{ step: "after:1", metric: "event:cell_voltage.volts" }, { step: "final", metric: "event:cell_voltage.volts" }] },
      { kind: "ratio", ratio: 2, samples: [{ step: "initial", metric: "moles:water", vessel: "v1" }, { step: "final", metric: "moles:water", vessel: "v1" }] },
    ], [{ step: "after:1", scene: scene([298]) }, { step: "initial", scene: scene([298]) }, { step: "final", scene: scene([298]) }]);
    expect(rows.map((row) => row.holds)).toEqual([null, null]);
    expect(rows[0]?.cells.every((cell) => cell.value === null)).toBe(true);
  });
});
