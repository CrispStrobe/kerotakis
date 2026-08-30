/**
 * BRD-071 input boundary for presentation-only rigid-body prototypes.
 *
 * Input devices compile to this small, versioned wire shape before a physics
 * implementation sees them. The intent contains no chemistry-bearing state:
 * it may arrange a scene object or become evidence for a BRD-070 proposal,
 * but it cannot edit amounts, phases, temperature, pressure, or events.
 */

export const RIGID_INTENT_VERSION = 1 as const;
export const RIGID_COORDINATE_SCALE = 10_000;

export type RigidInputSource = "mouse" | "touch" | "pen" | "keyboard";
export type RigidMotionPolicy = "animated" | "reduced_motion" | "headless" | "background";

export interface RigidBounds {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

export interface RigidPosition {
  x: number;
  y: number;
}

/** Integer coordinates make equality and replay independent of device floats. */
export interface RigidMoveIntent {
  version: typeof RIGID_INTENT_VERSION;
  action: "move_to";
  object: string;
  x: number;
  y: number;
}

export type RigidIntentResult =
  | { ok: true; intent: RigidMoveIntent }
  | { ok: false; reason: string };

export interface RigidPresentationFrame {
  x: number;
  y: number;
}

const finite = (value: number) => Number.isFinite(value);

function validateBounds(bounds: RigidBounds): string | null {
  if (![bounds.minX, bounds.maxX, bounds.minY, bounds.maxY].every(finite)) {
    return "rigid bounds must be finite";
  }
  if (bounds.minX > bounds.maxX || bounds.minY > bounds.maxY) {
    return "rigid bounds are inverted";
  }
  return null;
}

function quantize(value: number): number {
  return Math.round(value * RIGID_COORDINATE_SCALE);
}

export function intentPosition(intent: RigidMoveIntent): RigidPosition {
  return {
    x: intent.x / RIGID_COORDINATE_SCALE,
    y: intent.y / RIGID_COORDINATE_SCALE,
  };
}

/**
 * Compile any absolute pointing-device endpoint to the canonical wire action.
 * `source` is deliberately consumed but not serialized: equal gestures from
 * mouse, touch, pen, and assistive adapters must replay identically.
 */
export function compilePointMove(
  source: Exclude<RigidInputSource, "keyboard">,
  object: string,
  position: RigidPosition,
  bounds: RigidBounds,
): RigidIntentResult {
  if (!(["mouse", "touch", "pen"] as const).includes(source)) {
    return { ok: false, reason: "unsupported pointing-device source" };
  }
  return compileMove(object, position, bounds);
}

/** Compile a keyboard delta to the same absolute action used by pointers. */
export function compileKeyboardMove(
  object: string,
  current: RigidPosition,
  delta: RigidPosition,
  bounds: RigidBounds,
): RigidIntentResult {
  if (![current.x, current.y, delta.x, delta.y].every(finite)) {
    return { ok: false, reason: "rigid coordinates must be finite" };
  }
  return compileMove(object, { x: current.x + delta.x, y: current.y + delta.y }, bounds);
}

function compileMove(
  object: string,
  position: RigidPosition,
  bounds: RigidBounds,
): RigidIntentResult {
  const id = object.trim();
  if (!id) return { ok: false, reason: "rigid object identity is required" };
  const boundsError = validateBounds(bounds);
  if (boundsError) return { ok: false, reason: boundsError };
  if (![position.x, position.y].every(finite)) {
    return { ok: false, reason: "rigid coordinates must be finite" };
  }
  if (
    position.x < bounds.minX || position.x > bounds.maxX
    || position.y < bounds.minY || position.y > bounds.maxY
  ) {
    return { ok: false, reason: "rigid endpoint is outside its declared bounds" };
  }
  return {
    ok: true,
    intent: {
      version: RIGID_INTENT_VERSION,
      action: "move_to",
      object: id,
      x: quantize(position.x),
      y: quantize(position.y),
    },
  };
}

/**
 * Produce optional painting frames. The authoritative endpoint is always the
 * intent itself, so frame count and motion preference cannot change replay.
 */
export function presentationFrames(
  policy: RigidMotionPolicy,
  from: RigidPosition,
  intent: RigidMoveIntent,
  intermediateCount = 3,
): RigidPresentationFrame[] {
  if (policy !== "animated") return [];
  if (![from.x, from.y].every(finite) || !Number.isInteger(intermediateCount) || intermediateCount < 0) {
    return [];
  }
  const to = intentPosition(intent);
  return Array.from({ length: intermediateCount }, (_, index) => {
    const progress = (index + 1) / (intermediateCount + 1);
    return {
      x: from.x + (to.x - from.x) * progress,
      y: from.y + (to.y - from.y) * progress,
    };
  });
}
