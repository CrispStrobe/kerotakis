<script lang="ts">
  import type { ShelfItem } from "../session.svelte";

  let { item }: { item: ShelfItem } = $props();

  // Shape encodes phase (survives colourless viewing); fill is the
  // engine's own colour — reflective for the substance, computed
  // solution tint as fallback, glassy neutral when neither is curated.
  const phase = $derived(item.phase.toLowerCase());
  const fill = $derived.by(() => {
    const c = item.srgb ?? item.solution_srgb;
    return c ? `rgb(${c[0]},${c[1]},${c[2]})` : "var(--panel-raised)";
  });
  const curated = $derived(item.srgb != null || item.solution_srgb != null);
  const FLAMES: Record<string, string> = {
    orange: "#e8923a", yellow: "#e6c34a", red: "#d05545", crimson: "#c53a52",
    green: "#5ea55e", "apple-green": "#7db65a", lilac: "#a487c9",
    violet: "#8a63b8", blue: "#5a83c9", "brick-red": "#b65a3a",
  };
  const flameFill = $derived(item.flame ? (FLAMES[item.flame] ?? "var(--hot)") : null);
  const title = $derived(
    [
      item.appearance ? `${item.appearance} ${phase}` : phase,
      item.flame ? `burns ${item.flame}` : null,
    ]
      .filter(Boolean)
      .join(" · "),
  );
</script>

<span class="chip" {title}>
  <svg viewBox="0 0 20 20" aria-hidden="true">
    {#if phase === "solid"}
      <!-- a heap of powder -->
      <path d="M 3 15 Q 10 6 17 15 Z" fill={fill} class:uncurated={!curated} />
      <line x1="2" y1="15.5" x2="18" y2="15.5" class="ground" />
    {:else if phase === "gas"}
      <!-- a wisp -->
      <circle cx="10" cy="10" r="6" fill="none" class="gasring" style={`stroke:${fill}`} />
      <circle cx="10" cy="10" r="2.2" fill={fill} class:uncurated={!curated} opacity="0.5" />
    {:else}
      <!-- a droplet: liquid or aqueous -->
      <path
        d="M 10 3 Q 15 10 13.5 14 Q 12 17 10 17 Q 8 17 6.5 14 Q 5 10 10 3 Z"
        fill={fill}
        class:uncurated={!curated}
      />
    {/if}
    {#if flameFill}
      <path d="M 16.5 2 Q 15 5 16 6.5 Q 16.5 7.4 17 6.5 Q 18 5 16.5 2 Z" fill={flameFill} />
    {/if}
  </svg>
</span>

<style>
  .chip {
    display: inline-flex;
    width: 22px;
    height: 22px;
    flex: none;
  }
  svg {
    width: 100%;
    height: 100%;
  }
  .uncurated {
    stroke: var(--edge-strong);
    stroke-width: 0.8;
    stroke-dasharray: 2 1.5;
  }
  .ground {
    stroke: var(--edge-strong);
    stroke-width: 1;
  }
  .gasring {
    stroke-width: 1.4;
    stroke-dasharray: 3 2;
  }
</style>
