<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { Effect } from "../magnitudes";

  let { effect }: { effect: Effect } = $props();
  let marker: HTMLSpanElement;
  let path = $state("");
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
    path = `M ${x1} ${y1} Q ${(x1 + x2) / 2} ${Math.min(y1, y2) - lift} ${x2} ${y2}`;
  }

  onMount(() => {
    void tick().then(position);
    window.addEventListener("resize", position);
    return () => window.removeEventListener("resize", position);
  });
</script>

<span class="marker" bind:this={marker}></span>
{#if path}
  <svg class="bench-effect" aria-hidden="true">
    <path class="pour-glow" d={path} pathLength="1" style={`--duration:${duration};--stream:${2 + effect.magnitude * 5}px`} />
    <path class="pour-stream" d={path} pathLength="1" style={`--duration:${duration};--stream:${1 + effect.magnitude * 2.5}px`} />
    <circle class="landing" r={6 + effect.magnitude * 8}>
      <animateMotion dur={duration} path={path} fill="freeze" />
    </circle>
  </svg>
{/if}

<style>
  .marker { position: absolute; inset: 0; pointer-events: none; }
  .bench-effect { position: absolute; inset: 0; z-index: 5; width: 100%; height: 100%; overflow: visible; pointer-events: none; }
  .pour-glow, .pour-stream { fill: none; stroke-linecap: round; stroke-dasharray: 1; stroke-dashoffset: 1; animation: pour var(--duration) cubic-bezier(.2,.7,.25,1) forwards; }
  .pour-glow { stroke: color-mix(in srgb, var(--cool) 35%, white); stroke-width: var(--stream); opacity: 0.32; filter: blur(3px); }
  .pour-stream { stroke: var(--cool); stroke-width: var(--stream); opacity: 0.78; }
  .landing { fill: none; stroke: var(--cool); stroke-width: 2; opacity: 0; animation: land var(--duration) ease-out forwards; }
  @keyframes pour { 0% { stroke-dashoffset: 1; opacity: 0; } 12% { opacity: 0.85; } 58% { stroke-dashoffset: 0; } 82% { opacity: 0.75; } 100% { stroke-dashoffset: -1; opacity: 0; } }
  @keyframes land { 0%, 58% { opacity: 0; } 72% { opacity: 0.7; } 100% { opacity: 0; } }
  @media (prefers-reduced-motion: reduce) { .bench-effect { display: none; } }
</style>
