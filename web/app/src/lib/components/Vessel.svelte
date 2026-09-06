<script lang="ts">
  import type { SceneVessel } from "../host/EngineHost";
  import { KINDS, depositDisplayHeight, solidLayers, fillHeight, graduationTicks } from "../glassware";
  import FluidOverlay from "./FluidOverlay.svelte";
  import type { FluidSpecies } from "../fluidScene";
  import type { Effect } from "../magnitudes";
  import {
    FALLBACK_MOLAR_VOLUME_L,
    NORMAL_BOILING_K,
    compressedVolumeL,
    condensationFilm,
    depositParticles,
    headspaceVolumeL,
    incandescence,
  } from "../magnitudes";
  import { i18n, t } from "../i18n.svelte";
  import DeployedApparatus from "./DeployedApparatus.svelte";
  import { APPARATUS } from "../apparatus";
  import { engineText } from "../engineText";
  import { liveIgnitionEffect } from "../ignitionPresentation";
  import type { WebGpuEnvironmentSnapshot } from "../webGpuLifecycle";
  import IgnitionFlameCanvas from "./IgnitionFlameCanvas.svelte";
  import type { WebGpuMetricsRegistry } from "../webGpuMetricsRegistry";
  import { enzymeReadouts } from "../persistentReadouts";

  let {
    vessel,
    register,
    selected = false,
    onselect,
    ondropspecies,
    effects = [],
    titrationPlayback = null,
    onbadge,
    fluidLookup = null,
    transferTarget = false,
    deployedTool = null,
    linkedTool = null,
    apparatusWorking = false,
    apparatusValues = {},
    gpuIgnition = null,
    gpuMetricsRegistry = null,
  }: {
    vessel: SceneVessel;
    register: string;
    selected?: boolean;
    onselect?: (id: number) => void;
    ondropspecies?: (id: number, payload: { key: string; phase: string }) => void;
    effects?: Effect[];
    titrationPlayback?: { vessel: number; delivered: number; total: number } | null;
    onbadge?: (badge: { key: string; value: number; confidence: string }) => void;
    /** Species srgb+density lookup for the fluid overlay (GUI-065a);
     * absent = no fluid animation, static render only. */
    fluidLookup?: ((key: string) => FluidSpecies) | null;
    transferTarget?: boolean;
    deployedTool?: string | null;
    /** A freestanding workstation associated with this sample. It contributes
     * target/status UI but is deliberately not drawn inside the vessel. */
    linkedTool?: string | null;
    apparatusWorking?: boolean;
    apparatusValues?: Record<string, number | string>;
    gpuIgnition?: WebGpuEnvironmentSnapshot | null;
    gpuMetricsRegistry?: WebGpuMetricsRegistry | null;
  } = $props();

  // Transient effects: young enough that their animation is still running.
  const now = () => Date.now();
  // `Date.now()` is not reactive. While effects exist, advance one lightweight
  // clock so finite visual windows really end even if no other bench state
  // changes at that instant. This also keeps textual apparatus state honest.
  let effectClock = $state(Date.now());
  $effect(() => {
    if (effects.length === 0) return;
    effectClock = Date.now();
    const timer = setInterval(() => (effectClock = Date.now()), 100);
    return () => clearInterval(timer);
  });
  const effectAlive = (effect: Effect, fallbackMs: number, at = now()) =>
    at - effect.at < (effect.durationMs ?? fallbackMs);
  const active = (kind: string, withinMs: number) =>
    effects.some((e) => e.kind === kind && effectAlive(e, withinMs, effectClock));
  // GUI-059: magnitude of the most recent active effect of a given kind.
  const mag = (kind: string, withinMs: number) => {
    const n = effectClock;
    const recent = effects.filter((e) => e.kind === kind && effectAlive(e, withinMs, n));
    return recent.length > 0 ? (recent[recent.length - 1]!.magnitude ?? 1) : 0;
  };
  const latestEffect = (kind: string, withinMs: number) => {
    const recent = effects.filter((effect) => effect.kind === kind && effectAlive(effect, withinMs, effectClock));
    return recent.length > 0 ? recent[recent.length - 1] : undefined;
  };
  const thermometerEffect = $derived(latestEffect("thermometer", 2500));
  const phProbeEffect = $derived(latestEffect("ph_probe", 2500));
  const balanceEffect = $derived(latestEffect("balance", 2500));
  const pressureEffect = $derived(latestEffect("pressure_gauge", 2500));
  const volumeEffect = $derived(latestEffect("volume_meter", 2500));
  const conductivityEffect = $derived(latestEffect("conductivity_meter", 2500));
  const uvvisEffect = $derived(latestEffect("uvvis", 2500));
  const calorimeterEffect = $derived(latestEffect("calorimeter", 2500));
  const chromatographEffect = $derived(latestEffect("chromatograph", 5200));
  const inspectionEffect = $derived(latestEffect("inspect", 4500));
  const geigerEffect = $derived(latestEffect("geiger_counter", 3500));
  const flameTestEffect = $derived(latestEffect("flame_test", 3000));
  const ignitionEffect = $derived(liveIgnitionEffect(effects, effectClock));
  const settlingEffect = $derived(latestEffect("settle", 8000));
  const stirEffect = $derived.by(() => {
    const effect = latestEffect("swirl", 8000);
    return effect?.stir ? effect : undefined;
  });
  const gasTestEffect = $derived(latestEffect("gas_test", 4500));
  const waftEffect = $derived(latestEffect("waft", 4200));
  const pressureControlEffect = $derived(latestEffect("regulate", 4500));
  const sweepEffect = $derived(latestEffect("sweep", 3800));
  const irradiationEffect = $derived(latestEffect("irradiate", 4200));
  const electrolysisEffect = $derived(latestEffect("electrolyse", 8000));
  const thermalEffect = $derived.by(() => {
    const effect = latestEffect(deployedTool === "cool" ? "cool" : "heat", 2600);
    return effect?.thermal ? effect : undefined;
  });
  const electrolysisDeposit = $derived.by(() => {
    const species = electrolysisEffect?.electrolysis?.species;
    return species ? vessel.solids.find((solid) => solid.species === species) : undefined;
  });
  const latestFlameColour = $derived(ignitionEffect?.flameColour);

  function surfaceParticleX(index: number, count: number, cleared: number): number {
    const gap = Math.max(0, Math.min(0.94, cleared));
    if (gap < 0.001) return INNER_X + ((index + 0.5) / count) * INNER_W;
    const right = index % 2 === 1;
    const rank = Math.floor(index / 2);
    const perSide = Math.ceil(count / 2);
    const bandWidth = INNER_W * (1 - gap) / 2;
    const offset = ((rank + 0.5) / perSide) * bandWidth;
    return right ? INNER_X + INNER_W - offset : INNER_X + offset;
  }

  function surfaceColourX(index: number, count: number, spread: number): number {
    const centreOffset = (index - (count - 1) / 2) * 3.2;
    const direction = index % 2 === 0 ? -1 : 1;
    return 50 + centreOffset + direction * Math.max(0, Math.min(1, spread)) * (22 + (index % 3) * 4);
  }

  let dropReady = $state(false);
  /** A just-landed drop ripples once — pouring is an action, not a teleport. */
  let splashedAt = $state(0);

  function ondrop(e: DragEvent) {
    dropReady = false;
    const raw = e.dataTransfer?.getData("application/x-kero-species");
    if (!raw) return;
    e.preventDefault();
    try {
      ondropspecies?.(vessel.id, JSON.parse(raw));
      splashedAt = now();
    } catch {
      // A malformed drag payload is simply not a drop.
    }
  }

  const geom = $derived(KINDS[vessel.label] ?? KINDS.beaker!);
  const INNER_X = $derived(geom.ix);
  const INNER_W = $derived(geom.iw);
  const BOTTOM_Y = $derived(geom.by);
  const FULL_AT_L = $derived(geom.fullAtL);
  const FULL_H = $derived(geom.fh);

  const liquidH = $derived(
    vessel.liquid ? fillHeight(geom, vessel.liquid.volume_l) : 0,
  );
  const foamH = $derived.by(() => {
    if (!vessel.foam || vessel.foam.volume_liters <= 0) return 0;
    const liquidVolume = vessel.liquid?.volume_l ?? 0;
    const combined = fillHeight(
      geom,
      Math.min(FULL_AT_L, liquidVolume + vessel.foam.volume_liters),
      0,
    );
    return Math.max(0, combined - liquidH);
  });
  const foamOverflow = $derived(vessel.foam?.overflow_liters ?? 0);
  const foamColour = $derived(vessel.foam?.srgb ?? [245, 245, 245] as [number, number, number]);
  const snowFraction = $derived(vessel.swelling
    ? Math.min(1, vessel.swelling.swelling_ratio_g_per_g / Math.max(1, vessel.swelling.capacity_g_per_g))
    : 0);
  const snowH = $derived(vessel.swelling ? Math.max(7, FULL_H * (0.22 + snowFraction * 0.68)) : 0);
  const glowStrength = $derived(Math.min(1, (vessel.chemiluminescence?.relative_intensity ?? 0) / 4));
  const persistentEnzymeReadouts = $derived(enzymeReadouts(vessel));
  // The layer stack in pixels, bottom-up: each layer's share of the
  // total height is its share of the total volume, so the drawn split
  // IS the computed split. Falls back to one layer for older scenes.
  const stackedLayers = $derived.by(() => {
    if (!vessel.liquid || liquidH <= 0) return [];
    const source =
      vessel.layers && vessel.layers.length > 0
        ? vessel.layers
        : [
            {
              species: "solution",
              name: t("solution"),
              volume_l: vessel.liquid.volume_l,
              srgb: vessel.liquid.srgb,
              colour_word: vessel.liquid.colour_word,
            },
          ];
    const total = source.reduce((s, l) => s + l.volume_l, 0) || 1;
    let bottom = BOTTOM_Y;
    return source.map((l) => {
      const h = (l.volume_l / total) * liquidH;
      bottom -= h;
      return { ...l, y: bottom, h };
    });
  });
  // Only the top few deposits are drawn; the layer arithmetic counts the
  // same ones, or the stack stops short of the floor.
  const shownSolids = $derived(vessel.solids.filter((solid) => !solid.represented_by_bulk_object).slice(0, 3));
  const solidVolume = (solid: (typeof vessel.solids)[number]) =>
    (solid.volume_l ?? solid.moles * 0.01) * (solid.settled_fraction ?? 1);
  // Solids draw as a settled layer. The scene owns pure-solid volume from
  // molar mass and density; the renderer applies one documented perceptual
  // magnifier so bench-scale traces remain visible.
  const solidH = $derived(
    depositDisplayHeight(
      geom,
      shownSolids.reduce((sum, solid) => sum + solidVolume(solid), 0),
    ),
  );
  const shownSolidLayers = $derived(solidLayers(shownSolids.map(solidVolume), solidH, BOTTOM_Y));
  const rgb = (c: [number, number, number]) => `rgb(${c[0]},${c[1]},${c[2]})`;
  // The engine's srgb is TRANSMITTED light: pure water transmits white,
  // and painting that as an opaque white block is the wrong physics on
  // screen. Opacity follows how much the liquid tints — colourless water
  // reads as glassy, saturated permanganate as nearly solid colour.
  const liquidOpacity = (c: [number, number, number]) => {
    const tint = 1 - Math.min(c[0], c[1], c[2]) / 255;
    return 0.16 + 0.78 * tint;
  };
  const tempC = $derived(vessel.temperature_k - 273.15);
  const phReading = $derived(
    vessel.badges.reduce<number | undefined>(
      (value, badge) => (badge.key === "ph" ? badge.value : value),
      undefined,
    ),
  );
  const formatReading = (value: number, digits: number) =>
    value.toLocaleString(i18n.locale === "de" ? "de-DE" : "en-GB", {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });
  const wavelengthFromUnit = (unit?: string) => {
    const value = unit?.match(/([\d.]+)\s*nm/i)?.[1];
    return value ? Number(value) : null;
  };
  const wavelengthColour = (wavelength: number | null) => {
    if (wavelength === null) return "var(--instrument)";
    const clamped = Math.min(740, Math.max(380, wavelength));
    return `hsl(${270 - ((clamped - 380) / 360) * 270} 88% 54%)`;
  };
  const sealed = $derived(vessel.boundary !== "open");
  // Engine wire tags, not words: the badge showed "pressure_controlled" raw. Values reuse the keys the lids above already translate.
  const BOUNDARY_LABELS: Record<string, string> = { sealed: "sealed", pressure_controlled: "pressure-controlled", swept: "swept with carrier gas" };
  // Combustion is event-authoritative; temperature still owns glow/steam.
  const burning = $derived(ignitionEffect !== undefined);
  let ignitionFallbackVisible = $state(true);
  // GUI-099: the boil is held at the temperature the ENGINE computed, not at
  // a constant. `state_changed` names the plateau it actually used in
  // `phase.atK` — pressure shift and colligative elevation already in it —
  // and `boiling_point_routed` repeats it whenever the route was not the
  // normal boiling point. Between events scene v1 carries no standing
  // boiling point, so the stage falls back to pure water at one atmosphere
  // rather than inventing a correlation of its own.
  const boilEffect = $derived(latestEffect("boil", 3200));
  const boilingK = $derived(boilEffect?.phase?.atK ?? boilEffect?.temperatureK ?? NORMAL_BOILING_K);
  const boilSpecies = $derived(boilEffect?.phase?.species ?? "");
  // Moles of vapour this step actually made. A boil emits them as
  // `gas_evolved` (open) or `gas_contained` (sealed); the `evaporate` verb
  // emits them as `evaporated`. Whichever spoke, the plume and the bubbles
  // are sized by that one number, so a simmer and a flask boiling dry differ.
  const vapourMag = $derived.by(() => {
    const clock = effectClock;
    const gas = effects.filter(
      (effect) =>
        (effect.kind === "vent" || effect.kind === "contain") &&
        effectAlive(effect, 2600, clock) &&
        (boilSpecies === "" || effect.species === boilSpecies),
    );
    const last = gas.length > 0 ? gas[gas.length - 1]!.magnitude : 0;
    return Math.max(mag("evaporate", 2500), last);
  });
  const vapourMoles = $derived.by(() => {
    const clock = effectClock;
    const carriers = effects.filter(
      (effect) =>
        ["vent", "contain", "evaporate"].includes(effect.kind) &&
        effectAlive(effect, 2600, clock) &&
        effect.unit === "mol",
    );
    return carriers.length > 0 ? (carriers[carriers.length - 1]!.reading ?? 0) : 0;
  });
  const boiling = $derived(
    vessel.liquid !== null && (vessel.temperature_k >= boilingK - 0.25 || active("boil", 3200)),
  );
  const steaming = $derived(boiling || active("evaporate", 2500));
  // Above ~800 K a body glows in the visible, and its colour is a function of
  // temperature alone: the blackbody locus, deep red through amber to white.
  const incandescent = $derived(incandescence(vessel.temperature_k));
  // Condensation beads only once the wall is under the room's dew point —
  // which is why a beaker of ice water runs and a beaker of tap water does not.
  const condensation = $derived(condensationFilm(vessel.temperature_k));
  // GUI-099 ANIM-2: how many grains, and how big. The count is the amount,
  // the size is the room a mole of THIS substance takes up — so a fluffy
  // hydroxide and a dense sulfate stop drawing the same 1.2 px circle.
  const precipitateEffect = $derived(latestEffect("precipitate", 1800));
  const dissolveEffect = $derived(latestEffect("dissolve", 1400));
  const precipitateGrains = $derived(
    depositParticles(
      precipitateEffect?.solid?.moles ?? precipitateEffect?.reading ?? 0,
      precipitateEffect?.solid?.molarVolumeLPerMol ?? FALLBACK_MOLAR_VOLUME_L,
    ),
  );
  const dissolveGrains = $derived(
    depositParticles(
      dissolveEffect?.solid?.moles ?? dissolveEffect?.reading ?? 0,
      dissolveEffect?.solid?.molarVolumeLPerMol ?? FALLBACK_MOLAR_VOLUME_L,
    ),
  );
  // GUI-099 ANIM-2: the gas above the liquid, and where the lid that holds
  // it belongs. Exact where the engine named the trapped moles — V = nRT/P,
  // at the scene's own pressure and temperature — and Boyle's law off the
  // free volume and the scene's pressure once that event's window closes.
  // The piston used to be drawn at y=16 whatever the pressure, so squeezing
  // a gas moved nothing on screen.
  const sealEffect = $derived(latestEffect("seal", 4000));
  const capacityL = $derived(geom.capacity_ml / 1000);
  const freeVolumeL = $derived(Math.max(0, capacityL - (vessel.liquid?.volume_l ?? 0)));
  const headspace = $derived.by(() => {
    const control = pressureControlEffect?.pressureControl;
    const pressurePa = vessel.pressure_pa > 0 ? vessel.pressure_pa : (control?.pressurePa ?? 0);
    if (control && control.trappedGasMoles > 0 && pressurePa > 0) {
      const volumeL = headspaceVolumeL(control.trappedGasMoles, vessel.temperature_k, pressurePa);
      if (volumeL > 0) return { volumeL, moles: control.trappedGasMoles, pressurePa, source: "ideal-gas" };
    }
    const sealed = sealEffect?.headspace;
    if (sealed && sealed.volumeL > 0) {
      return { volumeL: sealed.volumeL, moles: sealed.moles, pressurePa, source: "engine" };
    }
    if (vessel.boundary !== "open" && pressurePa > 0) {
      return { volumeL: compressedVolumeL(freeVolumeL, pressurePa), moles: 0, pressurePa, source: "boyle" };
    }
    return { volumeL: freeVolumeL, moles: 0, pressurePa, source: "geometry" };
  });
  const liquidTopY = $derived(BOTTOM_Y - liquidH);
  const pistonY = $derived(
    Math.max(
      6,
      Math.min(
        liquidTopY - 2,
        BOTTOM_Y - fillHeight(geom, Math.min(capacityL, (vessel.liquid?.volume_l ?? 0) + headspace.volumeL), 0),
      ),
    ),
  );
  // A gas held above atmospheric reads denser. One atmosphere is invisible.
  const headspaceTint = $derived(
    Math.min(0.5, Math.max(0, (headspace.pressurePa - 101_325) / 400_000)),
  );
  const frosty = $derived(vessel.temperature_k < 272);
  const hot = $derived(Math.min(1, Math.max(0, (vessel.temperature_k - 310) / 300)));
  const cold = $derived(Math.min(1, Math.max(0, (273.15 - vessel.temperature_k) / 60)));
  const motionMag = $derived(Math.max(mag("swirl", 2200), mag("burst", 1800), mag("heat", 2200), mag("cool", 2200)));
  // A melt is the engine saying the ice went: the frost recedes with it.
  const frostIntensity = $derived(
    Math.max(cold, mag("cool", 2200), mag("freeze", 2200)) * (1 - mag("melt", 3200)),
  );
  const apparatusOperating = $derived(
    apparatusWorking ||
      (deployedTool === "stir" && active("swirl", 2200)) ||
      (deployedTool === "heat" && active("heat", 2200)) ||
      (deployedTool === "cool" && active("cool", 2200)) ||
      (deployedTool === "sweep" && active("sweep", 3800)) ||
      (deployedTool === "irradiate" && active("irradiate", 4200)) ||
      (deployedTool === "electrolyse" && active("electrolyse", 8000)),
  );
  const activeTool = $derived(deployedTool ?? linkedTool);
  const apparatusTitle = $derived(
    activeTool
      ? activeTool === "burette"
        ? "burette"
        : APPARATUS.find((spec) => spec.verb === activeTool)?.title ?? activeTool
      : null,
  );
  const apparatusRelationship = $derived(
    apparatusTitle
      ? linkedTool
        ? t("{tool} workstation for vessel v{vessel}", { tool: t(apparatusTitle), vessel: vessel.id + 1 })
        : t("{tool} installed: {state}", { tool: t(apparatusTitle), state: t(apparatusOperating ? "running…" : "ready") })
      : null,
  );
  const reducedMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;
  const buretteFraction = $derived(
    titrationPlayback ? Math.min(1, titrationPlayback.delivered / (titrationPlayback.total || 1)) : 0,
  );
