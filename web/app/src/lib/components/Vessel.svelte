<script lang="ts">
  import type { SceneVessel } from "../host/EngineHost";

  let {
    vessel,
    register,
    selected = false,
    onselect,
    ondropspecies,
  }: {
    vessel: SceneVessel;
    register: string;
    selected?: boolean;
    onselect?: (id: number) => void;
    ondropspecies?: (id: number, payload: { key: string; phase: string }) => void;
  } = $props();

  let dropReady = $state(false);

  function ondrop(e: DragEvent) {
    dropReady = false;
    const raw = e.dataTransfer?.getData("application/x-kero-species");
    if (!raw) return;
    e.preventDefault();
    try {
      ondropspecies?.(vessel.id, JSON.parse(raw));
    } catch {
      // A malformed drag payload is simply not a drop.
    }
  }

  // Drawing basis: a 400 mL beaker drawn 84 units tall inside a 100×140 box.
  const INNER_X = 14;
  const INNER_W = 72;
  const BOTTOM_Y = 122;
  const FULL_AT_L = 0.4;
  const FULL_H = 84;

  const liquidH = $derived(
    vessel.liquid
      ? Math.max(6, Math.min(FULL_H, (vessel.liquid.volume_l / FULL_AT_L) * FULL_H))
      : 0,
  );
  // Solids draw as a settled layer; depth follows amount, capped well below
  // the liquid so the layer reads as a deposit rather than a fill.
  const solidH = $derived(
    Math.min(
      18,
      vessel.solids.reduce((sum, s) => sum + s.moles, 0) * 600,
    ),
  );
  const rgb = (c: [number, number, number]) => `rgb(${c[0]},${c[1]},${c[2]})`;
  const tempC = $derived(vessel.temperature_k - 273.15);
  const sealed = $derived(vessel.boundary !== "open");
</script>

<button
  class="vessel"
  class:selected
  class:drop-ready={dropReady}
  aria-label={`${vessel.label} v${vessel.id + 1}: ${vessel.words}`}
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
  <svg viewBox="0 0 100 140" role="img">
    <title>{vessel.words}</title>

    {#if vessel.liquid && liquidH > 0}
      <rect
        x={INNER_X}
        y={BOTTOM_Y - liquidH}
        width={INNER_W}
        height={liquidH}
        fill={rgb(vessel.liquid.srgb)}
        opacity="0.92"
      />
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
    {/if}

    {#if solidH > 0}
      {#each vessel.solids.slice(0, 3) as solid, i (solid.species)}
        <rect
          x={INNER_X}
          y={BOTTOM_Y - (solidH * (vessel.solids.length - i)) / vessel.solids.length}
          width={INNER_W}
          height={(solidH / vessel.solids.length) * (i + 1)}
          fill={rgb(solid.srgb)}
          class:metallic={solid.metallic}
        >
          <title>{solid.colour_word} {solid.name}</title>
        </rect>
      {/each}
    {/if}

    {#if vessel.bubbling && liquidH > 0}
      {#each [30, 50, 66] as x, i (x)}
        <circle
          class="bubble"
          cx={x}
          cy={BOTTOM_Y - 4}
          r="2.4"
          style={`--rise:${liquidH - 8}px; animation-delay:${i * 0.45}s`}
        />
      {/each}
    {/if}

    <!-- The glass, drawn over the contents. -->
    <path
      class="glass"
      d="M 12 14 L 12 122 Q 12 128 20 128 L 80 128 Q 88 128 88 122 L 88 14"
    />
    {#if sealed}
      <rect class="lid" x="10" y="9" width="80" height="5" rx="2" />
    {/if}
  </svg>

  <div class="caption">
    <span class="label">{vessel.label} v{vessel.id + 1}</span>
    {#if register !== "lv1"}
      <span class="badge">{tempC.toFixed(1)} °C</span>
      {#each vessel.badges as badge (badge.key)}
        <span class="badge" data-confidence={badge.confidence}>
          {badge.key === "ph" ? "pH" : badge.key}
          {badge.value.toFixed(2)}
        </span>
      {/each}
      {#if sealed}<span class="badge">{vessel.boundary}</span>{/if}
    {/if}
  </div>
</button>

<style>
  .vessel {
    margin: 0;
    padding: 0.4rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    background: none;
    border: 1px solid transparent;
    border-radius: 10px;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  .vessel:hover {
    border-color: var(--edge);
  }
  .vessel.selected {
    border-color: var(--hot);
  }
  .vessel.drop-ready {
    border-color: var(--good);
    background: var(--panel);
  }
  svg {
    width: clamp(96px, 18vw, 150px);
  }
  .glass {
    fill: none;
    stroke: var(--edge-strong);
    stroke-width: 2.5;
    stroke-linecap: round;
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
  @media (prefers-reduced-motion: reduce) {
    .bubble {
      animation: none;
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
  }
</style>
