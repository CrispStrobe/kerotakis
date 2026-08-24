/**
 * The 118 elements — STRUCTURAL facts only (number, symbol, name,
 * group, period, block, category). Deliberately no numeric property
 * claims: masses and measured properties come from the registry's
 * provenance-carrying records where the lab has the element, and are
 * honestly absent otherwise. Numeric breadth arrives via the
 * licence-clean data ETL (roadmap), never by transcription.
 */

export interface ElementInfo {
  z: number;
  symbol: string;
  name: string;
  group: number;
  period: number;
  block: string;
  category: string;
}

export const ELEMENTS: ElementInfo[] = [
  { z: 1, symbol: "H", name: "Hydrogen", group: 1, period: 1, block: "s", category: "nonmetal" },
  { z: 2, symbol: "He", name: "Helium", group: 18, period: 1, block: "s", category: "noble" },
  { z: 3, symbol: "Li", name: "Lithium", group: 1, period: 2, block: "s", category: "alkali" },
  { z: 4, symbol: "Be", name: "Beryllium", group: 2, period: 2, block: "s", category: "alkaline" },
  { z: 5, symbol: "B", name: "Boron", group: 13, period: 2, block: "p", category: "metalloid" },
  { z: 6, symbol: "C", name: "Carbon", group: 14, period: 2, block: "p", category: "nonmetal" },
  { z: 7, symbol: "N", name: "Nitrogen", group: 15, period: 2, block: "p", category: "nonmetal" },
  { z: 8, symbol: "O", name: "Oxygen", group: 16, period: 2, block: "p", category: "nonmetal" },
  { z: 9, symbol: "F", name: "Fluorine", group: 17, period: 2, block: "p", category: "halogen" },
  { z: 10, symbol: "Ne", name: "Neon", group: 18, period: 2, block: "p", category: "noble" },
  { z: 11, symbol: "Na", name: "Sodium", group: 1, period: 3, block: "s", category: "alkali" },
  { z: 12, symbol: "Mg", name: "Magnesium", group: 2, period: 3, block: "s", category: "alkaline" },
  { z: 13, symbol: "Al", name: "Aluminium", group: 13, period: 3, block: "p", category: "post" },
  { z: 14, symbol: "Si", name: "Silicon", group: 14, period: 3, block: "p", category: "metalloid" },
  { z: 15, symbol: "P", name: "Phosphorus", group: 15, period: 3, block: "p", category: "nonmetal" },
  { z: 16, symbol: "S", name: "Sulfur", group: 16, period: 3, block: "p", category: "nonmetal" },
  { z: 17, symbol: "Cl", name: "Chlorine", group: 17, period: 3, block: "p", category: "halogen" },
  { z: 18, symbol: "Ar", name: "Argon", group: 18, period: 3, block: "p", category: "noble" },
  { z: 19, symbol: "K", name: "Potassium", group: 1, period: 4, block: "s", category: "alkali" },
  { z: 20, symbol: "Ca", name: "Calcium", group: 2, period: 4, block: "s", category: "alkaline" },
  { z: 21, symbol: "Sc", name: "Scandium", group: 3, period: 4, block: "d", category: "transition" },
  { z: 22, symbol: "Ti", name: "Titanium", group: 4, period: 4, block: "d", category: "transition" },
  { z: 23, symbol: "V", name: "Vanadium", group: 5, period: 4, block: "d", category: "transition" },
  { z: 24, symbol: "Cr", name: "Chromium", group: 6, period: 4, block: "d", category: "transition" },
  { z: 25, symbol: "Mn", name: "Manganese", group: 7, period: 4, block: "d", category: "transition" },
  { z: 26, symbol: "Fe", name: "Iron", group: 8, period: 4, block: "d", category: "transition" },
  { z: 27, symbol: "Co", name: "Cobalt", group: 9, period: 4, block: "d", category: "transition" },
  { z: 28, symbol: "Ni", name: "Nickel", group: 10, period: 4, block: "d", category: "transition" },
  { z: 29, symbol: "Cu", name: "Copper", group: 11, period: 4, block: "d", category: "transition" },
  { z: 30, symbol: "Zn", name: "Zinc", group: 12, period: 4, block: "d", category: "transition" },
  { z: 31, symbol: "Ga", name: "Gallium", group: 13, period: 4, block: "p", category: "post" },
  { z: 32, symbol: "Ge", name: "Germanium", group: 14, period: 4, block: "p", category: "metalloid" },
  { z: 33, symbol: "As", name: "Arsenic", group: 15, period: 4, block: "p", category: "metalloid" },
  { z: 34, symbol: "Se", name: "Selenium", group: 16, period: 4, block: "p", category: "nonmetal" },
  { z: 35, symbol: "Br", name: "Bromine", group: 17, period: 4, block: "p", category: "halogen" },
  { z: 36, symbol: "Kr", name: "Krypton", group: 18, period: 4, block: "p", category: "noble" },
  { z: 37, symbol: "Rb", name: "Rubidium", group: 1, period: 5, block: "s", category: "alkali" },
  { z: 38, symbol: "Sr", name: "Strontium", group: 2, period: 5, block: "s", category: "alkaline" },
  { z: 39, symbol: "Y", name: "Yttrium", group: 3, period: 5, block: "d", category: "transition" },
  { z: 40, symbol: "Zr", name: "Zirconium", group: 4, period: 5, block: "d", category: "transition" },
  { z: 41, symbol: "Nb", name: "Niobium", group: 5, period: 5, block: "d", category: "transition" },
  { z: 42, symbol: "Mo", name: "Molybdenum", group: 6, period: 5, block: "d", category: "transition" },
  { z: 43, symbol: "Tc", name: "Technetium", group: 7, period: 5, block: "d", category: "transition" },
  { z: 44, symbol: "Ru", name: "Ruthenium", group: 8, period: 5, block: "d", category: "transition" },
  { z: 45, symbol: "Rh", name: "Rhodium", group: 9, period: 5, block: "d", category: "transition" },
  { z: 46, symbol: "Pd", name: "Palladium", group: 10, period: 5, block: "d", category: "transition" },
  { z: 47, symbol: "Ag", name: "Silver", group: 11, period: 5, block: "d", category: "transition" },
  { z: 48, symbol: "Cd", name: "Cadmium", group: 12, period: 5, block: "d", category: "transition" },
  { z: 49, symbol: "In", name: "Indium", group: 13, period: 5, block: "p", category: "post" },
  { z: 50, symbol: "Sn", name: "Tin", group: 14, period: 5, block: "p", category: "post" },
  { z: 51, symbol: "Sb", name: "Antimony", group: 15, period: 5, block: "p", category: "metalloid" },
  { z: 52, symbol: "Te", name: "Tellurium", group: 16, period: 5, block: "p", category: "metalloid" },
  { z: 53, symbol: "I", name: "Iodine", group: 17, period: 5, block: "p", category: "halogen" },
  { z: 54, symbol: "Xe", name: "Xenon", group: 18, period: 5, block: "p", category: "noble" },
  { z: 55, symbol: "Cs", name: "Caesium", group: 1, period: 6, block: "s", category: "alkali" },
  { z: 56, symbol: "Ba", name: "Barium", group: 2, period: 6, block: "s", category: "alkaline" },
  { z: 57, symbol: "La", name: "Lanthanum", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 58, symbol: "Ce", name: "Cerium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 59, symbol: "Pr", name: "Praseodymium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 60, symbol: "Nd", name: "Neodymium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 61, symbol: "Pm", name: "Promethium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 62, symbol: "Sm", name: "Samarium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 63, symbol: "Eu", name: "Europium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 64, symbol: "Gd", name: "Gadolinium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 65, symbol: "Tb", name: "Terbium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 66, symbol: "Dy", name: "Dysprosium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 67, symbol: "Ho", name: "Holmium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 68, symbol: "Er", name: "Erbium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 69, symbol: "Tm", name: "Thulium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 70, symbol: "Yb", name: "Ytterbium", group: 3, period: 6, block: "f", category: "lanthanide" },
  { z: 71, symbol: "Lu", name: "Lutetium", group: 3, period: 6, block: "d", category: "lanthanide" },
  { z: 72, symbol: "Hf", name: "Hafnium", group: 4, period: 6, block: "d", category: "transition" },
  { z: 73, symbol: "Ta", name: "Tantalum", group: 5, period: 6, block: "d", category: "transition" },
  { z: 74, symbol: "W", name: "Tungsten", group: 6, period: 6, block: "d", category: "transition" },
  { z: 75, symbol: "Re", name: "Rhenium", group: 7, period: 6, block: "d", category: "transition" },
  { z: 76, symbol: "Os", name: "Osmium", group: 8, period: 6, block: "d", category: "transition" },
  { z: 77, symbol: "Ir", name: "Iridium", group: 9, period: 6, block: "d", category: "transition" },
  { z: 78, symbol: "Pt", name: "Platinum", group: 10, period: 6, block: "d", category: "transition" },
  { z: 79, symbol: "Au", name: "Gold", group: 11, period: 6, block: "d", category: "transition" },
  { z: 80, symbol: "Hg", name: "Mercury", group: 12, period: 6, block: "d", category: "transition" },
  { z: 81, symbol: "Tl", name: "Thallium", group: 13, period: 6, block: "p", category: "post" },
  { z: 82, symbol: "Pb", name: "Lead", group: 14, period: 6, block: "p", category: "post" },
  { z: 83, symbol: "Bi", name: "Bismuth", group: 15, period: 6, block: "p", category: "post" },
  { z: 84, symbol: "Po", name: "Polonium", group: 16, period: 6, block: "p", category: "post" },
  { z: 85, symbol: "At", name: "Astatine", group: 17, period: 6, block: "p", category: "halogen" },
  { z: 86, symbol: "Rn", name: "Radon", group: 18, period: 6, block: "p", category: "noble" },
  { z: 87, symbol: "Fr", name: "Francium", group: 1, period: 7, block: "s", category: "alkali" },
  { z: 88, symbol: "Ra", name: "Radium", group: 2, period: 7, block: "s", category: "alkaline" },
  { z: 89, symbol: "Ac", name: "Actinium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 90, symbol: "Th", name: "Thorium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 91, symbol: "Pa", name: "Protactinium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 92, symbol: "U", name: "Uranium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 93, symbol: "Np", name: "Neptunium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 94, symbol: "Pu", name: "Plutonium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 95, symbol: "Am", name: "Americium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 96, symbol: "Cm", name: "Curium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 97, symbol: "Bk", name: "Berkelium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 98, symbol: "Cf", name: "Californium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 99, symbol: "Es", name: "Einsteinium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 100, symbol: "Fm", name: "Fermium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 101, symbol: "Md", name: "Mendelevium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 102, symbol: "No", name: "Nobelium", group: 3, period: 7, block: "f", category: "actinide" },
  { z: 103, symbol: "Lr", name: "Lawrencium", group: 3, period: 7, block: "d", category: "actinide" },
  { z: 104, symbol: "Rf", name: "Rutherfordium", group: 4, period: 7, block: "d", category: "transition" },
  { z: 105, symbol: "Db", name: "Dubnium", group: 5, period: 7, block: "d", category: "transition" },
  { z: 106, symbol: "Sg", name: "Seaborgium", group: 6, period: 7, block: "d", category: "transition" },
  { z: 107, symbol: "Bh", name: "Bohrium", group: 7, period: 7, block: "d", category: "transition" },
  { z: 108, symbol: "Hs", name: "Hassium", group: 8, period: 7, block: "d", category: "transition" },
  { z: 109, symbol: "Mt", name: "Meitnerium", group: 9, period: 7, block: "d", category: "unknown" },
  { z: 110, symbol: "Ds", name: "Darmstadtium", group: 10, period: 7, block: "d", category: "unknown" },
  { z: 111, symbol: "Rg", name: "Roentgenium", group: 11, period: 7, block: "d", category: "unknown" },
  { z: 112, symbol: "Cn", name: "Copernicium", group: 12, period: 7, block: "d", category: "transition" },
  { z: 113, symbol: "Nh", name: "Nihonium", group: 13, period: 7, block: "p", category: "unknown" },
  { z: 114, symbol: "Fl", name: "Flerovium", group: 14, period: 7, block: "p", category: "unknown" },
  { z: 115, symbol: "Mc", name: "Moscovium", group: 15, period: 7, block: "p", category: "unknown" },
  { z: 116, symbol: "Lv", name: "Livermorium", group: 16, period: 7, block: "p", category: "unknown" },
  { z: 117, symbol: "Ts", name: "Tennessine", group: 17, period: 7, block: "p", category: "unknown" },
  { z: 118, symbol: "Og", name: "Oganesson", group: 18, period: 7, block: "p", category: "unknown" },
];

const SYMBOLS = new Set(ELEMENTS.map((e) => e.symbol));

/**
 * The element symbols appearing in a chemical formula ("Ca(OH)2" →
 * Ca, O, H). Greedy two-letter match against the real symbol set, so
 * "Co" is cobalt while "CO" is carbon + oxygen.
 */
export function elementsInFormula(formula: string): string[] {
  const found: string[] = [];
  let i = 0;
  while (i < formula.length) {
    const two = formula.slice(i, i + 2);
    const one = formula[i]!;
    if (/[A-Z]/.test(one) && /[a-z]/.test(two[1] ?? "") && SYMBOLS.has(two)) {
      if (!found.includes(two)) found.push(two);
      i += 2;
    } else if (SYMBOLS.has(one)) {
      if (!found.includes(one)) found.push(one);
      i += 1;
    } else {
      i += 1;
    }
  }
  return found;
}
