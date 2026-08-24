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

import { buildTransportLine } from "./titration";

describe("the column train compiles to the grammar", () => {
  it("builds the transport line in role order", () => {
    expect(
      buildTransportLine({ cells: [0, 1, 2], inlet: 3, receiver: 4, steps: 3 }),
    ).toBe("transport v1 v2 v3 from v4 to v5 steps 3");
    expect(
      buildTransportLine({ cells: [1], inlet: 0, receiver: 2, steps: 5, courant: 0.5 }),
    ).toBe("transport v2 from v1 to v3 steps 5 courant 0.5");
  });

  it("refuses role collisions and empty trains", () => {
    expect(buildTransportLine({ cells: [], inlet: 0, receiver: 1, steps: 3 })).toBeNull();
    expect(buildTransportLine({ cells: [0], inlet: 0, receiver: 1, steps: 3 })).toBeNull();
    expect(buildTransportLine({ cells: [0], inlet: 1, receiver: 2, steps: 0 })).toBeNull();
  });
});
