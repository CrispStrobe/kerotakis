/**
 * E1: stock solutions on the shelf. Mirrors the `[[stock]]` rows of
 * data/solutions/stock-solutions.toml (a test keeps the two in step); the
 * engine owns densities and refusals, this module only composes commands.
 */

export interface StockOption {
  /** Grammar token the engine parses after the amount: `37%`, `1M`; "" = pure. */
  token: string;
  /** The shelf stock key, when this is a listed stock. */
  key?: string;
  common?: boolean;
}

/** species key → listed stocks, in the data file's order. */
export const STOCKS: Record<string, StockOption[]> = {
  HCl: [
    { token: "37%", key: "hydrochloric_acid_37_percent", common: true },
    { token: "6M", key: "hydrochloric_acid_6_molar" },
    { token: "1M", key: "hydrochloric_acid_1_molar" },
  ],
  H2SO4: [
    { token: "96%", key: "sulfuric_acid_96_percent", common: true },
    { token: "1M", key: "sulfuric_acid_1_molar" },
  ],
  H2O2: [{ token: "30%", key: "hydrogen_peroxide_30_percent", common: true }],
  NH3: [{ token: "25%", key: "ammonia_25_percent", common: true }],
  NaOH: [{ token: "1M", key: "sodium_hydroxide_1_molar", common: true }],
  CH3COOH: [
    { token: "5%", key: "acetic_acid_5_percent", common: true },
    { token: "100%", key: "glacial_acetic_acid" },
  ],
};

/**
 * Concentrations the add form offers for a species. At lv1/lv2 the bottle
 * on the shelf is the common stock, never the pure compound; lv3 also
 * offers the pure substance (token ""). Empty when no stock is listed.
 */
export function concentrationOptions(register: string, species: string): StockOption[] {
  const stocks = STOCKS[species];
  if (!stocks) return [];
  return register === "lv3" ? [...stocks, { token: "" }] : [...stocks];
}

/** The default selection: the common stock. */
export function defaultConcentration(register: string, species: string): string {
  const options = concentrationOptions(register, species);
  return (options.find((o) => o.common) ?? options[0])?.token ?? "";
}

/**
 * Normalise a typed concentration to the grammar token, or null when it is
 * not one: `37 %`, `37%w/w`, `37 % (w/w)` → `37%`; `1 M` → `1M` (capital M
 * only, as the engine reads it). Range: 0 < % ≤ 100, M > 0.
 */
export function normaliseConcentration(raw: string): string | null {
  const s = raw.trim().replace(/\s+/g, "");
  const percent = /^(\d+(?:\.\d+)?|\.\d+)%(?:w\/w|\(w\/w\))?$/.exec(s);
  if (percent) {
    const v = Number(percent[1]);
    return v > 0 && v <= 100 ? `${percent[1]}%` : null;
  }
  const molar = /^(\d+(?:\.\d+)?|\.\d+)M$/.exec(s);
  if (molar) {
    const v = Number(molar[1]);
    return v > 0 && Number.isFinite(v) ? `${molar[1]}M` : null;
  }
  return null;
}

/** Units a solution is measured in: solution volume/mass, or solute moles. */
export const SOLUTION_UNITS = ["mL", "L", "g", "mol"] as const;

/** The `add` command for one dose; a pure dose keeps the old grammar. */
export function solutionCommand(
  vessel: number,
  species: string,
  amount: string,
  concentration: string,
): string {
  const base = `add v${vessel + 1} ${species} ${amount.trim()}`;
  if (!concentration) return base;
  const token = normaliseConcentration(concentration);
  if (!token) throw new Error(`not a concentration: ${concentration}`);
  return amount.trim().endsWith("mol") ? `${base} as ${token}` : `${base} ${token}`;
}
