/**
 * The burette's half of GUI-033: turn a filled-in form into the exact
 * `titrate` line the grammar speaks —
 * `titrate v1 NaOH 1M 1mL until ph 7 [max N]`.
 * The burette holds a STANDARD SOLUTION; molarity states its
 * concentration, as printed on every real bottle.
 */

export interface TitrationSetup {
  vessel: number;
  titrant: string;
  molarity: number;
  incrementMl: number;
  targetPh: number;
  maxSteps?: number;
}

export function buildTitrateLine(s: TitrationSetup): string | null {
  if (!s.titrant.trim()) return null;
  if (!(s.molarity > 0) || !(s.incrementMl > 0)) return null;
  if (!Number.isFinite(s.targetPh)) return null;
  const max = s.maxSteps && s.maxSteps > 0 ? ` max ${Math.floor(s.maxSteps)}` : "";
  return (
    `titrate v${s.vessel + 1} ${s.titrant.trim()} ` +
    `${s.molarity}M ${s.incrementMl}mL until ph ${s.targetPh}${max}`
  );
}

/**
 * The column train (transport): cells in order, an inlet, a receiver,
 * and the step count —
 * `transport v1 v2 from v3 to v4 steps 5 [courant f]`.
 */
export interface TransportSetup {
  cells: number[];
  inlet: number;
  receiver: number;
  steps: number;
  courant?: number;
}

export function buildTransportLine(s: TransportSetup): string | null {
  if (s.cells.length === 0 || !(s.steps > 0)) return null;
  const all = [...s.cells, s.inlet, s.receiver];
  if (new Set(all).size !== all.length) return null; // a vessel plays one role
  const cells = s.cells.map((c) => `v${c + 1}`).join(" ");
  const courant =
    s.courant !== undefined && s.courant > 0 && s.courant <= 1
      ? ` courant ${s.courant}`
      : "";
  return `transport ${cells} from v${s.inlet + 1} to v${s.receiver + 1} steps ${Math.floor(s.steps)}${courant}`;
}
