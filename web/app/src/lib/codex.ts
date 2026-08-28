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
  reveals_de?: string;
  next?: string;
  next_de?: string;
}

export interface CodexPrediction {
  question: string;
  question_de?: string;
  options: string[];
  /** Positional twin of `options`; same length or ignored. */
  options_de?: string[];
  answer: number;
  misconception?: string | null;
  misconception_de?: string | null;
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
  summary_de?: string | null;
  concepts?: string[];
  requires?: string[];
  apparatus?: string[];
  calculations?: string[];
  models?: string[];
  setup: { script: string };
  expect: CodexExpect;
  registers: Record<string, string>;
  curriculum?: CodexPlacement[];
}

/** One curriculum's placement of an entry (kerotakis-codex::Placement). */
export interface CodexPlacement {
  system: string;
  stage: string;
  ages?: CodexRange | null;
  source: string;
}

export function scriptKit(script: string): string[] {
  const kit = new Set<string>();
  for (const line of script.split("\n")) {
    const m = line.trim().match(/^(?:add|titrate|grind)\s+\S+\s+(\S+)/);
    if (m) kit.add(m[1]!);
  }
  return [...kit];
}

export function parseCodexIndex(raw: unknown): CodexEntry[] {
  // The export's document shape: `{ reactions, models, concepts }`
  // (kero codex export); older spellings tolerated.
  const doc = raw as { reactions?: unknown[]; entries?: unknown[] } | unknown[];
  const list = Array.isArray(doc) ? doc : (doc?.reactions ?? doc?.entries ?? []);
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

// ── Grouping for the browsers (GUI-053 concepts / GUI-055 curriculum) ──

/** Concepts with usage counts, most-taught first; ties alphabetical. */
export function conceptIndex(entries: CodexEntry[]): { concept: string; count: number }[] {
  const counts = new Map<string, number>();
  for (const e of entries) {
    for (const c of e.concepts ?? []) counts.set(c, (counts.get(c) ?? 0) + 1);
  }
  return [...counts.entries()]
    .map(([concept, count]) => ({ concept, count }))
    .sort((a, b) => b.count - a.count || a.concept.localeCompare(b.concept));
}

/** Concepts that co-occur with `concept`, strongest neighbours first. */
export function relatedConcepts(entries: CodexEntry[], concept: string): string[] {
  const counts = new Map<string, number>();
  for (const e of entries) {
    const cs = e.concepts ?? [];
    if (!cs.includes(concept)) continue;
    for (const c of cs) {
      if (c !== concept) counts.set(c, (counts.get(c) ?? 0) + 1);
    }
  }
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([c]) => c);
}

export interface CurriculumStage {
  stage: string;
  /** The citation(s) behind the placement claims — shown, never hidden. */
  sources: string[];
  entries: CodexEntry[];
}

/**
 * Placements grouped system → stage. Stages order by their age band's
 * lower bound where stated, then by name — cross-system stage names
 * ("KS3", "Jgst. 9") carry no comparable order of their own.
 */
export function curriculumIndex(
  entries: CodexEntry[],
): { system: string; stages: CurriculumStage[] }[] {
  const systems = new Map<string, Map<string, { ageMin: number; sources: Set<string>; entries: CodexEntry[] }>>();
  for (const e of entries) {
    for (const p of e.curriculum ?? []) {
      const stages = systems.get(p.system) ?? new Map();
      systems.set(p.system, stages);
      const s = stages.get(p.stage) ?? { ageMin: Infinity, sources: new Set<string>(), entries: [] };
      stages.set(p.stage, s);
      if (typeof p.ages?.min === "number") s.ageMin = Math.min(s.ageMin, p.ages.min);
      if (p.source) s.sources.add(p.source);
      if (!s.entries.includes(e)) s.entries.push(e);
    }
  }
  return [...systems.entries()]
    .map(([system, stages]) => ({
      system,
      stages: [...stages.entries()]
        .sort(([an, a], [bn, b]) => a.ageMin - b.ageMin || an.localeCompare(bn))
        .map(([stage, s]) => ({ stage, sources: [...s.sources], entries: s.entries })),
    }))
    .sort((a, b) => a.system.localeCompare(b.system));
}

// ── The concept graph (GUI-053) ────────────────────────────────────────

export interface ConceptNode {
  concept: string;
  /** How many entries teach it. */
  count: number;
  /** Longest prerequisite chain below it — the map's column. */
  depth: number;
}

export interface ConceptGraph {
  nodes: ConceptNode[];
  /** from = prerequisite concept, to = concept it unlocks. */
  edges: { from: string; to: string }[];
}

/**
 * Concepts as a layered DAG. Edges come from the entries themselves:
 * every concept an entry requires points at every concept it teaches.
 * Depth is longest-path layering; a cycle (bad content, not bad code)
 * parks the affected concepts at depth 0 rather than looping.
 */
export function conceptGraph(entries: CodexEntry[]): ConceptGraph {
  const counts = new Map<string, number>();
  const edgeSet = new Set<string>();
  const preds = new Map<string, Set<string>>();
  for (const e of entries) {
    for (const c of e.concepts ?? []) {
      counts.set(c, (counts.get(c) ?? 0) + 1);
      for (const r of e.requires ?? []) {
        if (r === c) continue;
        if (!counts.has(r)) counts.set(r, 0);
        edgeSet.add(`${r}→${c}`);
        let ps = preds.get(c);
        if (!ps) {
          ps = new Set();
          preds.set(c, ps);
        }
        ps.add(r);
      }
    }
  }
  const depthOf = new Map<string, number>();
  const visiting = new Set<string>();
  const depth = (c: string): number => {
    const known = depthOf.get(c);
    if (known !== undefined) return known;
    if (visiting.has(c)) return 0; // cycle guard
    visiting.add(c);
    const ps = [...(preds.get(c) ?? [])];
    const d = ps.length === 0 ? 0 : 1 + Math.max(...ps.map(depth));
    visiting.delete(c);
    depthOf.set(c, d);
    return d;
  };
  const nodes = [...counts.entries()]
    .map(([concept, count]) => ({ concept, count, depth: depth(concept) }))
    .sort((a, b) => a.depth - b.depth || a.concept.localeCompare(b.concept));
  const edges = [...edgeSet].map((k) => {
    const [from, to] = k.split("→");
    return { from: from!, to: to! };
  });
  return { nodes, edges };
}

/**
 * Which concepts a learner has met: those taught by any entry whose run
 * checked out (`done` holds entry ids). Honest by construction — nothing
 * is met by reading, only by a bench run that agreed with the claims.
 */
export function metConcepts(entries: CodexEntry[], done: ReadonlySet<string>): Set<string> {
  const met = new Set<string>();
  for (const e of entries) {
    if (!done.has(e.id)) continue;
    for (const c of e.concepts ?? []) met.add(c);
  }
  return met;
}

/** An entry is ready when everything it requires has been met. */
export function entryReady(e: CodexEntry, met: ReadonlySet<string>): boolean {
  return (e.requires ?? []).every((r) => met.has(r));
}
