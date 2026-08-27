<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { Effect } from "../magnitudes";
  import { t } from "../i18n.svelte";

  let { effect: benchEffect, layoutKey = "" }: { effect: Effect; layoutKey?: string } = $props();
  let marker: HTMLSpanElement;
  let path = $state("");
  let midpoint = $state({ x: 0, y: 0 });
  let visible = $state(true);
  let duration = $derived(
    `${Math.max(0.45, (benchEffect.durationMs ?? (1450 - benchEffect.magnitude * 650)) / 1000)}s`,
  );
  const residueMoles = $derived(
    benchEffect.filterResidue?.reduce((sum, solid) => sum + solid.moles, 0) ?? 0,
  );
  const dominantResidue = $derived(
    benchEffect.filterResidue?.reduce((dominant, solid) =>
      !dominant || solid.moles > dominant.moles ? solid : dominant, benchEffect.filterResidue[0]),
  );
  const startC = $derived((benchEffect.distillation?.startK ?? 273.15) - 273.15);
  const endC = $derived((benchEffect.distillation?.endK ?? benchEffect.distillation?.startK ?? 273.15) - 273.15);
  const stageMarks = $derived(
    Array.from({ length: Math.min(8, Math.max(1, benchEffect.distillation?.stages ?? 1)) }),
  );
  const magneticMoles = $derived(
    benchEffect.magnetic?.attracted.reduce((sum, solid) => sum + solid.moles, 0) ?? 0,
  );

  function position() {
    const bench = marker?.closest<HTMLElement>(".bench");
    const sourceVessel = bench?.querySelector<HTMLElement>(`[data-vessel-id="${benchEffect.source}"]`);
    const targetVessel = bench?.querySelector<HTMLElement>(`[data-vessel-id="${benchEffect.target}"]`);
    const source = sourceVessel?.closest<HTMLElement>(".vessel-position")?.querySelector<HTMLElement>('[data-port="out"]') ?? sourceVessel;
    const target = targetVessel?.closest<HTMLElement>(".vessel-position")?.querySelector<HTMLElement>('[data-port="in"]') ?? targetVessel;
    if (!bench || !source || !target) return;
    const b = bench.getBoundingClientRect();
    const a = source.getBoundingClientRect();
    const z = target.getBoundingClientRect();
    const x1 = a.left + a.width / 2 - b.left + bench.scrollLeft;
    const y1 = a.top + a.height * 0.34 - b.top + bench.scrollTop;
    const x2 = z.left + z.width / 2 - b.left + bench.scrollLeft;
    const y2 = z.top + z.height * 0.42 - b.top + bench.scrollTop;
    const lift = Math.max(36, Math.abs(x2 - x1) * 0.22);
    midpoint = { x: (x1 + x2) / 2, y: Math.min(y1, y2) - lift };
    path = `M ${x1} ${y1} Q ${midpoint.x} ${midpoint.y} ${x2} ${y2}`;
  }

  $effect(() => {
    // Reading this key makes the SVG reconnect whenever either vessel moves.
    layoutKey;
    void tick().then(position);
  });

  onMount(() => {
    void tick().then(position);
    window.addEventListener("resize", position);
    const bench = marker?.closest<HTMLElement>(".bench");
    const observer = bench && typeof ResizeObserver !== "undefined"
      ? new ResizeObserver(position)
      : null;
    if (bench) observer?.observe(bench);
    const remaining = Math.max(0, (benchEffect.durationMs ?? 4000) - (Date.now() - benchEffect.at));
    const expiry = window.setTimeout(() => (visible = false), remaining);
    return () => {
      window.removeEventListener("resize", position);
      observer?.disconnect();
      window.clearTimeout(expiry);
    };
  });
</script>

