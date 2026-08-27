<script lang="ts">
  import { t } from "../i18n.svelte";

  let {
    tool,
    working = false,
    values = {},
    surfaceY = 118,
  }: {
    tool: string;
    working?: boolean;
    values?: Record<string, number | string>;
    surfaceY?: number;
  } = $props();

  const toolNames: Record<string, string> = {
    burette: "burette",
    bunsen: "Bunsen burner",
    dilute: "wash bottle",
    evaporate: "evaporating dish",
    electrolyse: "electrodes and supply",
    grind: "mortar",
    heat: "hotplate",
    cool: "cooling bath",
    irradiate: "lamp",
    regulate: "piston lid",
    stir: "magnetic stirrer",
    sweep: "carrier-gas line",
  };

  const wavelength = $derived(Number(values.wavelength ?? 500));
  const lampColour = $derived.by(() => {
    if (wavelength < 380) return "#8b5cf6";
    if (wavelength < 450) return "#526dff";
    if (wavelength < 495) return "#20bde8";
    if (wavelength < 570) return "#33d17a";
    if (wavelength < 590) return "#ffd447";
    if (wavelength < 620) return "#ff8a34";
    return "#ff506d";
  });
  const amps = $derived(Math.max(0.001, Number(values.amps ?? 0.5)));
  const pulseDuration = $derived(`${Math.max(0.25, 1.2 - Math.min(1, amps / 2) * 0.8)}s`);
  const pressure = $derived(Math.max(0.1, Number(values.pressure ?? 1)));
  const flamePower = $derived(Math.max(0, Math.min(100, Number(values.flame ?? 50))));
  const airOpen = $derived(Math.max(0, Math.min(100, Number(values.air ?? 70))));
  const flameHeight = $derived(8 + flamePower * 0.22);
  const flameOuter = $derived(airOpen < 35 ? "#ffb321" : "#248cff");
  const flameInner = $derived(airOpen < 35 ? "#fff0a8" : "#bdeaff");
  const stirRpm = $derived(Math.max(0, Number(values.rpm ?? 500)));
  const heatWatts = $derived(Math.max(0, Number(values.watts ?? 250)));
</script>

