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

export interface StirredSolid {
  species: string;
  name: string;
  moles: number;
  colour: string;
}

export interface StirRun {
  rpm: number;
  seconds: number;
  barLengthM: number;
  tipSpeedMS: number;
  resuspendedFraction: number;
  rateCoupled: boolean;
  solids: StirredSolid[];
}

export interface GasTestRun {
  test: "pop" | "glowing_splint" | "limewater" | "damp_litmus" | string;
  positive: boolean;
  notes: string;
}

export interface WaftRun {
  notes: { species: string; description: string }[];
}

export interface PressureControlRun {
  pressurePa: number;
  initialVolumeL: number;
  trappedGasMoles: number;
}

export interface DilutionRun {
  volumeL: number;
  waterMoles: number;
}

export interface SweepRun {
  pressurePa: number;
}

export interface IrradiationRun {
  wavelengthNm: number;
  irradianceWM2: number;
  photolysisCoupled: boolean;
}

export interface ElectrolysisRun {
  species: string;
  amps: number;
  seconds: number;
  coulombs: number;
  electronMoles: number;
  productMoles: number;
  grams: number;
  electronsPerIon: number;
}

export interface ThermalRun {
  heating: boolean;
  requestedJ: number;
  deliveredJ: number;
  timeCoupled: boolean;
}

/** Engine-owned surface receiving material after a spill or breakage. */
export interface SpillRun {
  surface: "bench" | "tray" | "floor" | string;
  location: string;
  fraction: number;
}

/**
 * The engine's own numbers for one solid appearing in or leaving a vessel,
 * gathered from the event (species, moles) and the scene row that owns the
 * substance (pure-solid volume, colour). `molarVolumeLPerMol` is what makes
 * a mole of a fluffy hydroxide draw bigger than a mole of a dense sulfate.
 */
export interface SolidYield {
  species: string;
  name: string;
  /** Moles that precipitated or dissolved this step. */
  moles: number;
  /** Registry mass ÷ density, litres per mole. */
  molarVolumeLPerMol: number;
  /** `moles × molarVolumeLPerMol`, the volume the step actually moved. */
  volumeL: number;
  /** The species' own colour, as a CSS value. */
  colour?: string;
}

/**
 * The gas above the liquid: how much room it takes up, and what set that.
 * `volumeL` is the engine's own figure where an event carried one
 * (`vessel_sealed.headspace_volume`), and otherwise the ideal-gas volume
 * the trapped moles occupy at the held pressure and the vessel temperature.
 */
export interface HeadspaceRun {
  volumeL: number;
  moles: number;
  /** The held pressure, where the event stated one. */
  pressurePa?: number;
  /** The temperature the volume was computed at, where one was stated. */
  temperatureK?: number;
  /** "engine" when the event named the volume; "ideal-gas" when derived. */
  source: "engine" | "ideal-gas";
}

/**
 * One engine-computed phase transition. `atK` is the temperature the engine
 * actually used — a boiling plateau already carrying its pressure and
 * colligative shifts — so the stage can hold the boil at the engine's
 * number instead of a hard-coded 373 K.
 */
export interface PhaseChangeRun {
  species: string;
  /** Phase left, engine wire tag: solid | liquid | gas. */
  from: string;
  /** Phase entered, engine wire tag. */
  to: string;
  /** Transition temperature used, K. */
  atK: number;
  /** K away from the pure solvent's transition; negative when solutes lowered it. */
  shiftedByK: number;
  /** Vessel pressure that set the boiling point, kPa — routed boils only. */
  pressureKpa?: number;
  /** Which correlation answered, or why none did — routed boils only. */
  route?: string;
  /** The saturation model's own name from its pack row — routed boils only. */
  model?: string;
}

