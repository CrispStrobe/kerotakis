/**
 * GUI-033's parameter forms: seven verbs share one shape — a few fields,
 * one compiled grammar line. Each spec IS the affordance; the generic
 * ApparatusForm renders any of them, and every builder refuses nonsense
 * rather than emitting a broken command.
 */

export interface FormField {
  name: string;
  label: string;
  /** number | species (a shelf picker) | choice (a fixed list) */
  type: "number" | "species" | "choice";
  unit?: string;
  default: number | string;
  min?: number;
  max?: number;
  step?: number;
  /** `choice` only: the fixed options, in the order they are offered. */
  options?: { value: string; label: string }[];
}

/**
 * What is under the vessel, and how hot it can get.
 *
 * The engine grew a heat SOURCE: `heat v1 40kJ on candle` caps the vessel
 * at the flame that is heating it, because a candle cannot take anything
 * to 1500 °C however long it burns. Omitting the clause keeps the old
 * meaning — the bench default, a laboratory burner — so the panel that
 * said nothing was silently claiming a burner for every flame, including
 * the one the kids' kit calls a candle.
 *
 * The ceilings mirror `kerotakis-core::apparatus`; they are shown as a
 * readout so the choice is visible before the run rather than only in the
 * temperature that comes back. Engine-owned numbers, echoed here.
 */
export const HEAT_SOURCES: { value: string; label: string; ceilingC: number }[] = [
  { value: "burner", label: "Bunsen burner", ceilingC: 1500 },
  { value: "candle", label: "candle", ceilingC: 1400 },
  { value: "hotplate", label: "hotplate", ceilingC: 550 },
];

/**
 * The named source, or the burner the engine falls back to.
 *
 * A restored save from before the picker existed carries no source at all,
 * and a hand-edited one could carry a word the engine would refuse. Both
 * land on the default rather than on a command the bench rejects.
 */
export function heatSource(value: number | string | undefined): (typeof HEAT_SOURCES)[number] {
  const found = HEAT_SOURCES.find((source) => source.value === value);
  return found ?? HEAT_SOURCES[0]!;
}

