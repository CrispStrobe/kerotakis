/**
 * GUI-093 — what a reagent is *for*, derived rather than listed.
 *
 * The shelf already filters by phase, which is a physics axis; a learner
 * looking for "an acid" is thinking on a chemistry one. The temptation is
 * a table in this file mapping every species key to a role, and that table
 * would be wrong within a month: the registry is generated from the data
 * pack, so a species added there would silently arrive with no role and
 * nothing would say so.
 *
 * So nothing here names a species. Every role comes from a fact the engine
 * already computes and now ships with each shelf item:
 *
 *   - `reactive_groups` — `kerotakis_safety::groups`, the NOAA-style
 *     reactive-group row that CI already forces to be total over the
 *     registry. This is the authority for acid, base, oxidiser, reducer
 *     and active metal. The `hazards` labels are a lossy view of the same
 *     rows (`AcidStrong` and `BaseStrong` both render "corrosive"), so
 *     they are used only as the fallback for an older engine build.
 *   - `elements` / `charge` — the engine's own formula parser
 *     (`kerotakis_core::stoich::parse_formula`) applied to the registry
 *     formula. Composition, not string-matching, decides ion, oxide,
 *     hydroxide, salt and organic.
 *   - `indicator` — membership of `kerotakis_core::indicator::INDICATORS`,
 *     the table that actually computes the colour change.
 *   - `solvent` — membership of `kerotakis_core::nonaqueous::KNOWN_SOLVENTS`,
 *     plus water, the solvent the aqueous engine is built around.
 *   - `ELEMENTS` in `elements.ts` — structural periodic-table facts, for
 *     the single question composition cannot answer on its own: whether a
 *     symbol is a metal.
 *
 * Where those inputs decide nothing, the species is `unsorted`, and the
 * shelf says so in as many words. That is the honest answer for elemental
 * oxygen and for a stand-in formula like the enzymes' bare "C" — better
 * than a guess, and visible enough that the gap gets filled with data
 * rather than with a hand-written exception here.
 */
import { ELEMENTS } from "./elements";
import type { ShelfItem } from "./session.svelte";

/**
 * Roles in the order the chips are laid out — the reagent-bottle reading
 * order a bench shelf is usually arranged in, with the honest gap last.
 */
export const REAGENT_ROLES = [
  "acid",
  "base",
  "salt",
  "oxide",
  "metal",
  "organic",
  "solvent",
  "indicator",
  "oxidiser",
  "reducer",
  "ion",
  "unsorted",
] as const;

export type ReagentRole = (typeof REAGENT_ROLES)[number];

/** Chip label per role: a plural noun, translated like every other chip. */
export const ROLE_LABELS: Record<ReagentRole, string> = {
  acid: "acids",
  base: "bases",
  salt: "salts",
  oxide: "oxides",
  metal: "metals",
  organic: "organics",
  solvent: "solvents",
  indicator: "indicators",
  oxidiser: "oxidisers",
  reducer: "reducers",
  ion: "ions",
  unsorted: "unsorted",
};

/** Categories in `elements.ts` that describe a metallic element. */
const METALLIC_CATEGORIES = new Set([
  "alkali",
  "alkaline",
  "transition",
  "post",
  "lanthanide",
  "actinide",
]);

const METAL_SYMBOLS: ReadonlySet<string> = new Set(
  ELEMENTS.filter((element) => METALLIC_CATEGORIES.has(element.category)).map(
    (element) => element.symbol,
  ),
);

export function isMetalSymbol(symbol: string): boolean {
  return METAL_SYMBOLS.has(symbol);
}

/** The subset of a shelf item this derivation reads. Nothing else. */
export type RoleInput = Pick<
  ShelfItem,
  | "key"
  | "formula"
  | "hazards"
  | "reactive_groups"
  | "elements"
  | "charge"
  | "indicator"
  | "solvent"
  | "components"
>;

const ROLE_ORDER = new Map(REAGENT_ROLES.map((role, index) => [role, index]));

function sortRoles(roles: Iterable<ReagentRole>): ReagentRole[] {
  // i18n-ok: `ROLE_ORDER` is a layout rank, not a rendered string; the
  // chips are sorted for display by their translated labels in Shelf.
  return [...roles].sort((a, b) => (ROLE_ORDER.get(a) ?? 0) - (ROLE_ORDER.get(b) ?? 0));
}

/** The reactive-group rows the safety matrix assigns, mapped to roles. */
function rolesFromGroups(item: RoleInput, roles: Set<ReagentRole>): void {
  if (item.reactive_groups) {
    for (const group of item.reactive_groups) {
      if (group === "acid_strong" || group === "acidic_salt") roles.add("acid");
      // Ammonia and the amines are the matrix's weak bases; that row is
      // there because they react as bases, which is the same fact.
      else if (group === "base_strong" || group === "ammonia_amines") roles.add("base");
      else if (group === "oxidizer_strong" || group === "oxidizer_hypochlorite") {
        roles.add("oxidiser");
      } else if (group === "reducing_agent") roles.add("reducer");
      else if (group === "active_metal") roles.add("metal");
    }
    return;
  }
  // Older bridge: only the flattened labels arrived. They cannot separate
  // an acid from a base, but they still separate the two redox roles, so
  // take what is there rather than nothing.
  for (const hazard of item.hazards ?? []) {
    if (hazard === "oxidiser") roles.add("oxidiser");
    if (hazard === "reducing_agent") roles.add("reducer");
  }
}