<span class="marker" bind:this={marker}></span>
{#if path && visible}
  <svg class="bench-effect" aria-hidden="true" style={`--fluid:${benchEffect.fluidColour ?? "var(--cool)"}`}>
    {#if benchEffect.operation === "cell"}
      <path class="cable positive" d={path} />
      <path class="cable negative" d={path} transform="translate(0 8)" />
      <g class="meter" transform={`translate(${midpoint.x - 22} ${midpoint.y - 12})`}>
        <rect width="44" height="25" rx="7" />
        <path d="M 12 18 Q 22 5 32 18" /><circle cx="22" cy="17" r="2" />
      </g>
    {:else}
      <path class="rig-line" d={path} />
      {#if benchEffect.operation === "magnet"}
        <g class="magnet-tool" transform={`translate(${midpoint.x - 16} ${midpoint.y - 18})`}>
          <path class="magnet-body" d="M 3 0 V 13 A 13 13 0 0 0 29 13 V 0 H 20 V 13 A 4 4 0 0 1 12 13 V 0 Z" />
          <path class="magnet-pole north" d="M 3 0 H 12 V 6 H 3 Z" />
          <path class="magnet-pole south" d="M 20 0 H 29 V 6 H 20 Z" />
          <text x="7.5" y="4.6" text-anchor="middle">N</text>
          <text x="24.5" y="4.6" text-anchor="middle">S</text>
          {#if magneticMoles > 0}
            <text class="magnetic-reading" x="16" y="32" text-anchor="middle">{(magneticMoles * 1000).toPrecision(2)} mmol</text>
          {:else}
            <text class="magnetic-reading empty" x="16" y="32" text-anchor="middle">{t("no magnetic solid")}</text>
          {/if}
        </g>
        {#each benchEffect.magnetic?.attracted ?? [] as solid, speciesIndex (solid.species)}
          {#each [0, 1, 2] as grain (grain)}
            <circle
              class="magnetic-grain"
              r={1.2 + benchEffect.magnitude * 1.4}
              fill={solid.colour}
              style={`--magnet-delay:${speciesIndex * .12 + grain * .09}s`}
            >
              <animateMotion dur={`${1.25 - benchEffect.magnitude * .45}s`} begin={`${speciesIndex * .12 + grain * .09}s`} path={path} fill="freeze" />
            </circle>
          {/each}
        {/each}
      {:else if benchEffect.operation === "filter"}
        <g
          class="filter"
          class:loaded={residueMoles > 0}
          transform={`translate(${midpoint.x - 13} ${midpoint.y - 12})`}
          style={`--residue:${dominantResidue?.colour ?? "var(--cloud)"};--residue-load:${Math.max(.12, benchEffect.magnitude)}`}
        >
          <path class="stand" d="M 27 -8 V 30 M 23 30 H 31 M 18 2 H 27" />
          <path d="M 0 0 H 26 L 16 14 V 25 H 10 V 14 Z" />
          <path class="filter-paper" d="M 3 3 H 23 L 15 13 H 11 Z" />
          {#if residueMoles > 0}
            <path class="residue" d="M 4 4 H 22 L 19 7 Q 13 10 7 7 Z" />
            {#each [[8, 5], [12, 6.5], [16, 5], [19, 6.5], [10, 8], [15, 8]] as dot, i (i)}
              <circle class="residue-grain" cx={dot[0]} cy={dot[1]} r={0.45 + benchEffect.magnitude * .35} style={`--grain-delay:${i * .08}s`} />
            {/each}
            <text class="filter-reading" x="13" y="-3" text-anchor="middle">{(residueMoles * 1000).toPrecision(2)} mmol</text>
          {/if}
        </g>
      {:else if benchEffect.operation === "distil"}
        <g
          class="condenser"
          class:azeotropic={benchEffect.distillation?.azeotropic}
          transform={`translate(${midpoint.x - 31} ${midpoint.y - 16})`}
          style={`--condense-rate:${Math.max(.4, 1.3 - benchEffect.magnitude * .75)}s`}
        >
          <path class="column" d="M 4 28 V 5 H 13 V 28" />
          {#each stageMarks as _, i (i)}
            <path class="stage" d={`M 5 ${8 + i * (17 / Math.max(1, stageMarks.length - 1))} H 12`} />
          {/each}
          <path class="thermometer" d="M 8 5 V 0 H 17" />
          <circle class="thermometer-bulb" cx="8" cy="5" r="2" />
          <rect class="jacket" x="13" y="8" width="48" height="18" rx="8" />
          <path class="vapour-tube" d="M 13 13 H 57" />
          <path class="coolant" d="M 18 11 L 25 23 M 30 11 L 37 23 M 42 11 L 49 23" />
          <path class="coolant-port" d="M 18 8 V 3 M 56 26 V 31" />
          <circle class="condensate-drop" cx="58" cy="14" r="1.7" />
          <text class="temperature" x="31" y="4" text-anchor="middle">{startC.toFixed(1)}–{endC.toFixed(1)} °C</text>
          <text class="stages" x="8" y="35" text-anchor="middle">×{benchEffect.distillation?.stages ?? 1}</text>
          {#if (benchEffect.distillation?.energyKj ?? 0) > 0}
            <text class="energy" x="39" y="35" text-anchor="middle">{benchEffect.distillation!.energyKj.toFixed(1)} kJ</text>
          {/if}
          {#if benchEffect.distillation?.azeotropic}
            <text class="azeotrope" x="39" y="43" text-anchor="middle">{t("azeotrope")}</text>
          {/if}
        </g>
      {:else if benchEffect.operation === "drain"}
        <g
          class="separator"
          transform={`translate(${midpoint.x - 15} ${midpoint.y - 15})`}
          style={`--lower:${benchEffect.drain?.lowerColour ?? benchEffect.fluidColour ?? "var(--cool)"};--upper:${benchEffect.drain?.upperColour ?? "color-mix(in srgb, var(--cool) 20%, white)"};--drain-rate:${Math.max(.42, 1.25 - benchEffect.magnitude * .7)}s`}
        >
          <path class="separator-stand" d="M 29 -7 V 38 M 24 38 H 34 M 20 4 H 29" />
          <path class="separator-glass" d="M 7 0 H 19 L 21 5 Q 26 16 13 28 Q 0 16 5 5 Z M 13 28 V 34" />
          <path class="upper-layer" d="M 4.5 8 Q 13 10 21.5 8 Q 23 16 18 21 H 8 Q 3 16 4.5 8 Z" />
          <path class="lower-layer" d="M 8 21 H 18 Q 16 25 13 28 Q 10 25 8 21 Z" />
          <path class="stopcock" d="M 8 32 H 18 M 13 30 V 35" />
          <path class="drain-jet" d="M 13 35 V 44" />
          <text x="13" y="-4" text-anchor="middle">{t(benchEffect.drain?.solvent ?? "")}</text>
          <text x="13" y="50" text-anchor="middle">{((benchEffect.drain?.moles ?? 0) * 1000).toPrecision(2)} mmol</text>
        </g>
      {/if}
      {#if benchEffect.operation !== "magnet"}
        <path class="pour-glow" d={path} pathLength="1" style={`--duration:${duration};--stream:${2 + benchEffect.magnitude * 5}px`} />
        <path class="pour-stream" d={path} pathLength="1" style={`--duration:${duration};--stream:${1 + benchEffect.magnitude * 2.5}px`} />
        <circle class="landing" r={6 + benchEffect.magnitude * 8}>
          <animateMotion dur={duration} path={path} fill="freeze" />
        </circle>
      {/if}
    {/if}
  </svg>
{/if}

<style>
  .marker { position: absolute; inset: 0; pointer-events: none; }
  .bench-effect { position: absolute; inset: 0; z-index: 5; width: 100%; height: 100%; overflow: visible; pointer-events: none; }
  .pour-glow, .pour-stream { fill: none; stroke-linecap: round; stroke-dasharray: 1; stroke-dashoffset: 1; animation: pour var(--duration) cubic-bezier(.2,.7,.25,1) forwards; }
  .pour-glow { stroke: color-mix(in srgb, var(--fluid) 65%, white); stroke-width: var(--stream); opacity: 0.32; filter: blur(3px); }
  .pour-stream { stroke: var(--fluid); stroke-width: var(--stream); opacity: 0.78; }
  .landing { fill: none; stroke: var(--fluid); stroke-width: 2; opacity: 0; animation: land var(--duration) ease-out forwards; }
  .rig-line { fill: none; stroke: var(--edge-strong); stroke-width: 5; stroke-linecap: round; opacity: .75; }
  .filter path { fill: color-mix(in srgb, var(--surface) 86%, var(--cool)); stroke: var(--edge-strong); stroke-width: 1.5; }
  .filter .stand { fill: none; stroke: var(--edge-strong); stroke-width: 1.8; stroke-linecap: round; }
  .filter .filter-paper { fill: var(--cloud); stroke-width: 1; }
  .filter .residue { fill: var(--residue); fill-opacity: calc(.38 + var(--residue-load) * .55); stroke: color-mix(in srgb, var(--residue) 80%, var(--edge-strong)); stroke-width: .5; }
  .filter .residue-grain { fill: var(--residue); opacity: calc(.45 + var(--residue-load) * .45); }
  .filter.loaded .residue-grain { animation: settle-grain .55s ease-out both; animation-delay: var(--grain-delay); }
  .magnet-body { fill: color-mix(in srgb, var(--surface) 82%, var(--edge)); stroke: var(--edge-strong); stroke-width: 1.6; }
  .magnet-pole { stroke: var(--edge-strong); stroke-width: .6; }
  .magnet-pole.north { fill: var(--danger); }
  .magnet-pole.south { fill: var(--primary); }
  .magnet-tool text { fill: white; font: 800 4px system-ui, sans-serif; }
  .magnet-tool .magnetic-reading { fill: var(--ink); font-size: 5px; paint-order: stroke; stroke: var(--surface); stroke-width: 2px; }
  .magnet-tool .magnetic-reading.empty { fill: var(--muted); }
  .magnetic-grain { stroke: color-mix(in srgb, currentColor 70%, black); stroke-width: .5; opacity: 0; animation: magnetic-arrive .7s ease-out forwards; animation-delay: var(--magnet-delay); }
  .condenser .jacket { fill: color-mix(in srgb, var(--cool) 15%, var(--surface)); stroke: var(--edge-strong); stroke-width: 2; }
  .condenser path { fill: none; stroke-linecap: round; stroke-linejoin: round; }
  .condenser .column, .condenser .thermometer, .condenser .vapour-tube { stroke: var(--edge-strong); stroke-width: 1.6; }
  .condenser .stage { stroke: color-mix(in srgb, var(--instrument) 75%, var(--edge-strong)); stroke-width: .8; }
  .condenser .coolant, .condenser .coolant-port { stroke: var(--cool); stroke-width: 1.4; }
  .condenser .thermometer-bulb { fill: var(--hot); stroke: var(--edge-strong); stroke-width: .5; }
  .condenser text { fill: var(--ink); font: 700 5px system-ui, sans-serif; paint-order: stroke; stroke: var(--surface); stroke-width: 2px; }
  .condenser .azeotrope { fill: var(--hot); }
  .condensate-drop { fill: color-mix(in srgb, var(--cool) 35%, white); stroke: var(--cool); stroke-width: .5; animation: condenser-drop var(--condense-rate) ease-in infinite; }
  .separator-stand, .separator-glass, .separator .stopcock { fill: none; stroke: var(--edge-strong); stroke-linecap: round; stroke-linejoin: round; }
  .separator-stand { stroke-width: 1.8; }
  .separator-glass { stroke-width: 1.5; }
  .separator .stopcock { stroke-width: 1.7; transform-origin: 13px 32px; animation: stopcock-open .55s ease-out both; }
  .separator .upper-layer { fill: var(--upper); fill-opacity: .72; stroke: none; }
  .separator .lower-layer { fill: var(--lower); fill-opacity: .85; stroke: none; }
  .separator .drain-jet { fill: none; stroke: var(--lower); stroke-width: calc(1px + var(--stream, 1px)); stroke-linecap: round; stroke-dasharray: 4 2; animation: drain-jet var(--drain-rate) linear infinite; }
  .separator text { fill: var(--ink); font: 700 5px system-ui, sans-serif; paint-order: stroke; stroke: var(--surface); stroke-width: 2px; }
  .cable { fill: none; stroke-width: 4; stroke-linecap: round; stroke-dasharray: 10 5; animation: current 1s linear infinite; }
  .cable.positive { stroke: var(--danger); } .cable.negative { stroke: var(--primary); animation-direction: reverse; }
  .meter rect { fill: var(--surface); stroke: var(--instrument); stroke-width: 2; }
  .meter path { fill: none; stroke: var(--instrument); stroke-width: 1.5; }
  .meter circle { fill: var(--hot); }
  @keyframes pour { 0% { stroke-dashoffset: 1; opacity: 0; } 12% { opacity: 0.85; } 58% { stroke-dashoffset: 0; } 82% { opacity: 0.75; } 100% { stroke-dashoffset: -1; opacity: 0; } }
  @keyframes land { 0%, 58% { opacity: 0; } 72% { opacity: 0.7; } 100% { opacity: 0; } }
  @keyframes current { to { stroke-dashoffset: -30; } }
  @keyframes condenser-drop { from { opacity: 0; transform: translate(-4px, -2px); } 35% { opacity: 1; } to { opacity: 0; transform: translate(5px, 13px); } }
  @keyframes stopcock-open { from { transform: rotate(90deg); } }
  @keyframes drain-jet { to { stroke-dashoffset: -12; } }
  @keyframes settle-grain { from { opacity: 0; transform: translateY(-4px); } }
  @keyframes magnetic-arrive { 0% { opacity: 0; } 20%, 85% { opacity: 1; } 100% { opacity: .25; } }
  @media (prefers-reduced-motion: reduce) { .bench-effect { display: none; } }
</style>
