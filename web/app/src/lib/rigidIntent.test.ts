import { describe, expect, it } from "vitest";
import {
  compileKeyboardMove,
  compilePointMove,
  intentPosition,
  presentationFrames,
  RIGID_COORDINATE_SCALE,
  type RigidBounds,
} from "./rigidIntent";

const BOUNDS: RigidBounds = { minX: 0.08, maxX: 0.92, minY: 0.12, maxY: 0.88 };

describe("BRD-071 canonical rigid intents", () => {
  it("compiles mouse, touch, and pen endpoints to byte-identical actions", () => {
    const endpoint = { x: 0.347_891, y: 0.612_349 };
    const actions = (["mouse", "touch", "pen"] as const).map((source) =>
      compilePointMove(source, "vessel:v1", endpoint, BOUNDS),
    );
    expect(actions.every((result) => result.ok)).toBe(true);
    expect(actions[1]).toEqual(actions[0]);
    expect(actions[2]).toEqual(actions[0]);
    expect(JSON.stringify(actions[0])).toBe(JSON.stringify(actions[2]));
    expect(actions[0]).toEqual({
      ok: true,
      intent: { version: 1, action: "move_to", object: "vessel:v1", x: 3479, y: 6123 },
    });
  });

  it("compiles keyboard nudges to the same canonical endpoint as pointing input", () => {
    const keyboard = compileKeyboardMove(
      "vessel:v1",
      { x: 0.3, y: 0.6 },
      { x: 0.05, y: -0.06 },
      BOUNDS,
    );
    const touch = compilePointMove("touch", "vessel:v1", { x: 0.35, y: 0.54 }, BOUNDS);
    expect(keyboard).toEqual(touch);
  });

  it("refuses malformed identities, coordinates, deltas, and bounds", () => {
    expect(compilePointMove("mouse", " ", { x: 0.5, y: 0.5 }, BOUNDS)).toMatchObject({ ok: false });
    expect(compilePointMove("mouse", "vessel:v1", { x: Number.NaN, y: 0.5 }, BOUNDS)).toMatchObject({ ok: false });
    expect(compilePointMove("mouse", "vessel:v1", { x: 0.01, y: 0.5 }, BOUNDS)).toMatchObject({ ok: false });
    expect(compileKeyboardMove("vessel:v1", { x: 0.5, y: 0.5 }, { x: Infinity, y: 0 }, BOUNDS)).toMatchObject({ ok: false });
    expect(compilePointMove("pen", "vessel:v1", { x: 0.5, y: 0.5 }, { ...BOUNDS, maxX: Number.NaN })).toMatchObject({ ok: false });
    expect(compilePointMove("touch", "vessel:v1", { x: 0.5, y: 0.5 }, { ...BOUNDS, minY: 0.9 })).toMatchObject({ ok: false });
  });

  it("quantizes device floats to stable integer coordinates", () => {
    const result = compilePointMove("pen", "apparatus:mortar", { x: 0.500_049, y: 0.499_951 }, BOUNDS);
    expect(result).toMatchObject({ ok: true });
    if (!result.ok) throw new Error(result.reason);
    expect(Number.isInteger(result.intent.x)).toBe(true);
    expect(result.intent).toMatchObject({ x: RIGID_COORDINATE_SCALE / 2, y: RIGID_COORDINATE_SCALE / 2 });
    expect(intentPosition(result.intent)).toEqual({ x: 0.5, y: 0.5 });
  });

  it("changes painting frames with motion policy but never the endpoint action", () => {
    const result = compilePointMove("mouse", "vessel:v1", { x: 0.7, y: 0.4 }, BOUNDS);
    if (!result.ok) throw new Error(result.reason);
    const before = JSON.stringify(result.intent);

    expect(presentationFrames("animated", { x: 0.2, y: 0.6 }, result.intent, 4)).toHaveLength(4);
    expect(presentationFrames("reduced_motion", { x: 0.2, y: 0.6 }, result.intent, 4)).toEqual([]);
    expect(presentationFrames("headless", { x: 0.2, y: 0.6 }, result.intent, 4)).toEqual([]);
    expect(presentationFrames("background", { x: 0.2, y: 0.6 }, result.intent, 4)).toEqual([]);
    expect(JSON.stringify(result.intent)).toBe(before);
    expect(intentPosition(result.intent)).toEqual({ x: 0.7, y: 0.4 });
  });
});
