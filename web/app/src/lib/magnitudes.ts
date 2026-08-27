/**
 * GUI-059: effect magnitudes — maps engine event amounts onto visual
 * scale factors for the vessel's transient effects. Every factor names
 * its source event field so the link is auditable.
 *
 * All scale functions return a number in [0, 1] that the Vessel component
 * uses as a CSS variable. 0 = minimum visible, 1 = maximum (clamp).
 */

/** The shape of an engine event, loosely typed (serde JSON). */
export type EngineEvent = Record<string, unknown>;

/** One engine-computed chromatography peak, retained for physical playback. */
export interface ChromatographyBand {
  species: string;
  retentionTimeS: number;
  widthS: number;
  relativeArea: number;
  partitionK: number;
}

/** The typed result of looking closely, as computed by the appearance model. */
export interface InspectionAppearance {
  liquidRgb?: [number, number, number];
  cloudiness: number;
  deposit?: { species: string; rgb: [number, number, number] };
  bubbling: boolean;
}

/** A solid physically retained by filter paper, captured from the engine scene. */
export interface FilterResidue {
  species: string;
  name: string;
  moles: number;
  colour: string;
}

/** Engine-computed still cut used to configure the physical rig. */
export interface DistillationRun {
  waterMoles: number;
  ethanolMoles: number;
  startK: number;
  endK: number;
  stages: number;
  energyKj: number;
  azeotropic: boolean;
}

/** Lower-layer cut emitted by the separatory-funnel operator. */
export interface DrainRun {
  solvent: string;
  moles: number;
  lowerColour?: string;
  upperColour?: string;
}

export interface MagneticSolid {
  species: string;
  name: string;
  moles: number;
  colour: string;
}

/** Engine classification plus pre-transfer physical inventory. */
export interface MagneticRun {
  attractedSpecies: string[];
  remainedSpecies: string[];
  attracted: MagneticSolid[];
}

export interface SettlingPopulation {
  species: string;
  particleDiameterUm: number;
  terminalSpeedMS: number;
  distanceM: number;
  separatedFraction: number;
  direction: string;
  colour?: string;
}

/** Stokes-law gravity settling emitted while bench time advances. */
export interface SettlingRun {
  seconds: number;
  populations: SettlingPopulation[];
}

export interface CentrifugePopulation extends SettlingPopulation {
  particleSizeAssumed: boolean;
  particleDensityKgM3: number;
}

export interface CentrifugeRun {
  rpm: number;
  seconds: number;
  rotorRadiusM: number;
  rcf: number;
  sampleMassG: number;
  counterbalanceG: number;
  imbalanceG: number;
  fluidDensityKgM3: number;
  dynamicViscosityPaS: number;
  populations: CentrifugePopulation[];
  stateCoupled: boolean;
}

/** A visual effect with magnitude, produced by {@link effectFromEvent}. */
export interface Effect {
  kind: string;
  at: number;
  /** Visible lifetime for time-bearing operations, derived from engine seconds. */
  durationMs?: number;
  /** 0–1 visual intensity, from the event's amount field. */
  magnitude: number;
  /** Flame colour CSS value, if the event carries one. */
  flameColour?: string;
  /** Source and destination for spatial bench effects such as pouring. */
  source?: number;
  target?: number;
  /** Computed endpoint, used by thermal presentation without duplicating state. */
  temperatureK?: number;
  /** Scalar emitted by an engine-owned measurement event. */
  reading?: number;
  /** Unit emitted with the measurement. */
  unit?: string;
  /** Engine-owned chromatography result; presentation only normalises its axes. */
  bands?: ChromatographyBand[];
  voidTimeS?: number;
  plates?: number;
  outsideMethod?: string[];
  appearance?: InspectionAppearance;
  /** Physical setup connecting source and target vessels. */
  operation?: "pour" | "filter" | "drain" | "magnet" | "distil" | "cell";
  /** Computed pre-transfer source-liquid colour, captured before scene replacement. */
  fluidColour?: string;
  /** Engine-scene solids left on the paper during a filtration. */
  filterResidue?: FilterResidue[];
  /** Boiling range, column and energy bill emitted by the VLE solver. */
  distillation?: DistillationRun;
  /** Engine-selected lower layer and its pre-drain scene colours. */
  drain?: DrainRun;
  /** Solids selected by the engine's magnetic-property data. */
  magnetic?: MagneticRun;
  settling?: SettlingRun;
  centrifuge?: CentrifugeRun;
}

