import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { AdapterError, boundedViewport, validateFixture, type CandidateAdapter, type ViewerFixture } from "./adapter";
import { ComparisonController } from "./comparison";

const fixture: ViewerFixture = { id: "water", kind: "molecule", format: "sdf", text: "fixture", description: "Water", atoms: [{ id: 0, element: "O", x: 0, y: 0, z: 0 }], bonds: [] };

describe("comparison contract", () => {
  const host = () => ({
    replaceChildren() {},
    getBoundingClientRect: () => ({ width: 640, height: 480 }),
  }) as unknown as HTMLElement;
  it("bounds hostile viewports", () => expect(boundedViewport(Infinity, 50_000, 9)).toEqual({ width: 1, height: 960, dpr: 2 }));
  it("rejects dangling bonds", () => expect(() => validateFixture({ ...fixture, bonds: [{ from: 0, to: 1 }] })).toThrow(AdapterError));
  it("rejects hostile coordinates, bond orders and unit-cell angles", () => {
    expect(() => validateFixture({ ...fixture, atoms: [{ ...fixture.atoms[0], x: 1_000_001 }] })).toThrow(/finite coordinates/);
    expect(() => validateFixture({ ...fixture, atoms: [...fixture.atoms, { id: 1, element: "H", x: 0, y: 0, z: 0 }], bonds: [{ from: 0, to: 1, order: 9 }] })).toThrow(/bond/);
    expect(() => validateFixture({ ...fixture, unitCell: [1, 1, 1, 90, 180, 90] })).toThrow(/Unit-cell/);
  });
  it("rejects oversized source input before a renderer sees it", () => expect(() => validateFixture({ ...fixture, text: "x".repeat(2_000_001) })).toThrow(/source-byte limit/));
  it("reports unsupported fixtures without mounting", async () => {
    const updates = vi.fn();
    const adapter: CandidateAdapter = { id: "test", label: "Test", supports: () => false, mount: vi.fn() };
    const controller = new ComparisonController(host(), updates);
    await controller.show(adapter, fixture, true);
    expect(controller.status.state).toBe("unsupported");
    expect(adapter.mount).not.toHaveBeenCalled();
  });
  it("disposes the prior renderer before replacement", async () => {
    const dispose = vi.fn();
    const adapter: CandidateAdapter = { id: "test", label: "Test", supports: () => true, mount: vi.fn(async () => ({ setLabels() {}, select() {}, resize() {}, snapshot: () => ({ candidate: "test", fixture: "water", selectedAtomIds: [], labelsVisible: false, width: 1, height: 1, dpr: 1, status: "ready" }), dispose })) };
    const controller = new ComparisonController(host(), () => {});
    await controller.show(adapter, fixture, false);
    await controller.show(adapter, fixture, false);
    expect(dispose).toHaveBeenCalledTimes(1);
  });

  it("keeps the semantic table, keyboard-native choices and reduced-motion control in the route", () => {
    const source = readFileSync(new URL("./App.svelte", import.meta.url), "utf8");
    const styles = readFileSync(new URL("./style.css", import.meta.url), "utf8");
    expect(source).toContain('id="semantic-view"');
    expect(source).toContain("<table>");
    expect(source).toContain('type="radio" name="candidate"');
    expect(source).toContain('id="reduce-motion"');
    expect(source).toContain("neither authoritative chemistry");
    expect(styles).toContain("prefers-reduced-motion: reduce");
  });
});
