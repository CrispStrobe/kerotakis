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
  /**
   * `number` only: entered as free text rather than through
   * `<input type="number">`.
   *
   * A spinner's value is parsed by the BROWSER against the BROWSER's
   * locale, which is not the locale this app is speaking. A German reader
   * on an English-locale Chrome who types `7,5` into a number input hands
   * the app an empty string, and the panel says the field is empty while
   * the reader is looking at a number. Where the reader is expected to
   * type a decimal rather than nudge a step, the field takes text and
   * `num()` below does the parsing — for both separators.
   */
  decimal?: boolean;
  /**
   * Whether this field applies to the values currently set.
   *
   * A form whose fields are all always shown can only offer one way of
   * saying a thing. The bunsen panel now has two — derive the energy from
   * the flame, or state it — and the controls that belong to the other
   * one are not merely irrelevant, they are misleading: a flame slider
   * sitting under a typed energy reads as if it still did something.
   *
   * Hidden, never dropped: `values` keeps every field's entry, so
   * switching back finds the flame where it was left.
   */
  when?: (values: Record<string, number | string>) => boolean;
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

/**
 * A field's value as a number, in either decimal notation.
 *
 * The reader of this app is German (I18N), and a German reader writes
 * `7,5`. `Number("7,5")` is `NaN`, so without this line a typed comma is
 * the same as an empty field — a run button that greys out with no
 * explanation. The command that comes out the other end always carries a
 * POINT, because the grammar has one spelling of a number and it is not
 * the reader's.
 *
 * A lone comma is always the DECIMAL point, never a thousands group —
 * `1,234` is 1.234. That reading is safe here only because the fields
 * that take it are bounded well under a thousand (`BUNSEN_MAX_KJ` is
 * 150), so a grouping separator has nothing to group; a field that could
 * hold four digits would have to ask rather than guess. A string with
 * more than one comma, or with both a comma and a point, is refused
 * outright: that is a typo, not a convention.
 */
const num = (v: number | string | undefined): number | null => {
  if (typeof v === "string") {
    const trimmed = v.trim();
    if (trimmed === "") return null;
    const commas = (trimmed.match(/,/g) ?? []).length;
    if (commas > 1 || (commas === 1 && trimmed.includes("."))) return null;
    const n = Number(trimmed.replace(",", "."));
    return Number.isFinite(n) ? n : null;
  }
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

/**
 * The most this panel's own model can deliver: full flame, open collar,
 * the longest exposure the exposure field accepts.
 *
 * Derived from the bounds the fields already declare rather than picked,
 * so it cannot drift away from them. It is what a typed energy is judged
 * against (GUI-113): the point of typing the number is to say exactly how
 * much, not to escape the burner the panel is modelling.
 */
export const BUNSEN_MAX_KJ = 0.005 * 100 * 300 * 1;

/** Which control says how much energy the flame delivers (GUI-113). */
export const ENERGY_ENTRY = { derived: "flame", typed: "energy" } as const;

const bunsenTyped = (values: Record<string, number | string>): boolean =>
  values.entry === ENERGY_ENTRY.typed;

const bunsenEnergyKj = (values: Record<string, number | string>): number | null => {
  // The owner's ask: "we should be able to directly change, for Erhitzen,
  // what amount of 'zugeführte Energie' we add". The panel already PRINTED
  // that number as a readout; in this mode the readout is the input, and
  // the flame controls that used to compute it step aside.
  if (bunsenTyped(values)) {
    const typed = num(values.energy);
    if (typed === null || typed <= 0 || typed > BUNSEN_MAX_KJ) return null;
    // Three decimals, the same precision the derived path rounds to — a
    // pasted 0.1+0.2 must not reach the grammar as 0.30000000000000004.
    return Number(typed.toFixed(3));
  }
  const flame = num(values.flame);
  const air = num(values.air ?? 100);
  const seconds = pos(values.seconds);
  if (flame === null || flame <= 0 || flame > 100 || air === null || air < 0 || air > 100 || seconds === null || seconds > 300) {
    return null;
  }
  const collarEfficiency = 0.55 + 0.45 * air / 100;
  return Number((0.005 * flame * seconds * collarEfficiency).toFixed(3));
};

/**
 * Why a typed energy is refused, in words, or null.
 *
 * `build` returning null already greys the run button out; on a field the
 * reader has just typed into, a grey button with no sentence beside it is
 * the app refusing to say what is wrong with what they wrote. The three
 * refusals are distinct on purpose — an unreadable number, a number that
 * is not an amount, and a number larger than this burner has.
 */
const bunsenEnergyWarning = (values: Record<string, number | string>): string | null => {
  if (!bunsenTyped(values)) return null;
  const raw = values.energy;
  if (typeof raw === "string" && raw.trim() === "") return null;
  const typed = num(raw);
  if (typed === null) return "that is not a number — write it as 7.5 or 7,5";
  if (typed <= 0) return "an energy of nothing heats nothing — write a number above zero";
  if (typed > BUNSEN_MAX_KJ) {
    return "more than this flame can deliver: at full power with the collar open it reaches 150 kJ in the longest exposure the panel allows";
  }
  return null;
};

export const APPARATUS: ApparatusSpec[] = [
  {
    verb: "bunsen",
    commandVerb: "heat",
    title: "candle / Bunsen flame",
    blurb: "adjust a flame, then heat or test ignition",
    fields: [
      // GUI-113. Which of the two controls owns the energy — and it is a
      // control of its own rather than a mode hidden in a blank field,
      // because "leave it empty and the flame decides" is a rule nobody
      // can see and a screen reader cannot announce.
      {
        name: "entry",
        label: "energy from",
        type: "choice",
        default: ENERGY_ENTRY.derived,
        options: [
          { value: ENERGY_ENTRY.derived, label: "flame and exposure" },
          { value: ENERGY_ENTRY.typed, label: "a number I type" },
        ],
      },
      {
        name: "energy",
        label: "delivered energy",
        type: "number",
        unit: "kJ",
        decimal: true,
        default: 7.5,
        min: 0,
        max: BUNSEN_MAX_KJ,
        when: bunsenTyped,
      },
      { name: "flame", label: "flame power", type: "number", unit: "%", default: 50, min: 0, max: 100, step: 5, when: (f) => !bunsenTyped(f) },
      { name: "air", label: "air collar", type: "number", unit: "%", default: 70, min: 0, max: 100, step: 5, when: (f) => !bunsenTyped(f) },
      { name: "seconds", label: "exposure", type: "number", unit: "s", default: 30, min: 1, max: 300, when: (f) => !bunsenTyped(f) },
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
    warning: bunsenEnergyWarning,
    readouts: (f) => {
      const energyKj = bunsenEnergyKj(f);
      const ceiling = { label: "flame ceiling", value: heatSource(f.source).ceilingC, unit: "°C", digits: 0 };
      // Typed, the energy is on the field the reader is looking at;
      // echoing it back as a readout is the panel telling them what they
      // just wrote. The ceiling is the fact the flame still owns.
      if (bunsenTyped(f)) return [ceiling];
      return energyKj === null
        ? [ceiling]
        : [{ label: "delivered energy", value: energyKj, unit: "kJ", digits: 3 }, ceiling];
    },
    secondary: {
      label: "touch flame to contents",
      build: (v, f) => {
        // `ignite` has no energy in it: touching the flame to the contents
        // is the same gesture whichever control set the burner, so the
        // typed mode keeps it rather than losing a verb to a radio button.
        if (bunsenTyped(f)) return `ignite v${v + 1}`;
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