/** Clamp `x` into [0, 1], scaling linearly from 0 at `lo` to 1 at `hi`. */
function scale(x: number, lo: number, hi: number): number {
  if (hi <= lo) return x >= lo ? 1 : 0;
  return Math.max(0, Math.min(1, (x - lo) / (hi - lo)));
}

/**
 * Named-colour → CSS colour for flame rendering.
 * Keys match the engine's `FlameTest.colour` / `Ignited.flame` strings.
 */
const FLAME_COLOURS: Record<string, string> = {
  lilac: "#c8a2c8",
  violet: "#9b30ff",
  yellow: "#ffd700",
  orange: "#ff8c00",
  "brick-red": "#cb4154",
  red: "#ff2400",
  green: "#00e676",
  "blue-green": "#0dbf8c",
  blue: "#1e90ff",
  crimson: "#dc143c",
  white: "#ffffff",
};
const FLAME_KEYS = Object.keys(FLAME_COLOURS).sort((a, b) => b.length - a.length);

function flameCss(colour: unknown): string | undefined {
  const phrase = String(colour ?? "").trim().toLowerCase();
  if (!phrase) return undefined;
  const key = FLAME_COLOURS[phrase] ? phrase : FLAME_KEYS.find((candidate) => phrase.includes(candidate));
  return key ? FLAME_COLOURS[key] : undefined;
}

// ── Per-event-kind magnitude extractors ──────────────────────────────
// Each returns [magnitude, flameColour?]. The magnitude tracks a single
// event field named in the comment; the constants bound what "small" and
// "large" look like on the bench.

// event.moles — GasEvolved: 0.001 mol is a wisp, 0.1 mol is vigorous.
function gasMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.001, 0.1);
}

// event.moles — Precipitated: 0.0005 mol is a few specks, 0.05 mol is
// a heavy snowfall.
function precipMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.0005, 0.05);
}

// event.moles — Evaporated/Distilled: 0.01 mol is gentle, 0.5 mol is
// a rolling boil.
function steamMag(e: EngineEvent): number {
  const moles = Number(e.moles ?? 0) + Number(e.water ?? 0) + Number(e.ethanol ?? 0);
  return scale(moles, 0.01, 0.5);
}

// event.fraction — Transferred: the visible stream follows the amount
// the engine says actually moved, not the control's requested amount.
function transferMag(e: EngineEvent): number {
  return scale(Number(e.fraction ?? 0.5), 0.02, 1);
}

// event.from/to (Kelvin) — a 2 K nudge is subtle; 150 K is dramatic.
function thermalMag(e: EngineEvent): number {
  return scale(Math.abs(Number(e.to ?? e.temperature ?? 0) - Number(e.from ?? e.temperature ?? 0)), 2, 150);
}

// event.moles — Electrolysed: 0.0001 mol is a trickle, 0.01 mol is
// vigorous bubbling.
function electroMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.0001, 0.01);
}

// event.fraction_a + event.fraction_b — Mixed: total pour fraction
// from 0 (nothing moved) to 1 (both vessels emptied).
function mixMag(e: EngineEvent): number {
  const a = Number(e.fraction_a ?? 0);
  const b = Number(e.fraction_b ?? 0);
  return scale(a + b, 0.1, 1.5);
}

// event.tip_speed_m_s — a 25 mm bar at 50 rpm is a gentle turn;
// 2000 rpm is the configured bench maximum. The animation follows the
// physical linear speed emitted by the engine, not the requested UI value.
function stirMag(e: EngineEvent): number {
  return scale(Number(e.tip_speed_m_s ?? 0), 0.065, 2.62);
}

// event.surface_area_m2 — useful powder areas span orders of magnitude.
// 0.001 m² is a few coarse grains; 10 m² is a fine powder.
function grindMag(e: EngineEvent): number {
  const area = Math.max(0.001, Number(e.surface_area_m2 ?? 0.001));
  return scale(Math.log10(area), -3, 1);
}

// event.rcf — rotor blur follows computed centrifugal acceleration. A small
// classroom spinner starts near 10×g; the configured mini-centrifuge tops out
// around 20,000×g depending on radius.
function centrifugeMag(e: EngineEvent): number {
  const rcf = Math.max(10, Number(e.rcf ?? 10));
  return scale(Math.log10(rcf), 1, 4.3);
}

// event.volume (Liters) — Diluted: 0.01 L is a squirt, 0.5 L is a flood.
function diluteMag(e: EngineEvent): number {
  return scale(Number(e.volume ?? 0), 0.01, 0.5);
}

