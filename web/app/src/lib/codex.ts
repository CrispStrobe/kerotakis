/**
 * Consumer types for the codex export (the serde of kerotakis-codex's
 * structs — Rust is the source of truth) and the honest checker: it never
 * computes chemistry, it COMPARES the engine's actual events and final
 * state against the entry's committed claims. Loaded from
 * `codex/index.json` beside the payload; quiet absence until the export
 * ships.
 */

export interface CodexRange {
  min?: number;
  max?: number;
}

export interface CodexDiagnosis {
  option: number;
  reveals: string;
  next?: string;
}

export interface CodexPrediction {
  question: string;
  options: string[];
  answer: number;
  misconception?: string | null;
  diagnosis?: CodexDiagnosis[];
}

export interface CodexExpect {
  events?: string[];
  absent?: string[];
  predict?: CodexPrediction | null;
  ph?: CodexRange | null;
  temperature_c?: CodexRange | null;
}

export interface CodexEntry {
  id: string;
  equation?: string | null;
  summary?: string | null;
  concepts?: string[];
  requires?: string[];
  apparatus?: string[];
  calculations?: string[];
  models?: string[];
  setup: { script: string };
  expect: CodexExpect;
  registers: Record<string, string>;
}

export function parseCodexIndex(raw: unknown): CodexEntry[] {
  const list = Array.isArray(raw)
    ? raw
    : ((raw as { entries?: unknown[] })?.entries ?? []);
  return (list as CodexEntry[]).filter(
    (e) => typeof e?.id === "string" && typeof e?.setup?.script === "string",
  );
}

export interface CheckResult {
  events: { want: string; seen: boolean }[];
  forbidden: { want: string; violated: boolean }[];
  ph: { range: CodexRange; value: number | null; ok: boolean } | null;
  temperature_c: { range: CodexRange; value: number | null; ok: boolean } | null;
  allOk: boolean;
}

const inRange = (v: number, r: CodexRange) =>
  (r.min === undefined || v >= r.min) && (r.max === undefined || v <= r.max);

/**
 * Compare what the engine DID against what the entry claims. `observed`
 * carries both bare tags and `tag:species` keys from the run's typed
 * events; scene values come from the final render model.
 */
export function checkExpect(
  expect: CodexExpect,
  observed: string[],
  scene: { phValues: number[]; temperaturesC: number[] },
): CheckResult {
  const seen = new Set(observed);
  const events = (expect.events ?? []).map((want) => ({
    want,
    seen: seen.has(want),
  }));
  const forbidden = (expect.absent ?? []).map((want) => ({
    want,
    violated: seen.has(want),
  }));
  const check = (range: CodexRange | null | undefined, values: number[]) => {
    if (!range) return null;
    // The claim is unscoped to a vessel; any vessel satisfying it counts.
    const value = values.find((v) => inRange(v, range)) ?? values[0] ?? null;
    return { range, value, ok: values.some((v) => inRange(v, range)) };
  };
  const ph = check(expect.ph, scene.phValues);
  const temperature_c = check(expect.temperature_c, scene.temperaturesC);
  const allOk =
    events.every((e) => e.seen) &&
    forbidden.every((f) => !f.violated) &&
    (ph === null || ph.ok) &&
    (temperature_c === null || temperature_c.ok);
  return { events, forbidden, ph, temperature_c, allOk };
}
