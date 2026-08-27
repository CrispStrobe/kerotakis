export type AssemblyPart = {
  id: string;
  label: string;
  symbol: string;
  state?: "ready" | "attention";
};

export type ApparatusAssembly = {
  parts: AssemblyPart[];
  edges: [string, string][];
};

const line = (...parts: AssemblyPart[]): ApparatusAssembly => ({
  parts,
  edges: parts.slice(1).map((part, index) => [parts[index]!.id, part.id]),
});

const part = (id: string, label: string, symbol: string, state?: AssemblyPart["state"]): AssemblyPart =>
  ({ id, label, symbol, state });

/** Physical parts shown for a deployed operator. This is presentation state;
 * the operator remains the authority for whether the setup can run. */
export function assemblyFor(
  tool: string,
  values: Record<string, number | string>,
): ApparatusAssembly {
  switch (tool) {
    case "stir":
      return line(part("drive", "magnetic drive", "↻"), part("bar", "stir bar", "━"), part("sample", "sample vessel", "▽"));
    case "heat":
      return line(part("power", "power", "⚡"), part("plate", "hotplate", "▰"), part("sample", "sample vessel", "▽"));
    case "cool":
      return line(part("bath", "cooling bath", "❄"), part("sample", "sample vessel", "▽"));
    case "centrifuge": {
      const sample = Number(values.sampleMass ?? 0);
      const counterbalance = Number(values.counterbalance ?? 0);
      const balanced = Number.isFinite(sample) && Number.isFinite(counterbalance) && Math.abs(sample - counterbalance) <= 0.1;
      return {
        parts: [
          part("sample", "sample tube", "▯"),
          part("rotor", "rotor", "✣"),
          part("balance", "counterbalance tube", "▯", balanced ? "ready" : "attention"),
        ],
        edges: [["sample", "rotor"], ["balance", "rotor"]],
      };
    }
    case "electrolyse":
      return line(part("supply", "power supply", "⚡"), part("leads", "electrical leads", "⌁"), part("electrodes", "electrode pair", "Ⅱ"), part("sample", "sample vessel", "▽"));
    case "irradiate":
      return line(part("lamp", "lamp", "☀"), part("sample", "sample vessel", "▽"));
    case "regulate":
      return line(part("piston", "piston lid", "↕"), part("seal", "sealed connection", "◉"), part("sample", "sample vessel", "▽"));
    case "sweep":
      return line(part("source", "gas source", "◉"), part("inlet", "inlet hose", "⌁"), part("sample", "sample vessel", "▽"), part("outlet", "safe exhaust", "→"));
    case "grind":
      return line(part("mortar", "mortar", "◡"), part("pestle", "pestle", "╱"), part("solid", "solid sample", "◆", values.species ? "ready" : "attention"));
    case "dilute":
      return line(part("bottle", "wash bottle", "♙"), part("sample", "sample vessel", "▽"));
    case "evaporate":
      return line(part("heater", "heater", "♨"), part("dish", "evaporating dish", "◡"));
    default:
      return line(part("tool", tool.replaceAll("_", " "), "◉"), part("sample", "sample vessel", "▽"));
  }
}
