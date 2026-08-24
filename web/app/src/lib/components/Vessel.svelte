<script lang="ts">
  import type { SceneVessel } from "../host/EngineHost";

  let {
    vessel,
    register,
    selected = false,
    onselect,
    ondropspecies,
    effects = [],
  }: {
    vessel: SceneVessel;
    register: string;
    selected?: boolean;
    onselect?: (id: number) => void;
    ondropspecies?: (id: number, payload: { key: string; phase: string }) => void;
    effects?: { kind: string; at: number }[];
  } = $props();

  // Transient effects: young enough that their animation is still running.
  const now = () => Date.now();
  const active = (kind: string, withinMs: number) =>
    effects.some((e) => e.kind === kind && now() - e.at < withinMs);

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

  // Per-kind glassware geometry inside the 100×140 box. `inner` is the
  // fill silhouette (liquids and solids are clipped to it, so a conical
  // flask holds its liquid in a cone); `glass` is the stroked outline.
  const KINDS: Record<
    string,
    { ix: number; iw: number; by: number; fh: number; fullAtL: number; glass: string; inner: string; svgW: number }
  > = {
    beaker: {
      ix: 14, iw: 72, by: 122, fh: 84, fullAtL: 0.4, svgW: 150,
      glass: "M 12 14 L 12 122 Q 12 128 20 128 L 80 128 Q 88 128 88 122 L 88 14",
      inner: "M 13 14 L 13 127 L 87 127 L 87 14 Z",
    },
    flask: {
      ix: 14, iw: 72, by: 122, fh: 92, fullAtL: 0.3, svgW: 150,
      glass: "M 42 8 L 42 44 L 14 118 Q 12 128 22 128 L 78 128 Q 88 128 86 118 L 58 44 L 58 8",
      inner: "M 43 8 L 43 45 L 15 120 Q 14 127 22 127 L 78 127 Q 86 127 85 120 L 57 45 L 57 8 Z",
    },
    tube: {
      ix: 38, iw: 24, by: 118, fh: 100, fullAtL: 0.05, svgW: 90,
      glass: "M 38 10 L 38 114 Q 38 128 50 128 Q 62 128 62 114 L 62 10",
      inner: "M 39 10 L 39 114 Q 39 127 50 127 Q 61 127 61 114 L 61 10 Z",
    },
    cylinder: {
      ix: 38, iw: 24, by: 124, fh: 108, fullAtL: 0.1, svgW: 90,
      glass: "M 38 8 L 38 124 L 62 124 L 62 8 M 30 130 L 70 130",
      inner: "M 39 8 L 39 123 L 61 123 L 61 8 Z",
    },
    crucible: {
      ix: 24, iw: 52, by: 122, fh: 34, fullAtL: 0.08, svgW: 150,
      glass: "M 18 92 L 26 126 Q 27 128 30 128 L 70 128 Q 73 128 74 126 L 82 92",
      inner: "M 20 93 L 27 127 L 73 127 L 80 93 Z",
    },
  };
  const geom = $derived(KINDS[vessel.label] ?? KINDS.beaker);
  const INNER_X = $derived(geom.ix);
  const INNER_W = $derived(geom.iw);
  const BOTTOM_Y = $derived(geom.by);
  const FULL_AT_L = $derived(geom.fullAtL);
  const FULL_H = $derived(geom.fh);

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
  <svg viewBox="0 0 100 140" role="img" style={`width:clamp(64px,14vw,${geom.svgW}px)`}>
    <title>{vessel.words}</title>
    <defs>
      <clipPath id={`vclip-${vessel.id}`}>
        <path d={geom.inner} />
      </clipPath>
    </defs>

    <g clip-path={`url(#vclip-${vessel.id})`}>
    {#if vessel.liquid && liquidH > 0}
      <rect
        x={INNER_X}
        y={BOTTOM_Y - liquidH}
        width={INNER_W}
        height={liquidH}
        fill={rgb(vessel.liquid.srgb)}
        opacity={liquidOpacity(vessel.liquid.srgb)}
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

    </g>

    <!-- State-driven effects: every one traces to a computed number. -->
    {#if hot > 0.02}
      <ellipse class="glow" cx="50" cy="132" rx="34" ry="5" style={`opacity:${0.15 + hot * 0.5}`} />
    {/if}
    {#if burning}
      <g class="flame" aria-hidden="true">
        <path class="outer" d="M 50 -2 Q 42 12 47 20 Q 50 25 53 20 Q 58 12 50 -2 Z" />
        <path class="inner" d="M 50 6 Q 46 13 49 18 Q 50 20 51 18 Q 54 13 50 6 Z" />
      </g>
    {/if}
    {#if steaming}
      {#each [34, 50, 66] as x, i (x)}
        <path
          class="steam"
          d={`M ${x} ${BOTTOM_Y - liquidH - 4} q 3 -6 0 -12 q -3 -6 0 -12`}
          style={`animation-delay:${i * 0.5}s`}
        />
      {/each}
    {/if}
    {#if frosty}
      <g class="frost" aria-hidden="true">
        {#each [[18, 40], [80, 60], [22, 90], [78, 105]] as [fx, fy], i (i)}
          <path d={`M ${fx} ${fy} l 4 0 M ${fx + 2} ${fy - 2} l 0 4 M ${fx} ${fy - 2} l 4 4 M ${fx} ${fy + 2} l 4 -4`} />
        {/each}
      </g>
    {/if}

    <!-- Event-driven transients (GUI-026): each fires only because the
         engine emitted the matching event. -->
    {#if active("precipitate", 1800) && liquidH > 0}
      {#each [30, 44, 58, 70] as x, i (x)}
        <circle
          class="falling"
          cx={x}
          cy={BOTTOM_Y - liquidH + 6}
          r="1.8"
          style={`--fall:${Math.max(8, liquidH - 10)}px; animation-delay:${i * 0.15}s`}
        />
      {/each}
    {/if}
    {#if active("dissolve", 1400) && liquidH > 0}
      <circle class="dissolving" cx="50" cy={BOTTOM_Y - 10} r="4" />
    {/if}
    {#if active("electrolyse", 3500) && liquidH > 0}
      {#each [30, 70] as x (x)}
        {#each [0, 1, 2] as i (i)}
          <circle
            class="bubble"
            cx={x + (i - 1) * 2}
            cy={BOTTOM_Y - 6}
            r="1.6"
            style={`--rise:${liquidH - 10}px; animation-delay:${i * 0.35}s`}
          />
        {/each}
      {/each}
    {/if}
    {#if active("plate", 2000)}
      <rect class="shimmer" x={INNER_X} y={BOTTOM_Y - Math.max(solidH, 6)} width={INNER_W} height={Math.max(solidH, 6)} />
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
    <path class="glass" d={geom.glass} />
    {#if vessel.boundary === "sealed"}
      <rect class="lid" x="10" y="9" width="80" height="5" rx="2">
        <title>sealed</title>
      </rect>
    {:else if vessel.boundary === "pressure_controlled"}
      <!-- A floating piston: the lid that moves to hold the set pressure. -->
      <rect class="lid" x="14" y="16" width="72" height="4" rx="1">
        <title>pressure-controlled</title>
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
        <title>swept with carrier gas</title>
      </g>
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
    height: auto;
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
    animation: flicker 0.35s ease-in-out infinite alternate;
    transform-origin: 50px 20px;
  }
  .flame .inner {
    fill: var(--warn);
    animation: flicker 0.28s ease-in-out infinite alternate-reverse;
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
      opacity: 0.65;
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
  @media (prefers-reduced-motion: reduce) {
    .bubble,
    .flame .outer,
    .flame .inner,
    .steam,
    .falling,
    .dissolving,
    .shimmer {
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
  }
</style>
