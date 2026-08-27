<script lang="ts">
  import { t } from "../i18n.svelte";

  let { tool, working = false, performedAt, intensity = 0.5, values = {} }: {
    tool: string;
    working?: boolean;
    performedAt?: number;
    intensity?: number;
    values?: Record<string, number | string>;
  } = $props();

  const grindDuration = $derived(`${Math.max(0.18, 0.55 - intensity * 0.3)}s`);
</script>

{#if tool === "grind"}
  <figure
    class="standalone mortar"
    class:working
    class:performed={performedAt !== undefined}
    style:--grind-duration={grindDuration}
    aria-label={t("mortar on the bench")}
  >
    <svg viewBox="0 0 100 82" role="img" aria-label={t("mortar and pestle")}>
      <ellipse class="rim" cx="50" cy="34" rx="32" ry="10" />
      <path class="bowl" d="M18 34 Q22 69 50 73 Q78 69 82 34 Q66 43 50 43 Q34 43 18 34Z" />
      <path class="pestle" d="M28 8 L59 49" />
    </svg>
    <figcaption>
      <strong>{t("mortar")}</strong>
      {#if values.species}<small>{t(String(values.species))} · {values.diameter ?? 50} µm</small>{/if}
    </figcaption>
  </figure>
{/if}

<style>
  .standalone {
    position: absolute;
    z-index: 7;
    right: -2.5rem;
    bottom: 0.1rem;
    width: 78px;
    margin: 0;
    pointer-events: none;
    filter: drop-shadow(0 8px 7px var(--shadow));
  }
  svg { display: block; width: 100%; overflow: visible; }
  .rim, .bowl { fill: color-mix(in srgb, var(--surface) 76%, var(--instrument)); stroke: var(--edge-strong); stroke-width: 2; }
  .rim { fill: color-mix(in srgb, var(--surface) 58%, var(--instrument)); }
  .pestle { fill: none; stroke: var(--edge-strong); stroke-width: 9; stroke-linecap: round; transform-origin: 59px 49px; }
  .working .pestle { animation: grind var(--grind-duration) ease-in-out infinite alternate; }
  .performed:not(.working) .pestle { animation: grind var(--grind-duration) ease-in-out 8 alternate; }
  figcaption { display: grid; justify-items: center; margin-top: -0.2rem; color: var(--ink); font-size: 0.55rem; line-height: 1.15; }
  figcaption small { color: var(--dim); }
  @keyframes grind { to { transform: rotate(-18deg) translateY(-2px); } }
  @media (prefers-reduced-motion: reduce) { .working .pestle { animation: none; } }
</style>
