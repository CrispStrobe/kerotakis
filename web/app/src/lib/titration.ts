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
