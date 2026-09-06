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
  /**
   * GUI-099: what actually comes off each end of the cell, from the engine's
   * own half-reactions. Hydrogen leaves the cathode twice as fast as oxygen
   * leaves the anode, and until the engine carried both halves the bench had
   * to size both electrodes by the charge they shared — correct, but blind
   * to the one ratio the experiment exists to show. Undefined for a log
   * written before the fields existed.
   */
  anodeSpecies?: string;
  anodeMoles?: number;
  cathodeSpecies?: string;
  cathodeMoles?: number;
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
  /**
   * Where the volume came from: `"scene"` is the engine's standing
   * `headspace_volume_l`, `"engine"` a volume an event named at the moment
   * it happened, `"ideal-gas"` the client's own `V = nRT/P`.
   */
  source: "scene" | "engine" | "ideal-gas";
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
  /**
   * GUI-099: which of the six transitions the ENGINE says this is —
   * `melting`, `freezing`, `boiling`, `condensation`, `sublimation`,
   * `deposition`. Dry ice going straight to fog is a `to: gas` with no
   * liquid, no bubbles and no plateau, and a client reading `to` alone drew
   * it as a rolling boil. Undefined in a log written before the field.
   */
  kind?: string;
  /** Moles that changed phase in this step, where the engine said. */
  moles?: number;
}

/** Engine-computed dispersion of one liquid inside another. */
export interface EmulsionRun {
  material: string;
  fromDispersedFraction: number;
  toDispersedFraction: number;
  dispersedVolumeL: number;
  /** Seconds for half the dispersed phase to coalesce back out. */
  halfLifeSeconds: number;
}

/** Engine-computed anaerobic run: what the yeast ate, made, and how long for. */
export interface FermentationRun {
  sucroseMoles: number;
  ethanolMoles: number;
  carbonDioxideMoles: number;
  activeYeastGrams: number;
  seconds: number;
  /** `carbon_dioxide_moles ÷ seconds` — the rate the bubbling follows. */
  molesPerSecond: number;
}

/** Engine-computed production rate for a gas-forming reaction. */
export interface GasProductionRun {
  molesPerSecond: number;
}

/** Engine-computed persistence of a newly formed foam head. */
export interface FoamRun {
  /** Seconds in which the modeled foam head falls to half its height. */
  halfLifeSeconds: number;
}

/** Engine-computed transmission of one UV band through a sample. */
export interface UvRun {
  material: string;
  wavelengthNm: number;
  band: string;
  /** Fraction of the incident light that got through, 0–1. */
  transmittedFraction: number;
  mechanism: string;
}

/**
 * Engine-computed Henry's-law split of one volatile between the liquid and
 * an owned headspace. `gasFraction` is the share of the species' whole
 * inventory now sitting in the gas — the standing state, which is what
 * tints the band — while `toGas` is only which way this one step went,
 * which is what points the arrows.
 */
export interface HeadspacePartitionRun {
  species: string;
  /** `true`: liquid → headspace this step; `false`: headspace → liquid. */
  toGas: boolean;
  /** Moles that crossed. */
  moles: number;
  /** Share of the species' whole inventory now in the headspace, 0–1. */
  gasFraction: number;
  /** Equilibrium partial pressure, Pa. */
  partialPressurePa: number;
  /** Henry's constant at the vessel temperature, mol/(L·atm). */
  henryMolPerLAtm: number;
  /** Provenance of the coefficient, for the tooltip. */
  source: string;
}

/**
 * The gas/liquid equilibrium a finite headspace settled at: the pressure a
 * gauge would read and the moles that are holding it there. Two numbers
 * that have to agree with the piston drawn from the same headspace, which
 * is the whole point of showing them together.
 */
export interface HeadspaceEquilibriumRun {
  pressurePa: number;
  totalMoles: number;
}

/**
 * How far past its own limit a solution is holding a solid. The engine
 * refuses to precipitate it — that is what makes rock candy possible — so
 * the only honest visual is the distance itself, and `ratio` is it.
 */
export interface SaturationRun {
  species: string;
  /** What is in solution now, mol. */
  dissolved: number;
  /** What this much solvent holds at this temperature, mol. */
  capacity: number;
  /** `dissolved ÷ capacity`; 1 is saturation, above it is metastable. */
  ratio: number;
  /** `dissolved − capacity`, the moles the solvent should not be holding. */
  excessMoles: number;
}

/**
 * The engine's corrosion verdict together with how far it has already got.
 * The extent is an amount and never a rate — the rate lives in the
 * kinetics registry and arrives as `Reacted` — so what it can size is a
 * picture of a nail that is *already* rusted, not one rusting faster.
 */
export interface CorrosionRun {
  species: string;
  corroding: boolean;
  why: string;
  /** Moles of the metal now locked in its oxide, where the engine knew. */
  corrodedMoles?: number;
  /** That over all the metal the vessel holds in either form, 0–1. */
  corrodedFraction?: number;
}

/**
 * One kinetic interval: how far a curated reaction ran, and over how much
 * bench time. The pair is the point — the same tenth of a mole in one
 * second and in one hour are different observations.
 */
export interface ReactionRun {
  reaction: string;
  equation: string;
  moles: number;
  seconds: number;
  /** `moles ÷ seconds`, the extent rate the tempo follows. */
  molesPerSecond: number;
  /** The catalyst in force, where one was. */
  catalyst?: string;
  /** Activation energy actually used, J/mol. */
  activationEnergyJPerMol: number;
}

/** Heat a curated kinetic reaction let go during one interval. */
export interface ExothermRun {
  reaction: string;
  energyJ: number;
}

/**
 * KID-13, the dancing raisin. `liftGasFraction` is the attached gas volume,
 * as a fraction of the object's own volume, needed before it goes up —
 * zero meaning it floats unaided, which is the case the visual must not
 * draw bubbles for.
 */
export interface BubbleRideRun {
  object: string;
  objectDensityGPerMl: number;
  liquidDensityGPerMl: number;
  liftGasFraction: number;
}

/**
 * BRD-032: what the sorbent took and what the beaker still holds. Both
 * halves travel because neither can be read without the other — "the
 * charcoal adsorbed the dye" is exactly the sentence that misleads.
 */