/** A visual effect with magnitude, produced by {@link effectFromEvent}. */
export interface Effect {
  kind: string;
  at: number;
  /** Visible lifetime for time-bearing operations, derived from engine seconds. */
  durationMs?: number;
  /** 0–1 visual intensity, from the event's amount field. */
  magnitude: number;
  /** Engine-accepted transfer fraction. Visuals may scale to this value,
   * but must never infer or revise it from pointer travel or animation. */
  acceptedTransferFraction?: number;
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
  stir?: StirRun;
  gasTest?: GasTestRun;
  waft?: WaftRun;
  pressureControl?: PressureControlRun;
  dilution?: DilutionRun;
  sweep?: SweepRun;
  irradiation?: IrradiationRun;
  electrolysis?: ElectrolysisRun;
  thermal?: ThermalRun;
  /** Presentation metadata only; the engine remains owner of spilled material. */
  spill?: SpillRun;
  /** Engine-computed transition, including the temperature it happened at. */
  phase?: PhaseChangeRun;
  /** The engine's species id, where the event names one substance. */
  species?: string;
  /** Engine-owned amount and molar volume of a solid appearing or leaving. */
  solid?: SolidYield;
  /** Engine-owned headspace, for the lid and the piston. */
  headspace?: HeadspaceRun;
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
// event.moles — GasEvolved/GasContained: a millimole is a few beads on
// the glass, a tenth of a mole is a rolling fizz. Logarithmic on purpose:
// a spoon of baking soda in vinegar makes ~0.01 mol of CO₂ (a quarter of
// a litre of gas), which the old linear ramp drew as two shy bubbles.
function gasMag(e: EngineEvent): number {
  const moles = Number(e.moles ?? 0);
  if (!(moles > 0)) return 0;
  return scale(Math.log10(moles), Math.log10(0.001), Math.log10(0.1));
}

// event.moles — Precipitated: 0.0005 mol is a few specks, 0.05 mol is
// a heavy snowfall.
function precipMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.0005, 0.05);
}