// event.energy_j — Ignited: 100 J is a small flash and 50 kJ fills the
// available flame envelope. FlameTest has no combustion energy of its own,
// so it stays a restrained burner flame. Colour always comes from
// event.flame / event.colour.
function flameMag(e: EngineEvent): [number, string | undefined] {
  const css = flameCss(e.flame ?? e.colour);
  const magnitude = e.event === "ignited"
    ? (e.energy_j === undefined ? 0.35 : scale(Number(e.energy_j), 100, 50_000))
    : 0.28;
  return [magnitude, css];
}

/**
 * Map one engine event to a visual effect with magnitude.
 * Returns null if the event kind has no visual mapping.
 */
export function effectFromEvent(e: EngineEvent): Effect | null {
  const kind = String(e.event ?? "");
  const now = Date.now();

  switch (kind) {
    case "gas_evolved":
      return { kind: "vent", at: now, magnitude: gasMag(e) };
    case "gas_produced":
      return { kind: "vent", at: now, magnitude: gasMag(e) };
    case "foam_changed":
      return {
        kind: "foam",
        at: now,
        magnitude: scale(Number(e.height_cm ?? 0), 0.5, 30),
      };
    case "precipitated":
      return { kind: "precipitate", at: now, magnitude: precipMag(e) };
    case "evaporated":
      return { kind: "evaporate", at: now, magnitude: steamMag(e) };
    case "distilled":
      return {
        kind: "evaporate",
        at: now,
        magnitude: steamMag(e),
        source: Number(e.from ?? 0),
        target: Number(e.to ?? 0),
        operation: "distil",
        distillation: {
          waterMoles: Number(e.water ?? 0),
          ethanolMoles: Number(e.ethanol ?? 0),
          startK: Number(e.at ?? 0),
          endK: Number(e.ended ?? e.at ?? 0),
          stages: Math.max(1, Number(e.stages ?? 1)),
          energyKj: Math.max(0, Number(e.energy_kj ?? 0)),
          azeotropic: Boolean(e.azeotropic),
        },
      };
    case "electrolysed":
      return { kind: "electrolyse", at: now, magnitude: electroMag(e) };
    case "mixed":
      return {
        kind: "swirl",
        at: now,
        magnitude: mixMag(e),
        source: Number(e.a ?? 0),
        target: Number(e.into ?? 0),
      };
    case "stirred":
      return {
        kind: "swirl",
        at: now,
        magnitude: stirMag(e),
        durationMs: Math.min(8000, Math.max(1200, Number(e.seconds ?? 2.2) * 1000)),
      };
    case "ground":
      return { kind: "grind", at: now, magnitude: grindMag(e) };
    case "centrifuged":
      return {
        kind: "centrifuge",
        at: now,
        magnitude: centrifugeMag(e),
        durationMs: Math.min(8000, Math.max(1200, Number(e.seconds ?? 2.2) * 1000)),
        centrifuge: {
          rpm: Number(e.rpm ?? 0),
          seconds: Number(e.seconds ?? 0),
          rotorRadiusM: Number(e.rotor_radius_m ?? 0),
          rcf: Number(e.rcf ?? 0),
          sampleMassG: Number(e.sample_mass_g ?? 0),
          counterbalanceG: Number(e.counterbalance_g ?? 0),
          imbalanceG: Number(e.imbalance_g ?? 0),
          fluidDensityKgM3: Number(e.fluid_density_kg_m3 ?? 0),
          dynamicViscosityPaS: Number(e.dynamic_viscosity_pa_s ?? 0),
          populations: (Array.isArray(e.separations) ? e.separations : []).map((value) => {
            const separation = value && typeof value === "object" ? value as Record<string, unknown> : {};
            return {
              species: String(separation.species ?? ""),
              particleDiameterUm: Number(separation.particle_diameter_um ?? 0),
              particleSizeAssumed: Boolean(separation.particle_size_assumed),
              particleDensityKgM3: Number(separation.particle_density_kg_m3 ?? 0),
              terminalSpeedMS: Number(separation.terminal_speed_m_s ?? 0),
              distanceM: Number(separation.distance_m ?? 0),
              separatedFraction: Math.max(0, Math.min(1, Number(separation.separated_fraction ?? 0))),
              direction: String(separation.direction ?? ""),
            };
          }),
          stateCoupled: Boolean(e.state_coupled),
        },
      };
    case "gravity_settled": {
      const populations = (Array.isArray(e.separations) ? e.separations : []).map((value) => {
        const separation = value && typeof value === "object" ? value as Record<string, unknown> : {};
        return {
          species: String(separation.species ?? ""),
          particleDiameterUm: Number(separation.particle_diameter_um ?? 0),
          terminalSpeedMS: Number(separation.terminal_speed_m_s ?? 0),
          distanceM: Number(separation.distance_m ?? 0),
          separatedFraction: Math.max(0, Math.min(1, Number(separation.separated_fraction ?? 0))),
          direction: String(separation.direction ?? ""),
        };
      });
      const seconds = Math.max(0, Number(e.seconds ?? 0));
      return {
        kind: "settle",
        at: now,
        durationMs: Math.min(8000, Math.max(1200, seconds * 1000)),
        magnitude: populations.reduce((strongest, population) => Math.max(strongest, population.separatedFraction), 0),
        settling: { seconds, populations },
      };
    }
    case "transferred":
      return {
        kind: "pour",
        at: now,
        magnitude: transferMag(e),
        source: Number(e.from ?? 0),
        target: Number(e.to ?? 0),
        operation: "pour",
      };
    case "filtered":
      return {
        kind: "pour",
        at: now,
        // The session replaces this fallback with a scale derived from the
        // source vessel's actual retained-solid inventory when available.
        magnitude: 0.45,
        source: Number(e.from ?? 0),
        target: Number(e.to ?? 0),
        operation: "filter",
      };
    case "magnet_separated": {
      const attractedSpecies = Array.isArray(e.attracted) ? e.attracted.map(String) : [];
      return {
        kind: "magnet",
        at: now,
        durationMs: 3000,
        magnitude: scale(attractedSpecies.length, 0, 4),
        source: Number(e.from ?? 0),
        target: Number(e.to ?? 0),
        operation: "magnet",
        magnetic: {
          attractedSpecies,
          remainedSpecies: Array.isArray(e.remained) ? e.remained.map(String) : [],
          attracted: [],
        },
      };
    }
    case "drained":
      return {
        kind: "pour",
        at: now,
        magnitude: scale(Number(e.moles ?? 0), 0.001, 2),
        source: Number(e.from ?? 0),
        target: Number(e.to ?? 0),
        operation: "drain",
        drain: {
          solvent: String(e.solvent ?? ""),
          moles: Number(e.moles ?? 0),
        },
      };
    case "cell_voltage":
      return {
        kind: "connection",
        at: now,
        magnitude: scale(Math.abs(Number(e.volts ?? 0)), 0.05, 2.5),
        source: Number(e.anode ?? 0),
        target: Number(e.cathode ?? 0),
        operation: "cell",
      };
    case "diluted":
      return { kind: "swirl", at: now, magnitude: diluteMag(e) };
    case "dissolved":
      return { kind: "dissolve", at: now, magnitude: 1 };
    case "plated":
      return { kind: "plate", at: now, magnitude: 1 };
    case "temperature_changed": {
      const to = Number(e.to ?? 0);
      return {
        kind: to >= Number(e.from ?? to) ? "heat" : "cool",
        at: now,
        magnitude: thermalMag(e),
        temperatureK: to,
      };
    }
    case "heat_of_mixing": {
      const joules = Number(e.joules ?? 0);
      return {
        kind: joules >= 0 ? "heat" : "cool",
        at: now,
        magnitude: scale(Math.abs(joules), 5, 5000),
      };
    }
    case "state_changed":
      return {
        kind: String(e.to ?? "") === "solid" ? "freeze" : "phase-change",
        at: now,
        magnitude: scale(Math.abs(Number(e.shifted_by ?? 0)), 0, 20),
        temperatureK: Number(e.at ?? 0),
      };
    case "burst":
      return {
        kind: "burst",
        at: now,
        magnitude: scale(Number(e.at_pa ?? 0) / Math.max(1, Number(e.rating_pa ?? 1)), 1, 2),
      };
    case "ignited":
    case "flame_test": {
      const [mag, colour] = flameMag(e);
      return { kind: kind === "flame_test" ? "flame_test" : "ignite", at: now, magnitude: mag, flameColour: colour };
    }
    default:
      return null;
  }
}

/**
 * Vessel ID from an event — events use `vessel`, `from`, or `into`.
 */
export function vesselOf(e: EngineEvent): number {
  return Number(e.vessel ?? e.from ?? e.into ?? e.anode ?? 0);
}