export interface ApparatusSpec {
  /** Stable equipment identity; it need not equal the engine command verb. */
  verb: string;
  commandVerb?: string;
  title: string;
  blurb: string;
  fields: FormField[];
  build: (vessel: number, values: Record<string, number | string>) => string | null;
  secondary?: {
    label: string;
    build: (vessel: number, values: Record<string, number | string>) => string | null;
  };
  /** Immediate physical consequences of the chosen controls. Chemistry stays engine-owned. */
  readouts?: (values: Record<string, number | string>) => {
    label: string;
    value: number;
    unit: string;
    digits: number;
  }[];
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

const energyReadout = (watts: number | string | undefined, seconds: number | string | undefined) => {
  const power = pos(watts);
  const duration = pos(seconds);
  if (power === null || duration === null) return [];
  const joules = power * duration;
  return [{
    label: "delivered energy",
    value: joules >= 1000 ? joules / 1000 : joules,
    unit: joules >= 1000 ? "kJ" : "J",
    digits: joules >= 1000 ? 2 : 0,
  }];
};

const bunsenEnergyKj = (values: Record<string, number | string>): number | null => {
  const flame = num(values.flame);
  const air = num(values.air ?? 100);
  const seconds = pos(values.seconds);
  if (flame === null || flame <= 0 || flame > 100 || air === null || air < 0 || air > 100 || seconds === null || seconds > 300) {
    return null;
  }
  const collarEfficiency = 0.55 + 0.45 * air / 100;
  return Number((0.005 * flame * seconds * collarEfficiency).toFixed(3));
};

export const APPARATUS: ApparatusSpec[] = [
  {
    verb: "bunsen",
    commandVerb: "heat",
    title: "candle / Bunsen flame",
    blurb: "adjust a flame, then heat or test ignition",
    fields: [
      { name: "flame", label: "flame power", type: "number", unit: "%", default: 50, min: 0, max: 100, step: 5 },
      { name: "air", label: "air collar", type: "number", unit: "%", default: 70, min: 0, max: 100, step: 5 },
      { name: "seconds", label: "exposure", type: "number", unit: "s", default: 30, min: 1, max: 300 },
      // Which flame this actually is. The title has always said "candle /
      // Bunsen flame"; until the engine took a source, that slash was the
      // only place the difference existed.
      {
        name: "source",
        label: "heat source",
        type: "choice",
        default: "burner",
        options: [
          { value: "burner", label: "Bunsen burner" },
          { value: "candle", label: "candle" },
        ],
      },
    ],
    build: (v, f) => {
      // Bounded first near-field model: up to 500 W reaches the selected
      // vessel. Opening the collar raises the teaching heat-transfer
      // efficiency from 55% to 100%; the engine still owns temperature and
      // resulting chemistry. This is not a soot/CO combustion model.
      const energyKj = bunsenEnergyKj(f);
      // The source is named even when it is the default: a command that
      // omits it reads as "whatever the bench assumes", and the whole
      // point of the clause is that the flame is no longer an assumption.
      return energyKj === null ? null : `heat v${v + 1} ${energyKj}kJ on ${heatSource(f.source).value}`;
    },
    readouts: (f) => {
      const energyKj = bunsenEnergyKj(f);
      const ceiling = { label: "flame ceiling", value: heatSource(f.source).ceilingC, unit: "°C", digits: 0 };
      return energyKj === null
        ? [ceiling]
        : [{ label: "delivered energy", value: energyKj, unit: "kJ", digits: 3 }, ceiling];
    },
    secondary: {
      label: "touch flame to contents",
      build: (v, f) => {
        const flame = num(f.flame);
        return flame !== null && flame > 0 && flame <= 100 ? `ignite v${v + 1}` : null;
      },
    },
  },
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
    readouts: (f) => {
      const rpm = pos(f.rpm);
      if (rpm === null) return [];
      // Same 25 mm stir-bar path used by the engine's computed Stirred event.
      const tipSpeed = Math.PI * 0.025 * rpm / 60;
      return [{ label: "stir-bar tip speed", value: tipSpeed, unit: "m/s", digits: 3 }];
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
      // A hotplate is a hotplate: no picker, but the clause is still
      // written, because the bench's silent default is a BURNER and a
      // hotplate that borrows the burner's ceiling reaches 950 °C it does
      // not have.
      return watts === null || seconds === null ? null : `heat v${v + 1} ${watts * seconds}J on hotplate`;
    },
    readouts: (f) => [
      ...energyReadout(f.watts, f.seconds),
      { label: "plate ceiling", value: heatSource("hotplate").ceilingC, unit: "°C", digits: 0 },
    ],
  },
  {
    verb: "cool",
    title: "cooling bath",
    blurb: "set cooling power and time",
    fields: [
      { name: "watts", label: "cooling power", type: "number", unit: "W", default: 100, min: 1, max: 2000, step: 10 },
      { name: "seconds", label: "duration", type: "number", unit: "s", default: 30, min: 1, max: 3600 },
    ],
    build: (v, f) => {
      const watts = pos(f.watts);
      const seconds = pos(f.seconds);
      return watts === null || seconds === null ? null : `cool v${v + 1} ${watts * seconds}J`;
    },
    readouts: (f) => energyReadout(f.watts, f.seconds).map((readout) => ({ ...readout, label: "removed energy" })),
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
    readouts: (f) => {
      const rpm = pos(f.rpm);
      const radiusCm = pos(f.radius);
      if (rpm === null || radiusCm === null) return [];
      const angularSpeed = rpm * Math.PI * 2 / 60;
      const rcf = angularSpeed ** 2 * (radiusCm / 100) / 9.80665;
      return [{ label: "relative centrifugal force", value: rcf, unit: "× g", digits: 0 }];
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
    readouts: (f) => {
      const amps = pos(f.amps);
      const minutes = pos(f.minutes);
      if (amps === null || minutes === null) return [];
      const coulombs = amps * minutes * 60;
      // The engine's Faraday constant (C/mol e−), used by displacement::electrolyse.
      const electronMoles = coulombs / 96_485.332_12;
      return [
        { label: "electrical charge", value: coulombs, unit: "C", digits: coulombs < 100 ? 1 : 0 },
        { label: "electron amount", value: electronMoles, unit: "mol e⁻", digits: 5 },
      ];
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
    readouts: (f) => {
      const wavelengthNm = pos(f.wavelength);
      if (wavelengthNm === null) return [];
      // Same E = hc/λ constants as photochem::LightSource; eV is a display conversion.
      const joules = 6.626e-34 * 2.998e8 / (wavelengthNm * 1e-9);
      const electronVolts = joules / 1.602_176_634e-19;
      return [{ label: "photon energy", value: electronVolts, unit: "eV", digits: 3 }];
    },
  },
  {
    verb: "regulate",
    title: "balloon or gas bag",
    blurb: "hold a chosen pressure and gas volume with a flexible boundary",
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
