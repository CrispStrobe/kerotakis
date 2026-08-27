<script lang="ts">
  import type { SceneVessel } from "../host/EngineHost";
  import { KINDS, solidLayer, fillHeight, graduationTicks } from "../glassware";
  import FluidOverlay from "./FluidOverlay.svelte";
  import type { FluidSpecies } from "../fluidScene";
  import type { Effect } from "../magnitudes";
  import { i18n, t } from "../i18n.svelte";
  import DeployedApparatus from "./DeployedApparatus.svelte";
  import { APPARATUS } from "../apparatus";

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
    apparatusWorking = false,
    apparatusValues = {},
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
    apparatusWorking?: boolean;
    apparatusValues?: Record<string, number | string>;
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
  const settlingEffect = $derived(latestEffect("settle", 8000));
  const stirEffect = $derived.by(() => {
    const effect = latestEffect("swirl", 8000);
    return effect?.stir ? effect : undefined;
  });
  const gasTestEffect = $derived(latestEffect("gas_test", 4500));
  const waftEffect = $derived(latestEffect("waft", 4200));
  const latestFlameColour = $derived.by(() => {
    const n = effectClock;
    const recent = effects.filter((e) => (e.kind === "ignite" || e.kind === "flame_test") && n - e.at < 3000 && e.flameColour);
    return recent.length > 0 ? recent[recent.length - 1]!.flameColour : undefined;
  });

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
  const shownSolids = $derived(vessel.solids.slice(0, 3));
  // Solids draw as a settled layer; depth follows amount, capped well below
  // the liquid so the layer reads as a deposit rather than a fill.
  const solidH = $derived(
    Math.min(
      18,
      vessel.solids.reduce((sum, s) => sum + s.moles * (s.settled_fraction ?? 1), 0) * 600,
    ),
  );
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
  // State-driven effects, straight from the computed temperature.
  const burning = $derived(vessel.temperature_k > 600 || active("ignite", 3000));
  const steaming = $derived(
    (vessel.liquid !== null && vessel.temperature_k >= 368) || active("evaporate", 2500),
  );
  const frosty = $derived(vessel.temperature_k < 272);
  const hot = $derived(Math.min(1, Math.max(0, (vessel.temperature_k - 310) / 300)));
  const cold = $derived(Math.min(1, Math.max(0, (273.15 - vessel.temperature_k) / 60)));
  const motionMag = $derived(Math.max(mag("swirl", 2200), mag("burst", 1800), mag("heat", 2200), mag("cool", 2200)));
  const frostIntensity = $derived(Math.max(cold, mag("cool", 2200), mag("freeze", 2200)));
  const apparatusOperating = $derived(
    apparatusWorking ||
      (deployedTool === "stir" && active("swirl", 2200)) ||
      (deployedTool === "heat" && active("heat", 2200)) ||
      (deployedTool === "cool" && active("cool", 2200)),
  );
  const apparatusTitle = $derived(
    deployedTool
      ? deployedTool === "burette"
        ? "burette"
        : APPARATUS.find((spec) => spec.verb === deployedTool)?.title ?? deployedTool
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
  class:whirling={active("swirl", 2200)}
  class:apparatus-working={apparatusOperating}
  class:bursting={active("burst", 1800)}
  data-vessel-id={vessel.id}
  style={`--swirl-duration:${2.2 - motionMag * 1.25}s;--stir-duration:${1.15 - motionMag * 0.65}s;--heat-duration:${1.8 - Math.max(hot, mag("heat", 2200)) * 0.8}s;--heat-opacity:${0.25 + Math.max(hot, mag("heat", 2200)) * 0.65};--pour-angle:${9 + mag("pour", 2200) * 23}deg`}
>
  <button
    class="glassbtn"
    class:pouring={active("pour", 2200)}
    aria-label={`${t(vessel.label)} v${vessel.id + 1}: ${t(vessel.words)}${transferTarget ? ` · ${t("transfer target")}` : ""}${apparatusTitle ? ` · ${t("{tool} installed: {state}", { tool: t(apparatusTitle), state: t(apparatusOperating ? "running…" : "ready") })}` : ""}`}
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
    </defs>

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

    {#if vessel.foam && foamH > 0}
      {@const foamY = BOTTOM_Y - liquidH - foamH}
      <g
        class="foam-state"
        class:rising={active("foam", 3000)}
        style={`transform-origin:50px ${BOTTOM_Y - liquidH}px`}
      >
        <rect
          class="foam-fill"
          x={INNER_X}
          y={foamY}
          width={INNER_W}
          height={foamH}
        >
          <title>{t("modeled foam: {height} cm high", { height: vessel.foam.height_cm.toFixed(1) })}</title>
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

    {#if solidH > 0}
      {#each shownSolids as solid, i (solid.species)}
        {@const layer = solidLayer(i, shownSolids.length, solidH, BOTTOM_Y)}
        <rect
          x={INNER_X}
          y={layer.y}
          width={INNER_W}
          height={layer.h}
          fill={rgb(solid.srgb)}
          class:metallic={solid.metallic}
        >
          <title>{t(solid.colour_word)} {t(solid.name)}</title>
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

    {#if fluidLookup}
      <FluidOverlay {vessel} {effects} lookup={fluidLookup} />
    {/if}
    </g>

    {#if vessel.foam && foamOverflow > 0}
      {@const spillScale = Math.min(1, foamOverflow / Math.max(0.01, FULL_AT_L))}
      <g class="foam-overflow" aria-hidden="true" style={`--spill:${spillScale}`}>
        <ellipse cx="50" cy="7" rx={12 + spillScale * 13} ry={3 + spillScale * 3} />
        <path d={`M ${38 - spillScale * 4} 8 Q ${28 - spillScale * 8} ${18 + spillScale * 8} ${30 - spillScale * 9} ${38 + spillScale * 30}`} />
        <path d={`M ${62 + spillScale * 4} 8 Q ${72 + spillScale * 8} ${18 + spillScale * 8} ${70 + spillScale * 9} ${38 + spillScale * 30}`} />
      </g>
    {/if}

    {#if deployedTool}
      <DeployedApparatus tool={deployedTool} working={apparatusOperating} values={apparatusValues} effect={deployedTool === "stir" ? stirEffect : undefined} surfaceY={BOTTOM_Y - Math.max(liquidH, 4)} />
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
    {#if burning}
      {@const flameMagnitude = mag("ignite", 3000)}
      {@const flameScale = 0.42 + flameMagnitude * 0.88}
      {@const flameDuration = 0.48 - flameMagnitude * 0.25}
      <g class="flame" aria-hidden="true" style={`--flame-duration:${flameDuration}s;transform-origin:50px 20px;transform:scale(${flameScale})`}>
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
        <title>{gasTest.notes}</title>
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
    {#if steaming}
      {@const steamMag = mag("evaporate", 2500)}
      {@const steamOpacity = 0.3 + steamMag * 0.7}
      {#each [34, 50, 66] as x, i (x)}
        <path
          class="steam"
          d={`M ${x} ${BOTTOM_Y - liquidH - 4} q 3 -6 0 -12 q -3 -6 0 -12`}
          style={`animation-delay:${i * 0.5}s;--steam-opacity:${steamOpacity}`}
        />
      {/each}
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

    <!-- Event-driven transients (GUI-026): each fires only because the
         engine emitted the matching event. -->
    {#if active("precipitate", 1800) && liquidH > 0}
      {@const pMag = mag("precipitate", 1800)}
      {@const pCount = Math.max(2, Math.round(2 + pMag * 6))}
      {@const pRadius = 1.2 + pMag * 1.2}
      {#each Array.from({length: pCount}, (_, i) => INNER_X + 4 + (i / (pCount - 1)) * (INNER_W - 8)) as x, i (i)}
        <circle
          class="falling"
          cx={x}
          cy={BOTTOM_Y - liquidH + 6}
          r={pRadius}
          style={`--fall:${Math.max(8, liquidH - 10)}px; animation-delay:${i * 0.12}s`}
        />
      {/each}
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
    {#if active("dissolve", 1400) && liquidH > 0}
      <circle class="dissolving" cx="50" cy={BOTTOM_Y - 10} r="4" />
    {/if}
    {#if active("electrolyse", 3500) && liquidH > 0}
      {@const eMag = mag("electrolyse", 3500)}
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
      {@const bMag = mag("vent", 2600)}
      {@const bCount = Math.max(2, Math.round(2 + bMag * 4))}
      {@const bRadius = 1.6 + bMag * 1.4}
      {#each Array.from({length: bCount}, (_, i) => INNER_X + 6 + (i / Math.max(1, bCount - 1)) * (INNER_W - 12)) as x, i (i)}
        <circle
          class="bubble"
          cx={x}
          cy={BOTTOM_Y - 4}
          r={bRadius}
          style={`--rise:${liquidH - 8}px; animation-delay:${i * 0.35}s`}
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
    {#if vessel.boundary === "sealed"}
      <rect class="lid" x="10" y="9" width="80" height="5" rx="2">
        <title>{t("sealed")}</title>
      </rect>
    {:else if vessel.boundary === "pressure_controlled"}
      <!-- A floating piston: the lid that moves to hold the set pressure. -->
      <rect class="lid" x="14" y="16" width="72" height="4" rx="1">
        <title>{t("pressure-controlled")}</title>
      </rect>
      <line class="piston" x1="50" y1="4" x2="50" y2="16" />
      <line class="piston" x1="42" y1="4" x2="58" y2="4" />
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
  </button>

  {#if dropReady}<span class="drop-hint">{t("add here")}</span>{/if}

  <figcaption class="caption">
    <span class="label">{t(vessel.label)} v{vessel.id + 1}</span>
    {#if apparatusTitle}
      <span
        class="apparatus-status"
        class:running={apparatusOperating}
        title={t("{tool} installed: {state}", { tool: t(apparatusTitle), state: t(apparatusOperating ? "running…" : "ready") })}
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
          {badge.key === "ph" ? "pH" : t(badge.key)}
          {badge.value.toFixed(2)}
        </button>
      {/each}
      {#if sealed}<span class="badge">{t(vessel.boundary)}</span>{/if}
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
    fill: color-mix(in srgb, white 88%, var(--instrument));
    stroke: color-mix(in srgb, var(--instrument) 42%, var(--edge));
    stroke-width: 0.55;
  }
  .foam-state.rising {
    animation: foam-rise 900ms cubic-bezier(.2, .8, .25, 1) both;
  }
  .foam-cell {
    fill: color-mix(in srgb, white 30%, transparent);
    stroke: color-mix(in srgb, var(--instrument) 48%, var(--edge));
    stroke-width: 0.45;
  }
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
  .dissolving {
    fill: none;
    stroke: var(--ink);
    stroke-width: 1.2;
    animation: dissolve 1.3s ease-out forwards;
    transform-origin: 50px 112px;
  }
  @keyframes dissolve {
    from {
      opacity: 0.8;
      transform: scale(1);
    }
    to {
      opacity: 0;
      transform: scale(3.5);
    }
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
    .burette-fill {
      transition: none;
    }
    .bubble,
    .foam-state,
    .foam-overflow,
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
