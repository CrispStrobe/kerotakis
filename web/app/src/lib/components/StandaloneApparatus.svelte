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
  const rotorDuration = $derived(`${Math.max(0.08, 0.65 - intensity * 0.5)}s`);
  const rotorImbalance = $derived(
    Math.abs(Number(values.sampleMass ?? 0) - Number(values.counterbalance ?? 0)),
  );
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
{:else if tool === "centrifuge"}
  <figure
    class="standalone centrifuge"
    class:working
    class:performed={performedAt !== undefined}
    style:--rotor-duration={rotorDuration}
    aria-label={t("mini centrifuge on the bench")}
  >
    <svg viewBox="0 0 110 88" role="img" aria-label={t("mini centrifuge") }>
      <path class="centrifuge-base" d="M12 33 Q12 20 26 18 H84 Q98 20 98 33 L103 73 Q101 82 91 82 H19 Q9 82 7 73Z" />
      <ellipse class="lid" class:danger={rotorImbalance > 0.1} cx="55" cy="32" rx="39" ry="22" />
      <g class="rotor">
        <circle class="hub" cx="55" cy="32" r="6" />
        <path class="rotor-arm" d="M24 32 H86 M55 10 V54" />
        <g class="tube tube-a" transform="translate(25 27) rotate(-90 5 5)"><path d="M1 1 H9 V15 Q5 20 1 15Z" /></g>
        <g class="tube tube-b" transform="translate(75 27) rotate(90 5 5)"><path d="M1 1 H9 V15 Q5 20 1 15Z" /></g>
      </g>
      <rect class="display" x="34" y="62" width="42" height="12" rx="3" />
      <text x="55" y="71" text-anchor="middle">{Number(values.rpm ?? 3000).toFixed(0)} rpm</text>
    </svg>
    <figcaption>
      <strong>{t("mini centrifuge")}</strong>
      <small>{values.radius ?? 8} cm · {values.seconds ?? 60} s</small>
      <small class="balance" class:danger={rotorImbalance > 0.1}>{rotorImbalance > 0.1 ? `⚠ ${rotorImbalance.toFixed(2)} g` : `✓ ${t("balanced")}`}</small>
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
  .centrifuge { width: 96px; right: -3.1rem; }
  .centrifuge-base { fill: color-mix(in srgb, var(--primary) 22%, var(--surface)); stroke: var(--edge-strong); stroke-width: 2; }
  .lid { fill: color-mix(in srgb, var(--cool) 18%, var(--surface)); stroke: var(--edge-strong); stroke-width: 2; }
  .rotor { transform-origin: 55px 32px; }
  .rotor-arm { fill: none; stroke: var(--edge-strong); stroke-width: 5; stroke-linecap: round; }
  .hub { fill: var(--hot); stroke: var(--edge-strong); stroke-width: 2; }
  .tube path { fill: color-mix(in srgb, var(--cool) 35%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1.5; }
  .display { fill: var(--edge-strong); }
  .centrifuge text { fill: var(--surface); font-size: 7px; font-weight: 800; }
  .lid.danger { stroke: var(--danger); }
  .balance.danger { color: var(--danger); }
  .working .rotor { animation: spin var(--rotor-duration) linear infinite; }
  .performed:not(.working) .rotor { animation: spin var(--rotor-duration) linear 12; }
  figcaption { display: grid; justify-items: center; margin-top: -0.2rem; color: var(--ink); font-size: 0.55rem; line-height: 1.15; }
  figcaption small { color: var(--dim); }
  @keyframes grind { to { transform: rotate(-18deg) translateY(-2px); } }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .working .pestle, .working .rotor, .performed .rotor { animation: none; } }
</style>
