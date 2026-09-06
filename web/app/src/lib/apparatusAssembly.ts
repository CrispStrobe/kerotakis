/**
 * The physical setup, as a thing on the stage rather than a list beside it.
 *
 * This used to feed one component: a row of labelled chips headed
 * "PHYSISCHER AUFBAU", set in the middle column of the apparatus panel.
 * A chip is 4–7.5 rem wide, the column was 12 rem, and "Ballon oder
 * Gasbeutel — Dichte Verbindung — Gefäß" needs about 24 rem. So the row
 * scrolled sideways inside a column that was itself squeezing the
 * controls, and a learner read two words at a time of a sentence about
 * equipment they could already see drawn on the vessel.
 *
 * `DeployedApparatus.svelte` has been drawing that equipment on the
 * vessel's own SVG all along — the balloon on the neck, the hose, the
 * gauge, the electrode pair, the mortar. The parts list was never a
 * second picture; it was a NAMING of the first one, printed where the
 * picture is not.
 *
 * So each part now carries `at`: where, in the vessel's own `0 0 100 140`
 * viewBox, the piece it names is drawn. `ApparatusAssembly.svelte` renders
 * as an annotation layer inside that same SVG — a hairline down the
 * assembly and a marker on each piece, with the part's name as the
 * marker's `<title>` — and the panel keeps only what a picture cannot
 * say: which part still needs attention, and the words behind the (i).
 *
 * The coordinates are DeployedApparatus's, read off its own paths. They
 * are presentation only: the operator remains the authority for whether
 * the setup can run, and a wrong anchor moves a dot, never a result.
 */

export type AssemblyPart = {
  id: string;
  label: string;
  symbol: string;
  state?: "ready" | "attention";
  /** Where this piece is drawn, in the vessel's own `0 0 100 140` viewBox. */
  at: readonly [number, number];
};

export type ApparatusAssembly = {
  parts: AssemblyPart[];
  edges: [string, string][];
};

const line = (...parts: AssemblyPart[]): ApparatusAssembly => ({
  parts,
  edges: parts.slice(1).map((part, index) => [parts[index]!.id, part.id]),
});

const part = (
  id: string,
  label: string,
  symbol: string,
  at: readonly [number, number],
  state?: AssemblyPart["state"],
): AssemblyPart => ({ id, label, symbol, at, state });

/**
 * The vessel body itself, which is where "sample vessel" always points.
 *
 * Not the geometric centre: the glass is drawn from y≈4 to y≈127 and the
 * liquid sits in its lower half, so a marker at 100 would land in the
 * liquid where the fill colour is. 62 is the shoulder — over the glass,
 * clear of both the fill and the apparatus above it.
 */
const VESSEL = [50, 62] as const;

/** Physical parts shown for a deployed operator. This is presentation state;
 * the operator remains the authority for whether the setup can run. */
export function assemblyFor(
  tool: string,
  values: Record<string, number | string>,
): ApparatusAssembly {
  switch (tool) {
    case "bunsen":
      return line(
        part("wax", "candle wax", "▮", [50, 131]),
        part("wick", "wick", "│", [50, 110]),
        part("flame", "ignition flame", "♨", [50, 86]),
        part("sample", "sample vessel", "▽", VESSEL),
      );
    case "stir":
      return line(
        part("drive", "magnetic drive", "↻", [50, 129]),
        part("bar", "stir bar", "━", [50, 121]),
        part("sample", "sample vessel", "▽", VESSEL),
      );
    case "heat":
      return line(
        part("power", "power", "⚡", [69, 129]),
        part("plate", "hotplate", "▰", [50, 121]),
        part("sample", "sample vessel", "▽", VESSEL),
      );
    case "cool":
      return line(
        part("bath", "cooling bath", "❄", [50, 112]),
        part("sample", "sample vessel", "▽", VESSEL),
      );
    case "centrifuge": {
      const sample = Number(values.sampleMass ?? 0);
      const counterbalance = Number(values.counterbalance ?? 0);
      const balanced = Number.isFinite(sample) && Number.isFinite(counterbalance) && Math.abs(sample - counterbalance) <= 0.1;
      return {
        parts: [
          part("sample", "sample tube", "▯", [38, 62]),
          part("rotor", "rotor", "✣", [50, 96]),
          part("balance", "counterbalance tube", "▯", [62, 62], balanced ? "ready" : "attention"),
        ],
        edges: [["sample", "rotor"], ["balance", "rotor"]],
      };
    }
    case "electrolyse":
      return line(
        part("supply", "power supply", "⚡", [50, 10]),
        part("leads", "electrical leads", "⌁", [18, 24]),
        part("electrodes", "electrode pair", "Ⅱ", [30, 58]),
        part("sample", "sample vessel", "▽", [50, 100]),
      );
    case "irradiate":
      return line(
        part("lamp", "lamp", "☀", [38, 23]),
        part("sample", "sample vessel", "▽", [58, 100]),
      );
    case "regulate":
      return line(
        part("bag", "balloon or gas bag", "◯", [50, 25]),
        part("seal", "sealed connection", "◉", [50, 48]),
        part("sample", "sample vessel", "▽", [50, 90]),
      );
    case "sweep":
      return line(
        part("source", "gas source", "◉", [10, 45]),
        part("inlet", "inlet hose", "⌁", [29, 30]),
        part("sample", "sample vessel", "▽", VESSEL),
        part("outlet", "safe exhaust", "→", [88, 36]),
      );
    case "grind":
      return line(
        part("mortar", "mortar", "◡", [76, 111]),
        part("pestle", "pestle", "╱", [66, 88]),
        part("solid", "solid sample", "◆", [76, 116], values.species ? "ready" : "attention"),
      );
    case "dilute":
      return line(
        part("bottle", "wash bottle", "♙", [78, 100]),
        part("sample", "sample vessel", "▽", [50, 96]),
      );
    case "evaporate":
      return line(
        part("heater", "heater", "♨", [50, 129]),
        part("dish", "evaporating dish", "◡", [50, 117]),
      );
    default:
      return line(
        part("tool", tool.replaceAll("_", " "), "◉", [50, 20]),
        part("sample", "sample vessel", "▽", VESSEL),
      );
  }
}

/**
 * The tools `DeployedApparatus.svelte` actually draws on the vessel.
 *
 * Kept here rather than inferred, because the annotation is only honest
 * where there is a picture under it: a marker floating over a bare beaker
 * points at nothing. `centrifuge` is the one operator with an assembly and
 * no drawing — its tubes leave the bench — so its panel keeps the named
 * parts inline instead of annotating a vessel that is not there.
 */
const DRAWN_ON_STAGE: readonly string[] = [
  "bunsen",
  "cool",
  "dilute",
  "electrolyse",
  "evaporate",
  "grind",
  "heat",
  "irradiate",
  "regulate",
  "stir",
  "sweep",
];

export function drawnOnStage(tool: string): boolean {
  return DRAWN_ON_STAGE.includes(tool);
}

/**
 * The one thing the panel still has to say in words.
 *
 * A picture shows what is connected; it cannot show that the
 * counterbalance is 0.2 g out or that no solid has been chosen yet. Those
 * parts are what the strip reports, and the rest of the naming lives
 * behind the (i) and on each marker's `<title>`.
 */
export function assemblyAttention(assembly: ApparatusAssembly): AssemblyPart[] {
  return assembly.parts.filter((item) => item.state === "attention");
}
