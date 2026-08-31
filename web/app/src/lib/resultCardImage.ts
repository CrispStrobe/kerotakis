import type { ResultSummary } from "./resultSummary";

export type ResultCardImageText = {
  title: string;
  vessel?: string;
  equation: string;
  observation: string;
  results: string;
  provenance: string;
  emptyEquation: string;
  emptyObservation: string;
};

const escapeXml = (value: string): string => value
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")
  .replaceAll('"', "&quot;")
  .replaceAll("'", "&apos;");

/** Wrap user/engine prose without relying on SVG foreignObject or browser layout. */
export function wrapCardText(value: string, width = 72, lines = 3): string[] {
  const words = value.trim().replace(/\s+/g, " ").split(" ").filter(Boolean)
    .flatMap((word) => word.length <= width
      ? [word]
      : Array.from({ length: Math.ceil(word.length / width) }, (_, index) =>
        word.slice(index * width, (index + 1) * width)));
  const output: string[] = [];
  for (const word of words) {
    const previous = output.at(-1);
    if (!previous || previous.length + word.length + 1 > width) output.push(word);
    else output[output.length - 1] = `${previous} ${word}`;
  }
  if (output.length <= lines) return output;
  const clipped = output.slice(0, lines);
  clipped[lines - 1] = `${clipped[lines - 1]!.slice(0, Math.max(1, width - 1))}…`;
  return clipped;
}

function textLines(lines: string[], x: number, y: number, className: string): string {
  return lines.map((line, index) =>
    `<text x="${x}" y="${y + index * 24}" class="${className}">${escapeXml(line)}</text>`,
  ).join("");
}

/** A deterministic, self-contained hand-in artifact for one computed result. */
export function resultCardSvg(
  result: ResultSummary,
  text: ResultCardImageText,
  format: (value: number) => string,
): string {
  const equation = result.equation || text.emptyEquation;
  const observation = result.observation || text.emptyObservation;
  const quantities = result.quantities.map((quantity) =>
    `${text.results}: ${quantity.label} ${format(quantity.value)} ${quantity.unit}`,
  );
  if (result.temperatureDeltaK !== undefined) {
    quantities.push(`ΔT ${result.temperatureDeltaK > 0 ? "+" : ""}${format(result.temperatureDeltaK)} K`);
  }
  const resultLines = (quantities.length > 0 ? quantities : [text.results])
    .flatMap((line) => wrapCardText(line, 82, 4))
    .slice(0, 4);
  const accessible = [text.title, result.kind, equation, observation, ...resultLines, text.provenance].join(". ");
  const heading = text.vessel ? `${text.title} · ${text.vessel}` : text.title;
  const renderedHeading = wrapCardText(heading, 82, 1)[0] ?? "";
  const renderedKind = wrapCardText(result.kind, 42, 1)[0] ?? "";

  return `<svg xmlns="http://www.w3.org/2000/svg" width="800" height="530" viewBox="0 0 800 530" role="img" aria-labelledby="title description"><title id="title">${escapeXml(heading)}</title><desc id="description">${escapeXml(accessible)}</desc><rect width="800" height="530" rx="28" fill="#f7f4ec"/><rect x="24" y="24" width="752" height="482" rx="20" fill="#fffdf8" stroke="#b8c7bd" stroke-width="2"/><style>text{font-family:ui-sans-serif,system-ui,-apple-system,sans-serif;fill:#17211c}.eyebrow{font-size:15px;font-weight:700;letter-spacing:1.4px;fill:#52645a}.kind{font-size:30px;font-weight:800}.label{font-size:13px;font-weight:700;letter-spacing:1px;fill:#52645a}.equation{font-family:ui-monospace,SFMono-Regular,monospace;font-size:20px;font-weight:700}.body{font-size:17px}.number{font-size:16px;font-weight:700}.provenance{font-size:13px;fill:#52645a}</style><text x="52" y="60" class="eyebrow">${escapeXml(renderedHeading)}</text><text x="52" y="101" class="kind">${escapeXml(renderedKind)}</text><line x1="52" x2="748" y1="120" y2="120" stroke="#d8dfda"/><text x="52" y="151" class="label">${escapeXml(text.equation)}</text>${textLines(wrapCardText(equation, 62, 2), 52, 181, "equation")}<text x="52" y="239" class="label">${escapeXml(text.observation)}</text>${textLines(wrapCardText(observation, 76, 2), 52, 269, "body")}<text x="52" y="337" class="label">${escapeXml(text.results)}</text>${textLines(resultLines, 52, 365, "number")}<line x1="52" x2="748" y1="450" y2="450" stroke="#d8dfda"/>${textLines(wrapCardText(text.provenance, 100, 2), 52, 472, "provenance")}</svg>`;
}

export function resultCardFilename(result: ResultSummary, extension: "svg" | "png"): string {
  const stem = result.kind.toLocaleLowerCase("en-US").normalize("NFKD")
    .replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "result";
  return `kerotakis-${stem}.${extension}`;
}
