import { describe, expect, it } from "vitest";
import { mixLine, twoVesselLine, vesselQuickActions } from "./directActions";

describe("direct vessel actions", () => {
  it("targets the selected vessel and follows its boundary", () => {
    const open = vesselQuickActions(2, "open");
    expect(open.find((a) => a.id === "stir")?.line).toBe("stir v3");
    expect(open.find((a) => a.id === "seal")?.line).toBe("seal v3 500mL");
    expect(vesselQuickActions(2, "sealed").find((a) => a.id === "open")?.line).toBe("open v3");
  });

  it("compiles direct pours and transfer tools to public grammar", () => {
    expect(twoVesselLine("decant", 0, 2, 0.75)).toBe("decant v1 v3 0.75");
    expect(twoVesselLine("filter", 1, 0)).toBe("filter v2 v1");
    expect(twoVesselLine("distil", 3, 1, 1)).toBe("distil v4 v2 1");
  });

  it("refuses impossible targets and fractions before dispatch", () => {
    expect(twoVesselLine("decant", 0, 0, 0.5)).toBeNull();
    expect(twoVesselLine("decant", 0, 1, 0)).toBeNull();
    expect(twoVesselLine("distil", 0, 1, 1.1)).toBeNull();
  });

  it("compiles three distinct vessel taps to the public MIX grammar", () => {
    expect(mixLine(0, 1, 2)).toBe("mix v1 0.5 v2 0.5 into v3");
    expect(mixLine(0, 1, 2, 0.25, 0.75)).toBe("mix v1 0.25 v2 0.75 into v3");
    expect(mixLine(0, 0, 2)).toBeNull();
    expect(mixLine(0, 1, 2, 0, 0.5)).toBeNull();
  });
});