export interface AdsorptionRun {
  sorbate: string;
  sorbent: string;
  /** Moles now held on the surface. */
  heldMoles: number;
  /** Moles still in solution, which is what a filtration would pour. */
  stillDissolvedMoles: number;
  /** The isotherm's own unit, mg of sorbate per g of sorbent. */
  loadingMgPerG: number;
  /** What the curated isotherm does not claim. */
  boundary: string;
}

/**
 * A shear-thickening mixture pushed. Nothing reacts and no mole moves:
 * this is how the mixture *responds*, which is why the only honest visual
 * is resistance to the thing doing the pushing.
 */
export interface ThickeningRun {
  solid: string;
  /** 0 at the onset mixture, 1 at the full one. */
  strength: number;
  solidMassFraction: number;
  tipSpeedMS: number;
  /** Sheared hard enough to thicken, rather than merely stirred. */
  shearedHard: boolean;
}

/** Moles of acidity cancelled — the commonest reaction in a school lab. */
export interface NeutralisationRun {
  moles: number;
}

/**
 * KID-12: the flame went out because of the AIR, not the fuel. A candle
 * under a jar quits while roughly four fifths of the jar's oxygen is
 * still there, so `oxygenFraction` is the number that contradicts "it
 * used up all the oxygen", and `burnedMoles` says how much fuel it
 * managed first — zero meaning it never caught at all.
 */
export interface FlameStarvedRun {
  fuel: string;
  burnedMoles: number;
  oxygenFraction: number;
}

/**
 * BRD-041: a fuel standing in air, warm, and below the temperature it
 * would light itself at. Nothing burns, and that is the answer — so the
 * only thing to draw is the gap.
 */
export interface AutoignitionGapRun {
  fuel: string;
  autoignitionK: number;
  temperatureK: number;
  /** `autoignitionK − temperatureK`, the K still to go. */
  gapK: number;
}

/** A radionuclide tracer's opening activity — what the Geiger will read. */
export interface NuclideSpikeRun {
  nuclide: string;
  moles: number;
  activityBq: number;
}

/**
 * A neutral solute split between two layers on its computed partition
 * coefficient. `fractionLower` is the share that sat in the lower layer,
 * and so the share that left when the stopcock opened.
 */
export interface SolutePartitionRun {
  species: string;
  fractionLower: number;
}

/**
 * Water crossing a membrane. The sign is the whole observation: an egg in
 * syrup shrinks and an egg in water swells, and both are the same event.
 */
export interface OsmosisRun {
  material: string;
  waterMoles: number;
  massChangeG: number;
}

/**
 * A settled thermal equilibrium. `holdsNothing` matters more than the
 * temperature does: the burn consumed everything, and the number beside
 * it is the exhaust's rather than the glass's — which reached a reader
 * once as "thermal equilibrium at 2496 °C" over an empty beaker.
 */
export interface ThermalEquilibriumRun {
  temperatureK: number;
  reactionEnergyJ?: number;
  holdsNothing: boolean;
}

/** A metal coming out of solution onto a more reactive one. */
export interface PlatingRun {
  species: string;
  onto: string;
  moles: number;
}

/**
 * What went, and what is left. The remainder is optional on purpose: the
 * event used to carry only what went, and "is used up" claimed a
 * completeness it could not see — half a magnesium ribbon beside its
 * plated copper was reported gone. Absent means the emitter did not say,
 * and the visual must then claim no remainder either.
 */
export interface ConsumptionRun {
  species: string;
  moles: number;
  remainingMoles?: number;
}

/** A solid ground finer: the size of the grains and the area they expose. */
export interface GrindRun {
  species: string;
  diameterUm: number;
  solidMoles: number;
  surfaceAreaM2: number;
  /** False until a heterogeneous kinetic law actually consumes this area. */
  rateCoupled: boolean;
}

/** One parcel of parent that became daughter while bench time ran. */
export interface DecayRun {
  parent: string;
  daughter: string;
  mode: string;
  moles: number;
  halfLifeS: number;
  /** `ln2 ÷ half-life × moles`, the decay rate the ticks follow. */
  molesPerSecond: number;
}

/**
 * How long an instrument's reading stays on the vessel.
 *
 * 2.5 s, which is what this was, is long enough to notice something
 * flickered and not long enough to read it — the owner's words from the
 * German deploy were "too shortly and too tiny". Six seconds is a reading
 * you can look up from your phone and back at; the next command replaces
 * it either way, because a newer reading of the same instrument wins.
 *
 * ONE value, imported by both halves: `session.svelte.ts` stamps it on the
 * effect (which is what decides when the effect is dropped from the array)
 * and `Vessel.svelte` uses it as the drawing window. Split, they disagree
 * and the shorter one silently governs — which is exactly what happened
 * here: the drawing window was 2.5 s and the effect lived 4 s.
 */
export const INSTRUMENT_READING_MS = 6000;

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
  /** Engine-owned dispersion, for the droplet field. */
  emulsion?: EmulsionRun;
  /** Engine-owned anaerobic run, for the slow bubbling. */
  fermentation?: FermentationRun;
  /** Engine-owned gas production rate, for bubble cadence. */
  gasProduction?: GasProductionRun;
  /** Engine-owned foam half-life, for collapse timing. */
  foam?: FoamRun;
  /** Engine-owned UV transmission, for the beam. */
  uv?: UvRun;
  /** Engine-owned Henry's-law split, for the headspace tint and arrows. */
  headspacePartition?: HeadspacePartitionRun;
  /** Engine-settled headspace pressure and amount, for the gauge. */
  headspaceEquilibrium?: HeadspaceEquilibriumRun;
  /** Engine-owned distance past saturation, for the haze. */
  saturation?: SaturationRun;
  /** Engine-read corrosion extent, for the bloom on the metal. */
  corrosion?: CorrosionRun;
  /** Engine-computed kinetic interval, for the extent readout and tempo. */
  reaction?: ReactionRun;
  /** Engine-computed heat let go, for the exotherm halo. */
  exotherm?: ExothermRun;
  /** Engine-computed lift threshold, for the riding object. */
  bubbleRide?: BubbleRideRun;
  /** Engine-computed sorbent loading and remainder, for the darkening. */
  adsorption?: AdsorptionRun;
  /** Engine-computed shear response, for the resisting stirrer. */
  thickening?: ThickeningRun;
  /** Engine-computed acidity cancelled, for the neutralisation marks. */
  neutralisation?: NeutralisationRun;
  /** Engine-computed oxygen the flame quit at, for the guttering flame. */
  flameStarved?: FlameStarvedRun;
  /** Engine-computed distance to autoignition, for the gap bar. */
  autoignitionGap?: AutoignitionGapRun;
  /** Engine-computed opening activity, for the tracer ticks. */
  nuclideSpike?: NuclideSpikeRun;
  /** Engine-computed solute split, for the dots across the two layers. */
  solutePartition?: SolutePartitionRun;
  /** Engine-computed water crossing a membrane, for the swelling. */
  osmosis?: OsmosisRun;
  /** Engine-settled temperature, for the equilibrium badge. */
  thermalEquilibrium?: ThermalEquilibriumRun;
  /** Engine-computed deposit, for the plating's thickness. */
  plating?: PlatingRun;
  /** Engine-computed amount gone and, where known, what is left. */
  consumption?: ConsumptionRun;
  /** Engine-computed grain size and exposed area, for the powder. */
  grind?: GrindRun;
  /** Engine-computed decay parcel and half-life, for the ticks. */
  decay?: DecayRun;
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

