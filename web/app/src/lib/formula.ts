/**
 * One typesetter for chemical formulas in the shell: `SO4-2` → `SO₄²⁻`,
 * `CuSO4·5H2O` → `CuSO₄·5H₂O`. A port of `typeset` in
 * `crates/kerotakis-core/src/ionic.rs` (digits in the body are subscripts;
 * a trailing charge — `+`, `--`, or a sign followed by digits — is raised),
 * plus hydrate segments whose leading coefficient stays on the line.
 *
 * It only ever rewrites something that IS a formula: every capital must
 * open a real element symbol. Names ("acetic acid"), vessel names ("v1"),
 * quantities ("3.2638 mol") and temperatures ("78.2 °C") pass through.
 */

const SUB = "₀₁₂₃₄₅₆₇₈₉";
const SUP = "⁰¹²³⁴⁵⁶⁷⁸⁹";

const ELEMENTS = new Set(
  ("H He Li Be B C N O F Ne Na Mg Al Si P S Cl Ar K Ca Sc Ti V Cr Mn Fe Co Ni Cu Zn Ga Ge As Se Br Kr " +
    "Rb Sr Y Zr Nb Mo Tc Ru Rh Pd Ag Cd In Sn Sb Te I Xe Cs Ba La Ce Pr Nd Pm Sm Eu Gd Tb Dy Ho Er Tm Yb Lu " +
    "Hf Ta W Re Os Ir Pt Au Hg Tl Pb Bi Po At Rn Fr Ra Ac Th Pa U Np Pu Am Cm Bk Cf Es Fm Md No Lr " +
    "Rf Db Sg Bh Hs Mt Ds Rg Cn Nh Fl Mc Lv Ts Og").split(" "),
);

const PHASE = /\((aq|s|l|g)\)$/;
const HYDRATE_DOT = /[·•]/;

/** The `split_trailing_charge` of ionic.rs. */
function splitCharge(text: string): [string, number] {
  const t = text.trimEnd();
  const signs = /[+-]+$/.exec(t);
  if (signs) {
    const sign = signs[0][0] === "-" ? -1 : 1;
    return [t.slice(0, signs.index), sign * signs[0].length];
  }
  const tail = /[+-](\d+)$/.exec(t);
  if (tail) {
    const sign = t[tail.index] === "-" ? -1 : 1;
    return [t.slice(0, tail.index), sign * Number(tail[1])];
  }
  return [t, 0];
}

/** Element symbols with counts and bracketed groups, nothing else. */
function isBody(body: string): boolean {
  if (!/^(?:[A-Z][a-z]?\d*|[()[\]]\d*)+$/.test(body)) return false;
  let depth = 0;
  for (const m of body.matchAll(/[A-Z][a-z]?|[()[\]]/g)) {
    const s = m[0];
    if (s === "(" || s === "[") depth++;
    else if (s === ")" || s === "]") {
      if (--depth < 0) return false;
    } else if (!ELEMENTS.has(s)) return false;
  }
  return depth === 0 && /[A-Z]/.test(body);
}

function parse(text: string): { segments: [string, string][]; charge: number; phase: string } | null {
  const phaseMatch = PHASE.exec(text);
  const phase = phaseMatch ? phaseMatch[0] : "";
  const core = phase ? text.slice(0, -phase.length) : text;
  const [body, charge] = splitCharge(core);
  const parts = body.split(HYDRATE_DOT);
  const segments: [string, string][] = [];
  for (const [i, part] of parts.entries()) {
    // Only a hydrate segment may carry a leading coefficient (`5H2O`).
    const m = /^(\d*)(.*)$/.exec(part)!;
    const coefficient = i === 0 ? "" : m[1]!;
    const rest = i === 0 ? part : m[2]!;
    if (!isBody(rest)) return null;
    segments.push([coefficient, rest]);
  }
  if (charge !== 0 && parts.length > 1) return null;
  return { segments, charge, phase };
}

/** Is this whole string a chemical formula the typesetter would accept? */
export function isFormula(text: string): boolean {
  return parse(text) !== null;
}

/** Typeset one formula; anything that is not a formula is returned unchanged. */
export function typesetFormula(text: string): string {
  const parsed = parse(text);
  if (!parsed) return text;
  const separator = text.match(HYDRATE_DOT)?.[0] ?? "·";
  let out = parsed.segments
    .map(([coefficient, body]) => coefficient + body.replace(/\d/g, (d) => SUB[Number(d)]!))
    .join(separator);
  const magnitude = Math.abs(parsed.charge);
  if (magnitude > 1) out += [...String(magnitude)].map((d) => SUP[Number(d)]!).join("");
  if (parsed.charge > 0) out += "⁺";
  if (parsed.charge < 0) out += "⁻";
  return out + parsed.phase;
}

// A candidate token: starts with a capital or bracket, not glued to a word
// or a number on either side. Charge and phase suffixes are optional.
const TOKEN =
  /(?<![\w.·])([A-Z([][A-Za-z0-9()[\]·]*(?:\d*[+-]+|[+-]\d+)?(?:\((?:aq|s|l|g)\))?)(?![\w])/g;

/**
 * Typeset the formula tokens inside prose, conservatively: a token changes
 * only when it parses as a formula AND carries a count or a charge, so
 * "NaCl", "CO", "C" (as in °C) and every number are left exactly as written.
 */
export function typesetFormulaTokens(text: string): string {
  return text.replace(TOKEN, (token: string) => {
    if (!/\d|[+-]/.test(token)) return token;
    return isFormula(token) ? typesetFormula(token) : token;
  });
}