// event.moles — Evaporated/Distilled: 0.01 mol is gentle, 0.5 mol is
// a rolling boil. Shared with the boil so the plume and the bubbles are
// sized by one number (see `vapourIntensity`).
function steamMag(e: EngineEvent): number {
  const moles = Number(e.moles ?? 0) + Number(e.water ?? 0) + Number(e.ethanol ?? 0);
  return vapourIntensity(moles);
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
 * Water's normal boiling point, K. Used only while no engine phase event is
 * live: `state_changed` carries the plateau the solver actually held the
 * vessel at (pressure shift and colligative elevation already in it), and
 * that number wins whenever it is on the bench. Scene v1 has no standing
 * `boiling_point_k`, so between events the stage falls back to pure water at
 * one atmosphere rather than inventing a correlation of its own.
 */
export const NORMAL_BOILING_K = 373.15;

/** Below this the bench draws no incandescence: a solid is merely hot. */
export const INCANDESCENCE_ONSET_K = 800;

/**
 * Vapour intensity from the moles of vapour a step actually made.
 * A hundredth of a mole (≈0.25 L of steam) is a gentle simmer; half a
 * mole is a rolling boil. Monotone and bounded in [0, 1].
 */
export function vapourIntensity(moles: number): number {
  return scale(moles, 0.01, 0.5);
}

/**
 * Visible incandescence of a body at `temperatureK`, or null when it glows
 * only in the infrared. `fraction` is how strongly the glow reads (0 at the
 * onset, 1 by white heat); `rgb` is the colour, from the standard
 * blackbody-locus approximation to Planck's law — deep red at 800 K,
 * orange near 1500 K, amber near 2500 K, near-white above 3500 K.
 */
export function incandescence(
  temperatureK: number,
): { fraction: number; rgb: [number, number, number] } | null {
  if (!(temperatureK >= INCANDESCENCE_ONSET_K)) return null;
  const t = Math.min(temperatureK, 6500) / 100;
  const byte = (value: number) => Math.max(0, Math.min(255, Math.round(value)));
  const red = t <= 66 ? 255 : byte(329.698_727_446 * Math.pow(t - 60, -0.133_204_759_2));
  const green = t <= 66
    ? byte(99.470_802_586_1 * Math.log(t) - 161.119_568_166_1)
    : byte(288.122_169_528_3 * Math.pow(t - 60, -0.075_514_849_2));
  const blue = t >= 66 ? 255 : t <= 19 ? 0 : byte(138.517_731_223_1 * Math.log(t - 10) - 305.044_792_730_7);
  return {
    fraction: scale(temperatureK, INCANDESCENCE_ONSET_K, 2000),
    rgb: [byte(red), byte(green), byte(blue)],
  };
}

/**
 * Dew point of air at `ambientK` and `relativeHumidity` (0–1), by the
 * Magnus–Tetens approximation. Room air at 20 °C and 50 % RH dews at
 * about 9 °C — which is why a beaker of ice water beads and a beaker of
 * tap water does not.
 */
export function dewPointK(ambientK: number, relativeHumidity: number): number {
  const a = 17.62;
  const b = 243.12;
  const celsius = ambientK - 273.15;
  const rh = Math.max(0.001, Math.min(1, relativeHumidity));
  const gamma = Math.log(rh) + (a * celsius) / (b + celsius);
  return b * gamma / (a - gamma) + 273.15;
}

/**
 * How heavily a vessel wall at `surfaceK` beads with condensation, 0–1:
 * nothing until the glass is below the room's dew point, saturated 15 K
 * below it. Returns 0 once the wall is below freezing, where the same
 * water arrives as frost and the frost layer draws it instead.
 */
export function condensationFilm(
  surfaceK: number,
  ambientK = 293.15,
  relativeHumidity = 0.5,
): number {
  if (surfaceK < 273.15) return 0;
  return scale(dewPointK(ambientK, relativeHumidity) - surfaceK, 0, 15);
}

/** event.shifted_by — a fraction of a kelvin is invisible; 20 K is a big shift. */
function phaseMag(e: EngineEvent): number {
  return scale(Math.abs(Number(e.shifted_by ?? 0)), 0.05, 20);
}

/** The visual kind one engine phase transition asks for. */
function phaseKind(from: string, to: string): string {
  if (to === "gas") return "boil";
  if (to === "solid") return "freeze";
  if (to === "liquid") return from === "gas" ? "condense" : "melt";
  return "phase-change";
}

/** Molar gas constant, J/(mol·K). */
const GAS_CONSTANT_J_PER_MOL_K = 8.314_462_618;

/**
 * A typical ionic solid's molar volume, L/mol, used only when the scene has
 * no row for the species — which happens when a solid dissolves completely
 * in the same step it is reported. Calcite is 0.0369, gypsum 0.0745, halite
 * 0.0270; 0.030 sits in the middle of that and is never the number a visual
 * claims when the real one is available.
 */
export const FALLBACK_MOLAR_VOLUME_L = 0.03;

/**
 * The volume `moles` of ideal gas occupies at `pressurePa` and
 * `temperatureK`, in litres. This is what moves a floating piston: hold the
 * pressure and add gas and it rises, squeeze the same gas harder and it
 * falls. Returns 0 for a non-positive pressure or amount.
 */
export function headspaceVolumeL(
  moles: number,
  temperatureK: number,
  pressurePa: number,
): number {
  if (!(moles > 0) || !(temperatureK > 0) || !(pressurePa > 0)) return 0;
  return (moles * GAS_CONSTANT_J_PER_MOL_K * temperatureK) / pressurePa * 1000;
}

/**
 * Boyle's law: what a gas that filled `volumeAtReferenceL` at
 * `referencePressurePa` shrinks (or swells) to at `pressurePa`, isothermally.
 * The standing fallback for a pressure-controlled vessel between events —
 * the scene carries `pressure_pa` but not the trapped moles, so this is what
 * keeps the piston in the right place once the event window has closed.
 */
export function compressedVolumeL(
  volumeAtReferenceL: number,
  pressurePa: number,
  referencePressurePa = 101_325,
): number {
  if (!(volumeAtReferenceL > 0) || !(pressurePa > 0)) return 0;
  return volumeAtReferenceL * (referencePressurePa / pressurePa);
}

/** One deposit drawn as grains: how many, and how big each one is. */
export interface DepositParticles {
  count: number;
  /** Volume of a single drawn grain, litres. */
  particleVolumeL: number;
  /** Grain radius relative to a 10 µL reference grain; bounded [0.5, 3]. */
  radiusScale: number;
}

/**
 * Grains for `moles` of a solid whose molar volume is `molarVolumeLPerMol`.
 *
 * The count follows the amount on a log ramp — a tenth of a millimole is a
 * few specks, a tenth of a mole is a snowfall — and the SIZE follows the
 * volume each grain then has to carry. That is the part that was missing:
 * a mole of a fluffy hydroxide occupies three times a mole of a dense
 * sulfate, and until now both drew the same 1.2 px circle.
 */
export function depositParticles(
  moles: number,
  molarVolumeLPerMol = FALLBACK_MOLAR_VOLUME_L,
): DepositParticles {
  const amount = Math.max(0, moles);
  const molar = molarVolumeLPerMol > 0 ? molarVolumeLPerMol : FALLBACK_MOLAR_VOLUME_L;
  const count = amount > 0
    ? Math.round(3 + 9 * scale(Math.log10(amount), -4, -1))
    : 0;
  const particleVolumeL = count > 0 ? (amount * molar) / count : 0;
  // A 10 µL grain is the reference; radius goes as the cube root of volume.
  const radiusScale = particleVolumeL > 0
    ? Math.max(0.5, Math.min(3, Math.cbrt(particleVolumeL / 1e-5)))
    : 0.5;
  return { count, particleVolumeL, radiusScale };
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
      return { kind: "vent", at: now, magnitude: gasMag(e), species: String(e.species ?? ""), reading: Number(e.moles ?? 0), unit: "mol" };
    case "gas_produced":
      return { kind: "vent", at: now, magnitude: gasMag(e), species: String(e.species ?? ""), reading: Number(e.moles ?? 0), unit: "mol" };
    case "gas_contained":
      // A sealed vessel keeps its gas: the same moles, but they stay in the
      // headspace and raise the pressure instead of leaving through the mouth.
      // Unmapped before GUI-099, so a sealed flask boiling showed nothing.
      return {
        kind: "contain",
        at: now,
        durationMs: 3000,
        magnitude: gasMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
      };
    case "foam_changed":
      return {
        kind: "foam",
        at: now,
        magnitude: scale(Number(e.height_cm ?? 0), 0.5, 30),
      };
    case "surface_spread":
      return {
        kind: "surface-spread",
        at: now,
        magnitude: scale(Number(e.to_cleared_fraction ?? 0), 0.05, 0.9),
      };
    case "surface_colour_spread":
      return {
        kind: "magic-milk",
        at: now,
        magnitude: scale(Number(e.to_spread_fraction ?? 0), 0.05, 0.9),
      };
    case "curdling_changed":
      return {
        kind: "curdle",
        at: now,
        magnitude: scale(Number(e.separation_progress ?? 0), 0.01, 1),
      };
    case "precipitated":
      return {
        kind: "precipitate",
        at: now,
        magnitude: precipMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
      };
    case "evaporated":
      return {
        kind: "evaporate",
        at: now,
        magnitude: steamMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
      };
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
      return {
        kind: "electrolyse",
        at: now,
        magnitude: electroMag(e),
        durationMs: Math.min(8000, Math.max(1200, Number(e.seconds ?? 2.2) * 1000)),
        electrolysis: {
          species: String(e.species ?? ""),
          amps: Number(e.amps ?? 0),
          seconds: Number(e.seconds ?? 0),
          coulombs: Number(e.coulombs ?? 0),
          electronMoles: Number(e.electrons ?? 0),
          productMoles: Number(e.moles ?? 0),
          grams: Number(e.grams ?? 0),
          electronsPerIon: Number(e.per_ion ?? 0),
        },
      };
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
        stir: {
          rpm: Number(e.rpm ?? 0),
          seconds: Number(e.seconds ?? 0),
          barLengthM: Number(e.bar_length_m ?? 0),
          tipSpeedMS: Number(e.tip_speed_m_s ?? 0),
          resuspendedFraction: Math.max(0, Math.min(1, Number(e.resuspended_fraction ?? 0))),
          rateCoupled: Boolean(e.rate_coupled),
          solids: [],
        },
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
        acceptedTransferFraction:
          Number.isFinite(Number(e.fraction)) && Number(e.fraction) >= 0 && Number(e.fraction) <= 1
            ? Number(e.fraction)
            : 0,
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
      return {
        kind: "swirl",
        at: now,
        magnitude: diluteMag(e),
        durationMs: 2800,
        dilution: {
          volumeL: Number(e.volume ?? 0),
          waterMoles: Number(e.moles ?? 0),
        },
      };
    case "dissolved":
      return {
        kind: "dissolve",
        at: now,
        magnitude: precipMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
      };
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
    case "energy_transferred": {
      const deliveredJ = Math.max(0, Number(e.delivered_j ?? 0));
      const heating = Boolean(e.heating);
      return {
        kind: heating ? "heat" : "cool",
        at: now,
        durationMs: 2600,
        magnitude: scale(deliveredJ, 100, 50_000),
        reading: deliveredJ,
        unit: "J",
        thermal: {
          heating,
          requestedJ: Math.max(0, Number(e.requested_j ?? 0)),
          deliveredJ,
          timeCoupled: Boolean(e.time_coupled),
        },
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
    case "state_changed": {
      const from = String(e.from ?? "");
      const to = String(e.to ?? "");
      const atK = Number(e.at ?? 0);
      return {
        kind: phaseKind(from, to),
        at: now,
        durationMs: 3200,
        magnitude: phaseMag(e),
        temperatureK: atK,
        phase: {
          species: String(e.species ?? ""),
          from,
          to,
          atK,
          shiftedByK: Number(e.shifted_by ?? 0),
        },
      };
    }
    case "boiling_point_routed": {
      // Emitted only when the boil did NOT use the normal boiling point —
      // a partial vacuum, a pressurised vessel, a salted solvent. It names
      // the plateau, so it draws the same rolling boil `state_changed` does.
      const atK = Number(e.boiling ?? 0);
      return {
        kind: "boil",
        at: now,
        durationMs: 3200,
        magnitude: phaseMag(e),
        temperatureK: atK,
        phase: {
          species: String(e.species ?? ""),
          from: "liquid",
          to: "gas",
          atK,
          shiftedByK: Number(e.shifted_by ?? 0),
          pressureKpa: Number(e.pressure_kpa ?? 0),
          route: String(e.route ?? ""),
          model: String(e.model ?? ""),
        },
      };
    }
    case "burst":
      return {
        kind: "burst",
        at: now,
        magnitude: scale(Number(e.at_pa ?? 0) / Math.max(1, Number(e.rating_pa ?? 1)), 1, 2),
      };
    case "container_broken": {
      const destination = spillDestination(e.destination);
      return {
        kind: "break",
        at: now,
        durationMs: 5000,
        magnitude: scale(Number(e.impulse_ns ?? e.impulse ?? 1), 0.25, 8),
        source: Number(e.vessel ?? 0),
        spill: { ...destination, fraction: 1 },
      };
    }
    case "spill_created": {
      const fraction = Math.max(0, Math.min(1, Number(e.fraction ?? 0)));
      const destination = spillDestination(e.destination);
      return {
        kind: "spill",
        at: now,
        durationMs: 5000,
        magnitude: scale(fraction, 0.02, 1),
        acceptedTransferFraction: fraction,
        source: Number(e.source ?? e.from ?? 0),
        spill: { ...destination, fraction },
      };
    }
    case "ignited":
    case "flame_test": {
      const [mag, colour] = flameMag(e);
      const energyJ = kind === "ignited" && e.energy_j !== undefined ? Number(e.energy_j) : undefined;
      return {
        kind: kind === "flame_test" ? "flame_test" : "ignite",
        at: now,
        magnitude: mag,
        flameColour: colour,
        // The flame's size already followed this; carrying it makes the
        // driving number readable from the DOM rather than only inferable.
        ...(energyJ === undefined ? {} : { reading: energyJ, unit: "J" }),
      };
    }
    case "gas_tested":
      return {
        kind: "gas_test",
        at: now,
        durationMs: 4500,
        magnitude: Boolean(e.positive) ? .85 : .25,
        gasTest: {
          test: String(e.test ?? ""),
          positive: Boolean(e.positive),
          notes: String(e.notes ?? ""),
        },
      };
    case "smelled": {
      const notes = (Array.isArray(e.notes) ? e.notes : []).flatMap((entry) =>
        Array.isArray(entry) && entry.length >= 2
          ? [{ species: String(entry[0]), description: String(entry[1]) }]
          : [],
      );
      return {
        kind: "waft",
        at: now,
        durationMs: 4200,
        magnitude: Math.max(.2, Math.min(1, notes.length / 3)),
        waft: { notes },
      };
    }
    case "vessel_sealed": {
      // The engine names the headspace it just trapped, so the lid can sit
      // where the gas actually is instead of at a fixed y.
      const volumeL = Math.max(0, Number(e.headspace_volume ?? 0));
      return {
        kind: "seal",
        at: now,
        durationMs: 4000,
        magnitude: scale(volumeL, 0.005, 0.5),
        headspace: {
          volumeL,
          moles: Math.max(0, Number(e.trapped_air ?? 0)),
          source: "engine",
        },
      };
    }
    case "vessel_pressure_controlled": {
      const pressurePa = Number(e.pressure ?? 0);
      return {
        kind: "regulate",
        at: now,
        durationMs: 4500,
        magnitude: scale(pressurePa, 100_000, 500_000),
        pressureControl: {
          pressurePa,
          initialVolumeL: Number(e.initial_volume ?? 0),
          trappedGasMoles: Number(e.trapped_gas ?? 0),
        },
      };
    }
    case "vessel_swept": {
      const pressurePa = Number(e.pressure ?? 0);
      return {
        kind: "sweep",
        at: now,
        durationMs: 3800,
        magnitude: scale(pressurePa, 50_000, 500_000),
        sweep: { pressurePa },
      };
    }
    case "irradiated": {
      const irradianceWM2 = Math.max(0, Number(e.irradiance_w_m2 ?? 0));
      return {
        kind: "irradiate",
        at: now,
        durationMs: 4200,
        magnitude: scale(irradianceWM2, 0.1, 100),
        irradiation: {
          wavelengthNm: Number(e.wavelength_nm ?? 0),
          irradianceWM2,
          photolysisCoupled: Boolean(e.photolysis_coupled),
        },
      };
    }
    default:
      return null;
  }
}

function spillDestination(value: unknown): Pick<SpillRun, "surface" | "location"> {
  const destination = value && typeof value === "object"
    ? value as Record<string, unknown>
    : {};
  const surface = String(destination.surface ?? "bench");
  const location = String(destination.zone ?? destination.tray ?? "unknown");
  return { surface, location };
}

/**
 * Vessel ID from an event — events use `vessel`, `from`, or `into`.
 */
export function vesselOf(e: EngineEvent): number {
  return Number(e.vessel ?? e.from ?? e.into ?? e.anode ?? 0);
}