/**
 * The visual kind one engine phase transition asks for.
 *
 * The engine's own `kind` wins where it is on the wire: `to: "gas"` alone
 * cannot tell dry-ice fog from a rolling boil, and reading it as a boil is
 * exactly what the bench did. `from`/`to` remain the fallback for a log
 * written before the engine named its transitions.
 */
export function phaseKind(from: string, to: string, kind?: string): string {
  switch (kind) {
    case "boiling":
      return "boil";
    case "freezing":
      return "freeze";
    case "melting":
      return "melt";
    case "condensation":
      return "condense";
    case "sublimation":
      return "sublimate";
    case "deposition":
      return "deposit";
  }
  if (to === "gas") return from === "solid" ? "sublimate" : "boil";
  if (to === "solid") return from === "gas" ? "deposit" : "freeze";
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
 * Moles in one bubble a learner can actually see: about a millilitre of gas,
 * which at room conditions is ~41 µmol. Used to turn a production RATE into
 * a bubble tempo, so a ferment that makes 3 mmol of CO2 over an hour bubbles
 * once every few seconds rather than at an invented speed.
 */
export const VISIBLE_BUBBLE_MOLES = 4.1e-5;

/**
 * Seconds between visible bubbles for a gas produced at `molesPerSecond`.
 * Monotone decreasing in the rate and clamped to [0.25 s, 6 s] — below the
 * floor the eye reads a stream anyway, and above the ceiling the vessel
 * would appear inert.
 */
export function bubblePeriodS(molesPerSecond: number): number {
  if (!(molesPerSecond > 0)) return 6;
  return Math.max(0.25, Math.min(6, VISIBLE_BUBBLE_MOLES / molesPerSecond));
}

/**
 * Bubbles drawn at ONE electrode from the charge that passed, 1–8. Charge is
 * the honest driver: the engine names the moles of only one product, and the
 * counter-electrode's half-reaction is not on the wire, so both electrodes
 * are sized by the coulombs they shared rather than by an invented ratio.
 */
export function electrodeBubbles(coulombs: number): number {
  if (!(coulombs > 0)) return 1;
  return Math.max(1, Math.min(8, Math.round(1 + 7 * scale(Math.log10(coulombs), 0, 3))));
}

/**
 * Bubbles at each electrode, and what drove the split.
 *
 * With both half-reactions on the wire the COUNT IS THE RATIO: water
 * splitting evolves two moles of hydrogen at the cathode for every mole of
 * oxygen at the anode, so the cathode draws twice the bubbles — which is the
 * one observation the experiment is run to make. Without them, both fall
 * back to the charge the electrodes shared: equal by definition, therefore
 * never a guess, and equally never the ratio.
 */
export function electrodePairBubbles(
  coulombs: number,
  anodeMoles?: number,
  cathodeMoles?: number,
): { anode: number; cathode: number; source: "moles" | "charge" } {
  const shared = electrodeBubbles(coulombs);
  const a = Number(anodeMoles);
  const c = Number(cathodeMoles);
  if (!(a > 0) || !(c > 0)) return { anode: shared, cathode: shared, source: "charge" };
  const larger = Math.max(a, c);
  const scaled = (n: number) => Math.max(1, Math.min(8, Math.round(shared * (n / larger))));
  return { anode: scaled(a), cathode: scaled(c), source: "moles" };
}

/**
 * How heavy the haze over a solution holding more than it should is.
 *
 * Zero at and below saturation, because a saturated solution looks like any
 * other one and drawing something there would be a picture of the word.
 * Above it the ratio is the whole quantity: a syrup at twice its limit is
 * the one that grows rock candy, and it reads far heavier than one a hair
 * over. Bounded at 1 so a wild ratio cannot white the vessel out.
 */
export function supersaturationHaze(dissolved: number, capacity: number): number {
  if (!Number.isFinite(dissolved) || !Number.isFinite(capacity)) return 0;
  if (!(capacity > 0) || !(dissolved > capacity)) return 0;
  return scale(dissolved / capacity, 1, 2.5);
}

/**
 * Opacity for the headspace band during a Henry's-law partition, from the
 * share of the species' whole inventory that is now gas.
 *
 * Capped below the band's own pressure tint so a fully partitioned volatile
 * darkens the headspace without blacking it out — the band still has to
 * show the piston behind it.
 */
export function partitionTint(gasFraction: number): number {
  if (!Number.isFinite(gasFraction)) return 0;
  return Math.min(1, Math.max(0, gasFraction)) * 0.45;
}

/**
 * The rust bloom on a corroding metal: how many spots, and how strongly
 * they read, from the fraction of that metal already locked in its oxide.
 *
 * An untouched nail gets nothing — the verdict "this will corrode" is not
 * yet a picture of rust — and a nail entirely gone to oxide gets the full
 * field. The count is bounded at nine so a fully corroded solid reads as a
 * texture rather than as a swarm of circles.
 */
export function corrosionBloom(fraction: number): { spots: number; strength: number } {
  if (!Number.isFinite(fraction)) return { spots: 0, strength: 0 };
  const bounded = Math.min(1, Math.max(0, fraction));
  if (bounded <= 0) return { spots: 0, strength: 0 };
  return { spots: Math.max(1, Math.round(bounded * 9)), strength: 0.18 + bounded * 0.62 };
}

/**
 * How strongly a kinetic interval reads, and how fast it ran.
 *
 * Logarithmic on the extent for the same reason the gas ramp is: a school
 * bench reaction runs a hundredth of a mole and a demonstration runs a
 * tenth, and a linear ramp draws both as nothing. The rate is the honest
 * companion — the same tenth of a mole in a second and over an hour are
 * different observations, and only the pair separates them.
 */
export function reactionExtent(
  moles: number,
  seconds: number,
): { intensity: number; molesPerSecond: number } {
  const amount = Number.isFinite(moles) ? Math.max(0, moles) : 0;
  const elapsed = Number.isFinite(seconds) ? Math.max(0, seconds) : 0;
  const intensity = amount > 0 ? scale(Math.log10(amount), Math.log10(0.0001), Math.log10(0.1)) : 0;
  return { intensity, molesPerSecond: elapsed > 0 ? amount / elapsed : 0 };
}

/**
 * The exotherm's halo, from the joules a curated reaction let go.
 *
 * Same ramp as the heat of mixing, deliberately: dissolving a spoon of
 * lye and a self-heating hand warmer are the same kind of claim about the
 * same quantity, and drawing them on two scales would say they were not.
 */
export function exothermGlow(energyJ: number): number {
  if (!Number.isFinite(energyJ)) return 0;
  return scale(Math.max(0, energyJ), 5, 5000);
}

/**
 * How many cancellation marks an acid meeting a base draws.
 *
 * `H⁺ + OH⁻ → H₂O` is the commonest reaction a school bench runs and the
 * only one that used to happen with nothing at all against it. Bounded at
 * nine so a titration's last drop and a beaker of drain cleaner differ in
 * how many marks they draw rather than in how long the browser takes.
 */
export function neutralisationMarks(moles: number): number {
  if (!Number.isFinite(moles) || !(moles > 0)) return 0;
  const ramp = scale(Math.log10(moles), Math.log10(0.0001), Math.log10(0.1));
  return Math.max(1, Math.round(1 + ramp * 8));
}

/**
 * KID-13: what has to cling to a raisin before it goes up.
 *
 * `liftGasFraction` is the engine's threshold — attached gas volume as a
 * fraction of the object's own volume — so an object that floats unaided
 * reports zero and must draw NO clinging bubbles, because the bubbles are
 * not why it is up there. The count is that threshold: the more gas the
 * object needs, the more of it has to be visibly stuck on. Time follows
 * the same number, because gathering more gas takes longer.
 */
export function bubbleRideLift(
  objectDensityGPerMl: number,
  liquidDensityGPerMl: number,
  liftGasFraction: number,
): { needsGas: boolean; clingingBubbles: number; densityRatio: number; riseSeconds: number } {
  const object = Number.isFinite(objectDensityGPerMl) ? Math.max(0, objectDensityGPerMl) : 0;
  const liquid = Number.isFinite(liquidDensityGPerMl) ? Math.max(0, liquidDensityGPerMl) : 0;
  const need = Number.isFinite(liftGasFraction) ? Math.max(0, liftGasFraction) : 0;
  const densityRatio = liquid > 0 ? object / liquid : 0;
  if (!(need > 0)) {
    return { needsGas: false, clingingBubbles: 0, densityRatio, riseSeconds: 1.2 };
  }
  const bounded = Math.min(1, need);
  return {
    needsGas: true,
    clingingBubbles: Math.max(1, Math.round(1 + bounded * 7)),
    densityRatio,
    riseSeconds: 1.2 + bounded * 8,
  };
}

/**
 * How dark the sorbent goes, and how much of the dye actually left.
 *
 * Driven by the two amounts the event insists on carrying together —
 * `held` and `still_dissolved` — because their ratio is the answer to
 * "can charcoal take a food dye out of water" and the loading alone is
 * not. No isotherm ceiling is claimed here: the wire does not carry the
 * capacity, so the darkening is the fraction removed and the loading
 * travels beside it as a readout rather than as a fraction of something
 * nobody stated.
 */
export function adsorptionDarkening(
  heldMoles: number,
  stillDissolvedMoles: number,
): { removedFraction: number; darkening: number } {
  const held = Number.isFinite(heldMoles) ? Math.max(0, heldMoles) : 0;
  const left = Number.isFinite(stillDissolvedMoles) ? Math.max(0, stillDissolvedMoles) : 0;
  const total = held + left;
  const removedFraction = total > 0 ? held / total : 0;
  return { removedFraction, darkening: 0.15 + removedFraction * 0.6 };
}

/**
 * How hard a shear-thickening mixture pushes back.
 *
 * Zero unless the engine says it was sheared HARD: oobleck stirred slowly
 * is a liquid, and drawing resistance there would be a picture of the
 * recipe rather than of what happened. Above that it is `strength`, which
 * the engine already normalises from the onset mixture to the full one.
 */
export function shearResistance(strength: number, shearedHard: boolean): number {
  if (!shearedHard || !Number.isFinite(strength)) return 0;
  return Math.min(1, Math.max(0, strength));
}

/** Oxygen's share of dry air, the fraction every flame here starts from. */
export const AIR_OXYGEN_FRACTION = 0.209;

/**
 * How starved a flame was when it quit, and whether it ever caught.
 *
 * `caught` is `burned > 0`: zero moles burned means the air was already
 * too thin to light in, which is what a carbon-dioxide extinguisher
 * makes, and drawing a flame for it would contradict the event. The
 * guttering is how far the oxygen had fallen BELOW air's own fraction —
 * not how much oxygen is left, because the point KID-12 teaches is that
 * four fifths of it still is.
 */
export function flameGutter(
  oxygenFraction: number,
  burnedMoles: number,
): { caught: boolean; guttering: number } {
  const oxygen = Number.isFinite(oxygenFraction) ? Math.max(0, oxygenFraction) : 0;
  const burned = Number.isFinite(burnedMoles) ? Math.max(0, burnedMoles) : 0;
  const guttering = Math.min(1, Math.max(0, 1 - oxygen / AIR_OXYGEN_FRACTION));
  return { caught: burned > 0, guttering };
}

/**
 * How close a warm fuel stands to lighting itself, and how far that is.
 *
 * The gap is the answer BRD-041 gives — "it would sit there" — so the bar
 * fills toward 1 as the vessel approaches the autoignition temperature
 * and never reaches it, because reaching it is a different event.
 */
export function autoignitionApproach(
  temperatureK: number,
  autoignitionK: number,
): { approach: number; gapK: number } {
  const at = Number.isFinite(autoignitionK) ? autoignitionK : 0;
  const now = Number.isFinite(temperatureK) ? temperatureK : 0;
  if (!(at > 0)) return { approach: 0, gapK: 0 };
  return { approach: Math.min(1, Math.max(0, now / at)), gapK: Math.max(0, at - now) };
}

/**
 * How busy a tracer's opening activity reads.
 *
 * Logarithmic over the range a school tracer spans — a becquerel is one
 * disintegration a second and a sealed teaching source is megabecquerels
 * — so the ticks separate a background whisper from a working source.
 */
export function activityIntensity(activityBq: number): number {
  if (!Number.isFinite(activityBq) || !(activityBq > 0)) return 0;
  return scale(Math.log10(activityBq), 0, 6);
}

/**
 * A solute's dots split across the two layers, from `fraction_lower`.
 *
 * The two counts always sum to `total`, because the solute did not go
 * anywhere else: a split that loses a dot is a picture of a leak.
 */
export function soluteSplit(fractionLower: number, total = 10): { lower: number; upper: number } {
  const bounded = Number.isFinite(fractionLower) ? Math.min(1, Math.max(0, fractionLower)) : 0;
  const count = Math.max(0, Math.round(total));
  const lower = Math.round(bounded * count);
  return { lower, upper: count - lower };
}

/**
 * Which way the water went, and how far it moved the object.
 *
 * The SIGN is the whole observation — an egg in syrup shrinks and an egg
 * in water swells, and both arrive as the same event — so direction is
 * reported separately from size rather than folded into one signed
 * magnitude a visual would have to unpick.
 */
export function osmoticSwell(massChangeG: number): { direction: "in" | "out" | "none"; swell: number } {
  if (!Number.isFinite(massChangeG) || massChangeG === 0) return { direction: "none", swell: 0 };
  return {
    direction: massChangeG > 0 ? "in" : "out",
    swell: scale(Math.abs(massChangeG), 0.05, 12),
  };
}

/**
 * How thick a plated coating reads, from the moles that came out.
 *
 * The event's magnitude was a hard-coded `1`, so a copper blush on a nail
 * and a nail gone orange drew the same shimmer. Logarithmic, because a
 * displacement demonstration runs a tenth of a millimole and a plating
 * cell runs a hundredth of a mole.
 */
export function platingThickness(moles: number): number {
  if (!Number.isFinite(moles) || !(moles > 0)) return 0;
  return scale(Math.log10(moles), Math.log10(0.0001), Math.log10(0.02));
}

/**
 * What is left of a ribbon being eaten, where the engine said.
 *
 * `remaining` is optional on the wire and the difference matters: absent
 * means the emitter did not know, and a visual that shrank the ribbon
 * anyway would repeat the "is used up" claim that reported half a
 * magnesium ribbon gone. So an unknown remainder draws the ribbon being
 * eaten without asserting how much of it is left.
 */
export function consumptionRemainder(
  moles: number,
  remaining?: number,
): { knownRemainder: boolean; remainingFraction: number } {
  const gone = Number.isFinite(moles) ? Math.max(0, moles) : 0;
  if (remaining === undefined || !Number.isFinite(remaining)) {
    return { knownRemainder: false, remainingFraction: 1 };
  }
  const left = Math.max(0, remaining);
  const total = gone + left;
  return { knownRemainder: true, remainingFraction: total > 0 ? left / total : 0 };
}

/**
 * The powder a grind leaves: how big each grain is and how many are drawn.
 *
 * The grain radius comes from `diameter_um`, which is the actual size the
 * engine ground to — so grinding twice draws visibly finer powder rather
 * than the same specks with a different caption. The count follows the
 * area that exposed, on a log ramp, because that is what a rate would
 * later see.
 */
export function grindGrains(
  diameterUm: number,
  surfaceAreaM2: number,
): { count: number; radius: number } {
  const diameter = Number.isFinite(diameterUm) ? Math.max(0, diameterUm) : 0;
  const area = Number.isFinite(surfaceAreaM2) ? Math.max(0, surfaceAreaM2) : 0;
  const count = area > 0 ? Math.max(3, Math.round(3 + scale(Math.log10(area), -4, 1) * 15)) : 3;
  // 1 µm reads as the smallest speck the stage can draw, 2 mm as a chip.
  const radius = 0.35 + scale(Math.log10(Math.max(1, diameter)), 0, Math.log10(2000)) * 2.4;
  return { count, radius };
}

/**
 * Seconds for one sweep-arrow cycle, from the carrier gas pressure.
 *
 * The two arrows were static whatever the sweep, which drew a purge at
 * half an atmosphere and one at five the same way. Faster at higher
 * pressure, and bounded at both ends so neither becomes a strobe nor
 * appears stopped.
 */
export function sweepPeriodS(pressurePa: number): number {
  const pressure = Number.isFinite(pressurePa) ? Math.max(0, pressurePa) : 0;
  return 2.6 - scale(pressure, 50_000, 500_000) * 2;
}

/**
 * How turbid the substrate still is, from the fraction the enzyme has
 * converted. Milk clouded with undigested lactose clears as lactase
 * works, and the clearing IS the fraction — a caption percentage beside
 * an unchanged liquid was a number nothing on the stage agreed with.
 */
export function substrateClearing(convertedFraction: number): number {
  if (!Number.isFinite(convertedFraction)) return 1;
  return 1 - Math.min(1, Math.max(0, convertedFraction));
}

/**
 * Decay ticks for a parcel that actually decayed, from the parent amount
 * and its half-life.
 *
 * `ln2 ÷ half-life × moles` is the activity — the same physics the Geiger
 * reads — so a long-lived tracer ticks slowly and a large parcel ticks
 * often, and the bench stops needing an instrument in hand before decay
 * is visible at all.
 */
export function decayTicks(
  moles: number,
  halfLifeS: number,
): { ticks: number; periodS: number; molesPerSecond: number } {
  const parcel = Number.isFinite(moles) ? Math.max(0, moles) : 0;
  const halfLife = Number.isFinite(halfLifeS) ? Math.max(0, halfLifeS) : 0;
  const molesPerSecond = halfLife > 0 && parcel > 0 ? (Math.LN2 / halfLife) * parcel : 0;
  if (!(molesPerSecond > 0)) return { ticks: 0, periodS: 6, molesPerSecond: 0 };
  const busy = scale(Math.log10(molesPerSecond), -12, -3);
  return { ticks: Math.max(1, Math.round(1 + busy * 7)), periodS: 3.2 - busy * 2.8, molesPerSecond };
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
    case "gas_produced": {
      const rawRate = Number(e.rate_moles_per_second ?? 0);
      return {
        kind: "vent",
        at: now,
        magnitude: gasMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
        gasProduction: {
          molesPerSecond: Number.isFinite(rawRate) ? Math.max(0, rawRate) : 0,
        },
      };
    }
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
    case "gas_absorbed":
      // The mirror of `gas_evolved`: gas crossing INWARD from a boundary
      // and staying in the liquid. It changed the vessel and drew nothing.
      // Same log ramp on the same moles, so a mole in and a mole out read
      // at one scale rather than at two that happen to look similar.
      return {
        kind: "absorb",
        at: now,
        durationMs: 2600,
        magnitude: gasMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
      };
    case "headspace_partitioned": {
      const gasFraction = Math.max(0, Math.min(1, Number(e.gas_fraction ?? 0)));
      return {
        kind: "headspace-partition",
        at: now,
        durationMs: 4200,
        // The standing share, not the step: the band shows where the
        // volatile IS, and the arrows show which way it just went.
        magnitude: gasFraction,
        species: String(e.species ?? ""),
        reading: Number(e.moles ?? 0),
        unit: "mol",
        headspacePartition: {
          species: String(e.species ?? ""),
          toGas: Boolean(e.to_gas),
          moles: Number(e.moles ?? 0),
          gasFraction,
          partialPressurePa: Number(e.partial_pressure_pa ?? 0),
          henryMolPerLAtm: Number(e.henry_mol_per_l_atm ?? 0),
          source: String(e.source ?? ""),
        },
      };
    }
    case "headspace_equilibrated": {
      const pressurePa = Math.max(0, Number(e.pressure ?? 0));
      return {
        kind: "headspace-equilibrium",
        at: now,
        durationMs: 4200,
        // A headspace that settled AT one atmosphere is unremarkable and
        // reads as nothing; the magnitude is how far over that it sits.
        magnitude: scale(pressurePa, 101_325, 500_000),
        reading: pressurePa,
        unit: "Pa",
        headspaceEquilibrium: {
          pressurePa,
          totalMoles: Math.max(0, Number(e.total_moles ?? 0)),
        },
      };
    }
    case "supersaturated": {
      const dissolved = Math.max(0, Number(e.dissolved ?? 0));
      const capacity = Math.max(0, Number(e.capacity ?? 0));
      return {
        kind: "supersaturate",
        at: now,
        durationMs: 5200,
        magnitude: supersaturationHaze(dissolved, capacity),
        species: String(e.species ?? ""),
        reading: dissolved,
        unit: "mol",
        saturation: {
          species: String(e.species ?? ""),
          dissolved,
          capacity,
          ratio: capacity > 0 ? dissolved / capacity : 0,
          excessMoles: Math.max(0, dissolved - capacity),
        },
      };
    }
    case "corroded": {
      // GUI-099: the verdict has carried an EXTENT since PR 4 and nothing
      // drew it. The magnitude is that extent and not the verdict, so a
      // nail the engine says will corrode but has not touched yet draws
      // no rust — which is what a beaker set up a second ago looks like.
      const fraction =
        e.corroded_fraction === undefined
          ? undefined
          : Math.max(0, Math.min(1, Number(e.corroded_fraction)));
      const corrodedMoles =
        e.corroded_moles === undefined ? undefined : Math.max(0, Number(e.corroded_moles));
      return {
        kind: "corrode",
        at: now,
        durationMs: 5000,
        magnitude: fraction ?? 0,
        species: String(e.species ?? ""),
        reading: corrodedMoles,
        unit: corrodedMoles === undefined ? undefined : "mol",
        corrosion: {
          species: String(e.species ?? ""),
          corroding: Boolean(e.corroding),
          why: String(e.why ?? ""),
          corrodedMoles,
          corrodedFraction: fraction,
        },
      };
    }
    case "reacted": {
      // Time passed and this is what it did. The extent and the seconds
      // travel together because the same tenth of a mole in a second and
      // over an hour are different observations, and the bench drew
      // neither of them.
      const moles = Math.max(0, Number(e.moles ?? 0));
      const seconds = Math.max(0, Number(e.seconds ?? 0));
      const extent = reactionExtent(moles, seconds);
      const catalyst = e.catalyst === undefined || e.catalyst === null ? undefined : String(e.catalyst);
      return {
        kind: "react",
        at: now,
        durationMs: 5200,
        magnitude: extent.intensity,
        reading: moles,
        unit: "mol",
        reaction: {
          reaction: String(e.reaction ?? ""),
          equation: String(e.equation ?? ""),
          moles,
          seconds,
          molesPerSecond: extent.molesPerSecond,
          catalyst,
          activationEnergyJPerMol: Number(e.activation_energy ?? 0),
        },
      };
    }
    case "reaction_heat_released": {
      const energyJ = Math.max(0, Number(e.energy_j ?? 0));
      return {
        kind: "exotherm",
        at: now,
        durationMs: 4200,
        magnitude: exothermGlow(energyJ),
        reading: energyJ,
        unit: "J",
        exotherm: { reaction: String(e.reaction ?? ""), energyJ },
      };
    }
    case "neutralised": {
      const moles = Math.max(0, Number(e.moles ?? 0));
      return {
        kind: "neutralise",
        at: now,
        durationMs: 3000,
        magnitude: moles > 0 ? scale(Math.log10(moles), Math.log10(0.0001), Math.log10(0.1)) : 0,
        reading: moles,
        unit: "mol",
        neutralisation: { moles },
      };
    }
    case "bubble_ride": {
      const liftGasFraction = Math.max(0, Number(e.lift_gas_fraction ?? 0));
      return {
        kind: "bubble-ride",
        at: now,
        durationMs: 9000,
        // How much gas it NEEDS is the whole observation: a raisin that
        // wants half its own volume in bubbles is the striking one.
        magnitude: Math.min(1, liftGasFraction),
        reading: liftGasFraction,
        unit: "fraction",
        bubbleRide: {
          object: String(e.object ?? ""),
          objectDensityGPerMl: Number(e.object_density_g_per_ml ?? 0),
          liquidDensityGPerMl: Number(e.liquid_density_g_per_ml ?? 0),
          liftGasFraction,
        },
      };
    }
    case "adsorbed": {
      const heldMoles = Math.max(0, Number(e.held ?? 0));
      const stillDissolvedMoles = Math.max(0, Number(e.still_dissolved ?? 0));
      const removed = adsorptionDarkening(heldMoles, stillDissolvedMoles);
      return {
        kind: "adsorb",
        at: now,
        durationMs: 5200,
        magnitude: removed.removedFraction,
        species: String(e.sorbate ?? ""),
        reading: heldMoles,
        unit: "mol",
        adsorption: {
          sorbate: String(e.sorbate ?? ""),
          sorbent: String(e.sorbent ?? ""),
          heldMoles,
          stillDissolvedMoles,
          loadingMgPerG: Number(e.loading_mg_per_g ?? 0),
          boundary: String(e.boundary ?? ""),
        },
      };
    }
    case "thickened": {
      const strength = Number(e.strength ?? 0);
      const shearedHard = Boolean(e.sheared_hard);
      return {
        kind: "thicken",
        at: now,
        durationMs: 3600,
        magnitude: shearResistance(strength, shearedHard),
        thickening: {
          solid: String(e.solid ?? ""),
          strength: Math.min(1, Math.max(0, Number.isFinite(strength) ? strength : 0)),
          solidMassFraction: Number(e.solid_mass_fraction ?? 0),
          tipSpeedMS: Number(e.tip_speed_m_s ?? 0),
          shearedHard,
        },
      };
    }
    case "flame_starved": {
      // KID-12. `burned: 0` means the flame never caught — the air was
      // already too thin to light in — so the visual must not draw one.
      const oxygenFraction = Math.max(0, Number(e.oxygen_fraction ?? 0));
      const burnedMoles = Math.max(0, Number(e.burned ?? 0));
      const gutter = flameGutter(oxygenFraction, burnedMoles);
      return {
        kind: "flame-starve",
        at: now,
        durationMs: 4600,
        magnitude: gutter.guttering,
        species: String(e.fuel ?? ""),
        reading: oxygenFraction,
        unit: "fraction",
        flameStarved: { fuel: String(e.fuel ?? ""), burnedMoles, oxygenFraction },
      };
    }
    case "below_autoignition": {
      const autoignitionK = Math.max(0, Number(e.autoignition ?? 0));
      const temperatureK = Number(e.temperature ?? 0);
      const gap = autoignitionApproach(temperatureK, autoignitionK);
      return {
        kind: "below-autoignition",
        at: now,
        durationMs: 4600,
        magnitude: gap.approach,
        species: String(e.fuel ?? ""),
        temperatureK,
        reading: gap.gapK,
        unit: "K",
        autoignitionGap: {
          fuel: String(e.fuel ?? ""),
          autoignitionK,
          temperatureK,
          gapK: gap.gapK,
        },
      };
    }
    case "nuclide_spiked": {
      const activityBq = Math.max(0, Number(e.activity_bq ?? 0));
      return {
        kind: "spike",
        at: now,
        durationMs: 5000,
        magnitude: activityIntensity(activityBq),
        reading: activityBq,
        unit: "Bq",
        nuclideSpike: {
          nuclide: String(e.nuclide ?? ""),
          moles: Math.max(0, Number(e.moles ?? 0)),
          activityBq,
        },
      };
    }
    case "partitioned": {
      const fractionLower = Math.min(1, Math.max(0, Number(e.fraction_lower ?? 0)));
      return {
        kind: "solute-partition",
        at: now,
        durationMs: 5000,
        magnitude: fractionLower,
        species: String(e.species ?? ""),
        reading: fractionLower,
        unit: "fraction",
        solutePartition: { species: String(e.species ?? ""), fractionLower },
      };
    }
    case "osmosis_changed": {
      const massChangeG = Number(e.mass_change_g ?? 0);
      const swell = osmoticSwell(massChangeG);
      return {
        kind: "osmosis",
        at: now,
        durationMs: 6000,
        magnitude: swell.swell,
        reading: massChangeG,
        unit: "g",
        osmosis: {
          material: String(e.material ?? ""),
          waterMoles: Number(e.water_moles ?? 0),
          massChangeG,
        },
      };
    }
    case "thermal_equilibrium": {
      const temperatureK = Number(e.temperature ?? 0);
      const reactionEnergyJ =
        e.reaction_energy_j === undefined ? undefined : Number(e.reaction_energy_j);
      return {
        kind: "thermal-equilibrium",
        at: now,
        durationMs: 5000,
        // The heat the solve converted, where it could say; otherwise the
        // badge is a reading and claims no strength of its own.
        magnitude: reactionEnergyJ === undefined ? 0 : exothermGlow(reactionEnergyJ),
        temperatureK,
        reading: temperatureK,
        unit: "K",
        thermalEquilibrium: {
          temperatureK,
          reactionEnergyJ,
          holdsNothing: Boolean(e.holds_nothing),
        },
      };
    }
    case "consumed": {
      const moles = Math.max(0, Number(e.moles ?? 0));
      const remainingMoles = e.remaining === undefined || e.remaining === null
        ? undefined
        : Math.max(0, Number(e.remaining));
      return {
        kind: "consume",
        at: now,
        durationMs: 4200,
        magnitude: moles > 0 ? scale(Math.log10(moles), Math.log10(0.0001), Math.log10(0.1)) : 0,
        species: String(e.species ?? ""),
        reading: moles,
        unit: "mol",
        consumption: { species: String(e.species ?? ""), moles, remainingMoles },
      };
    }
    case "decayed": {
      const moles = Math.max(0, Number(e.moles ?? 0));
      const halfLifeS = Math.max(0, Number(e.half_life_s ?? 0));
      const ticking = decayTicks(moles, halfLifeS);
      return {
        kind: "decay",
        at: now,
        durationMs: 6000,
        // The activity, not the parcel: a long-lived tracer that barely
        // moved should not read like a hot source that did.
        magnitude: ticking.ticks > 0 ? Math.min(1, ticking.ticks / 8) : 0,
        reading: moles,
        unit: "mol",
        decay: {
          parent: String(e.parent ?? ""),
          daughter: String(e.daughter ?? ""),
          mode: String(e.mode ?? ""),
          moles,
          halfLifeS,
          molesPerSecond: ticking.molesPerSecond,
        },
      };
    }
    case "foam_changed": {
      const rawHalfLife = Number(e.half_life_seconds ?? 0);
      const halfLifeSeconds = Number.isFinite(rawHalfLife) ? Math.max(0, rawHalfLife) : 0;
      return {
        kind: "foam",
        at: now,
        durationMs: halfLifeSeconds > 0 ? halfLifeSeconds * 1000 : undefined,
        magnitude: scale(Number(e.height_cm ?? 0), 0.5, 30),
        foam: halfLifeSeconds > 0 ? { halfLifeSeconds } : undefined,
      };
    }
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
          anodeSpecies: e.anode_species === undefined ? undefined : String(e.anode_species),
          anodeMoles: e.anode_moles === undefined ? undefined : Number(e.anode_moles),
          cathodeSpecies: e.cathode_species === undefined ? undefined : String(e.cathode_species),
          cathodeMoles: e.cathode_moles === undefined ? undefined : Number(e.cathode_moles),
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
      // The magnitude on the log-area ramp was all there was; the grain
      // SIZE the engine ground to never reached the vessel, so grinding
      // twice drew the same specks with a different caption.
      return {
        kind: "grind",
        at: now,
        durationMs: 4600,
        magnitude: grindMag(e),
        species: String(e.species ?? ""),
        reading: Number(e.surface_area_m2 ?? 0),
        unit: "m2",
        grind: {
          species: String(e.species ?? ""),
          diameterUm: Math.max(0, Number(e.diameter_um ?? 0)),
          solidMoles: Math.max(0, Number(e.solid_moles ?? 0)),
          surfaceAreaM2: Math.max(0, Number(e.surface_area_m2 ?? 0)),
          rateCoupled: Boolean(e.rate_coupled),
        },
      };
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
    case "plated": {
      // The magnitude was a hard-coded 1: a copper blush on a nail and a
      // nail gone orange drew the same shimmer.
      const moles = Math.max(0, Number(e.moles ?? 0));
      return {
        kind: "plate",
        at: now,
        durationMs: 4200,
        magnitude: platingThickness(moles),
        species: String(e.species ?? ""),
        reading: moles,
        unit: "mol",
        plating: { species: String(e.species ?? ""), onto: String(e.onto ?? ""), moles },
      };
    }
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
      const transition = e.kind === undefined ? undefined : String(e.kind);
      const moles = e.moles === undefined ? undefined : Number(e.moles);
      return {
        kind: phaseKind(from, to, transition),
        at: now,
        durationMs: 3200,
        // A transition that says how much moved is sized by the amount; the
        // colligative shift is the fallback for logs that carry only it.
        magnitude: moles !== undefined && moles > 0 ? vapourIntensity(moles) : phaseMag(e),
        temperatureK: atK,
        reading: moles,
        unit: moles === undefined ? undefined : "mol",
        phase: {
          species: String(e.species ?? ""),
          from,
          to,
          atK,
          shiftedByK: Number(e.shifted_by ?? 0),
          kind: transition,
          moles,
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
    case "emulsion_changed": {
      // GUI-099 ANIM-3: `SceneVessel.emulsion` was read by no component in
      // the app, so shaking oil into water changed nothing on screen.
      const toDispersedFraction = Math.max(0, Math.min(1, Number(e.to_dispersed_fraction ?? 0)));
      const halfLifeSeconds = Math.max(0, Number(e.half_life_seconds ?? 0));
      return {
        kind: "emulsify",
        at: now,
        // The dispersion is visible for as long as it survives: the engine's
        // own half-life, clamped to a watchable window.
        durationMs: Math.min(9000, Math.max(1500, halfLifeSeconds * 1000)),
        magnitude: scale(toDispersedFraction, 0.02, 1),
        reading: toDispersedFraction,
        unit: "fraction",
        emulsion: {
          material: String(e.material ?? ""),
          fromDispersedFraction: Math.max(0, Math.min(1, Number(e.from_dispersed_fraction ?? 0))),
          toDispersedFraction,
          dispersedVolumeL: Math.max(0, Number(e.dispersed_volume_l ?? 0)),
          halfLifeSeconds,
        },
      };
    }
    case "fermented": {
      const carbonDioxideMoles = Math.max(0, Number(e.carbon_dioxide_moles ?? 0));
      const seconds = Math.max(0, Number(e.seconds ?? 0));
      const molesPerSecond = seconds > 0 ? carbonDioxideMoles / seconds : 0;
      return {
        kind: "ferment",
        at: now,
        // Hours of bench time compressed to a watchable window; the TEMPO
        // stays honest because it comes from the rate, not from this.
        durationMs: Math.min(12_000, Math.max(2500, 2500 + Math.log10(1 + seconds) * 2500)),
        magnitude: gasMag({ moles: carbonDioxideMoles }),
        reading: carbonDioxideMoles,
        unit: "mol",
        fermentation: {
          sucroseMoles: Math.max(0, Number(e.sucrose_moles ?? 0)),
          ethanolMoles: Math.max(0, Number(e.ethanol_moles ?? 0)),
          carbonDioxideMoles,
          activeYeastGrams: Math.max(0, Number(e.active_yeast_grams ?? 0)),
          seconds,
          molesPerSecond,
        },
      };
    }
    case "uv_attenuated": {
      const transmittedFraction = Math.max(0, Math.min(1, Number(e.transmitted_fraction ?? 1)));
      return {
        kind: "uv",
        at: now,
        durationMs: 4600,
        // The magnitude is how much was STOPPED: a sunscreen that works
        // draws a strong effect, not a faint one.
        magnitude: 1 - transmittedFraction,
        reading: transmittedFraction,
        unit: "fraction",
        uv: {
          material: String(e.material ?? ""),
          wavelengthNm: Number(e.wavelength_nm ?? 0),
          band: String(e.band ?? ""),
          transmittedFraction,
          mechanism: String(e.mechanism ?? ""),
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
