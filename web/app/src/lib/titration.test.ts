import { describe, expect, it } from "vitest";
import { buildTitrateLine } from "./titration";

describe("the burette compiles to the grammar", () => {
  it("builds the canonical titrate line", () => {
    expect(
      buildTitrateLine({
        vessel: 0,
        titrant: "NaOH",
        molarity: 1,
        incrementMl: 1,
        targetPh: 7,
      }),
    ).toBe("titrate v1 NaOH 1M 1mL until ph 7");
    expect(
      buildTitrateLine({
        vessel: 1,
        titrant: "HCl",
        molarity: 0.1,
        incrementMl: 0.5,
        targetPh: 4.5,
        maxSteps: 200,
      }),
    ).toBe("titrate v2 HCl 0.1M 0.5mL until ph 4.5 max 200");
  });

  it("refuses nonsense rather than emitting a broken line", () => {
    const base = { vessel: 0, titrant: "NaOH", molarity: 1, incrementMl: 1, targetPh: 7 };
    expect(buildTitrateLine({ ...base, titrant: "  " })).toBeNull();
    expect(buildTitrateLine({ ...base, molarity: 0 })).toBeNull();
    expect(buildTitrateLine({ ...base, incrementMl: -1 })).toBeNull();
    expect(buildTitrateLine({ ...base, targetPh: NaN })).toBeNull();
  });
});
