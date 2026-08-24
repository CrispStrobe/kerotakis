/**
 * Parse a relation's arg-spec string ("e0=<V> n=<electrons> [b=<x>]")
 * into form fields. The free-form pair syntax (ionic-strength's
 * "<z>:<m> ..." style) yields a single free-text field instead.
 */
export interface RelationField {
  name: string;
  hint: string;
  optional: boolean;
}

export function parseArgSpec(spec: string): { fields: RelationField[]; freeform: boolean } {
  const tokens = spec.split(/\s+/).filter(Boolean);
  const fields: RelationField[] = [];
  for (const raw of tokens) {
    const optional = raw.startsWith("[") && raw.endsWith("]");
    const token = optional ? raw.slice(1, -1) : raw;
    const m = token.match(/^([A-Za-z][A-Za-z0-9]*)=<([^>]+)>$/);
    if (!m) return { fields: [], freeform: true };
    fields.push({ name: m[1]!, hint: m[2]!, optional });
  }
  return { fields, freeform: fields.length === 0 };
}

/** Assemble `k=v` args from filled fields, skipping empty optionals. */
export function buildArgs(
  fields: RelationField[],
  values: Record<string, string>,
): string[] | null {
  const out: string[] = [];
  for (const f of fields) {
    const v = (values[f.name] ?? "").trim();
    if (!v) {
      if (f.optional) continue;
      return null;
    }
    if (!Number.isFinite(Number(v))) return null;
    out.push(`${f.name}=${v}`);
  }
  return out;
}
