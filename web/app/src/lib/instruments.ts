export type InstrumentSpec = { token: string; label: string; glyph: string; purpose: string };

export const INSTRUMENTS: InstrumentSpec[] = [
  { token: "smell", label: "safe waft", glyph: "≋", purpose: "check headspace odour safely" },
  { token: "thermometer", label: "thermometer", glyph: "🌡", purpose: "measure sample temperature" },
  { token: "ph", label: "pH meter", glyph: "pH", purpose: "measure aqueous acidity" },
  { token: "balance", label: "balance", glyph: "⚖", purpose: "measure total material mass" },
  { token: "volume", label: "gas volume meter", glyph: "mL", purpose: "measure sealed gas volume" },
  { token: "conductivity", label: "conductivity meter", glyph: "⚡", purpose: "estimate ionic conductivity" },
  { token: "pressure", label: "pressure gauge", glyph: "bar", purpose: "measure headspace pressure" },
  { token: "calorimeter", label: "calorimeter", glyph: "kJ", purpose: "measure enthalpy relative to 25 °C" },
  { token: "uvvis", label: "UV-Vis", glyph: "λ", purpose: "measure an absorbance spectrum" },
  { token: "eyes", label: "look closely", glyph: "🔍", purpose: "inspect visible appearance" },
  { token: "chromatograph", label: "chromatograph", glyph: "Rf", purpose: "separate and compare components" },
  { token: "geiger", label: "Geiger counter", glyph: "Bq", purpose: "measure radioactive activity" },
];

export const instrumentVerb = (token: string) => `measure:${token}`;

export function instrumentCommand(vessel: number, token: string): string {
  const target = `v${vessel + 1}`;
  if (token === "chromatograph") return `chromatograph ${target}`;
  if (token === "smell") return `smell ${target}`;
  return `measure ${target} ${token}`;
}
