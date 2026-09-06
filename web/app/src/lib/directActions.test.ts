import { describe, expect, it } from "vitest";
import { heatSource } from "./apparatus";
import { DOCK_INSTRUMENTS, mixLine, twoVesselLine, vesselQuickActions } from "./directActions";

describe("direct vessel actions", () => {
  it("targets the selected vessel and follows its boundary", () => {
    const open = vesselQuickActions(2, "open");
    expect(open.find((a) => a.id === "stir")?.line).toBe("stir v3 500rpm 10s");
    expect(open.find((a) => a.id === "seal")?.line).toBe("seal v3 500mL");
    expect(vesselQuickActions(2, "sealed").find((a) => a.id === "open")?.line).toBe("open v3");
  });

  it("names the heat source even when it is the bench default", () => {
    // The defect: `heat v3 10kJ` with no `on <source>` clause. The engine
    // caps a vessel at the flame heating it and falls back to a laboratory
    // burner when the clause is missing — so the line claimed a burner by
    // omission, which is the one thing the kids' candle is not. Stated from
    // the same table `ApparatusForm` builds its own line from.
    expect(vesselQuickActions(2, "open").find((a) => a.id === "heat")?.line)
      .toBe(`heat v3 10kJ on ${heatSource(undefined).value}`);
    expect(heatSource(undefined).value).toBe("burner");
    // `cool` takes no source: the grammar has no clause to name one with,
    // so a bare line is complete rather than silently defaulted.
    expect(vesselQuickActions(2, "open").find((a) => a.id === "cool")?.line).toBe("cool v3 10kJ");
  });

  it("keeps the dock's three readings, and says which they are", () => {
    // Decision 2: these stay hard-coded because a landmark that moves is
    // not a landmark. The constant is what the quick-access strip excludes.
    const lines = vesselQuickActions(0, "open").filter((a) => a.line.startsWith("measure "));
    expect(lines.map((a) => a.line)).toEqual(DOCK_INSTRUMENTS.map((token) => `measure v1 ${token}`));
  });

  it("compiles direct pours and transfer tools to public grammar", () => {
    expect(twoVesselLine("decant", 0, 2, 0.75)).toBe("decant v1 v3 0.75");
    expect(twoVesselLine("filter", 1, 0)).toBe("filter v2 v1");
    expect(twoVesselLine("magnet", 1, 3)).toBe("magnet v2 v4");
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