</script>

<figure
  class="vessel"
  class:selected
  class:drop-ready={dropReady}
  class:transfer-target={transferTarget}
  class:workstation-target={linkedTool !== null}
  class:whirling={active("swirl", 2200)}
  class:apparatus-working={apparatusOperating}
  class:bursting={active("burst", 1800)}
  data-vessel-id={vessel.id}
  data-temperature-k={vessel.temperature_k.toFixed(2)}
  data-boiling-k={boilingK.toFixed(2)}
  style={`--swirl-duration:${2.2 - motionMag * 1.25}s;--stir-duration:${1.15 - motionMag * 0.65}s;--heat-duration:${1.8 - Math.max(hot, mag("heat", 2200)) * 0.8}s;--heat-opacity:${0.25 + Math.max(hot, mag("heat", 2200)) * 0.65};--pour-angle:${9 + mag("pour", 2200) * 23}deg`}
>
  <button
    class="glassbtn"
    class:pouring={active("pour", 2200)}
    aria-label={`${t(vessel.label)} v${vessel.id + 1}: ${t(vessel.words)}${transferTarget ? ` · ${t("transfer target")}` : ""}${apparatusRelationship ? ` · ${apparatusRelationship}` : ""}`}
    aria-pressed={selected}
    onclick={() => onselect?.(vessel.id)}
    ondragover={(e) => {
      if (e.dataTransfer?.types.includes("application/x-kero-species")) {
        e.preventDefault();
        dropReady = true;
      }
    }}
    ondragleave={() => (dropReady = false)}
    {ondrop}
  >
  <svg viewBox="0 0 100 140" role="img" style={`width:clamp(64px,14vw,${geom.svgW}px)`}>
    <title>{t(vessel.words)}</title>
    <defs>
      <clipPath id={`vclip-${vessel.id}`}>
        <path d={geom.inner} />
      </clipPath>
      <clipPath id={`inspect-clip-${vessel.id}`}>
        <circle cx="77" cy="39" r="18" />
      </clipPath>
      <!-- Horizontal glass-curvature tint: denser refraction at the walls,
           near-clear in the middle — what makes a cylinder read as round. -->
      <linearGradient id={`vglass-${vessel.id}`} x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#bcd6e4" stop-opacity="0.28" />
        <stop offset="0.16" stop-color="#bcd6e4" stop-opacity="0.07" />
        <stop offset="0.5" stop-color="#eaf5fb" stop-opacity="0.03" />
        <stop offset="0.86" stop-color="#bcd6e4" stop-opacity="0.08" />
        <stop offset="1" stop-color="#bcd6e4" stop-opacity="0.26" />
      </linearGradient>
      <!-- Vertical depth over the liquid: lit at the surface, dim at the
           bottom. Pure shading — the colour underneath stays the engine's. -->
      <linearGradient id={`vdepth-${vessel.id}`} x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#ffffff" stop-opacity="0.16" />
        <stop offset="0.3" stop-color="#000000" stop-opacity="0" />
        <stop offset="1" stop-color="#000000" stop-opacity="0.2" />
      </linearGradient>
      <radialGradient id={`vglow-${vessel.id}`}>
        <stop offset="0" stop-color="#5de8ff" stop-opacity="0.9" />
        <stop offset="0.45" stop-color="#28aee9" stop-opacity="0.5" />
        <stop offset="1" stop-color="#1770d8" stop-opacity="0" />
      </radialGradient>
    </defs>

    {#if vessel.chemiluminescence && glowStrength > 0.002}
      <g class="computed-glow" aria-hidden="true" style={`--glow-strength:${glowStrength}`}>
        <ellipse cx="50" cy={BOTTOM_Y - Math.max(liquidH, 10) / 2} rx="39" ry={Math.max(29, liquidH * 0.8)} fill={`url(#vglow-${vessel.id})`} />
      </g>
    {/if}

    <g clip-path={`url(#vclip-${vessel.id})`}>
    <!-- The empty glass itself, before any contents. -->
    <path d={geom.inner} fill={`url(#vglass-${vessel.id})`} />
    {#if vessel.liquid && liquidH > 0}
      <!-- Layers (GUI-058): the engine's computed phase split, drawn
           bottom-up — hexane floats on water because the LLE said so.
           A mixed solution is one layer and renders exactly as before. -->
      {#each stackedLayers as layer (layer.species + layer.y)}
        <rect
          x={INNER_X}
          y={layer.y}
          width={INNER_W}
          height={layer.h}
          fill={rgb(layer.srgb)}
          opacity={liquidOpacity(layer.srgb)}
        >
          <title>{t(layer.colour_word)} {t(layer.name)}</title>
        </rect>
        <path
          class="meniscus"
          d={`M ${INNER_X + 1} ${layer.y + 1.5} Q 50 ${layer.y - 1.5} ${INNER_X + INNER_W - 1} ${layer.y + 1.5}`}
        />
      {/each}
      {#if vessel.liquid.cloudiness > 0.01}
        <rect
          x={INNER_X}
          y={BOTTOM_Y - liquidH}
          width={INNER_W}
          height={liquidH}
          fill="var(--cloud)"
          opacity={0.85 * vessel.liquid.cloudiness}
        />
      {/if}
      <rect
        x={INNER_X}
        y={BOTTOM_Y - liquidH}
        width={INNER_W}
        height={liquidH}
        fill={`url(#vdepth-${vessel.id})`}
      />
    {/if}

    {#if vessel.swelling && snowH > 0}
      <g class="swollen-snow">
        <title>{t("computed superabsorbent snow: {water} g water retained, {ratio} times the dry polymer mass", {
          water: vessel.swelling.retained_water_g.toFixed(1),
          ratio: vessel.swelling.swelling_ratio_g_per_g.toFixed(1),
        })}</title>
        <path d={`M ${INNER_X - 2} ${BOTTOM_Y} L ${INNER_X - 2} ${BOTTOM_Y - snowH * 0.48} Q ${INNER_X + INNER_W * 0.16} ${BOTTOM_Y - snowH * 0.68} ${INNER_X + INNER_W * 0.30} ${BOTTOM_Y - snowH * 0.62} Q 50 ${BOTTOM_Y - snowH * 1.02} ${INNER_X + INNER_W * 0.68} ${BOTTOM_Y - snowH * 0.67} Q ${INNER_X + INNER_W * 0.88} ${BOTTOM_Y - snowH * 0.76} ${INNER_X + INNER_W + 2} ${BOTTOM_Y - snowH * 0.46} L ${INNER_X + INNER_W + 2} ${BOTTOM_Y} Z`} />
        {#each Array.from({ length: 13 }, (_, i) => i) as i (i)}
          <circle cx={INNER_X + 3 + ((i * 17) % Math.max(7, INNER_W - 6))} cy={BOTTOM_Y - 3 - ((i * 13) % Math.max(5, snowH * 0.55))} r={0.65 + (i % 3) * 0.25} />
        {/each}
      </g>
    {/if}

    {#if vessel.curds && vessel.liquid && liquidH > 0}
      {@const curdCount = Math.max(4, Math.round(4 + vessel.curds.separation_progress * 16))}
      <g class="milk-curds" class:forming={active("curdle", 2600)}>
        <title>{t("soft curds with {mass} g modeled aggregate solids separated from {material}", {
          mass: vessel.curds.solids_mass_g.toFixed(2),
          material: t(vessel.curds.material),
        })}</title>
        {#each Array.from({ length: curdCount }, (_, i) => i) as i (i)}
          <ellipse
            cx={INNER_X + 4 + ((i * 13) % Math.max(8, INNER_W - 8))}
            cy={BOTTOM_Y - 3 - ((i * 7) % Math.max(5, liquidH * 0.62))}
            rx={1.8 + (i % 3) * 0.65}
            ry={1.0 + (i % 2) * 0.5}
            fill={rgb(vessel.curds.srgb)}
          />
        {/each}
      </g>
    {/if}

    {#if vessel.foam && foamH > 0}
      {@const foamY = BOTTOM_Y - liquidH - foamH}
      <g
        class="foam-state"
        class:rising={active("foam", 3000)}
        style={`transform-origin:50px ${BOTTOM_Y - liquidH}px;--foam-colour:${rgb(foamColour)}`}
      >
        <rect
          class="foam-fill"
          x={INNER_X}
          y={foamY}
          width={INNER_W}
          height={foamH}
        >
          <title>{t("modeled {colour} foam: {height} cm high", {
            colour: t(vessel.foam.colour_word ?? "colourless"),
            height: vessel.foam.height_cm.toFixed(1),
          })}</title>
        </rect>
        {#each Array.from({ length: Math.max(5, Math.round(5 + Math.min(1, vessel.foam.volume_liters / FULL_AT_L) * 11)) }, (_, i) => i) as i (i)}
          <circle
            class="foam-cell"
            cx={INNER_X + 4 + ((i * 17) % Math.max(6, INNER_W - 8))}
            cy={foamY + 3 + ((i * 11) % Math.max(4, foamH - 4))}
            r={1.2 + (i % 3) * 0.55}
          />
        {/each}
      </g>
    {/if}

    {#if vessel.surface_particles && vessel.liquid && liquidH > 0}
      {@const particleCount = Math.max(5, Math.round(5 + vessel.surface_particles.coverage_fraction * 20))}
      <g
        class="surface-particles"
        class:spreading={active("surface-spread", 2600)}
        style={`transform-origin:50px ${BOTTOM_Y - liquidH}px`}
      >
        <title>{t("modeled floating {material}; central clearing {percent}%", {
          material: t(vessel.surface_particles.material),
          percent: Math.round(vessel.surface_particles.cleared_fraction * 100),
        })}</title>
        {#each Array.from({ length: particleCount }, (_, i) => i) as i (i)}
          <circle
            class="surface-particle"
            cx={surfaceParticleX(i, particleCount, vessel.surface_particles.cleared_fraction)}
            cy={BOTTOM_Y - liquidH - 0.8 - (i % 3) * 0.55}
            r={0.65 + (i % 2) * 0.25}
          />
        {/each}
      </g>
    {/if}

    {#if vessel.surface_colours && vessel.surface_colours.length > 0 && vessel.liquid && liquidH > 0}
      <g
        class="surface-colours"
        class:spreading={active("magic-milk", 3000)}
        style={`transform-origin:50px ${BOTTOM_Y - liquidH}px`}
      >
        <title>{t("modeled food-colour drops and streaks on the milk surface")}</title>
        {#each vessel.surface_colours as spot, i (`${spot.material}-${i}`)}
          {@const spotX = surfaceColourX(i, vessel.surface_colours.length, spot.spread_fraction)}
          {@const surfaceY = BOTTOM_Y - liquidH - 1.2 + (i % 2) * 0.7}
          {@const streak = 3 + spot.spread_fraction * (18 + (i % 3) * 3)}
          <path
            class="surface-colour-streak"
            style={`--spot-colour:${rgb(spot.srgb)}`}
            d={`M 50 ${surfaceY} Q ${50 + (i % 2 === 0 ? -5 : 5)} ${surfaceY + 2.5}, ${spotX} ${surfaceY + (i % 3 - 1) * 1.8}`}
            pathLength={Math.max(1, streak)}
          />
          <ellipse
            class="surface-colour-drop"
            style={`--spot-colour:${rgb(spot.srgb)};--spot-x:${spotX}px`}
            cx={spotX}
            cy={surfaceY}
            rx={1.8 + spot.relative_amount * 1.5 + spot.spread_fraction * 2.2}
            ry={1.1 + spot.relative_amount * 0.8}
          />
        {/each}
      </g>
    {/if}

    {#each (vessel.bulk_objects ?? []).slice(0, 3) as object, i (object.recipe_id)}
      {@const objectWidth = Math.min(INNER_W * 0.52, 13 + Math.sqrt(Math.max(0, object.amount_g)) * 2.2)}
      {@const objectX = INNER_X + 4 + i * Math.min(12, (INNER_W - objectWidth - 8) / 2)}
      {@const objectY = object.position === "floating" && liquidH > 0
        ? BOTTOM_Y - liquidH - 3
        : BOTTOM_Y - 7 - i * 2}
      <rect
        class="bulk-object"
        class:floating={object.position === "floating"}
        x={objectX}
        y={objectY}
        width={objectWidth}
        height="7"
        rx="3.5"
        fill={rgb(object.srgb)}
      >
        <title>{t(object.material)} · {object.bulk_density_g_per_ml.toPrecision(3)} g/mL · {t(object.position)}</title>
      </rect>
      {#each (vessel.coatings ?? []).filter((coating) => coating.recipe_id === object.recipe_id) as coating (coating.kind)}
        <rect
          class="persistent-coating"
          class:paint={coating.kind === "paint"}
          class:passive={coating.kind === "passive_film"}
          x={objectX - 1}
          y={objectY - 1}
          width={objectWidth + 2}
          height="9"
          rx="4.5"
          fill="none"
        >
          <title>{t(coating.words)}</title>
        </rect>
      {/each}
    {/each}

    {#each (vessel.material_objects ?? []).slice(0, 3) as object, i (`${object.recipe_id}-${i}`)}
      {@const objectWidth = Math.min(INNER_W * 0.58, 14 + Math.sqrt(Math.max(0, object.mass_g)) * 2.4)}
      <ellipse
        class="prepared-object"
        cx={INNER_X + INNER_W / 2 + (i - 1) * 9}
        cy={BOTTOM_Y - 9 - i * 3}
        rx={objectWidth / 2}
        ry={6}
        fill={`color-mix(in srgb, #d7b36a ${Math.round((1 - object.browned_fraction) * 100)}%, #704321)`}
      >
        <title>{t(object.material)} · {object.mass_g.toFixed(2)} g · {Math.round(object.browned_fraction * 100)}% {t("browned")}</title>
      </ellipse>
    {/each}

    {#if vessel.soap_scum && vessel.soap_scum.aggregate_mass_g > 0}
      <path class="soap-scum" d={`M ${INNER_X + 5} ${BOTTOM_Y - 3} q 12 -7 23 0 t 23 0`}>
        <title>{vessel.soap_scum.aggregate_mass_g.toFixed(3)} g {t("soap-scum aggregate")}</title>
      </path>
    {/if}

    {#if vessel.lemon_paper_mark}
      <g class="lemon-paper-mark">
        <rect x={INNER_X + 12} y={BOTTOM_Y - 15} width={INNER_W - 24} height="10" rx="1" />
        <path
          class:browned={vessel.lemon_paper_mark.browned_fraction > 0}
          style={`--mark-opacity:${Math.max(0.12, vessel.lemon_paper_mark.browned_fraction)}`}
          d={`M ${INNER_X + 18} ${BOTTOM_Y - 10} q 8 -4 15 0 t 15 0`}
        >
          <title>{vessel.lemon_paper_mark.dry ? t("dry lemon mark") : t("wet lemon mark")} · {Math.round(vessel.lemon_paper_mark.browned_fraction * 100)}% {t("brown")}</title>
        </path>
      </g>
    {/if}

    {#if solidH > 0}
      {#each shownSolids as solid, i (solid.species)}
        {@const layer = shownSolidLayers[i]!}
        <rect
          x={INNER_X}
          y={layer.y}
          width={INNER_W}
          height={layer.h}
          fill={rgb(solid.srgb)}
          class:metallic={solid.metallic}
        >
          <title>{t(solid.colour_word)} {t(solid.name)} · {t("volume")} {(solidVolume(solid) * 1000).toPrecision(3)} mL</title>
        </rect>
      {/each}
      <!-- A lit rim on top of the deposit, so it reads as a settled layer
           with a surface rather than a painted band. -->
      <line
        class="solid-rim"
        x1={INNER_X + 2}
        x2={INNER_X + INNER_W - 2}
        y1={BOTTOM_Y - solidH}
        y2={BOTTOM_Y - solidH}
      />
    {/if}

    {#if vessel.gel && vessel.gel.gelled_fraction > 0}
      {@const gelStrength = Math.min(1, Math.max(0, vessel.gel.gelled_fraction))}
      {@const gelHeight = Math.max(12, Math.max(liquidH, solidH) * (0.45 + 0.45 * gelStrength))}
      <g class="gel-body" style={`--gel-opacity:${0.16 + gelStrength * 0.28}`}>
        <path
          d={`M ${INNER_X + 4} ${BOTTOM_Y - 3} C ${INNER_X + 10} ${BOTTOM_Y - gelHeight}, ${INNER_X + INNER_W - 10} ${BOTTOM_Y - gelHeight}, ${INNER_X + INNER_W - 4} ${BOTTOM_Y - 3} Z`}
        >
          <title>{t("translucent cohesive gel")} · {Math.round(vessel.gel.gelled_fraction * 100)}% {t("of polymer gelled")}</title>
        </path>
        <path class="gel-strand" d={`M ${INNER_X + 10} ${BOTTOM_Y - 9} Q 50 ${BOTTOM_Y - gelHeight - 2} ${INNER_X + INNER_W - 10} ${BOTTOM_Y - 9}`} aria-hidden="true" />
      </g>
    {/if}

    {#if fluidLookup}
      <FluidOverlay {vessel} {effects} lookup={fluidLookup} />
    {/if}

    {#if incandescent}
      <!-- GUI-099 red heat: above ~800 K the contents glow in the visible,
           and the colour is a function of temperature alone (the blackbody
           locus) — dull red at 900 K, amber near 2000 K, near-white above
           3500 K. Nothing here is a constant: both the colour and the
           strength come from `vessel.temperature_k`. -->
      {@const glowRgb = `rgb(${incandescent.rgb[0]} ${incandescent.rgb[1]} ${incandescent.rgb[2]})`}
      {@const glowH = Math.max(solidH, liquidH, 8)}
      <rect
        class="incandescence"
        x={INNER_X}
        y={BOTTOM_Y - glowH}
        width={INNER_W}
        height={glowH}
        data-incandescence-k={vessel.temperature_k.toFixed(1)}
        data-incandescence-fraction={incandescent.fraction.toFixed(3)}
        data-incandescence-rgb={incandescent.rgb.join(",")}
        style={`fill:${glowRgb};opacity:${(0.3 + incandescent.fraction * 0.6).toFixed(3)}`}
      >
        <title>{t("glowing at {temperature} K", { temperature: Math.round(vessel.temperature_k) })}</title>
      </rect>
    {/if}
    </g>

    {#if vessel.foam && foamOverflow > 0}
      {@const spillScale = Math.min(1, foamOverflow / Math.max(0.01, FULL_AT_L))}
      <g class="foam-overflow" aria-hidden="true" style={`--spill:${spillScale};--foam-colour:${rgb(foamColour)}`}>
        <ellipse cx="50" cy="7" rx={12 + spillScale * 13} ry={3 + spillScale * 3} />
        <path d={`M ${38 - spillScale * 4} 8 Q ${28 - spillScale * 8} ${18 + spillScale * 8} ${30 - spillScale * 9} ${38 + spillScale * 30}`} />
        <path d={`M ${62 + spillScale * 4} 8 Q ${72 + spillScale * 8} ${18 + spillScale * 8} ${70 + spillScale * 9} ${38 + spillScale * 30}`} />
      </g>
    {/if}

    {#if deployedTool}
      <DeployedApparatus
        tool={deployedTool}
        working={apparatusOperating}
        values={apparatusValues}
        depositColour={electrolysisDeposit ? rgb(electrolysisDeposit.srgb) : undefined}
        depositName={electrolysisDeposit?.name}
        effect={deployedTool === "stir"
          ? stirEffect
          : deployedTool === "heat" || deployedTool === "cool"
            ? thermalEffect
          : deployedTool === "regulate"
            ? pressureControlEffect
            : deployedTool === "sweep"
              ? sweepEffect
              : deployedTool === "irradiate"
                ? irradiationEffect
                : deployedTool === "electrolyse"
                  ? electrolysisEffect
                  : undefined}
        surfaceY={BOTTOM_Y - Math.max(liquidH, 4)}
      />
    {/if}

    <!-- State-driven effects: every one traces to a computed number. -->
    {#if hot > 0.02}
      <ellipse class="glow" cx="50" cy="132" rx="34" ry="5" style={`opacity:${0.15 + hot * 0.5}`} />
    {/if}
    {#if hot > 0.02 || active("heat", 2200)}
      <g class="heater" aria-hidden="true">
        <rect x="25" y="130" width="50" height="7" rx="2" />
        {#each [35, 50, 65] as x, i (x)}
          <path class="heat-wave" d={`M ${x} 128 q -4 -5 0 -10 q 4 -5 0 -10`} style={`animation-delay:${i * 0.18}s`} />
        {/each}
      </g>
    {/if}
    {#if burning && ignitionFallbackVisible}
      {@const flameMagnitude = mag("ignite", 3000)}
      {@const flameScale = 0.42 + flameMagnitude * 0.88}
      {@const flameDuration = 0.48 - flameMagnitude * 0.25}
      <g
        class="flame"
        aria-hidden="true"
        data-flame-energy-j={ignitionEffect?.unit === "J" ? (ignitionEffect.reading ?? 0).toExponential(3) : "unquantified"}
        data-flame-scale={flameScale.toFixed(3)}
        style={`--flame-duration:${flameDuration}s;transform-origin:50px 20px;transform:scale(${flameScale})`}
      >
        <path class="outer" d="M 50 -2 Q 42 12 47 20 Q 50 25 53 20 Q 58 12 50 -2 Z"
          style={latestFlameColour ? `fill:${latestFlameColour};stroke:var(--edge-strong);stroke-width:.55;filter:drop-shadow(0 0 3px ${latestFlameColour})` : ""} />
        <path class="inner" d="M 50 6 Q 46 13 49 18 Q 50 20 51 18 Q 54 13 50 6 Z" />
      </g>
    {/if}
    {#if flameTestEffect}
      {@const testMagnitude = flameTestEffect.magnitude}
      {@const testColour = flameTestEffect.flameColour ?? "var(--hot)"}
      <g class="flame-test-rig" aria-label={t("Bunsen burner flame test")}>
        <ellipse class="burner-foot" cx="18" cy="132" rx="13" ry="3" />
        <path class="burner-base" d="M8 130 Q18 123 28 130 Z" />
        <rect class="burner-barrel" x="14" y="101" width="8" height="27" rx="2" />
        <rect class="burner-collar" x="12" y="113" width="12" height="5" rx="2" />
        <path class="burner-hose" d="M9 128 C2 128 3 117 -2 117" />
        <path class="test-flame" d="M18 99 Q9 89 18 72 Q27 89 18 99 Z" style={`fill:${testColour};filter:drop-shadow(0 0 ${2 + testMagnitude * 4}px ${testColour});--test-flame-rate:${0.52 - testMagnitude * 0.22}s`} />
        <path class="test-flame-core" d="M18 97 Q14 91 18 82 Q22 91 18 97 Z" />
        <path class="wire-handle" d="M96 63 H62 Q50 63 42 73 L25 86" />
        <circle class="wire-loop" cx="21" cy="89" r="5" />
      </g>
    {/if}
    {#if gasTestEffect?.gasTest}
      {@const gasTest = gasTestEffect.gasTest}
      <g
        class="gas-test-rig"
        class:positive={gasTest.positive}
        data-test={gasTest.test}
        aria-label={t("{test}: {result}", {
          test: t(gasTest.test),
          result: t(gasTest.positive ? "positive" : "negative"),
        })}
      >
        {#if gasTest.test === "pop"}
          <path class="test-stick" d="M94 26 L65 18" />
          <path class="test-flame-small" d="M65 18 Q59 13 64 6 Q70 13 65 18Z" />
          {#if gasTest.positive}
            <circle class="pop-wave" cx="54" cy="17" r="8" />
            <text class="result-word" x="54" y="4" text-anchor="middle">{t("pop!")}</text>
          {/if}
        {:else if gasTest.test === "glowing_splint"}
          <path class="test-stick" d="M94 17 L55 17" />
          <circle class="ember" cx="55" cy="17" r="3" />
          {#if gasTest.positive}<path class="relit-flame" d="M55 14 Q48 7 55 -2 Q62 7 55 14Z" />{/if}
        {:else if gasTest.test === "limewater"}
          <path class="delivery-tube" d="M50 8 Q82 -1 86 24 V55" />
          <path class="gas-test-tube" d="M76 30 V73 Q76 82 86 82 Q96 82 96 73 V30" />
          <path class="limewater-fill" class:milky={gasTest.positive} d="M78 56 H94 V73 Q94 79 86 79 Q78 79 78 73Z" />
          {#if gasTest.positive}
            {#each [[82, 62], [88, 66], [84, 72], [91, 75], [90, 59]] as point, i (i)}
              <circle class="lime-particle" cx={point[0]} cy={point[1]} r="1.3" style={`animation-delay:${i * .12}s`} />
            {/each}
          {/if}
        {:else if gasTest.test === "damp_litmus"}
          <path class="forceps" d="M94 14 L61 30 M94 21 L61 33" />
          <path class="litmus-strip" class:changed={gasTest.positive} d="M58 27 L45 31 L48 41 L61 35Z" />
          <path class="water-drop" d="M52 25 Q48 20 52 16 Q56 20 52 25Z" />
        {/if}
        <g class="gas-test-result">
          <rect x="54" y="86" width="43" height="12" rx="5" />
          <circle cx="61" cy="92" r="3" />
          <text x="66" y="94">{t(gasTest.positive ? "positive" : "negative")}</text>
        </g>
        <title>{engineText(gasTest.notes)}</title>
      </g>
    {/if}
    {#if waftEffect?.waft}
      <g
        class="waft-rig"
        aria-label={t("safe waft at vessel v{vessel}: {result}", {
          vessel: vessel.id + 1,
          result: waftEffect.waft.notes.length > 0
            ? waftEffect.waft.notes.map((note) => t(note.species)).join(", ")
            : t("no odour detected"),
        })}
      >
        <path class="waft-hand" d="M92 11 Q84 6 76 10 L67 17 Q63 20 66 23 Q69 25 73 22 L78 19 Q73 24 77 27 Q80 29 84 25 L91 19" />
        {#each [0, 1, 2] as current (current)}
          <path class="waft-current" d={`M ${42 + current * 7} 14 Q ${50 + current * 6} ${7 - current * 2} ${63 + current * 5} ${15 + current * 2}`} style={`--waft-delay:${current * .22}s;--waft-strength:${.45 + waftEffect.magnitude * .55}`} />
        {/each}
        <g class="waft-result">
          <rect x="54" y="31" width="43" height={waftEffect.waft.notes.length > 0 ? 12 + Math.min(2, waftEffect.waft.notes.length) * 6 : 16} rx="5" />
          <text class="waft-rule" x="75.5" y="38" text-anchor="middle">{t("waft — never smell directly")}</text>
          {#if waftEffect.waft.notes.length > 0}
            {#each waftEffect.waft.notes.slice(0, 2) as note, i (note.species)}
              <text class="waft-note" x="75.5" y={44 + i * 6} text-anchor="middle">{t(note.species)}</text>
            {/each}
          {:else}
            <text class="waft-note" x="75.5" y="46" text-anchor="middle">{t("no odour detected")}</text>
          {/if}
        </g>
      </g>
    {/if}
    {#if boiling && liquidH > 0}
      <!-- GUI-099 rolling boil. The gate is the engine's own plateau
           (`state_changed.at`), not 368 K; the bubble count, size and tempo
           follow the moles of vapour that same step actually made. -->
      {@const boilCount = Math.max(4, Math.round(4 + vapourMag * 14))}
      {@const boilRadius = 1.3 + vapourMag * 2.1}
      {@const boilPeriod = Math.max(0.5, 1.5 - vapourMag * 0.95)}
      <g
        class="rolling-boil"
        data-boiling-k={boilingK.toFixed(2)}
        data-vapour-moles={vapourMoles.toExponential(3)}
        data-vapour-intensity={vapourMag.toFixed(3)}
        aria-label={t("rolling boil at {temperature} °C, {moles} mol of vapour", {
          temperature: formatReading(boilingK - 273.15, 1),
          moles: formatReading(vapourMoles, 3),
        })}
      >
        {#each Array.from({ length: boilCount }, (_, i) => i) as i (i)}
          <circle
            class="boil-bubble"
            cx={INNER_X + 4 + ((i * 37) % Math.max(1, INNER_W - 8))}
            cy={BOTTOM_Y - 3}
            r={boilRadius * (0.6 + ((i * 5) % 7) * 0.09)}
            style={`--rise:${Math.max(6, liquidH - 4)}px;animation-duration:${boilPeriod}s;animation-delay:${((i * 0.19) % boilPeriod).toFixed(2)}s`}
          />
        {/each}
        <path
          class="boil-surface"
          d={`M ${INNER_X + 2} ${BOTTOM_Y - liquidH} q ${INNER_W / 4} ${-2 - vapourMag * 3} ${INNER_W / 2} 0 q ${INNER_W / 4} ${2 + vapourMag * 3} ${INNER_W / 2 - 4} 0`}
          style={`--churn:${Math.max(0.35, 0.9 - vapourMag * 0.5)}s`}
        />
      </g>
    {/if}
    {#if steaming}
      <!-- The plume: how many columns there are, how far they climb and how
           opaque they read all follow the same vapour moles. -->
      {@const plumeCount = Math.max(2, Math.round(2 + vapourMag * 4))}
      {@const plumeReach = 12 + vapourMag * 16}
      <g
        class="steam-plume"
        data-vapour-intensity={vapourMag.toFixed(3)}
        data-vapour-moles={vapourMoles.toExponential(3)}
        aria-hidden="true"
      >
        {#each Array.from({ length: plumeCount }, (_, i) => INNER_X + 6 + (i / Math.max(1, plumeCount - 1)) * (INNER_W - 12)) as x, i (i)}
          <path
            class="steam"
            d={`M ${x} ${BOTTOM_Y - liquidH - 4} q 3 ${-plumeReach / 2} 0 ${-plumeReach} q -3 ${-plumeReach / 2} 0 ${-plumeReach}`}
            style={`animation-delay:${(i * 0.42).toFixed(2)}s;--steam-opacity:${(0.3 + vapourMag * 0.7).toFixed(2)}`}
          />
        {/each}
      </g>
    {/if}
    {#if frosty || active("cool", 2200) || active("freeze", 2200)}
      {@const frostPoints = [[18, 40], [80, 60], [22, 90], [78, 105], [30, 55], [68, 78], [40, 100], [60, 42], [50, 68], [28, 112], [72, 116]]}
      {@const frostCount = Math.round(3 + frostIntensity * 8)}
      <g class="frost" aria-hidden="true" style={`opacity:${0.35 + frostIntensity * 0.65}`}>
        {#each frostPoints.slice(0, frostCount) as [fx = 0, fy = 0], i (i)}
          <path d={`M ${fx} ${fy} l 4 0 M ${fx + 2} ${fy - 2} l 0 4 M ${fx} ${fy - 2} l 4 4 M ${fx} ${fy + 2} l 4 -4`} />
        {/each}
      </g>
    {/if}
    {#if condensation > 0.02}
      <!-- GUI-099: the wall is below the ROOM's dew point (Magnus, 20 °C and
           50 % RH), so room water is coming out of the air onto the glass.
           Below freezing the frost layer above draws the same water instead. -->
      {@const dropCount = Math.round(4 + condensation * 12)}
      <g
        class="condensation"
        aria-hidden="true"
        data-condensation={condensation.toFixed(3)}
        data-surface-k={vessel.temperature_k.toFixed(1)}
        style={`opacity:${(0.3 + condensation * 0.6).toFixed(2)}`}
      >
        {#each Array.from({ length: dropCount }, (_, i) => i) as i (i)}
          <ellipse
            cx={INNER_X + 2 + ((i * 23) % Math.max(1, INNER_W - 4))}
            cy={26 + ((i * 41) % Math.max(1, BOTTOM_Y - 36))}
            rx={0.9 + (i % 3) * 0.35 + condensation * 0.6}
            ry={1.2 + (i % 3) * 0.45 + condensation * 0.8}
          />
        {/each}
      </g>
    {/if}

    <!-- Event-driven transients (GUI-026): each fires only because the
         engine emitted the matching event. -->
    {#if precipitateEffect && liquidH > 0 && precipitateGrains.count > 0}
      <!-- GUI-099 ANIM-2: the count is the moles the engine precipitated;
           the grain radius is the cube root of the volume each grain then
           carries (`moles × molar volume ÷ count`); the colour is the
           species' own `srgb` off the scene row, not a generic grey. -->
      {@const pCount = precipitateGrains.count}
      {@const pRadius = Math.max(0.7, Math.min(3.4, 1.1 * precipitateGrains.radiusScale))}
      <g
        class="precipitating"
        data-precipitate-moles={(precipitateEffect.solid?.moles ?? precipitateEffect.reading ?? 0).toExponential(3)}
        data-molar-volume-l={(precipitateEffect.solid?.molarVolumeLPerMol ?? FALLBACK_MOLAR_VOLUME_L).toExponential(3)}
        data-deposit-volume-l={precipitateGrains.particleVolumeL.toExponential(3)}
        data-grain-count={pCount}
        aria-label={t("{moles} mol of {species} coming out of solution", {
          moles: formatReading(precipitateEffect.solid?.moles ?? precipitateEffect.reading ?? 0, 4),
          species: t(precipitateEffect.solid?.name ?? precipitateEffect.species ?? "solid"),
        })}
      >
        {#each Array.from({ length: pCount }, (_, i) => INNER_X + 4 + (i / Math.max(1, pCount - 1)) * (INNER_W - 8)) as x, i (i)}
          <circle
            class="falling"
            cx={x}
            cy={BOTTOM_Y - liquidH + 6}
            r={pRadius}
            style={`--fall:${Math.max(8, liquidH - 10)}px; animation-delay:${(i * 0.12).toFixed(2)}s${precipitateEffect.solid?.colour ? `;fill:${precipitateEffect.solid.colour}` : ""}`}
          />
        {/each}
      </g>
    {/if}
    {#if settlingEffect && liquidH > 0}
      {@const strongest = settlingEffect.settling?.populations.reduce((value, population) => Math.max(value, population.separatedFraction), 0) ?? 0}
      <g
        class="gravity-settling"
        aria-label={t("gravity settling: {percent}% in {seconds} seconds", {
          percent: Math.round(strongest * 100),
          seconds: settlingEffect.settling?.seconds.toFixed(1) ?? "0",
        })}
      >
        <rect class="settling-readout" x={INNER_X + 2} y={BOTTOM_Y - liquidH + 2} width="35" height="9" rx="3" />
        <text x={INNER_X + 19.5} y={BOTTOM_Y - liquidH + 8} text-anchor="middle">↓{Math.round(strongest * 100)}% · {settlingEffect.settling?.seconds.toFixed(1)} s</text>
        {#each settlingEffect.settling?.populations ?? [] as population, populationIndex (population.species)}
          {@const travel = Math.max(3, Math.min(liquidH - 8, (population.distanceM / .04) * Math.max(4, liquidH - 8)))}
          {@const count = Math.max(1, Math.round(1 + population.separatedFraction * 4))}
          {#each Array.from({ length: count }, (_, i) => i) as grain (grain)}
            <circle
              class="settling-grain"
              cx={INNER_X + 5 + ((populationIndex * 19 + grain * 13) % Math.max(7, INNER_W - 10))}
              cy={BOTTOM_Y - liquidH + 13 + ((grain * 7) % Math.max(4, liquidH * .28))}
              r={.8 + Math.min(1.8, population.particleDiameterUm / 100)}
              fill={population.colour ?? "var(--cloud)"}
              style={`--settle-distance:${travel}px;--settle-duration:${Math.min(8, Math.max(1.2, (settlingEffect.durationMs ?? 1200) / 1000))}s;--settle-delay:${populationIndex * .16 + grain * .1}s`}
            />
          {/each}
        {/each}
      </g>
    {/if}
    {#if now() - splashedAt < 900}
      <g class="splash" aria-hidden="true">
        <ellipse cx="50" cy={BOTTOM_Y - Math.max(liquidH, 4)} rx="6" ry="1.6" />
        <ellipse cx="50" cy={BOTTOM_Y - Math.max(liquidH, 4)} rx="11" ry="2.6" style="animation-delay:0.12s" />
      </g>
    {/if}
    {#if dissolveEffect && liquidH > 0}
      <!-- The mirror of the precipitate: the same grains, shrinking away.
           A speck and a spoonful of salt used to dissolve as one r=4 circle. -->
      {@const dCount = Math.max(1, dissolveGrains.count)}
      {@const dRadius = Math.max(0.9, Math.min(4, 1.3 * dissolveGrains.radiusScale))}
      <g
        class="dissolving-grains"
        data-dissolve-moles={(dissolveEffect.solid?.moles ?? dissolveEffect.reading ?? 0).toExponential(3)}
        data-molar-volume-l={(dissolveEffect.solid?.molarVolumeLPerMol ?? FALLBACK_MOLAR_VOLUME_L).toExponential(3)}
        data-grain-count={dCount}
        aria-label={t("{moles} mol of {species} going into solution", {
          moles: formatReading(dissolveEffect.solid?.moles ?? dissolveEffect.reading ?? 0, 4),
          species: t(dissolveEffect.solid?.name ?? dissolveEffect.species ?? "solid"),
        })}
      >
        {#each Array.from({ length: dCount }, (_, i) => i) as i (i)}
          <circle
            class="dissolving"
            cx={INNER_X + 5 + ((i * 29) % Math.max(1, INNER_W - 10))}
            cy={BOTTOM_Y - 6 - ((i * 11) % Math.max(3, Math.round(liquidH * 0.3)))}
            r={dRadius}
            style={`animation-delay:${(i * 0.09).toFixed(2)}s${dissolveEffect.solid?.colour ? `;fill:${dissolveEffect.solid.colour}` : ""}`}
          />
        {/each}
      </g>
    {/if}
    {#if active("electrolyse", 8000) && liquidH > 0}
      {@const eMag = mag("electrolyse", 8000)}
      {@const eBubbles = Math.max(1, Math.round(1 + eMag * 3))}
      {@const eRadius = 1.0 + eMag * 1.0}
      {#each [30, 70] as x (x)}
        {#each Array.from({length: eBubbles}, (_, i) => i) as i (i)}
          <circle
            class="bubble"
            cx={x + (i - Math.floor(eBubbles / 2)) * 2}
            cy={BOTTOM_Y - 6}
            r={eRadius}
            style={`--rise:${liquidH - 10}px; animation-delay:${i * 0.25}s`}
          />
        {/each}
      {/each}
    {/if}
    {#if active("vent", 2600) && !sealed}
      <!-- Gas leaving the open mouth: wisps above the rim, not in the liquid. -->
      {#each [42, 50, 58] as x, i (x)}
        <path
          class="vent"
          d={`M ${x} 8 q 3 -5 0 -10 q -3 -5 0 -10`}
          style={`animation-delay:${i * 0.4}s`}
        />
      {/each}
    {/if}
    {#if active("drip", 2400)}
      <!-- The burette's drop: falls from above the mouth to the surface. -->
      <circle
        class="drip"
        cx="50"
        cy="2"
        r="2"
        style={`--fall-to:${BOTTOM_Y - Math.max(liquidH, 6) - 2}px`}
      />
    {/if}
    {#if active("swirl", 2000) && liquidH > 0}
      {@const sMag = mag("swirl", 2000)}
      {@const sScale = 0.4 + sMag * 0.6}
      {#if deployedTool !== "stir"}
        <g class="stir-plate" aria-hidden="true">
          <ellipse cx="50" cy="122" rx="27" ry="5" />
          <rect x="22" y="124" width="56" height="10" rx="3" />
        </g>
      {/if}
      <ellipse
        class="swirl"
        cx="50"
        cy={BOTTOM_Y - liquidH / 2}
        rx={(INNER_W / 2 - 6) * sScale}
        ry={Math.min(8, liquidH / 3) * sScale}
      />
      <g class="stirrer" aria-hidden="true" style={`transform-origin:50px ${BOTTOM_Y - 6}px`}>
        <rect x="42" y={BOTTOM_Y - 8} width="16" height="4" rx="2" />
      </g>
      {#if stirEffect?.stir}
        <g
          class="stir-result"
          aria-label={t("stirring resuspended {percent}% at {speed} meters per second", {
            percent: Math.round(stirEffect.stir.resuspendedFraction * 100),
            speed: stirEffect.stir.tipSpeedMS.toFixed(3),
          })}
        >
          <rect x={INNER_X + 2} y={BOTTOM_Y - liquidH + 2} width="43" height="13" rx="3" />
          <text x={INNER_X + 23.5} y={BOTTOM_Y - liquidH + 8} text-anchor="middle">↻ {Math.round(stirEffect.stir.resuspendedFraction * 100)}% · {stirEffect.stir.tipSpeedMS.toFixed(3)} m/s</text>
          <text class="rate-boundary" x={INNER_X + 23.5} y={BOTTOM_Y - liquidH + 13} text-anchor="middle">
            {t(stirEffect.stir.rateCoupled ? "reaction rates coupled" : "mixing only — rates unchanged")}
          </text>
          {#each stirEffect.stir.solids.slice(0, 3) as solid, solidIndex (solid.species)}
            {@const particleCount = Math.max(1, Math.round(1 + stirEffect.stir.resuspendedFraction * 4))}
            {#each Array.from({ length: particleCount }, (_, i) => i) as particle (particle)}
              <circle
                class="resuspended-particle"
                cx={INNER_X + 6 + ((solidIndex * 17 + particle * 11) % Math.max(7, INNER_W - 12))}
                cy={BOTTOM_Y - 5}
                r={.8 + Math.min(1.4, solid.moles * 12)}
                fill={solid.colour}
                style={`--resuspend-rise:${Math.max(5, (liquidH - 12) * stirEffect.stir.resuspendedFraction)}px;--resuspend-delay:${solidIndex * .13 + particle * .1}s;--resuspend-duration:${Math.max(.45, 1.35 - stirEffect.magnitude * .7)}s`}
              />
            {/each}
          {/each}
        </g>
      {/if}
    {/if}
    {#if active("burst", 1800)}
      {@const burstMag = mag("burst", 1800)}
      <g class="burst" aria-hidden="true" style={`--burst-distance:${18 + burstMag * 30}px`}>
        {#each [0, 45, 90, 135, 180, 225, 270, 315] as angle (angle)}
          <path d="M 47 65 l 6 -4 l -1 7 z" style={`--angle:${angle}deg`} />
        {/each}
        <circle cx="50" cy="65" r={18 + burstMag * 14} />
      </g>
    {/if}
    {#if active("plate", 2000)}
      <rect class="shimmer" x={INNER_X} y={BOTTOM_Y - Math.max(solidH, 6)} width={INNER_W} height={Math.max(solidH, 6)} />
    {/if}

    <!-- GUI-062: instruments drawn on the bench while their operation is live. -->
    {#if titrationPlayback}
      {@const colH = 56}
      {@const fillH = colH * (1 - buretteFraction)}
      {@const colTop = 4}
      <g class="instrument burette-inst" aria-label={t("burette")}>
        <line class="stand" x1="97" y1="0" x2="97" y2="130" />
        <line class="stand-base" x1="91" y1="130" x2="100" y2="130" />
        <line class="clamp" x1="92" y1="10" x2="97" y2="10" />
        <rect class="burette-col" x="90" y={colTop} width="5" height={colH} rx="1" />
        <rect class="burette-fill" x="91" y={colTop + (colH - fillH)} width="3" height={fillH} rx="0.5" />
        <path class="burette-tap" d="M 90 60 L 95 60 L 94 63 L 92.5 66 L 91 63 Z" />
        <rect class="burette-tip" x="91.5" y="66" width="2" height="4" />
        <line class="burette-tube" x1="92.5" y1="70" x2="50" y2="6" />
        {#if !reducedMotion}
          <circle class="burette-drop" cx="50" cy="6" r="1.4" />
        {/if}
      </g>
    {/if}
    {#if thermometerEffect}
      {@const tipY = BOTTOM_Y - Math.max(liquidH * 0.5, 10)}
      {@const reading = formatReading(thermometerEffect.reading ?? tempC, 1)}
      <g class="instrument thermometer-inst" aria-label={t("temperature probe: {value} °C", { value: reading })}>
        <rect class="meter-body thermometer-meter" x="14" y="2" width="25" height="19" rx="4" />
        <rect class="meter-screen" x="17" y="6" width="19" height="8" rx="1.5" />
        <text class="meter-value" x="26.5" y="12" text-anchor="middle">{reading}</text>
        <text class="meter-unit" x="26.5" y="18" text-anchor="middle">°C</text>
        <rect class="therm-stem" x="32" y="4" width="2" height={tipY - 4} rx="0.8" />
        <ellipse class="therm-bulb" cx="33" cy={tipY} rx="2.8" ry="3.2" />
        <rect class="therm-mercury" x="32.3" y={Math.max(tipY - 18, 10)} width="1.4" height={Math.min(18, tipY - 10)} rx="0.5" />
      </g>
    {/if}
    {#if phProbeEffect}
      {@const tipY = BOTTOM_Y - Math.max(liquidH * 0.5, 10)}
      {@const measuredPh = phProbeEffect.reading ?? phReading}
      {@const reading = measuredPh === undefined ? "—" : formatReading(measuredPh, 2)}
      <g class="instrument ph-inst" aria-label={t("pH probe: {value}", { value: reading })}>
        <rect class="meter-body ph-meter" x="68" y="2" width="28" height="19" rx="4" />
        <rect class="meter-screen" x="71" y="6" width="22" height="8" rx="1.5" />
        <text class="meter-value" x="82" y="12" text-anchor="middle">{reading}</text>
        <text class="meter-unit" x="82" y="18" text-anchor="middle">pH</text>
        <rect class="probe-stem" x="64" y="4" width="2" height={tipY - 4} rx="0.8" />
        <ellipse class="probe-tip" cx="65" cy={tipY} rx="2.2" ry="3.6" />
        <path class="probe-wire" d="M65 4 C65 -1 72 -1 75 3" />
      </g>
    {/if}
    {#if balanceEffect}
      {@const reading = formatReading(balanceEffect.reading ?? vessel.mass_g, 2)}
      <g class="instrument balance-inst" aria-label={t("balance reading: {value} g", { value: reading })}>
        <rect class="balance-pan" x="19" y="120" width="62" height="5" rx="2.5" />
        <path class="balance-neck" d="M28 125 H72 L78 132 H22 Z" />
        <rect class="balance-body" x="16" y="131" width="68" height="8" rx="3" />
        <rect class="balance-screen" x="57" y="132.5" width="20" height="5" rx="1" />
        <text class="balance-value" x="67" y="136.4" text-anchor="middle">{reading} g</text>
        <circle class="balance-key" cx="22" cy="135" r="1.5" />
        <circle class="balance-key" cx="27" cy="135" r="1.5" />
      </g>
    {/if}
    {#if pressureEffect}
      {@const pressure = pressureEffect.reading ?? vessel.pressure_pa / 1000}
      {@const reading = formatReading(pressure, 1)}
      {@const needleAngle = -120 + Math.min(1, Math.max(0, pressure / 500)) * 240}
      <g class="instrument pressure-inst" aria-label={t("pressure gauge reading: {value} kPa", { value: reading })}>
        <path class="gauge-hose" d="M68 31 C61 35 60 42 57 49" />
        <circle class="gauge-case" cx="79" cy="20" r="15" />
        <path class="gauge-safe" d="M68.5 27.5 A13 13 0 1 1 89.5 27.5" />
        <path class="gauge-warning" d="M89.5 27.5 A13 13 0 0 0 92 19" />
        {#each [-120, -80, -40, 0, 40, 80, 120] as angle (angle)}
          <line class="gauge-tick" x1="79" y1="7.5" x2="79" y2="10" transform={`rotate(${angle} 79 20)`} />
        {/each}
        <line class="gauge-needle" x1="79" y1="20" x2="79" y2="10" transform={`rotate(${needleAngle} 79 20)`} />
        <circle class="gauge-pin" cx="79" cy="20" r="2" />
        <rect class="gauge-readout" x="68" y="24" width="22" height="6" rx="1.5" />
        <text class="gauge-value" x="79" y="28.4" text-anchor="middle">{reading}</text>
        <text class="gauge-unit" x="79" y="34" text-anchor="middle">kPa</text>
      </g>
    {/if}
    {#if volumeEffect}
      {@const volume = Math.max(0, volumeEffect.reading ?? 0)}
      {@const reading = formatReading(volume, volume < 100 ? 1 : 0)}
      {@const fraction = Math.min(1, volume / 1000)}
      {@const pistonX = 38 - fraction * 28}
      <g class="instrument volume-inst" aria-label={t("gas volume meter reading: {value} mL", { value: reading })}>
        <path class="syringe-hose" d="M43 23 H50 C57 23 55 39 58 48" />
        <rect class="syringe-barrel" x="5" y="16" width="38" height="14" rx="3" />
        <rect class="syringe-gas" x={pistonX} y="18" width={43 - pistonX} height="10" rx="1" />
        <line class="syringe-piston" x1={pistonX} y1="16" x2={pistonX} y2="30" />
        <line class="syringe-rod" x1="0" y1="23" x2={pistonX} y2="23" />
        <line class="syringe-handle" x1="1" y1="17" x2="1" y2="29" />
        {#each [10, 20, 30, 40] as x (x)}<line class="syringe-tick" x1={x} y1="16" x2={x} y2="19" />{/each}
        <rect class="syringe-screen" x="12" y="32" width="25" height="7" rx="1.5" />
        <text class="syringe-value" x="24.5" y="37" text-anchor="middle">{reading} mL</text>
      </g>
    {/if}
    {#if conductivityEffect}
      {@const conductivity = Math.max(0, conductivityEffect.reading ?? 0)}
      {@const reading = formatReading(conductivity, conductivity < 100 ? 1 : 0)}
      {@const signal = Math.min(1, Math.log10(conductivity + 1) / 6)}
      {@const tipY = BOTTOM_Y - Math.max(liquidH * 0.5, 10)}
      <g class="instrument conductivity-inst" style={`--conductivity:${signal};--conductivity-opacity:${0.25 + signal * 0.65}`} aria-label={t("modeled conductivity estimate: {value} µS/cm", { value: reading })}>
        <rect class="conductivity-meter" x="68" y="2" width="29" height="21" rx="4" />
        <rect class="conductivity-screen" x="71" y="6" width="23" height="8" rx="1.5" />
        <text class="conductivity-value" x="82.5" y="12" text-anchor="middle">≈{reading}</text>
        <text class="conductivity-unit" x="82.5" y="18" text-anchor="middle">µS/cm</text>
        <path class="conductivity-wire" d="M73 22 C68 28 65 27 64 33" />
        <rect class="conductivity-probe" x="61" y="31" width="6" height={Math.max(8, tipY - 31)} rx="2.5" />
        <line class="conductivity-electrode" x1="62.5" y1={tipY - 5} x2="62.5" y2={tipY} />
        <line class="conductivity-electrode" x1="65.5" y1={tipY - 5} x2="65.5" y2={tipY} />
      </g>
    {/if}
    {#if calorimeterEffect}
      {@const energy = calorimeterEffect.reading ?? 0}
      {@const reading = formatReading(energy, 2)}
      <g class="instrument calorimeter-inst" aria-label={t("calorimeter reading: {value} kJ relative to 25 °C", { value: reading })}>
        <path class="calorimeter-jacket" d={`M${Math.max(10, INNER_X - 5)} 66 L${Math.max(10, INNER_X - 5)} 128 Q50 138 ${Math.min(90, INNER_X + INNER_W + 5)} 128 L${Math.min(90, INNER_X + INNER_W + 5)} 66`} />
        <path class="calorimeter-lid" d={`M${Math.max(9, INNER_X - 6)} 66 H${Math.min(91, INNER_X + INNER_W + 6)}`} />
        <rect class="calorimeter-screen" x="34" y="121" width="32" height="9" rx="2" />
        <text class="calorimeter-value" x="50" y="127" text-anchor="middle">{energy >= 0 ? "+" : ""}{reading} kJ</text>
      </g>
    {/if}
    {#if uvvisEffect}
      {@const absorbance = Math.max(0, uvvisEffect.reading ?? 0)}
      {@const reading = formatReading(absorbance, 3)}
      {@const wavelength = wavelengthFromUnit(uvvisEffect.unit)}
      {@const wavelengthText = wavelength === null ? "—" : formatReading(wavelength, 0)}
      {@const beam = wavelengthColour(wavelength)}
      {@const transmittance = Math.pow(10, -absorbance)}
      <g class="instrument uvvis-inst" style={`--beam:${beam};--transmittance:${0.12 + transmittance * 0.88}`} aria-label={t("UV-Vis peak absorbance: {value} AU at {wavelength} nm", { value: reading, wavelength: wavelengthText })}>
        <rect class="uvvis-body" x="2" y="43" width="39" height="42" rx="7" />
        <path class="uvvis-lid" d="M5 54 Q21.5 43 38 54" />
        <rect class="uvvis-cuvette" x="18" y="50" width="8" height="18" rx="1" />
        <line class="uvvis-beam input" x1="6" y1="59" x2="18" y2="59" />
        <line class="uvvis-beam output" x1="26" y1="59" x2="37" y2="59" />
        <rect class="uvvis-screen" x="8" y="71" width="27" height="9" rx="2" />
        <text class="uvvis-value" x="21.5" y="76" text-anchor="middle">A {reading}</text>
        <text class="uvvis-wavelength" x="21.5" y="83" text-anchor="middle">{wavelengthText} nm</text>
      </g>
    {/if}
    {#if chromatographEffect && chromatographEffect.bands?.length}
      {@const maxRetention = Math.max(chromatographEffect.voidTimeS ?? 0, ...chromatographEffect.bands.map((band) => band.retentionTimeS), 1)}
      {@const count = chromatographEffect.bands.length}
      <g class="instrument chromatograph-inst" aria-label={t("chromatography column: {count} computed retention bands", { count })}>
        <path class="chromatograph-tube" d="M18 29 C28 29 28 38 34 43" />
        <rect class="chromatograph-body" x="2" y="9" width="18" height="84" rx="7" />
        <rect class="chromatograph-column" x="7" y="18" width="8" height="58" rx="4" />
        <rect class="chromatograph-solvent" x="8" y="12" width="6" height="8" rx="2" />
        <path class="chromatograph-flow" d="M11 21 V72" />
        {#each chromatographEffect.bands as band, i (`${band.species}-${band.retentionTimeS}`)}
          {@const position = 23 + (band.retentionTimeS / maxRetention) * 47}
          {@const thickness = Math.max(1.6, Math.min(5.5, 1.6 + (band.widthS / maxRetention) * 24))}
          <rect
            class="chromatograph-band band-{i % 5}"
            x="7.6"
            y={position}
            width="6.8"
            height={thickness}
            rx={Math.min(2, thickness / 2)}
            style={`--band-travel:${position - 22}px;--band-delay:${i * 0.16}s;--band-opacity:${0.38 + Math.min(1, band.relativeArea) * 0.62}`}
          >
            <title>{t("{species}: retention {time} s, relative area {area}%", { species: t(band.species), time: formatReading(band.retentionTimeS, 1), area: formatReading(band.relativeArea * 100, 0) })}</title>
          </rect>
        {/each}
        <rect class="chromatograph-screen" x="1" y="80" width="20" height="13" rx="2.5" />
        <text class="chromatograph-count" x="11" y="86" text-anchor="middle">{count} {t(count === 1 ? "band" : "bands")}</text>
        <text class="chromatograph-plates" x="11" y="91" text-anchor="middle">N {chromatographEffect.plates ?? "—"}</text>
      </g>
    {/if}
    {#if inspectionEffect?.appearance}
      {@const appearance = inspectionEffect.appearance}
      {@const cloudCount = Math.round(2 + Math.min(1, appearance.cloudiness) * 14)}
      {@const cloudiness = formatReading(Math.min(1, appearance.cloudiness) * 100, 0)}
      <g class="instrument inspection-inst" aria-label={`${t("magnified computed appearance")}. ${t("computed turbidity {value}%", { value: cloudiness })}`}>
        <g clip-path={`url(#inspect-clip-${vessel.id})`}>
          <circle class="inspection-field" cx="77" cy="39" r="18" />
          {#if appearance.liquidRgb}
            <rect class="inspection-liquid" x="58" y="32" width="38" height="27" fill={rgb(appearance.liquidRgb)} />
            <path class="inspection-meniscus" d="M59 33 Q77 29 95 33" />
          {/if}
          {#each Array.from({ length: cloudCount }, (_, i) => i) as i (i)}
            <circle
              class="inspection-speck"
              cx={61 + ((i * 11) % 32)}
              cy={35 + ((i * 17) % 20)}
              r={0.7 + (i % 3) * 0.32}
              opacity={0.18 + Math.min(1, appearance.cloudiness) * 0.68}
            />
          {/each}
          {#if appearance.deposit}
            <path class="inspection-deposit" fill={rgb(appearance.deposit.rgb)} d="M58 53 Q67 48 76 53 Q85 47 96 53 V60 H58 Z">
              <title>{t(appearance.deposit.species)}</title>
            </path>
          {/if}
          {#if appearance.bubbling}
            {#each [64, 72, 81, 89] as x, i (x)}
              <circle class="inspection-bubble" cx={x} cy={52 - (i % 2) * 5} r={1.4 + (i % 2) * 0.7} style={`animation-delay:${i * 0.22}s`} />
            {/each}
          {/if}
        </g>
        <circle class="magnifier-rim" cx="77" cy="39" r="19.5" />
        <path class="magnifier-handle" d="M90.5 53 L101 66" />
      </g>
    {/if}
    {#if geigerEffect}
      {@const activity = Math.max(0, geigerEffect.reading ?? 0)}
      {@const reading = formatReading(activity, activity < 10 ? 1 : 0)}
      {@const cadence = Math.max(0.13, 0.95 - geigerEffect.magnitude * 0.82)}
      <g class="instrument geiger-inst" style={`--count-cadence:${cadence}s`} aria-label={t("Geiger counter reading: {value} Bq", { value: reading })}>
        <path class="geiger-wire" d="M27 29 C37 34 39 43 43 50" />
        <rect class="geiger-body" x="3" y="5" width="27" height="31" rx="5" />
        <rect class="geiger-screen" x="7" y="10" width="19" height="10" rx="2" />
        <text class="geiger-value" x="16.5" y="16.5" text-anchor="middle">{reading}</text>
        <text class="geiger-unit" x="16.5" y="24.5" text-anchor="middle">Bq</text>
        <circle class="geiger-led" cx="8" cy="29" r="2" />
        <rect class="geiger-probe" x="40" y="44" width="8" height="34" rx="4" transform="rotate(-18 44 61)" />
        {#each [0, 1, 2] as pulse (pulse)}
          <circle class="geiger-pulse" cx="45" cy="76" r={4 + pulse * 3} style={`animation-delay:${pulse * 0.11}s`} />
        {/each}
      </g>
    {/if}

    {#if vessel.bubbling && liquidH > 0}
      <!-- GUI-059: the fizz is sized by the gas the step actually made. Two
           staggered columns of bubbles so a real effervescence reads as a
           curtain rising through the liquid, not a row of beads; the count,
           size and tempo all follow the magnitude. -->
      {@const bMag = mag("vent", 4000)}
      {@const bCount = Math.max(3, Math.round(3 + bMag * 11))}
      {@const bRadius = 1.2 + bMag * 1.6}
      {@const bPeriod = 2.4 - bMag * 1.2}
      {#each Array.from({length: bCount}, (_, i) => INNER_X + 5 + (i / Math.max(1, bCount - 1)) * (INNER_W - 10)) as x, i (i)}
        <circle
          class="bubble"
          cx={x}
          cy={BOTTOM_Y - 4 - (i % 3) * 3}
          r={bRadius * (0.7 + ((i * 7) % 5) * 0.12)}
          style={`--rise:${liquidH - 8}px; animation-duration:${bPeriod}s; animation-delay:${(i * 0.37) % bPeriod}s`}
        />
      {/each}
    {/if}

    {#if vessel.label === "cylinder"}
      {#each graduationTicks(geom) as tick (tick.ml)}
        <line class="tick" x1="39" x2="46" y1={tick.y} y2={tick.y} />
        <text class="tick-label" x="37" y={tick.y + 1.5} text-anchor="end">{tick.ml}</text>
      {/each}
    {/if}

    <!-- Grounding shadow: the vessel stands on the bench, not in space. -->
    <ellipse class="shadow" cx="50" cy="131" rx={INNER_W / 2 + 6} ry="3.5" />

    <!-- The glass, drawn over the contents, with a sheen that makes it
         read as glass rather than wireframe. -->
    <path class="glass" d={geom.glass} />
    <path
      class="sheen"
      d={`M ${INNER_X + 3} 18 Q ${INNER_X + 1} ${BOTTOM_Y / 2} ${INNER_X + 4} ${BOTTOM_Y - 8}`}
    />
    <path
      class="sheen faint"
      d={`M ${INNER_X + INNER_W - 4} 24 Q ${INNER_X + INNER_W - 2} ${BOTTOM_Y / 2} ${INNER_X + INNER_W - 5} ${BOTTOM_Y - 14}`}
    />
    {#if vessel.boundary !== "open" && headspace.volumeL > 0}
      <!-- GUI-099 ANIM-2: the gas above the liquid, drawn where it actually
           is. Held above atmospheric it reads denser; at one atmosphere it
           is invisible, which is what a headspace at rest should look like. -->
      {@const bandTop = vessel.boundary === "pressure_controlled" ? pistonY + 4 : 14}
      {@const bandHeight = Math.max(0, liquidTopY - bandTop)}
      {#if bandHeight > 1}
        <rect
          class="headspace"
          x={INNER_X + 1}
          y={bandTop}
          width={INNER_W - 2}
          height={bandHeight}
          data-headspace-l={headspace.volumeL.toExponential(3)}
          data-headspace-moles={headspace.moles.toExponential(3)}
          data-headspace-pressure-pa={Math.round(headspace.pressurePa)}
          data-headspace-source={headspace.source}
          style={`opacity:${(0.08 + headspaceTint).toFixed(3)}`}
        >
          <title>{t("{litres} L of headspace gas at {pressure} kPa", {
            litres: formatReading(headspace.volumeL, 3),
            pressure: formatReading(headspace.pressurePa / 1000, 1),
          })}</title>
        </rect>
      {/if}
    {/if}
    {#if vessel.boundary === "sealed"}
      <rect class="lid" x="10" y="9" width="80" height="5" rx="2">
        <title>{t("sealed")}</title>
      </rect>
    {:else if vessel.boundary === "pressure_controlled"}
      <!-- A floating piston: the lid that moves to hold the set pressure.
           Its height is the volume the trapped gas occupies at that
           pressure — squeeze it and the piston comes down. -->
      <g
        class="piston-assembly"
        data-piston-y={pistonY.toFixed(2)}
        data-headspace-l={headspace.volumeL.toExponential(3)}
        data-headspace-source={headspace.source}
      >
        <rect class="lid" x="14" y={pistonY} width="72" height="4" rx="1">
          <title>{t("{litres} L of headspace gas at {pressure} kPa", {
            litres: formatReading(headspace.volumeL, 3),
            pressure: formatReading(headspace.pressurePa / 1000, 1),
          })}</title>
        </rect>
        <line class="piston" x1="50" y1={Math.max(0, pistonY - 12)} x2="50" y2={pistonY} />
        <line class="piston" x1="42" y1={Math.max(0, pistonY - 12)} x2="58" y2={Math.max(0, pistonY - 12)} />
      </g>
    {:else if vessel.boundary === "swept"}
      <!-- Carrier gas in one side, out the other. -->
      <g class="sweep" aria-hidden="true">
        <line x1="2" y1="18" x2="30" y2="18" />
        <path d="M 30 18 l -5 -3 v 6 z" />
        <line x1="70" y1="12" x2="98" y2="12" />
        <path d="M 98 12 l -5 -3 v 6 z" />
        <title>{t("swept with carrier gas")}</title>
      </g>
    {/if}
  </svg>
  {#if ignitionEffect && gpuIgnition}
    {#if gpuMetricsRegistry}
      <span class="ignition-flame-gpu" aria-hidden="true">
        <IgnitionFlameCanvas
          effect={ignitionEffect}
          vesselIdentity={vessel.id}
          gpu={gpuIgnition}
          metricsRegistry={gpuMetricsRegistry}
          onfallbackchange={(visible) => (ignitionFallbackVisible = visible)}
        />
      </span>
    {/if}
  {/if}
  </button>

  {#if dropReady}<span class="drop-hint">{t("add here")}</span>{/if}

  <span class="observation-status" role="status" aria-live="polite" aria-atomic="true">{t(vessel.words)}</span>

  <figcaption class="caption">
    <span class="label">{t(vessel.label)} v{vessel.id + 1}</span>
    {#if vessel.gel}
      <small class="gel-status">{Math.round(vessel.gel.gelled_fraction * 100)}% {t("of polymer gelled")}</small>
    {/if}
    {#each persistentEnzymeReadouts as progress (progress.material + progress.family)}
      <span
        class="persistent-readout"
        aria-label={t("{family} enzyme model: {percent}% of {substrate} converted in {material}", {
          family: t(progress.family),
          percent: progress.percent,
          substrate: t(progress.substrate),
          material: t(progress.material),
        })}
      >
        <small>{t(progress.family)} · {t("enzyme conversion")}</small>
        <strong>{progress.percent}%</strong>
      </span>
    {/each}
    {#if apparatusTitle}
      <span
        class="apparatus-status"
        class:running={apparatusOperating}
        title={apparatusRelationship ?? undefined}
      >
        <span class="status-light" aria-hidden="true"></span>
        <strong>{t(apparatusTitle)}</strong>
        <small>{t(apparatusOperating ? "running…" : "ready")}</small>
      </span>
    {/if}
    {#if register !== "lv1"}
      <button
        class="badge"
        onclick={() => onbadge?.({ key: "temperature", value: tempC, confidence: "computed" })}
      >
        {tempC.toFixed(1)} °C
      </button>
      {#each vessel.badges as badge (badge.key)}
        <button
          class="badge"
          data-confidence={badge.confidence}
          onclick={() => onbadge?.(badge)}
        >
          {badge.key === "ph" ? "pH" : t(badge.key.replaceAll("_", " "))}
          {badge.value.toFixed(2)}
        </button>
      {/each}
      {#if sealed}<span class="badge">{t(BOUNDARY_LABELS[vessel.boundary] ?? vessel.boundary)}</span>{/if}
    {/if}
  </figcaption>
</figure>

<style>
  .vessel {
    margin: 0;
    position: relative;
    padding: 0.4rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    border: 1px solid transparent;
    border-radius: 16px;
    transition:
      transform 160ms ease,
      border-color 160ms ease,
      background-color 160ms ease,
      box-shadow 160ms ease;
  }
  .glassbtn {
    background: none;
    border: 0;
    padding: 0;
    color: inherit;
    font: inherit;
    cursor: pointer;
    display: block;
    border-radius: 12px;
    transform-origin: 52% 78%;
    position: relative;
  }
  .ignition-flame-gpu {
    position: absolute;
    z-index: 2;
    left: 50%;
    top: -4px;
    width: 48px;
    height: 56px;
    transform: translateX(-50%);
    pointer-events: none;
  }
  .glassbtn.pouring { animation: vessel-pour 1.5s cubic-bezier(.3,.05,.3,1) both; }
  @keyframes vessel-pour {
    0%, 100% { transform: rotate(0deg) translateY(0); }
    25%, 70% { transform: rotate(var(--pour-angle)) translateY(-3px); }
  }
  .vessel:hover {
    border-color: var(--edge);
    background: color-mix(in srgb, var(--surface) 58%, transparent);
  }
  .vessel.selected {
    border-color: var(--action);
    background: color-mix(in srgb, var(--action) 7%, var(--surface));
    box-shadow: 0 8px 20px var(--shadow);
    transform: translateY(-3px);
  }
  .vessel.drop-ready {
    border-color: var(--good);
    background: color-mix(in srgb, var(--success) 13%, var(--surface));
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--success) 14%, transparent);
  }
  .vessel.workstation-target .glassbtn {
    box-shadow:
      0 0 0 2px color-mix(in srgb, var(--instrument) 42%, transparent),
      0 9px 20px var(--shadow);
  }
  .vessel.transfer-target {
    border-color: var(--instrument);
    background: color-mix(in srgb, var(--instrument) 8%, var(--surface));
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--instrument) 13%, transparent);
    animation: target-breathe 1.4s ease-in-out infinite alternate;
  }
  @keyframes target-breathe {
    to { box-shadow: 0 0 0 6px color-mix(in srgb, var(--instrument) 7%, transparent); }
  }
  .drop-hint {
    position: absolute;
    top: -1.5rem;
    left: 50%;
    translate: -50% 0;
    padding: 0.2rem 0.5rem;
    border-radius: 999px;
    color: var(--surface);
    background: var(--success);
    font-size: 0.65rem;
    font-weight: 750;
    white-space: nowrap;
  }
  svg {
    height: auto;
  }
  .glass {
    fill: none;
    stroke: var(--edge-strong);
    stroke-width: 2.5;
    stroke-linecap: round;
  }
  .tick {
    stroke: var(--edge-strong);
    stroke-width: 0.8;
    opacity: 0.7;
  }
  .tick-label {
    font-size: 5.5px;
    fill: var(--dim);
    font-family: sans-serif;
  }
  .sheen {
    fill: none;
    stroke: var(--cloud);
    stroke-width: 1.6;
    stroke-linecap: round;
    opacity: 0.22;
  }
  .sheen.faint {
    stroke-width: 1;
    opacity: 0.12;
  }
  .solid-rim {
    stroke: rgb(255 255 255 / 30%);
    stroke-width: 1;
  }
  .bulk-object {
    stroke: color-mix(in srgb, var(--ink) 58%, transparent);
    stroke-width: 0.75;
    filter: drop-shadow(0 1px 1px var(--shadow));
  }
  .bulk-object.floating {
    transform-box: fill-box;
    transform-origin: center;
    animation: object-bob 2.8s ease-in-out infinite alternate;
  }
  .persistent-coating {
    pointer-events: none;
    stroke-width: 2;
    stroke-dasharray: 2 1;
  }
  .persistent-coating.paint {
    stroke: #3f78a8;
  }
  .persistent-coating.passive {
    stroke: rgb(196 225 235 / 70%);
    stroke-width: 1.2;
  }
  .prepared-object {
    stroke: color-mix(in srgb, var(--ink) 55%, transparent);
    stroke-width: 0.8;
    filter: drop-shadow(0 1px 1px var(--shadow));
  }
  .gel-body > path:first-of-type {
    fill: #9f8bd7;
    fill-opacity: var(--gel-opacity);
    stroke: #7865b2;
    stroke-width: 1.1;
    stroke-opacity: 0.72;
  }
  .gel-strand {
    fill: none;
    stroke: #e8e0ff;
    stroke-width: 1.3;
    stroke-linecap: round;
    opacity: 0.65;
  }
  .soap-scum {
    fill: none;
    stroke: #ded8c7;
    stroke-width: 3;
    stroke-linecap: round;
    opacity: 0.9;
  }
  .lemon-paper-mark rect { fill: #f4f0df; stroke: #9e957b; stroke-width: 0.6; }
  .lemon-paper-mark path { fill: none; stroke: #d9cf9c; stroke-width: 1.4; opacity: 0.25; }
  .lemon-paper-mark path.browned { stroke: #7a431e; opacity: var(--mark-opacity); }
  @keyframes object-bob {
    to { transform: translateY(-1.2px) rotate(1.5deg); }
  }
  .vent {
    fill: none;
    stroke: var(--cloud);
    stroke-width: 1.2;
    stroke-linecap: round;
    opacity: 0;
    animation: vent-rise 1.6s ease-out infinite;
  }
  @keyframes vent-rise {
    0% {
      opacity: 0;
      transform: translateY(4px);
    }
    30% {
      opacity: 0.5;
    }
    100% {
      opacity: 0;
      transform: translateY(-6px);
    }
  }
  .drip {
    fill: var(--cool);
    animation: drip-fall 0.8s ease-in infinite;
  }
  @keyframes drip-fall {
    0% {
      opacity: 0;
      transform: translateY(0);
    }
    15% {
      opacity: 0.9;
    }
    90% {
      opacity: 0.9;
      transform: translateY(var(--fall-to, 90px));
    }
    100% {
      opacity: 0;
      transform: translateY(var(--fall-to, 90px));
    }
  }
  .swirl {
    fill: none;
    stroke: var(--cloud);
    stroke-width: 1.3;
    stroke-dasharray: 6 5;
    opacity: 0.45;
    animation: swirl-turn var(--swirl-duration, 2s) linear forwards;
  }
  .stirrer {
    transform-origin: center;
  }
  .apparatus-working .stirrer { animation: stir-tool var(--stir-duration, 1s) linear infinite; }
  .stirrer rect { fill: var(--surface); stroke: var(--edge-strong); stroke-width: 1.2; }
  .stir-plate ellipse { fill: var(--surface); stroke: var(--edge-strong); stroke-width: 1.2; }
  .stir-plate rect { fill: color-mix(in srgb, var(--instrument) 28%, var(--edge-strong)); stroke: var(--edge-strong); stroke-width: 1; }
  @keyframes stir-tool { to { transform: rotate(360deg); } }
  .heater rect { fill: color-mix(in srgb, var(--hot) 40%, var(--edge-strong)); }
  .heat-wave { fill: none; stroke: var(--hot); stroke-width: 1.2; opacity: 0; animation: heat-rise var(--heat-duration, 1.5s) ease-out infinite; }
  @keyframes heat-rise { 0% { opacity: 0; transform: translateY(4px); } 35% { opacity: var(--heat-opacity, 0.5); } 100% { opacity: 0; transform: translateY(-8px); } }
  .burst path { fill: var(--edge-strong); transform-box: fill-box; transform-origin: center; animation: shard-fly 1.1s cubic-bezier(.12,.65,.25,1) forwards; }
  .burst path:nth-child(2n) { fill: var(--cloud); }
  .burst circle { fill: none; stroke: var(--danger); stroke-width: 3; opacity: 0; animation: pressure-wave 0.9s ease-out forwards; }
  @keyframes shard-fly { to { opacity: 0; transform: rotate(var(--angle)) translateX(var(--burst-distance)) rotate(220deg); } }
  @keyframes pressure-wave { 0% { opacity: 0.8; transform: scale(0.25); transform-origin: 50px 65px; } 100% { opacity: 0; transform: scale(2.1); transform-origin: 50px 65px; } }
  .vessel.bursting { animation: burst-shock 0.42s linear 2; }
  @keyframes burst-shock { 25% { transform: translate(-5px, 1px) rotate(-1deg); } 75% { transform: translate(5px, -1px) rotate(1deg); } }
  @keyframes swirl-turn {
    0% {
      stroke-dashoffset: 0;
      opacity: 0.45;
    }
    100% {
      stroke-dashoffset: -44;
      opacity: 0;
    }
  }
  .splash ellipse {
    fill: none;
    stroke: var(--cloud);
    stroke-width: 1.1;
    opacity: 0;
    transform-box: fill-box;
    transform-origin: center;
    animation: splash-ring 0.9s ease-out forwards;
  }
  @keyframes splash-ring {
    0% {
      opacity: 0.55;
      transform: scale(0.4);
    }
    100% {
      opacity: 0;
      transform: scale(1.25);
    }
  }
  .shadow {
    fill: var(--shadow, rgb(0 0 0 / 30%));
  }
  .meniscus {
    fill: none;
    stroke: var(--cloud);
    stroke-width: 1;
    opacity: 0.3;
  }
  .computed-glow {
    opacity: calc(0.18 + var(--glow-strength) * 0.68);
    filter: blur(2px);
    pointer-events: none;
  }
  .swollen-snow path {
    fill: color-mix(in srgb, #f8fbff 86%, #a9dcf0);
    stroke: color-mix(in srgb, #8fc6db 65%, var(--edge));
    stroke-width: 0.7;
  }
  .swollen-snow circle {
    fill: #ffffff;
    opacity: 0.85;
  }
  /* Liquid and deposits move smoothly between computed states — the
     motion is presentation only; every keyframe endpoint is engine data. */
  svg rect {
    transition:
      y 0.5s ease,
      height 0.5s ease,
      fill 0.5s ease,
      opacity 0.5s ease;
  }
  @media (prefers-reduced-motion: reduce) {
    svg rect {
      transition: none;
    }
  }
  .lid {
    fill: var(--edge-strong);
  }
  .piston {
    stroke: var(--edge-strong);
    stroke-width: 2;
  }
  .sweep line {
    stroke: var(--dim);
    stroke-width: 1.2;
  }
  .sweep path {
    fill: var(--dim);
  }
  .metallic {
    stroke: var(--ink);
    stroke-width: 0.6;
    stroke-dasharray: 3 1.5;
  }
  .bubble {
    fill: none;
    stroke: var(--dim);
    stroke-width: 0.8;
    animation: rise 2.2s linear infinite;
  }
  .foam-fill,
  .foam-overflow ellipse,
  .foam-overflow path {
    fill: color-mix(in srgb, white 62%, var(--foam-colour, var(--instrument)));
    stroke: color-mix(in srgb, var(--foam-colour, var(--instrument)) 56%, var(--edge));
    stroke-width: 0.55;
  }
  .foam-state.rising {
    animation: foam-rise 900ms cubic-bezier(.2, .8, .25, 1) both;
  }
  .foam-cell {
    fill: color-mix(in srgb, white 30%, transparent);
    stroke: color-mix(in srgb, var(--foam-colour, var(--instrument)) 48%, var(--edge));
    stroke-width: 0.45;
  }
  .surface-particle {
    fill: #31261f;
    stroke: #0f0d0b;
    stroke-width: 0.25;
  }
  .surface-particles.spreading {
    animation: surface-spread 780ms cubic-bezier(.16, .82, .2, 1) both;
  }
  .surface-colour-streak {
    fill: none;
    stroke: var(--spot-colour);
    stroke-width: 2.1;
    stroke-linecap: round;
    opacity: 0.82;
  }
  .surface-colour-drop {
    fill: var(--spot-colour);
    stroke: color-mix(in srgb, var(--spot-colour) 72%, var(--edge));
    stroke-width: 0.35;
    opacity: 0.92;
  }
  .surface-colours.spreading .surface-colour-streak {
    animation: milk-colour-streak 1050ms cubic-bezier(.12, .78, .18, 1) both;
  }
  .surface-colours.spreading .surface-colour-drop {
    animation: milk-colour-drop 1050ms cubic-bezier(.12, .78, .18, 1) both;
  }
  .milk-curds ellipse {
    transform-box: fill-box;
    transform-origin: center;
    stroke: color-mix(in srgb, var(--edge) 38%, transparent);
    stroke-width: 0.3;
    filter: drop-shadow(0 0.5px 0.5px color-mix(in srgb, var(--edge) 30%, transparent));
  }
  .milk-curds.forming ellipse {
    animation: curds-form 900ms cubic-bezier(.18, .78, .22, 1) both;
  }
  .milk-curds.forming ellipse:nth-child(3n) { animation-delay: 90ms; }
  .milk-curds.forming ellipse:nth-child(3n + 1) { animation-delay: 180ms; }
  .foam-overflow path {
    fill: none;
    stroke-width: calc(2px + var(--spill) * 4px);
    stroke-linecap: round;
  }
  .foam-overflow {
    animation: foam-spill 1.4s ease-in-out infinite alternate;
  }
  @keyframes foam-rise {
    from { transform: scaleY(0.05); opacity: 0.35; }
    to { transform: scaleY(1); opacity: 1; }
  }
  @keyframes foam-spill {
    from { transform: translateY(0); }
    to { transform: translateY(2px); }
  }
  @keyframes curds-form {
    from { transform: translateY(-10px) scale(0.25); opacity: 0; }
    to { transform: translateY(0) scale(1); opacity: 1; }
  }
  @keyframes surface-spread {
    from { transform: scaleX(0.16); opacity: 0.72; }
    to { transform: scaleX(1); opacity: 1; }
  }
  @keyframes milk-colour-streak {
    from { stroke-dasharray: 0 100; opacity: 0.55; }
    to { stroke-dasharray: 100 0; opacity: 0.82; }
  }
  @keyframes milk-colour-drop {
    from { transform: translateX(calc(50px - var(--spot-x, 50px))) scale(.65); }
    to { transform: translateX(0) scale(1); }
  }
  @keyframes rise {
    from {
      transform: translateY(0);
      opacity: 0.9;
    }
    to {
      transform: translateY(calc(-1 * var(--rise, 60px)));
      opacity: 0;
    }
  }
  .glow {
    fill: var(--hot);
    filter: blur(3px);
  }
  .flame .outer {
    fill: var(--hot);
    animation: flicker var(--flame-duration, 0.4s) ease-in-out infinite alternate;
    transform-origin: 50px 20px;
  }
  .flame .inner {
    fill: var(--warn);
    animation: flicker calc(var(--flame-duration, 0.4s) * 0.82) ease-in-out infinite alternate-reverse;
    transform-origin: 50px 18px;
  }
  @keyframes flicker {
    from {
      transform: scaleY(1) scaleX(1);
      opacity: 0.95;
    }
    to {
      transform: scaleY(1.18) scaleX(0.92);
      opacity: 0.75;
    }
  }
  .steam {
    fill: none;
    stroke: var(--dim);
    stroke-width: 1.4;
    stroke-linecap: round;
    opacity: 0;
    animation: waft 2.4s ease-out infinite;
  }
  @keyframes waft {
    0% {
      opacity: 0;
      transform: translateY(4px);
    }
    25% {
      opacity: var(--steam-opacity, 0.65);
    }
    100% {
      opacity: 0;
      transform: translateY(-14px);
    }
  }
  .frost path {
    stroke: var(--cool);
    stroke-width: 1;
    opacity: 0.8;
  }
  .condensation ellipse {
    fill: color-mix(in srgb, var(--cool) 30%, transparent);
    stroke: color-mix(in srgb, var(--cool) 55%, transparent);
    stroke-width: 0.3;
  }
  /* Steady, not flickering: a body at one temperature glows at one colour.
     No animation here keeps a red-hot crucible free of per-frame work. */
  .incandescence {
    mix-blend-mode: screen;
    filter: blur(1.8px);
  }
  .rolling-boil .boil-bubble {
    fill: color-mix(in srgb, var(--surface) 70%, transparent);
    stroke: color-mix(in srgb, var(--dim) 70%, transparent);
    stroke-width: 0.4;
    animation-name: rise;
    animation-timing-function: linear;
    animation-iteration-count: infinite;
  }
  .rolling-boil .boil-surface {
    fill: none;
    stroke: color-mix(in srgb, var(--surface) 62%, transparent);
    stroke-width: 1.1;
    stroke-linecap: round;
    animation: churn var(--churn, 0.7s) ease-in-out infinite alternate;
  }
  @keyframes churn {
    from { transform: translateY(-0.6px) scaleX(1); }
    to { transform: translateY(0.8px) scaleX(0.97); }
  }
  .falling {
    fill: var(--cloud);
    animation: fall 1.5s ease-in forwards;
  }
  .gravity-settling text { fill: var(--ink); font: 700 5px system-ui, sans-serif; }
  .settling-readout { fill: color-mix(in srgb, var(--surface) 82%, transparent); stroke: var(--edge); stroke-width: .5; }
  .settling-grain { stroke: color-mix(in srgb, var(--edge-strong) 65%, transparent); stroke-width: .4; animation: gravity-settle var(--settle-duration) cubic-bezier(.25,.65,.45,1) both; animation-delay: var(--settle-delay); }
  @keyframes gravity-settle { from { opacity: .9; transform: translateY(0); } to { opacity: .35; transform: translateY(var(--settle-distance)); } }
  .stir-result rect { fill: color-mix(in srgb, var(--surface) 84%, transparent); stroke: var(--instrument); stroke-width: .5; }
  .stir-result text { fill: var(--ink); font: 700 4.5px system-ui, sans-serif; }
  .stir-result .rate-boundary { fill: var(--dim); font-size: 3.7px; }
  .resuspended-particle { stroke: var(--edge-strong); stroke-width: .35; animation: resuspend var(--resuspend-duration) ease-in-out infinite alternate; animation-delay: var(--resuspend-delay); }
  @keyframes resuspend { to { transform: translateY(calc(-1 * var(--resuspend-rise))) translateX(3px); opacity: .45; } }
  @keyframes fall {
    from {
      transform: translateY(0);
      opacity: 1;
    }
    to {
      transform: translateY(var(--fall, 40px));
      opacity: 0.2;
    }
  }
  /* A dissolving grain SHRINKS. The old rule scaled it to 3.5x, which read
     as a puff rather than a crystal going into solution; each grain now has
     its own origin so a whole population can shrink where it stands. */
  .dissolving {
    fill: none;
    stroke: var(--ink);
    stroke-width: 1.2;
    transform-box: fill-box;
    transform-origin: center;
    animation: dissolve 1.3s ease-out forwards;
  }
  @keyframes dissolve {
    from {
      opacity: 0.85;
      transform: scale(1);
    }
    to {
      opacity: 0;
      transform: scale(0.06);
    }
  }
  .headspace {
    fill: var(--cool);
    pointer-events: none;
  }
  .piston-assembly .lid {
    transition: y 0.45s cubic-bezier(0.3, 0.7, 0.35, 1);
  }
  .piston-assembly .piston {
    transition: y1 0.45s cubic-bezier(0.3, 0.7, 0.35, 1), y2 0.45s cubic-bezier(0.3, 0.7, 0.35, 1);
  }
  .shimmer {
    fill: var(--cloud);
    opacity: 0;
    animation: shimmer 1.8s ease-in-out;
  }
  @keyframes shimmer {
    0%,
    100% {
      opacity: 0;
    }
    50% {
      opacity: 0.35;
    }
  }
  .instrument {
    opacity: 0;
    animation: instrument-in 0.4s ease-out forwards;
  }
  @keyframes instrument-in {
    from { opacity: 0; transform: translateY(-6px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .stand {
    stroke: var(--edge-strong);
    stroke-width: 1.4;
  }
  .stand-base {
    stroke: var(--edge-strong);
    stroke-width: 2;
  }
  .clamp {
    stroke: var(--edge-strong);
    stroke-width: 1.6;
  }
  .burette-col {
    fill: none;
    stroke: var(--edge-strong);
    stroke-width: 0.8;
  }
  .burette-fill {
    fill: var(--cool);
    opacity: 0.55;
    transition: y 0.4s ease, height 0.4s ease;
  }
  .burette-tap {
    fill: var(--edge-strong);
  }
  .burette-tip {
    fill: var(--edge-strong);
  }
  .burette-tube {
    stroke: var(--dim);
    stroke-width: 0.6;
    stroke-dasharray: 2 2;
    opacity: 0.5;
  }
  .burette-drop {
    fill: var(--cool);
    animation: drip-fall 0.8s ease-in infinite;
  }
  .therm-stem {
    fill: var(--edge-strong);
    opacity: 0.8;
  }
  .therm-bulb {
    fill: var(--hot, #c44);
    stroke: var(--edge-strong);
    stroke-width: 0.6;
  }
  .therm-mercury {
    fill: var(--hot, #c44);
    opacity: 0.7;
  }
  .probe-stem {
    fill: var(--dim);
    opacity: 0.8;
  }
  .probe-tip {
    fill: var(--cloud);
    stroke: var(--dim);
    stroke-width: 0.6;
  }
  .probe-wire {
    fill: none;
    stroke: var(--dim);
    stroke-width: 0.8;
  }
  .meter-body {
    fill: color-mix(in srgb, var(--instrument) 34%, var(--surface));
    stroke: var(--edge-strong);
    stroke-width: 0.8;
    filter: drop-shadow(0 1px 1px var(--shadow));
  }
  .thermometer-meter { fill: color-mix(in srgb, var(--hot) 20%, var(--surface)); }
  .ph-meter { fill: color-mix(in srgb, var(--discovery) 22%, var(--surface)); }
  .meter-screen {
    fill: color-mix(in srgb, var(--success) 18%, var(--ink));
    stroke: var(--edge-strong);
    stroke-width: 0.45;
  }
  .meter-value,
  .meter-unit {
    fill: var(--surface);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-weight: 800;
    pointer-events: none;
  }
  .meter-value { font-size: 4.5px; }
  .meter-unit { fill: var(--ink); font-size: 3.5px; }
  .balance-inst { opacity: 1; transform-origin: 50px 132px; animation: balance-settle .38s ease-out forwards; }
  .balance-pan { fill: color-mix(in srgb, var(--surface) 72%, var(--edge)); stroke: var(--edge-strong); stroke-width: .7; }
  .balance-neck { fill: color-mix(in srgb, var(--instrument) 16%, var(--surface)); stroke: var(--edge-strong); stroke-width: .7; }
  .balance-body { fill: color-mix(in srgb, var(--instrument) 28%, var(--surface)); stroke: var(--edge-strong); stroke-width: .8; }
  .balance-screen { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); stroke: var(--edge-strong); stroke-width: .4; }
  .balance-value { fill: var(--surface); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 3.1px; font-weight: 850; }
  .balance-key { fill: var(--primary); }
  @keyframes balance-settle { 0% { transform: translateY(3px) scaleY(.96); } 55% { transform: translateY(-1px) scaleY(1.01); } 100% { transform: none; } }
  .pressure-inst { transform-origin: 79px 20px; }
  .gauge-hose { fill: none; stroke: var(--edge-strong); stroke-width: 2; stroke-linecap: round; }
  .gauge-case { fill: var(--surface); stroke: var(--edge-strong); stroke-width: 2; filter: drop-shadow(0 1px 1px var(--shadow)); }
  .gauge-safe, .gauge-warning { fill: none; stroke-width: 2.2; stroke-linecap: round; }
  .gauge-safe { stroke: var(--success); }
  .gauge-warning { stroke: var(--bad); }
  .gauge-tick { stroke: var(--dim); stroke-width: .7; }
  .gauge-needle { stroke: var(--hot); stroke-width: 1.5; stroke-linecap: round; transition: transform .45s cubic-bezier(.2,.8,.2,1); }
  .gauge-pin { fill: var(--hot); stroke: var(--surface); stroke-width: .7; }
  .gauge-readout { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); }
  .gauge-value { fill: var(--surface); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 3.8px; font-weight: 850; }
  .gauge-unit { fill: var(--ink); font-size: 3px; font-weight: 800; }
  .volume-inst { transform-origin: 43px 23px; }
  .syringe-hose { fill: none; stroke: var(--edge-strong); stroke-width: 1.5; stroke-linecap: round; }
  .syringe-barrel { fill: color-mix(in srgb, var(--glass) 78%, transparent); stroke: var(--edge-strong); stroke-width: 1; }
  .syringe-gas { fill: color-mix(in srgb, var(--cool) 18%, var(--surface)); transition: x .45s ease, width .45s ease; }
  .syringe-piston, .syringe-rod, .syringe-handle { stroke: var(--instrument); stroke-width: 1.4; transition: x1 .45s ease, x2 .45s ease; }
  .syringe-tick { stroke: var(--dim); stroke-width: .55; }
  .syringe-screen, .conductivity-screen { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); }
  .syringe-value, .conductivity-value { fill: var(--surface); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 3.5px; font-weight: 850; }
  .conductivity-meter { fill: color-mix(in srgb, var(--cool) 24%, var(--surface)); stroke: var(--edge-strong); stroke-width: .8; filter: drop-shadow(0 1px 1px var(--shadow)); }
  .conductivity-unit { fill: var(--ink); font-size: 3px; font-weight: 800; }
  .conductivity-wire { fill: none; stroke: var(--dim); stroke-width: 1; }
  .conductivity-probe { fill: var(--cool); fill-opacity: var(--conductivity-opacity); stroke: var(--edge-strong); stroke-width: .7; }
  .conductivity-electrode { stroke: var(--action); stroke-width: 1; opacity: calc(.35 + var(--conductivity) * .65); }
  .calorimeter-jacket { fill: color-mix(in srgb, var(--cool) 12%, var(--surface)); fill-opacity: .88; stroke: var(--instrument); stroke-width: 2.2; }
  .calorimeter-lid { fill: none; stroke: var(--instrument); stroke-width: 2.2; stroke-linecap: round; }
  .calorimeter-screen, .uvvis-screen { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); stroke: var(--edge-strong); stroke-width: .45; }
  .calorimeter-value, .uvvis-value { fill: var(--surface); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 3.7px; font-weight: 850; }
  .uvvis-body { fill: color-mix(in srgb, var(--discovery) 20%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1; filter: drop-shadow(0 1px 1px var(--shadow)); }
  .uvvis-lid { fill: color-mix(in srgb, var(--discovery) 12%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1; }
  .uvvis-cuvette { fill: color-mix(in srgb, var(--glass) 72%, transparent); stroke: var(--edge-strong); stroke-width: .7; }
  .uvvis-beam { stroke: var(--beam); stroke-width: 2; stroke-linecap: round; }
  .uvvis-beam.output { opacity: var(--transmittance); }
  .uvvis-wavelength { fill: var(--ink); font-size: 3px; font-weight: 800; }
  .chromatograph-inst { transform-origin: 11px 51px; }
  .chromatograph-body { fill: color-mix(in srgb, var(--discovery) 14%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1; filter: drop-shadow(0 1px 1px var(--shadow)); }
  .chromatograph-column { fill: color-mix(in srgb, var(--glass) 72%, transparent); stroke: var(--instrument); stroke-width: .8; }
  .chromatograph-solvent { fill: color-mix(in srgb, var(--cool) 28%, var(--surface)); stroke: var(--edge-strong); stroke-width: .6; }
  .chromatograph-tube, .chromatograph-flow { fill: none; stroke: var(--instrument); stroke-width: 1.1; stroke-linecap: round; }
  .chromatograph-flow { stroke-dasharray: 2 2; opacity: .55; animation: column-flow .6s linear infinite; }
  .chromatograph-band { opacity: var(--band-opacity); transform-origin: center; animation: band-elute 2.8s cubic-bezier(.22,.7,.25,1) var(--band-delay) both; }
  .chromatograph-band.band-0 { fill: var(--action); }
  .chromatograph-band.band-1 { fill: var(--hot); }
  .chromatograph-band.band-2 { fill: var(--cool); }
  .chromatograph-band.band-3 { fill: var(--success); }
  .chromatograph-band.band-4 { fill: var(--discovery); }
  .chromatograph-screen { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); stroke: var(--edge-strong); stroke-width: .45; }
  .chromatograph-count { fill: var(--surface); font-size: 3.2px; font-weight: 850; }
  .chromatograph-plates { fill: var(--surface); font-size: 2.7px; font-weight: 700; opacity: .82; }
  @keyframes column-flow { to { stroke-dashoffset: -4; } }
  @keyframes band-elute { from { transform: translateY(calc(-1 * var(--band-travel))); opacity: .16; } to { transform: translateY(0); opacity: var(--band-opacity); } }
  .inspection-inst { transform-origin: 77px 39px; animation: inspect-in .38s cubic-bezier(.2,.8,.2,1) both; }
  .inspection-field { fill: color-mix(in srgb, var(--surface) 88%, var(--cool)); }
  .inspection-liquid { opacity: .72; }
  .inspection-meniscus { fill: none; stroke: color-mix(in srgb, var(--surface) 72%, var(--cool)); stroke-width: 1.2; }
  .inspection-speck { fill: var(--edge-strong); }
  .inspection-deposit { stroke: var(--edge-strong); stroke-width: .45; }
  .inspection-bubble { fill: none; stroke: var(--surface); stroke-width: .8; animation: inspection-rise 1.35s ease-out infinite; }
  .magnifier-rim { fill: none; stroke: var(--instrument); stroke-width: 3; filter: drop-shadow(0 2px 2px var(--shadow)); }
  .magnifier-handle { fill: none; stroke: var(--instrument); stroke-width: 6; stroke-linecap: round; filter: drop-shadow(0 2px 2px var(--shadow)); }
  @keyframes inspect-in { from { opacity: 0; transform: scale(.55) rotate(-8deg); } to { opacity: 1; transform: scale(1) rotate(0); } }
  @keyframes inspection-rise { to { opacity: 0; transform: translateY(-17px) scale(1.25); } }
  .geiger-body { fill: color-mix(in srgb, var(--hot) 22%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1; filter: drop-shadow(0 1px 1px var(--shadow)); }
  .geiger-screen { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); stroke: var(--edge-strong); stroke-width: .45; }
  .geiger-value { fill: var(--surface); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 4px; font-weight: 850; }
  .geiger-unit { fill: var(--ink); font-size: 3.2px; font-weight: 850; }
  .geiger-led { fill: var(--danger); animation: geiger-flash var(--count-cadence) step-end infinite; }
  .geiger-wire { fill: none; stroke: var(--edge-strong); stroke-width: 1.2; }
  .geiger-probe { fill: var(--instrument); stroke: var(--edge-strong); stroke-width: 1; }
  .geiger-pulse { fill: none; stroke: var(--danger); stroke-width: .8; transform-origin: 45px 76px; animation: geiger-count var(--count-cadence) ease-out infinite; }
  @keyframes geiger-flash { 0%, 18% { opacity: 1; } 19%, 100% { opacity: .18; } }
  @keyframes geiger-count { from { opacity: .75; transform: scale(.35); } to { opacity: 0; transform: scale(1.3); } }
  .burner-foot { fill: var(--shadow); opacity: .38; }
  .burner-base, .burner-barrel, .burner-collar { fill: color-mix(in srgb, var(--instrument) 34%, var(--edge-strong)); stroke: var(--edge-strong); stroke-width: .8; }
  .burner-hose, .wire-handle, .wire-loop { fill: none; stroke: var(--edge-strong); stroke-width: 1.5; stroke-linecap: round; }
  .wire-loop { stroke-width: 1; }
  .test-flame { stroke: var(--edge-strong); stroke-width: .45; transform-origin: 18px 99px; animation: test-flame var(--test-flame-rate) ease-in-out infinite alternate; }
  .test-flame-core { fill: color-mix(in srgb, var(--surface) 72%, var(--cool)); opacity: .78; }
  @keyframes test-flame { to { transform: scaleX(.82) scaleY(1.08) rotate(1.5deg); } }
  .gas-test-rig { filter: drop-shadow(0 2px 2px var(--shadow)); }
  .test-stick, .forceps, .delivery-tube, .gas-test-tube { fill: none; stroke: var(--edge-strong); stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
  .test-stick { stroke: #8b5a32; stroke-width: 3; }
  .test-flame-small, .relit-flame { fill: var(--hot); stroke: var(--danger); stroke-width: .6; animation: gas-flame .38s ease-in-out infinite alternate; transform-origin: center; }
  .ember { fill: #ff6538; stroke: #6e2b14; stroke-width: .6; }
  .pop-wave { fill: none; stroke: var(--hot); stroke-width: 2; animation: pop-wave .72s ease-out infinite; transform-origin: 54px 17px; }
  .result-word { fill: var(--danger); font: 900 7px system-ui, sans-serif; paint-order: stroke; stroke: var(--surface); stroke-width: 2px; }
  .gas-test-tube { stroke: color-mix(in srgb, var(--cool) 72%, var(--edge-strong)); }
  .limewater-fill { fill: color-mix(in srgb, var(--cool) 10%, var(--surface)); stroke: var(--cool); stroke-width: .7; }
  .limewater-fill.milky { fill: color-mix(in srgb, var(--surface) 86%, var(--cloud)); }
  .lime-particle { fill: var(--cloud); stroke: var(--edge); stroke-width: .35; animation: lime-cloud .7s ease-out both; }
  .litmus-strip { fill: #e85b70; stroke: var(--edge-strong); stroke-width: .6; transition: fill .45s ease; }
  .litmus-strip.changed { fill: #386fe5; }
  .water-drop { fill: var(--cool); opacity: .75; }
  .gas-test-result rect { fill: color-mix(in srgb, var(--surface) 88%, transparent); stroke: var(--edge); stroke-width: .6; }
  .gas-test-result circle { fill: var(--dim); }
  .gas-test-rig.positive .gas-test-result circle { fill: var(--success); }
  .gas-test-result text { fill: var(--ink); font: 800 5px system-ui, sans-serif; }
  @keyframes gas-flame { to { transform: scaleX(.78) scaleY(1.12) rotate(2deg); } }
  @keyframes pop-wave { from { opacity: .9; transform: scale(.35); } to { opacity: 0; transform: scale(1.8); } }
  @keyframes lime-cloud { from { opacity: 0; transform: translateY(6px); } to { opacity: .9; transform: translateY(0); } }
  .waft-hand { fill: color-mix(in srgb, #d79b73 74%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1.1; stroke-linejoin: round; }
  .waft-current { fill: none; stroke: var(--instrument); stroke-width: 1.4; stroke-linecap: round; stroke-dasharray: 4 3; opacity: 0; animation: safe-waft 1.25s ease-out infinite; animation-delay: var(--waft-delay); }
  .waft-result rect { fill: color-mix(in srgb, var(--surface) 90%, transparent); stroke: var(--instrument); stroke-width: .6; }
  .waft-result text { fill: var(--ink); font: 750 4px system-ui, sans-serif; }
  .waft-result .waft-rule { fill: var(--danger); font-size: 3.5px; font-weight: 850; }
  .waft-result .waft-note { fill: var(--instrument); }
  @keyframes safe-waft { 0% { opacity: 0; stroke-dashoffset: 8; } 25% { opacity: var(--waft-strength); } 100% { opacity: 0; stroke-dashoffset: -8; } }
  @media (prefers-reduced-motion: reduce) {
    .instrument {
      animation: none;
      opacity: 1;
    }
    .balance-inst { animation: none; }
    .gauge-needle { transition: none; }
    .syringe-gas, .syringe-piston, .syringe-rod { transition: none; }
    .chromatograph-flow, .chromatograph-band { animation: none; }
    .inspection-inst, .inspection-bubble { animation: none; }
    .geiger-led, .geiger-pulse { animation: none; }
    .test-flame { animation: none; }
    .test-flame-small, .relit-flame, .pop-wave, .lime-particle { animation: none; }
    .waft-current { animation: none; opacity: .55; }
    .glassbtn.pouring { animation: none; }
    .burette-fill,
    .piston-assembly .lid,
    .piston-assembly .piston {
      transition: none;
    }
    .bubble,
    .boil-bubble,
    .boil-surface,
    .foam-state,
    .foam-overflow,
    .milk-curds.forming ellipse,
    .surface-colours.spreading .surface-colour-streak,
    .surface-colours.spreading .surface-colour-drop,
    .flame .outer,
    .flame .inner,
    .steam,
    .falling,
    .settling-grain,
    .resuspended-particle,
    .dissolving,
    .shimmer,
    .stirrer,
    .heat-wave,
    .burst path,
    .burst circle,
    .vessel.bursting {
      animation: none;
    }
    .steam {
      opacity: 0.4;
    }
  }
  .caption {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    justify-content: center;
    font-size: 0.78rem;
  }
  .observation-status {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  .label {
    color: var(--dim);
  }
  .apparatus-status {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.28rem;
    padding: 0.18rem 0.42rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 35%, var(--edge));
    border-radius: 999px;
    color: var(--instrument);
    background: color-mix(in srgb, var(--instrument) 8%, var(--surface));
    font-size: 0.57rem;
    line-height: 1;
    white-space: nowrap;
  }
  .apparatus-status small { color: var(--dim); font-size: 0.52rem; }
  .status-light {
    width: 0.42rem;
    height: 0.42rem;
    border: 1px solid currentColor;
    border-radius: 50%;
    background: var(--surface);
  }
  .apparatus-status.running {
    color: var(--success);
    border-color: color-mix(in srgb, var(--success) 45%, var(--edge));
    background: color-mix(in srgb, var(--success) 9%, var(--surface));
  }
  .apparatus-status.running .status-light {
    background: var(--success);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--success) 16%, transparent);
    animation: status-pulse 0.8s ease-in-out infinite alternate;
  }
  @keyframes status-pulse { to { box-shadow: 0 0 0 6px transparent; } }
  .persistent-readout {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: .35rem;
    padding: .2rem .45rem;
    border: 1px solid color-mix(in srgb, var(--discovery) 38%, var(--edge));
    border-radius: 8px;
    color: var(--discovery);
    background: color-mix(in srgb, var(--discovery) 7%, var(--surface));
  }
  .persistent-readout small { color: var(--dim); font-size: .55rem; }
  .persistent-readout strong { font-variant-numeric: tabular-nums; }
  .badge {
    border: 1px solid var(--edge);
    border-radius: 999px;
    padding: 0 0.5rem;
    background: var(--panel);
    color: inherit;
    font: inherit;
    font-size: inherit;
    cursor: pointer;
    min-height: 26px;
  }
  button.badge:hover {
    border-color: var(--cool);
  }
</style>