<g class="apparatus" class:working aria-label={t("{tool} deployed", { tool: t(toolNames[tool] ?? tool) })}>
  {#if tool === "stir" || tool === "heat" || tool === "cool"}
    <g class="magnetic-plate">
      <ellipse class="plate" cx="50" cy="121" rx="27" ry="5" />
      <rect class="base" x="22" y="123" width="56" height="11" rx="3" />
      <circle class="dial" cx="69" cy="129" r="2" />
      <text x="50" y="133" text-anchor="middle">{tool === "stir" ? `${stirRpm.toFixed(0)} rpm` : `${heatWatts.toFixed(0)} W`}</text>
      {#if tool === "heat"}
        {#each [39, 50, 61] as x, i (x)}
          <path class="heat" d={`M ${x} 117 q -4 -7 0 -14 q 4 -7 0 -14`} style={`--heat-delay:${i * .16}s;--heat-rate:${Math.max(.45, 1.5 - Math.min(1, heatWatts / 1000))}s`} />
        {/each}
      {/if}
      {#if tool === "cool"}
        {#each [38, 50, 62] as x, i (x)}
          <g class="frost" style={`--frost-delay:${i * .2}s;--frost-rate:${Math.max(.55, 1.7 - Math.min(1, heatWatts / 800))}s`}>
            <path d={`M ${x - 3} 113 H ${x + 3} M ${x} 110 V 116 M ${x - 2} 111 L ${x + 2} 115 M ${x + 2} 111 L ${x - 2} 115`} />
          </g>
        {/each}
      {/if}
    </g>
  {:else if tool === "burette"}
    <g class="burette">
      <path class="stand" d="M 91 3 V 129 M 84 129 H 99 M 85 18 H 91" />
      <rect class="glass-part" x="82" y="4" width="5" height="70" rx="1" />
      <rect class="fluid" x="83" y="13" width="3" height="37" rx=".5" />
      <path class="metal" d="M 80 74 H 89 M 84.5 74 V 83 L 50 93" />
      {#if working}<circle class="drop" cx="50" cy="91" r="1.8" />{/if}
    </g>
  {:else if tool === "bunsen"}
    <g class="burner" style={`--flame-power:${flamePower / 100};--flame-outer:${flameOuter};--flame-inner:${flameInner}`}>
      <path class="burner-base" d="M 32 133 H 68 L 61 126 H 39 Z" />
      <rect class="burner-tube" x="43" y="91" width="14" height="37" rx="3" />
      <rect class="burner-collar" x="40" y="105" width="20" height="7" rx="3" />
      <circle class="air-hole" cx="47" cy="108.5" r={0.4 + airOpen * 0.018} />
      <circle class="air-hole" cx="53" cy="108.5" r={0.4 + airOpen * 0.018} />
      {#if flamePower > 0}
        <path class="flame-outer" d={`M 50 92 C 36 78 44 ${92 - flameHeight * 0.55} 50 ${92 - flameHeight} C 56 ${92 - flameHeight * 0.55} 64 78 50 92 Z`} />
        <path class="flame-inner" d={`M 50 91 C 44 84 48 ${91 - flameHeight * 0.45} 50 ${91 - flameHeight * 0.62} C 52 ${91 - flameHeight * 0.45} 56 84 50 91 Z`} />
      {/if}
      <text x="61" y="103">{flamePower > 0 ? `${flamePower.toFixed(0)}%` : "off"}</text>
      <text x="61" y="110">air {airOpen.toFixed(0)}%</text>
    </g>
  {:else if tool === "dilute"}
    <g class="wash-bottle">
      <path class="bottle" d="M 75 74 q 12 -2 15 7 v 34 q -2 7 -13 7 q -11 0 -13 -7 V 84 q 0 -8 11 -10 Z" />
      <path class="tube" d="M 74 76 V 66 q 0 -7 -7 -7 H 55 q -7 0 -7 8 v 24" />
      {#if working}
        <path class="water-stream" d={`M 48 88 Q 45 98 50 ${surfaceY - 3}`} />
      {/if}
    </g>
  {:else if tool === "evaporate"}
    <g class="hotplate">
      <path class="dish" d="M 25 112 Q 50 123 75 112 L 70 121 Q 50 130 30 121 Z" />
      <rect class="base" x="24" y="124" width="52" height="10" rx="3" />
      <circle class="dial" cx="68" cy="129" r="2" />
      {#if working}
        {#each [38, 50, 62] as x, i (x)}
          <path class="heat" d={`M ${x} 111 q -5 -7 0 -14 q 5 -7 0 -14`} style={`animation-delay:${i * 0.18}s`} />
        {/each}
      {/if}
    </g>
  {:else if tool === "electrolyse"}
    <g class="electrodes" style={`--pulse:${pulseDuration}`}>
      <path class="wire positive" d="M 30 10 H 18 V 32" />
      <path class="wire negative" d="M 70 10 H 82 V 32" />
      <rect class="power" x="34" y="2" width="32" height="16" rx="4" />
      <text x="50" y="13" text-anchor="middle">{amps.toFixed(2)} A</text>
      <rect class="electrode" x="28" y="26" width="4" height={Math.max(20, surfaceY - 34)} rx="1" />
      <rect class="electrode" x="68" y="26" width="4" height={Math.max(20, surfaceY - 34)} rx="1" />
      {#if working}
        {#each [30, 70] as x (x)}<circle class="charge" cx={x} cy={surfaceY - 10} r="2" />{/each}
      {/if}
    </g>
  {:else if tool === "grind"}
    <g class="mortar">
      <path class="bowl" d="M 61 98 Q 78 100 91 97 Q 88 121 76 124 Q 64 121 61 98 Z" />
      <path class="pestle" d="M 58 75 L 82 108" />
    </g>
  {:else if tool === "irradiate"}
    <g class="lamp" style={`--lamp:${lampColour}`}>
      <path class="lamp-arm" d="M 8 122 V 34 Q 8 22 20 22 H 27" />
      <path class="lamp-head" d="M 26 14 h 24 l 6 17 H 20 Z" />
      <path class="light-cone" d={`M 29 31 L 43 ${surfaceY} H 74 L 48 31 Z`} />
      <text x="8" y="133">{wavelength.toFixed(0)} nm</text>
    </g>
  {:else if tool === "regulate"}
    <g class="regulator">
      <rect class="lid-plate" x="19" y="18" width="62" height="6" rx="2" />
      <path class="piston" d="M 50 18 V 5 M 41 5 H 59" />
      <circle class="gauge" cx="82" cy="8" r="9" />
      <path class="needle" d={`M 82 8 l ${Math.min(7, 2 + pressure * 1.5)} -3`} />
      <text x="82" y="-4" text-anchor="middle">{pressure.toFixed(1)} bar</text>
    </g>
  {:else if tool === "sweep"}
    <g class="gas-line">
      <path class="hose" d="M 2 45 H 29 V 18 M 98 36 H 71 V 18" />
      <path class="arrow" d="M 14 41 l 8 4 l -8 4 Z M 86 32 l 8 4 l -8 4 Z" />
      {#if working}<circle class="gas-pulse" cx="5" cy="45" r="3" />{/if}
    </g>
  {/if}
</g>

<style>
  .apparatus { color: var(--instrument); pointer-events: none; }
  .stand, .metal, .tube, .wire, .lamp-arm, .hose, .piston, .needle { fill: none; stroke: var(--edge-strong); stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
  .burner-base, .burner-tube, .burner-collar { fill: color-mix(in srgb, var(--instrument) 35%, var(--edge-strong)); stroke: var(--edge-strong); stroke-width: 1; }
  .air-hole { fill: var(--surface); }
  .flame-outer { fill: var(--flame-outer, #248cff); opacity: calc(.45 + var(--flame-power) * .5); }
  .flame-inner { fill: var(--flame-inner, #bdeaff); opacity: calc(.5 + var(--flame-power) * .45); }
  .burner text { fill: var(--ink); font-size: 6px; font-weight: 700; }
  .working .flame-outer { animation: burner-flicker .35s ease-in-out infinite alternate; }
  .glass-part, .bottle { fill: color-mix(in srgb, var(--cool) 12%, transparent); stroke: var(--edge-strong); stroke-width: 1.2; }
  .fluid, .water-stream, .drop { fill: var(--cool); stroke: var(--cool); }
  .water-stream { fill: none; stroke-width: 2; stroke-dasharray: 5 3; animation: flow .65s linear infinite; }
  .drop { animation: drop 0.8s ease-in infinite; }
  .dish, .bowl { fill: color-mix(in srgb, var(--surface) 84%, var(--cool)); stroke: var(--edge-strong); stroke-width: 1.2; }
  .base, .power { fill: color-mix(in srgb, var(--instrument) 28%, var(--edge-strong)); stroke: var(--edge-strong); stroke-width: 1; }
  .dial { fill: var(--hot); }
  .heat { fill: none; stroke: var(--hot); stroke-width: 1.5; opacity: 0; }
  .working .heat { animation: rise 1.15s ease-out infinite; }
  .magnetic-plate .heat { animation-duration: var(--heat-rate, 1.15s); animation-delay: var(--heat-delay, 0s); }
  .frost { color: var(--cool); opacity: .35; }
  .working .frost { animation: frost-pulse var(--frost-rate, 1.2s) ease-in-out infinite alternate; animation-delay: var(--frost-delay, 0s); }
  .frost path { fill: none; stroke: currentColor; stroke-width: 1; stroke-linecap: round; }
  .positive { stroke: var(--danger); } .negative { stroke: var(--primary); }
  .electrode { fill: var(--edge-strong); }
  .electrodes text, .lamp text, .regulator text, .magnetic-plate text { fill: var(--ink); font-size: 6px; font-weight: 700; }
  .plate { fill: var(--surface); stroke: var(--edge-strong); stroke-width: 1.2; }
  .charge { fill: none; stroke: var(--instrument); animation: bubble var(--pulse) ease-out infinite; }
  .pestle { fill: none; stroke: var(--edge-strong); stroke-width: 7; stroke-linecap: round; transform-origin: 76px 108px; }
  .working .pestle { animation: grind .5s ease-in-out infinite alternate; }
  .lamp-head { fill: var(--edge-strong); }
  .light-cone { fill: var(--lamp); opacity: .16; filter: blur(1px); }
  .working .light-cone { animation: lamp-pulse .8s ease-in-out infinite alternate; }
  .lid-plate { fill: var(--edge-strong); }
  .gauge { fill: var(--surface); stroke: var(--instrument); stroke-width: 1.5; }
  .arrow { fill: var(--instrument); }
  .gas-pulse { fill: var(--instrument); animation: gas-flow 1.1s linear infinite; }
  @keyframes flow { to { stroke-dashoffset: -8; } }
  @keyframes drop { from { opacity: 0; transform: translateY(-8px); } 30% { opacity: 1; } to { opacity: 0; transform: translateY(22px); } }
  @keyframes rise { 0% { opacity: 0; transform: translateY(4px); } 35% { opacity: .75; } 100% { opacity: 0; transform: translateY(-8px); } }
  @keyframes bubble { to { opacity: 0; transform: translateY(-34px) scale(1.8); } }
  @keyframes grind { to { transform: rotate(-18deg) translateY(-2px); } }
  @keyframes lamp-pulse { to { opacity: .3; } }
  @keyframes gas-flow { to { transform: translateX(88px); opacity: 0; } }
  @keyframes burner-flicker { to { transform: scaleX(.9) translateX(5px); } }
  @keyframes frost-pulse { to { opacity: 1; transform: translateY(-3px) scale(1.18); } }
  @media (prefers-reduced-motion: reduce) { .apparatus * { animation: none !important; } }
</style>