/** Composition and charge: the questions the safety matrix does not ask. */
function rolesFromComposition(item: RoleInput, roles: Set<ReagentRole>): void {
  const counts = item.elements;
  if (!counts) return;
  const symbols = Object.keys(counts).filter((symbol) => (counts[symbol] ?? 0) > 0);
  if (symbols.length === 0) return;
  const charge = item.charge ?? 0;
  const metals = symbols.filter(isMetalSymbol);
  const hydrogen = counts.H ?? 0;
  const oxygen = counts.O ?? 0;
  const carbon = counts.C ?? 0;
  const nitrogen = counts.N ?? 0;

  // A dissolved ion is on the shelf as itself. It is not an acid or a
  // salt, and saying "ion" is more use than saying nothing.
  if (charge !== 0) roles.add("ion");

  if (symbols.length === 1) {
    // An uncharged single element: a metal if the periodic table says the
    // symbol is one. Sulfur and oxygen fall through, correctly.
    if (charge === 0 && metals.length === 1) roles.add("metal");
    return;
  }

  // A hydroxide: metal, oxygen and hydrogen in equal measure and nothing
  // else. This is what makes copper(II) hydroxide a base without anyone
  // writing its key down — the safety matrix has no row for it, because
  // it is not a NOAA hazard.
  const onlyMetalHydrogenOxygen = symbols.every(
    (symbol) => symbol === "H" || symbol === "O" || isMetalSymbol(symbol),
  );
  if (metals.length > 0 && oxygen > 0 && hydrogen === oxygen && onlyMetalHydrogenOxygen) {
    roles.add("base");
  }

  // An oxide: one other element and oxygen, no hydrogen, uncharged.
  if (charge === 0 && hydrogen === 0 && oxygen > 0 && symbols.length === 2) {
    roles.add("oxide");
  }

  // The H⁺ donor, read the way chemists write one: the acidic hydrogen
  // leads the formula (HCl, H2SO4, H3PO4) or closes a carboxyl group
  // (CH3COOH). Requiring something beyond hydrogen and oxygen keeps water
  // and hydrogen peroxide out, which is the whole reason for the clause.
  const beyondHydrogenOxygen = symbols.some((symbol) => symbol !== "H" && symbol !== "O");
  const leadsWithHydrogen = /^H(?![a-z])/.test(item.formula);
  if (hydrogen > 0 && beyondHydrogenOxygen && (leadsWithHydrogen || /COOH$/.test(item.formula))) {
    roles.add("acid");
  }

  // Organic: a carbon–hydrogen skeleton. The exclusion is the one-carbon
  // species written the same way and meant differently — carbonate,
  // hydrogencarbonate — where the carbon carries only oxygen.
  if (carbon > 0 && hydrogen > 0 && !(carbon === 1 && oxygen >= 2)) roles.add("organic");

  // A salt: a cation and a partner. Ammonium counts as the cation it is;
  // hydroxides and oxides are already spoken for and are not salts.
  const ammonium = nitrogen > 0 && hydrogen >= 4 * nitrogen;
  if (
    charge === 0
    && (metals.length > 0 || ammonium)
    && !roles.has("base")
    && !roles.has("oxide")
  ) {
    roles.add("salt");
  }
}

/**
 * Roles for one pure species. Materials go through `deriveShelfRoles`,
 * which needs the rest of the shelf to look their components up.
 */
export function rolesForSpecies(item: RoleInput): ReagentRole[] {
  const roles = new Set<ReagentRole>();
  rolesFromGroups(item, roles);
  if (item.indicator) roles.add("indicator");
  if (item.solvent) roles.add("solvent");
  rolesFromComposition(item, roles);
  if (roles.size === 0) roles.add("unsorted");
  return sortRoles(roles);
}

/**
 * Roles for every item on the shelf, keyed by species key.
 *
 * A named material is a mixture with no formula of its own, so it takes
 * the roles of what is in it — minus `solvent`, because what a bottle is
 * dissolved in is not what it is for. Sparkling water is carbon dioxide;
 * dish soap, whose only modelled component is water, is honestly
 * unsorted.
 */
export function deriveShelfRoles(
  items: readonly RoleInput[],
): Map<string, ReagentRole[]> {
  const speciesRoles = new Map<string, ReagentRole[]>();
  for (const item of items) {
    if (item.components) continue;
    speciesRoles.set(item.key, rolesForSpecies(item));
  }
  const all = new Map(speciesRoles);
  for (const item of items) {
    if (!item.components) continue;
    const roles = new Set<ReagentRole>();
    for (const component of item.components) {
      for (const role of speciesRoles.get(component) ?? []) {
        if (role !== "unsorted" && role !== "solvent") roles.add(role);
      }
    }
    if (roles.size === 0) roles.add("unsorted");
    all.set(item.key, sortRoles(roles));
  }
  return all;
}
