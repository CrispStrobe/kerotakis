import type { Scene } from "./host/EngineHost";

export type ResultQuantity = {
  label: string;
  value: number;
  unit: string;
};

export type ResultSummary = {
  kind: string;
  vessel?: number;
  equation?: string;
  observation?: string;
  temperatureDeltaK?: number;
  quantities: ResultQuantity[];
  boundary?: string;
  provenance?: string;
};

type EngineEvent = Record<string, unknown>;

const CLASSIFICATIONS: Record<string, string> = {
  precipitated: "precipitation",
  plated: "metal plating",
  electrolysed: "electrolysis",
  gas_evolved: "gas evolution",
  gas_absorbed: "gas absorption",
  gas_contained: "gas formation",
  distilled: "distillation",
  filtered: "filtration",
  chromatographed: "chromatography",
  measured: "measurement",
  temperature_changed: "temperature change",
  energy_transferred: "energy transfer",
  heat_of_mixing: "heat of mixing",
  stirred: "mixing",
  mixed: "mixing",
  transported: "transport",
  gravity_settled: "settling",
  centrifuged: "centrifugation",
  ground: "grinding",
  irradiated: "irradiation",
  org_reacted: "reaction",
  decayed: "radioactive decay",
  burst: "vessel failure",
  layers_formed: "phase separation",
  dissolved: "dissolution",
  dissolved_in_solvent: "dissolution",
  transferred: "transfer",
  added: "addition",
  material_added: "addition",
  observed: "observation",
  inert: "no reaction",
  inert_in_solvent: "no reaction",
};

const PRIORITY = [
  "burst", "org_reacted", "electrolysed", "precipitated", "plated",
  "gas_evolved", "gas_absorbed", "gas_contained", "distilled", "filtered",
  "chromatographed", "layers_formed", "decayed", "heat_of_mixing",
  "measured", "temperature_changed", "energy_transferred", "centrifuged",
  "stirred", "mixed", "transported", "gravity_settled", "ground", "irradiated", "dissolved", "dissolved_in_solvent",
  "transferred", "added", "material_added", "observed", "inert", "inert_in_solvent",
];

function number(event: EngineEvent, key: string): number | undefined {
  const value = event[key];
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function eventVessel(event: EngineEvent): number | undefined {
  for (const key of ["vessel", "into", "receiver", "to", "from"]) {
    const value = number(event, key);
    if (value !== undefined) return value;
  }
  return undefined;
}

function quantities(event: EngineEvent): ResultQuantity[] {
  const values: ResultQuantity[] = [];
  const push = (label: string, key: string, unit: string, scale = 1) => {
    const value = number(event, key);
    if (value !== undefined) values.push({ label, value: value * scale, unit });
  };
  push("amount", "moles", "mol");
  push("mass", "grams", "g");
  push("energy", "delivered_j", "kJ", 0.001);
  if (!values.some((value) => value.label === "energy")) push("energy", "joules", "kJ", 0.001);
  push("speed", "rpm", "rpm");
  push("duration", "seconds", "s");
  push("voltage", "volts", "V");
  push("charge", "coulombs", "C");
  push("activity", "activity_bq", "Bq");
  push("resuspended", "resuspended_fraction", "%", 100);
  push("source A", "fraction_a", "%", 100);
  push("source B", "fraction_b", "%", 100);
  push("transferred", "fraction", "%", 100);
  const measured = number(event, "value");
  if (measured !== undefined && typeof event.unit === "string") {
    values.push({ label: "reading", value: measured, unit: event.unit });
  }
  return values.slice(0, 3);
}

function boundary(event: EngineEvent): string | undefined {
  if (event.event === "stirred" && event.rate_coupled === false) {
    return "suspension changed; reaction rates are not yet coupled";
  }
  if (event.event === "irradiated" && event.photolysis_coupled === false) {
    return "light was applied; photolysis is not yet coupled";
  }
  return undefined;
}

function provenance(event: EngineEvent): string | undefined {
  if (typeof event.provenance === "string" && event.provenance.trim()) {
    return event.provenance.trim();
  }
  if (event.provenance && typeof event.provenance === "object" && !Array.isArray(event.provenance)) {
    const record = event.provenance as Record<string, unknown>;
    const parts = [record.engine, record.dataset, record.model, record.source]
      .filter((part): part is string => typeof part === "string" && part.trim().length > 0);
    return parts.length > 0 ? parts.join(" · ") : undefined;
  }
  return undefined;
}

function significantEvent(events: EngineEvent[]): EngineEvent | undefined {
  for (const kind of PRIORITY) {
    for (let index = events.length - 1; index >= 0; index -= 1) {
      if (events[index]?.event === kind) return events[index];
    }
  }
  return undefined;
}

function observation(rendered: string[], equation?: string): string | undefined {
  for (let index = rendered.length - 1; index >= 0; index -= 1) {
    const line = rendered[index];
    if (!line) continue;
    const text = line.trim();
    if (text.length > 0 && text !== equation && !/^(?:.+\s)?(?:→|⇌)(?:\s.+)?$/.test(text)) {
      return text;
    }
  }
  return undefined;
}

/** Build a truthful UI digest from one accepted engine command. */
export function summarizeResult(
  events: unknown[],
  rendered: string[],
  before: Scene | null,
  after: Scene | null,
): ResultSummary | null {
  const typed = events.filter(
    (event): event is EngineEvent => Boolean(event && typeof event === "object" && !Array.isArray(event)),
  );
  const event = significantEvent(typed);
  if (!event || typeof event.event !== "string") return null;

  const vessel = eventVessel(event);
  const equation = typeof event.equation === "string" ? event.equation : undefined;
  const beforeTemperature = before?.vessels.find((item) => item.id === vessel)?.temperature_k;
  const afterTemperature = after?.vessels.find((item) => item.id === vessel)?.temperature_k;
  const temperatureDeltaK = beforeTemperature !== undefined && afterTemperature !== undefined
    ? afterTemperature - beforeTemperature
    : undefined;

  return {
    kind: CLASSIFICATIONS[event.event] ?? event.event.replaceAll("_", " "),
    vessel,
    equation,
    observation: observation(rendered, equation),
    temperatureDeltaK: temperatureDeltaK !== undefined && Math.abs(temperatureDeltaK) >= 0.05
      ? temperatureDeltaK
      : undefined,
    quantities: quantities(event),
    boundary: boundary(event),
    provenance: provenance(event),
  };
}
