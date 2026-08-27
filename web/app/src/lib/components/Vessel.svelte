<script lang="ts">
  import type { SceneVessel } from "../host/EngineHost";
  import { KINDS, solidLayer, fillHeight, graduationTicks } from "../glassware";
  import FluidOverlay from "./FluidOverlay.svelte";
  import type { FluidSpecies } from "../fluidScene";
  import type { Effect } from "../magnitudes";
  import { t } from "../i18n.svelte";
  import DeployedApparatus from "./DeployedApparatus.svelte";

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
  const active = (kind: string, withinMs: number) =>
    effects.some((e) => e.kind === kind && now() - e.at < withinMs);
  // GUI-059: magnitude of the most recent active effect of a given kind.
  const mag = (kind: string, withinMs: number) => {
    const n = now();
    const recent = effects.filter((e) => e.kind === kind && n - e.at < withinMs);
    return recent.length > 0 ? (recent[recent.length - 1]!.magnitude ?? 1) : 0;
  };
  const latestFlameColour = $derived.by(() => {
    const n = now();
    const recent = effects.filter((e) => e.kind === "ignite" && n - e.at < 3000 && e.flameColour);
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
      vessel.solids.reduce((sum, s) => sum + s.moles, 0) * 600,
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
  class:bursting={active("burst", 1800)}
  data-vessel-id={vessel.id}
  style={`--swirl-duration:${2.2 - motionMag * 1.25}s;--stir-duration:${1.15 - motionMag * 0.65}s;--heat-duration:${1.8 - Math.max(hot, mag("heat", 2200)) * 0.8}s;--heat-opacity:${0.25 + Math.max(hot, mag("heat", 2200)) * 0.65}`}
>
  <button
    class="glassbtn"
    aria-label={`${t(vessel.label)} v${vessel.id + 1}: ${t(vessel.words)}${transferTarget ? ` · ${t("transfer target")}` : ""}`}
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
      <DeployedApparatus tool={deployedTool} working={apparatusWorking} values={apparatusValues} surfaceY={BOTTOM_Y - Math.max(liquidH, 4)} />
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
      <ellipse
        class="swirl"
        cx="50"
        cy={BOTTOM_Y - liquidH / 2}
        rx={(INNER_W / 2 - 6) * sScale}
        ry={Math.min(8, liquidH / 3) * sScale}
      />
      <g class="stirrer" aria-hidden="true">
        <line x1="50" y1={Math.max(7, BOTTOM_Y - liquidH - 18)} x2="50" y2={BOTTOM_Y - 7} />
        <ellipse cx="50" cy={BOTTOM_Y - 6} rx="8" ry="2" />
      </g>
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
    {#if active("thermometer", 2500)}
      {@const tipY = BOTTOM_Y - Math.max(liquidH * 0.5, 10)}
      <g class="instrument thermometer-inst" aria-label={t("thermometer")}>
        <rect class="therm-stem" x="32" y="4" width="2" height={tipY - 4} rx="0.8" />
        <ellipse class="therm-bulb" cx="33" cy={tipY} rx="2.8" ry="3.2" />
        <rect class="therm-mercury" x="32.3" y={Math.max(tipY - 18, 10)} width="1.4" height={Math.min(18, tipY - 10)} rx="0.5" />
      </g>
    {/if}
    {#if active("ph_probe", 2500)}
      {@const tipY = BOTTOM_Y - Math.max(liquidH * 0.5, 10)}
      <g class="instrument ph-inst" aria-label={t("pH probe")}>
        <rect class="probe-stem" x="64" y="4" width="2" height={tipY - 4} rx="0.8" />
        <ellipse class="probe-tip" cx="65" cy={tipY} rx="2.2" ry="3.6" />
        <line class="probe-wire" x1="65" y1="4" x2="72" y2="-2" />
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
    transform-origin: 50px 70px;
    animation: stir-tool var(--stir-duration, 1s) ease-in-out infinite alternate;
  }
  .stirrer line, .stirrer ellipse { fill: none; stroke: var(--edge-strong); stroke-width: 1.5; }
  @keyframes stir-tool { to { transform: rotate(12deg) translateX(3px); } }
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
    stroke: var(--dim);
    stroke-width: 0.8;
  }
  @media (prefers-reduced-motion: reduce) {
    .instrument {
      animation: none;
      opacity: 1;
    }
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
