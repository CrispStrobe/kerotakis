/**
 * GUI-033's parameter forms: seven verbs share one shape — a few fields,
 * one compiled grammar line. Each spec IS the affordance; the generic
 * ApparatusForm renders any of them, and every builder refuses nonsense
 * rather than emitting a broken command.
 */

export interface FormField {
  name: string;
  label: string;
  /** number | species (a shelf picker) */
  type: "number" | "species";
  unit?: string;
  default: number | string;
  min?: number;
  max?: number;
  step?: number;
}

export interface ApparatusSpec {
  verb: string;
  title: string;
  blurb: string;
  fields: FormField[];
  build: (vessel: number, values: Record<string, number | string>) => string | null;
  warning?: (values: Record<string, number | string>) => string | null;
}

const num = (v: number | string | undefined): number | null => {
  const n = Number(v);
  return Number.isFinite(n) ? n : null;
};

const pos = (v: number | string | undefined): number | null => {
  const n = num(v);
  return n !== null && n > 0 ? n : null;
};

export const APPARATUS: ApparatusSpec[] = [
  {
    verb: "stir",
    title: "magnetic stirrer",
    blurb: "set rotation speed and mixing time",
    fields: [
      { name: "rpm", label: "rotation speed", type: "number", unit: "rpm", default: 500, min: 50, max: 2000, step: 50 },
      { name: "seconds", label: "duration", type: "number", unit: "s", default: 10, min: 1, max: 3600 },
    ],
    build: (v, f) => {
      const rpm = pos(f.rpm);
      const seconds = pos(f.seconds);
      return rpm === null || seconds === null ? null : `stir v${v + 1} ${rpm}rpm ${seconds}s`;
    },
  },
  {
    verb: "heat",
    title: "hotplate",
    blurb: "set heating power and time",
    fields: [
      { name: "watts", label: "heating power", type: "number", unit: "W", default: 250, min: 1, max: 2000, step: 10 },
      { name: "seconds", label: "duration", type: "number", unit: "s", default: 30, min: 1, max: 3600 },
    ],
    build: (v, f) => {
      const watts = pos(f.watts);
      const seconds = pos(f.seconds);
      return watts === null || seconds === null ? null : `heat v${v + 1} ${watts * seconds}J`;
    },
  },
  {
    verb: "centrifuge",
    title: "mini centrifuge",
    blurb: "separate particles by spinning a balanced tube",
    fields: [
      { name: "rpm", label: "rotation speed", type: "number", unit: "rpm", default: 3000, min: 100, max: 15000, step: 100 },
      { name: "seconds", label: "duration", type: "number", unit: "s", default: 60, min: 1, max: 3600 },
      { name: "radius", label: "rotor radius", type: "number", unit: "cm", default: 8, min: 3, max: 15, step: 0.5 },
      { name: "counterbalance", label: "counterbalance", type: "number", unit: "g", default: 0, min: 0, step: 0.01 },
    ],
    build: (v, f) => {
      const rpm = pos(f.rpm);
      const seconds = pos(f.seconds);
      const radius = pos(f.radius);
      const counterbalance = num(f.counterbalance);
      return rpm === null || seconds === null || radius === null || counterbalance === null || counterbalance < 0
        ? null
        : `centrifuge v${v + 1} ${rpm}rpm ${seconds}s ${radius}cm ${counterbalance}g`;
    },
    warning: (f) => {
      const sample = num(f.sampleMass);
      const counterbalance = num(f.counterbalance);
      if (sample === null || counterbalance === null) return null;
      const imbalance = Math.abs(sample - counterbalance);
      return imbalance > 0.1 ? "rotor out of balance — adjust the counterbalance" : null;
    },
  },
  {
    verb: "dilute",
    title: "wash bottle",
    blurb: "add water up to a volume",
    fields: [{ name: "volume", label: "to volume", type: "number", unit: "mL", default: 100, min: 1 }],
    build: (v, f) => {
      const volume = pos(f.volume);
      return volume === null ? null : `dilute v${v + 1} ${volume}mL`;
    },
  },
  {
    verb: "evaporate",
    title: "evaporating dish",
    blurb: "boil part of the liquid away",
    fields: [{ name: "fraction", label: "fraction", type: "number", default: 0.5, min: 0.05, max: 1, step: 0.05 }],
    build: (v, f) => {
      const fraction = num(f.fraction);
      return fraction === null || fraction <= 0 || fraction > 1
        ? null
        : `evaporate v${v + 1} ${fraction}`;
    },
  },
  {
    verb: "electrolyse",
    title: "electrodes and supply",
    blurb: "pass a current for a time",
    fields: [
      { name: "amps", label: "current", type: "number", unit: "A", default: 0.5, min: 0.001, step: 0.1 },
      { name: "minutes", label: "for", type: "number", unit: "min", default: 30, min: 1 },
    ],
    build: (v, f) => {
      const amps = pos(f.amps);
      const minutes = pos(f.minutes);
      return amps === null || minutes === null
        ? null
        : `electrolyse v${v + 1} ${amps}A ${minutes}min`;
    },
  },
  {
    verb: "grind",
    title: "mortar",
    blurb: "set a solid's particle size",
    fields: [
      { name: "species", label: "solid", type: "species", default: "" },
      { name: "diameter", label: "grain", type: "number", unit: "µm", default: 50, min: 1 },
    ],
    build: (v, f) => {
      const species = String(f.species ?? "").trim();
      const diameter = pos(f.diameter);
      return !species || diameter === null
        ? null
        : `grind v${v + 1} ${species} ${diameter}um`;
    },
  },
  {
    verb: "irradiate",
    title: "lamp",
    blurb: "shine light of one wavelength",
    fields: [
      { name: "wavelength", label: "wavelength", type: "number", unit: "nm", default: 254, min: 100, max: 1000 },
      { name: "irradiance", label: "irradiance", type: "number", unit: "W/m²", default: 10, min: 0.1 },
    ],
    build: (v, f) => {
      const wavelength = pos(f.wavelength);
      const irradiance = pos(f.irradiance);
      return wavelength === null || irradiance === null
        ? null
        : `irradiate v${v + 1} ${wavelength}nm ${irradiance}W/m2`;
    },
  },
  {
    verb: "regulate",
    title: "piston lid",
    blurb: "hold a set pressure over the vessel",
    fields: [
      { name: "pressure", label: "pressure", type: "number", unit: "bar", default: 1.5, min: 0.1, step: 0.1 },
      { name: "volume", label: "headspace", type: "number", unit: "mL", default: 500, min: 10 },
    ],
    build: (v, f) => {
      const pressure = pos(f.pressure);
      const volume = pos(f.volume);
      return pressure === null || volume === null
        ? null
        : `regulate v${v + 1} ${pressure}bar ${volume}mL`;
    },
  },
  {
    verb: "sweep",
    title: "carrier-gas line",
    blurb: "purge the headspace with inert gas",
    fields: [
      { name: "pressure", label: "pressure", type: "number", unit: "bar", default: 1, min: 0.1, step: 0.1 },
    ],
    build: (v, f) => {
      const pressure = pos(f.pressure);
      return pressure === null ? null : `sweep v${v + 1} ${pressure}bar`;
    },
  },
];
