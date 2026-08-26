<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { Effect } from "../magnitudes";

  let { effect }: { effect: Effect } = $props();
  let marker: HTMLSpanElement;
  let path = $state("");
  let midpoint = $state({ x: 0, y: 0 });
  let visible = $state(true);
  let duration = $derived(`${1.45 - effect.magnitude * 0.65}s`);

  function position() {
    const bench = marker?.closest<HTMLElement>(".bench");
    const source = bench?.querySelector<HTMLElement>(`[data-vessel-id="${effect.source}"]`);
    const target = bench?.querySelector<HTMLElement>(`[data-vessel-id="${effect.target}"]`);
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

  onMount(() => {
    void tick().then(position);
    window.addEventListener("resize", position);
    const expiry = window.setTimeout(() => (visible = false), 3500);
    return () => {
      window.removeEventListener("resize", position);
      window.clearTimeout(expiry);
    };
  });
</script>

<span class="marker" bind:this={marker}></span>
{#if path && visible}
  <svg class="bench-effect" aria-hidden="true">
    {#if effect.operation === "cell"}
      <path class="cable positive" d={path} />
      <path class="cable negative" d={path} transform="translate(0 8)" />
      <g class="meter" transform={`translate(${midpoint.x - 22} ${midpoint.y - 12})`}>
        <rect width="44" height="25" rx="7" />
        <path d="M 12 18 Q 22 5 32 18" /><circle cx="22" cy="17" r="2" />
      </g>
    {:else}
      <path class="rig-line" d={path} />
      {#if effect.operation === "filter"}
        <g class="filter" transform={`translate(${midpoint.x - 13} ${midpoint.y - 12})`}>
          <path d="M 0 0 H 26 L 16 14 V 25 H 10 V 14 Z" />
          <path class="filter-paper" d="M 3 3 H 23 L 15 13 H 11 Z" />
        </g>
      {:else if effect.operation === "distil"}
        <g class="condenser" transform={`translate(${midpoint.x - 24} ${midpoint.y - 9})`}>
          <rect width="48" height="18" rx="8" />
          <path d="M 8 4 L 15 14 M 21 4 L 28 14 M 34 4 L 41 14" />
        </g>
      {/if}
      <path class="pour-glow" d={path} pathLength="1" style={`--duration:${duration};--stream:${2 + effect.magnitude * 5}px`} />
      <path class="pour-stream" d={path} pathLength="1" style={`--duration:${duration};--stream:${1 + effect.magnitude * 2.5}px`} />
      <circle class="landing" r={6 + effect.magnitude * 8}>
        <animateMotion dur={duration} path={path} fill="freeze" />
      </circle>
    {/if}
  </svg>
{/if}

<style>
  .marker { position: absolute; inset: 0; pointer-events: none; }
  .bench-effect { position: absolute; inset: 0; z-index: 5; width: 100%; height: 100%; overflow: visible; pointer-events: none; }
  .pour-glow, .pour-stream { fill: none; stroke-linecap: round; stroke-dasharray: 1; stroke-dashoffset: 1; animation: pour var(--duration) cubic-bezier(.2,.7,.25,1) forwards; }
  .pour-glow { stroke: color-mix(in srgb, var(--cool) 35%, white); stroke-width: var(--stream); opacity: 0.32; filter: blur(3px); }
  .pour-stream { stroke: var(--cool); stroke-width: var(--stream); opacity: 0.78; }
  .landing { fill: none; stroke: var(--cool); stroke-width: 2; opacity: 0; animation: land var(--duration) ease-out forwards; }
  .rig-line { fill: none; stroke: var(--edge-strong); stroke-width: 5; stroke-linecap: round; opacity: .75; }
  .filter path { fill: color-mix(in srgb, var(--surface) 86%, var(--cool)); stroke: var(--edge-strong); stroke-width: 1.5; }
  .filter .filter-paper { fill: var(--cloud); stroke-width: 1; }
  .condenser rect { fill: color-mix(in srgb, var(--cool) 15%, var(--surface)); stroke: var(--edge-strong); stroke-width: 2; }
  .condenser path { fill: none; stroke: var(--cool); stroke-width: 1.5; }
  .cable { fill: none; stroke-width: 4; stroke-linecap: round; stroke-dasharray: 10 5; animation: current 1s linear infinite; }
  .cable.positive { stroke: var(--danger); } .cable.negative { stroke: var(--primary); animation-direction: reverse; }
  .meter rect { fill: var(--surface); stroke: var(--instrument); stroke-width: 2; }
  .meter path { fill: none; stroke: var(--instrument); stroke-width: 1.5; }
  .meter circle { fill: var(--hot); }
  @keyframes pour { 0% { stroke-dashoffset: 1; opacity: 0; } 12% { opacity: 0.85; } 58% { stroke-dashoffset: 0; } 82% { opacity: 0.75; } 100% { stroke-dashoffset: -1; opacity: 0; } }
  @keyframes land { 0%, 58% { opacity: 0; } 72% { opacity: 0.7; } 100% { opacity: 0; } }
  @keyframes current { to { stroke-dashoffset: -30; } }
  @media (prefers-reduced-motion: reduce) { .bench-effect { display: none; } }
</style>
