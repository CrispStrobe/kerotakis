export const VIEW_LIMITS = Object.freeze({
  maxAtoms: 20_000,
  maxBonds: 40_000,
  maxFrames: 120,
  maxGridPoints: 262_144,
  maxSourceBytes: 2_000_000,
  maxCoordinateMagnitude: 1_000_000,
  maxWidth: 1_280,
  maxHeight: 960,
  maxDpr: 2,
});

export type FixtureKind = "molecule" | "crystal" | "protein" | "orbital" | "trajectory";

export interface SemanticAtom {
  id: number;
  element: string;
  x: number;
  y: number;
  z: number;
  label?: string;
}

export interface SemanticBond {
  from: number;
  to: number;
  order?: number;
}

export interface ViewerFixture {
  id: string;
  kind: FixtureKind;
  format: string;
  text: string;
  description: string;
  atoms: readonly SemanticAtom[];
  bonds: readonly SemanticBond[];
  frameCount?: number;
  gridPointCount?: number;
  unitCell?: readonly [number, number, number, number, number, number];
}

export interface ViewerOptions {
  labelsVisible: boolean;
  reducedMotion: boolean;
}

export interface ViewerSnapshot {
  candidate: string;
  fixture: string;
  selectedAtomIds: readonly number[];
  labelsVisible: boolean;
  width: number;
  height: number;
  dpr: number;
  status: "ready" | "unsupported" | "error" | "disposed";
}

export interface ViewerSession {
  setLabels(visible: boolean): void | Promise<void>;
  select(atomIds: readonly number[]): void | Promise<void>;
  resize(width: number, height: number, dpr: number): void | Promise<void>;
  snapshot(): ViewerSnapshot;
  dispose(): void | Promise<void>;
}

export interface CandidateAdapter {
  id: string;
  label: string;
  supports(kind: FixtureKind): boolean;
  mount(host: HTMLElement, fixture: ViewerFixture, options: ViewerOptions): Promise<ViewerSession>;
}

export type AdapterErrorCode =
  | "unsupported-fixture"
  | "invalid-fixture"
  | "resource-limit"
  | "renderer-unavailable"
  | "renderer-failed";

export class AdapterError extends Error {
  constructor(public readonly code: AdapterErrorCode, message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "AdapterError";
  }
}

function finite(value: number): boolean {
  return Number.isFinite(value);
}

export function validateFixture(fixture: ViewerFixture): void {
  if (!fixture.id || !fixture.format || !fixture.description || !fixture.text) {
    throw new AdapterError("invalid-fixture", "Fixture identity, format, description and source text are required.");
  }
  if (new TextEncoder().encode(fixture.text).byteLength > VIEW_LIMITS.maxSourceBytes) {
    throw new AdapterError("resource-limit", "Fixture exceeds the source-byte limit.");
  }
  if (fixture.atoms.length > VIEW_LIMITS.maxAtoms || fixture.bonds.length > VIEW_LIMITS.maxBonds) {
    throw new AdapterError("resource-limit", "Fixture exceeds the atom or bond limit.");
  }
  if ((fixture.frameCount ?? 1) < 1 || (fixture.frameCount ?? 1) > VIEW_LIMITS.maxFrames) {
    throw new AdapterError("resource-limit", "Fixture exceeds the trajectory frame limit.");
  }
  if ((fixture.gridPointCount ?? 0) < 0 || (fixture.gridPointCount ?? 0) > VIEW_LIMITS.maxGridPoints) {
    throw new AdapterError("resource-limit", "Fixture exceeds the volume grid limit.");
  }
  const ids = new Set<number>();
  for (const atom of fixture.atoms) {
    if (!Number.isSafeInteger(atom.id) || atom.id < 0 || !atom.element || ![atom.x, atom.y, atom.z].every(finite)
      || [atom.x, atom.y, atom.z].some((value) => Math.abs(value) > VIEW_LIMITS.maxCoordinateMagnitude)) {
      throw new AdapterError("invalid-fixture", "Every atom needs a unique non-negative id, element and finite coordinates.");
    }
    if (ids.has(atom.id)) throw new AdapterError("invalid-fixture", "Atom ids must be unique.");
    ids.add(atom.id);
  }
  for (const bond of fixture.bonds) {
    if (!ids.has(bond.from) || !ids.has(bond.to) || bond.from === bond.to
      || (bond.order !== undefined && (!finite(bond.order) || bond.order <= 0 || bond.order > 4))) {
      throw new AdapterError("invalid-fixture", "Every bond must join two distinct fixture atoms.");
    }
    if (bond.order !== undefined && (!finite(bond.order) || bond.order <= 0)) {
      throw new AdapterError("invalid-fixture", "Bond orders must be finite and positive when supplied.");
    }
  }
  if (fixture.unitCell && (!fixture.unitCell.every(finite)
    || fixture.unitCell.slice(0, 3).some((value) => value <= 0 || value > VIEW_LIMITS.maxCoordinateMagnitude)
    || fixture.unitCell.slice(3).some((value) => value <= 0 || value >= 180))) {
    throw new AdapterError("invalid-fixture", "Unit-cell lengths and angles must be finite and positive.");
  }
}

export function boundedViewport(width: number, height: number, dpr: number) {
  const safe = (value: number, maximum: number) => Number.isFinite(value) ? Math.min(maximum, Math.max(1, Math.round(value))) : 1;
  return { width: safe(width, VIEW_LIMITS.maxWidth), height: safe(height, VIEW_LIMITS.maxHeight), dpr: Number.isFinite(dpr) ? Math.min(VIEW_LIMITS.maxDpr, Math.max(1, dpr)) : 1 };
}
