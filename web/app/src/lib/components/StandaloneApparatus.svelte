<script lang="ts">
  import { t } from "../i18n.svelte";
  import type { Effect } from "../magnitudes";

  let { tool, target, working = false, performedAt, intensity = 0.5, values = {}, effect }: {
    tool: string;
    target: number;
    working?: boolean;
    performedAt?: number;
    intensity?: number;
    values?: Record<string, number | string>;
    effect?: Effect;
  } = $props();

  const grindDuration = $derived(`${Math.max(0.18, 0.55 - intensity * 0.3)}s`);
  const rotorDuration = $derived(`${Math.max(0.08, 0.65 - intensity * 0.5)}s`);
  const rotorImbalance = $derived(
    effect?.centrifuge?.imbalanceG ?? Math.abs(Number(values.sampleMass ?? 0) - Number(values.counterbalance ?? 0)),
  );
  const centrifugeRpm = $derived(effect?.centrifuge?.rpm ?? Number(values.rpm ?? 3000));
  const centrifugeSeconds = $derived(effect?.centrifuge?.seconds ?? Number(values.seconds ?? 60));
  const centrifugeRadiusCm = $derived((effect?.centrifuge?.rotorRadiusM ?? Number(values.radius ?? 8) / 100) * 100);
  const centrifugeRcf = $derived(
    effect?.centrifuge?.rcf ?? 1.118e-5 * centrifugeRadiusCm * centrifugeRpm * centrifugeRpm,
  );
  const buretteFraction = $derived(
    Number(values.total ?? 0) > 0
      ? Math.min(1, Math.max(0, Number(values.delivered ?? 0) / Number(values.total)))
      : 0,
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
      <span class="target">{t("works with vessel v{vessel}", { vessel: target + 1 })}</span>
      {#if values.species}<small>{t(String(values.species))} · {values.diameter ?? 50} µm</small>{/if}
    </figcaption>
  </figure>
{:else if tool === "evaporate"}
  <figure
    class="standalone evaporation-station"
    class:working
    class:performed={performedAt !== undefined}
    style={`--evaporation-intensity:${Math.max(.12, intensity)};--dish-liquid:${effect?.fluidColour ?? "color-mix(in srgb, var(--cool) 25%, var(--surface))"}`}
    aria-label={t("evaporating dish station on the bench")}
  >
    <svg viewBox="0 0 110 88" role="img" aria-label={t("evaporating dish and heater") }>
      <ellipse class="station-shadow" cx="55" cy="78" rx="38" ry="5" />
      <rect class="heater-base" x="20" y="58" width="70" height="18" rx="5" />
      <ellipse class="heater-top" cx="55" cy="58" rx="30" ry="8" />
      <path class="porcelain-dish" d="M17 36 Q55 54 93 36 L84 57 Q55 70 26 57Z" />
      <path class="dish-liquid" d="M22 39 Q55 51 88 39 Q55 58 22 39Z" />
      <circle class="heater-dial" cx="80" cy="68" r="3" />
      {#each [39, 55, 71] as x, i (x)}
        <path class="evaporation-steam" d={`M ${x} 37 q -6 -8 0 -16 q 6 -8 0 -16`} style={`--steam-delay:${i * .18}s`} />
      {/each}
      {#if effect}
        <rect class="evaporation-display" x="35" y="63" width="35" height="9" rx="2" />
        <text x="52.5" y="69.5" text-anchor="middle">{((effect.reading ?? 0) * 1000).toPrecision(2)} mmol</text>
      {/if}
    </svg>
    <figcaption>
      <strong>{t("evaporating dish")}</strong>
      <span class="target">{t("works with vessel v{vessel}", { vessel: target + 1 })}</span>
      {#if effect}<small>{t("steam scaled from removed water")}</small>{/if}
    </figcaption>
  </figure>
{:else if tool === "dilute"}
  <figure
    class="standalone wash-station"
    class:working
    class:performed={performedAt !== undefined}
    style={`--wash-strength:${Math.max(.15, intensity)}`}
    aria-label={t("wash bottle station on the bench")}
  >
    <svg viewBox="0 0 110 88" role="img" aria-label={t("wash bottle adding water") }>
      <ellipse class="station-shadow" cx="45" cy="80" rx="30" ry="4" />
      <path class="wash-body" d="M25 27 Q45 19 65 27 L70 69 Q67 79 45 80 Q23 79 20 69Z" />
      <path class="wash-water" d="M23 48 Q45 43 67 48 L69 68 Q65 76 45 77 Q25 76 21 68Z" />
      <rect class="wash-cap" x="36" y="18" width="18" height="12" rx="3" />
      <path class="wash-nozzle" d="M45 18 V8 Q45 3 52 3 H76 Q84 3 84 10 V18" />
      <path class="wash-jet" d="M84 18 Q92 25 98 39" />
      {#if effect?.dilution}
        <rect class="wash-display" x="29" y="57" width="32" height="10" rx="3" />
        <text x="45" y="64" text-anchor="middle">{(effect.dilution.volumeL * 1000).toFixed(1)} mL</text>
      {/if}
    </svg>
    <figcaption>
      <strong>{t("wash bottle")}</strong>
      <span class="target">{t("delivers into vessel v{vessel}", { vessel: target + 1 })}</span>
      {#if effect?.dilution}<small>{(effect.dilution.waterMoles).toFixed(3)} mol H₂O</small>{/if}
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
        {#each (effect?.centrifuge?.populations ?? []).slice(0, 3) as population, i (population.species)}
          <circle class="centrifuge-pellet" cx={27 + i * 2.2} cy={32} r={1 + population.separatedFraction * 1.4} fill={population.colour ?? "var(--cloud)"} />
        {/each}
      </g>
      <rect class="display" x="27" y="60" width="56" height="16" rx="3" />
      <text x="55" y="67" text-anchor="middle">{centrifugeRpm.toFixed(0)} rpm</text>
      <text class="rcf" x="55" y="73" text-anchor="middle">{centrifugeRcf.toFixed(1)} × g</text>
    </svg>
    <figcaption>
      <strong>{t("mini centrifuge")}</strong>
      <span class="target">{t("works with vessel v{vessel}", { vessel: target + 1 })}</span>
      <small>{centrifugeRadiusCm.toFixed(1)} cm · {centrifugeSeconds.toFixed(1)} s</small>
      <small class="balance" class:danger={rotorImbalance > 0.1}>{rotorImbalance > 0.1 ? `⚠ ${rotorImbalance.toFixed(2)} g` : `✓ ${t("balanced")}`}</small>
      {#if effect?.centrifuge}
        <small class="coupling" class:forecast={!effect.centrifuge.stateCoupled}>
          {t(effect.centrifuge.stateCoupled ? "separation applied to vessel" : "visual forecast — vessel state unchanged")}
        </small>
        {#each effect.centrifuge.populations.slice(0, 2) as population (population.species)}
          <small class="separation-result">
            {t(population.species)} · {Math.round(population.separatedFraction * 100)}%
            {population.particleSizeAssumed ? ` · ${t("assumed particle size")}` : ""}
          </small>
        {/each}
      {/if}
    </figcaption>
  </figure>
{:else if tool === "burette"}
  <figure
    class="standalone burette-station"
    class:working
    aria-label={t("burette and retort stand on the bench")}
  >
    <svg viewBox="0 0 110 120" role="img" aria-label={t("burette and retort stand")}>
      <ellipse class="stand-foot" cx="69" cy="111" rx="34" ry="6" />
      <rect class="stand-base" x="42" y="102" width="54" height="9" rx="3" />
      <rect class="stand-rod" x="82" y="8" width="5" height="96" rx="2" />
      <path class="boss" d="M55 23 H87 V30 H55Z" />
      <path class="clamp-jaw" d="M55 22 L45 18 M55 30 L45 34" />
      <rect class="burette-glass" x="37" y="5" width="10" height="74" rx="4" />
      <rect
        class="burette-liquid"
        x="39"
        y={8 + 67 * buretteFraction}
        width="6"
        height={Math.max(2, 67 * (1 - buretteFraction))}
        rx="2"
      />
      {#each [16, 27, 38, 49, 60, 71] as y (y)}
        <line class="graduation" x1="37" y1={y} x2="42" y2={y} />
      {/each}
      <path class="stopcock" d="M31 80 H53 M42 76 V85" />
      <path class="burette-tip" d="M42 84 V98 L38 104" />
      {#if working}<circle class="burette-drop" cx="36" cy="108" r="2" />{/if}
    </svg>
    <figcaption>
      <strong>{t("burette and stand")}</strong>
      <span class="target">{t("delivers into vessel v{vessel}", { vessel: target + 1 })}</span>
      {#if Number(values.total ?? 0) > 0}
        <small>{Number(values.delivered ?? 0).toFixed(1)} / {Number(values.total).toFixed(1)} mL</small>
      {:else}
        <small>{t("ready for controlled addition")}</small>
      {/if}
    </figcaption>
  </figure>
{/if}

<style>
  .standalone {
    width: 100%;
    margin: 0;
    padding: 0.32rem 0.38rem 0.4rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 38%, var(--edge));
    border-radius: 13px;
    background: color-mix(in srgb, var(--surface) 92%, var(--instrument));
    pointer-events: none;
    filter: drop-shadow(0 8px 7px var(--shadow));
  }
  svg { display: block; width: 100%; overflow: visible; }
  .rim, .bowl { fill: color-mix(in srgb, var(--surface) 76%, var(--instrument)); stroke: var(--edge-strong); stroke-width: 2; }
  .rim { fill: color-mix(in srgb, var(--surface) 58%, var(--instrument)); }
  .pestle { fill: none; stroke: var(--edge-strong); stroke-width: 9; stroke-linecap: round; transform-origin: 59px 49px; }
  .working .pestle { animation: grind var(--grind-duration) ease-in-out infinite alternate; }
  .performed:not(.working) .pestle { animation: grind var(--grind-duration) ease-in-out 8 alternate; }
  .evaporation-station { background: color-mix(in srgb, var(--surface) 88%, var(--hot)); }
  .station-shadow { fill: var(--shadow); opacity: .35; }
  .heater-base { fill: color-mix(in srgb, var(--action) 30%, var(--edge-strong)); stroke: var(--edge-strong); stroke-width: 1.6; }
  .heater-top { fill: color-mix(in srgb, var(--hot) 18%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1.4; }
  .porcelain-dish { fill: color-mix(in srgb, var(--surface) 92%, var(--cool)); stroke: var(--edge-strong); stroke-width: 1.8; }
  .dish-liquid { fill: var(--dish-liquid); fill-opacity: .72; stroke: color-mix(in srgb, var(--dish-liquid) 70%, var(--edge-strong)); stroke-width: .8; }
  .heater-dial { fill: var(--hot); stroke: var(--edge-strong); stroke-width: .8; }
  .evaporation-steam { fill: none; stroke: var(--dim); stroke-width: calc(1px + var(--evaporation-intensity) * 1.6px); stroke-linecap: round; opacity: 0; }
  .working .evaporation-steam, .performed .evaporation-steam { animation: station-steam calc(1.4s - var(--evaporation-intensity) * .65s) ease-out 7; animation-delay: var(--steam-delay); }
  .evaporation-display { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); }
  .evaporation-station svg text { fill: var(--surface); font: 800 5px ui-monospace, monospace; }
  .wash-station { background: color-mix(in srgb, var(--surface) 86%, var(--cool)); }
  .wash-body { fill: color-mix(in srgb, var(--glass) 70%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1.8; }
  .wash-water { fill: color-mix(in srgb, var(--cool) 48%, var(--surface)); opacity: .72; }
  .wash-cap { fill: var(--instrument); stroke: var(--edge-strong); stroke-width: 1; }
  .wash-nozzle { fill: none; stroke: var(--edge-strong); stroke-width: 3; stroke-linecap: round; }
  .wash-jet { fill: none; stroke: var(--cool); stroke-width: calc(1px + var(--wash-strength) * 2px); stroke-linecap: round; stroke-dasharray: 4 3; opacity: 0; }
  .working .wash-jet, .performed .wash-jet { animation: wash-flow .65s linear 6; }
  .wash-display { fill: color-mix(in srgb, var(--success) 18%, var(--ink)); }
  .wash-station svg text { fill: var(--surface); font: 800 5px ui-monospace, monospace; }
  .centrifuge { width: 100%; }
  .centrifuge-base { fill: color-mix(in srgb, var(--primary) 22%, var(--surface)); stroke: var(--edge-strong); stroke-width: 2; }
  .lid { fill: color-mix(in srgb, var(--cool) 18%, var(--surface)); stroke: var(--edge-strong); stroke-width: 2; }
  .rotor { transform-origin: 55px 32px; }
  .rotor-arm { fill: none; stroke: var(--edge-strong); stroke-width: 5; stroke-linecap: round; }
  .hub { fill: var(--hot); stroke: var(--edge-strong); stroke-width: 2; }
  .tube path { fill: color-mix(in srgb, var(--cool) 35%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1.5; }
  .display { fill: var(--edge-strong); }
  .centrifuge text { fill: var(--surface); font-size: 7px; font-weight: 800; }
  .centrifuge text.rcf { font-size: 5px; fill: color-mix(in srgb, var(--success) 72%, white); }
  .centrifuge-pellet { stroke: var(--edge-strong); stroke-width: .4; }
  .lid.danger { stroke: var(--danger); }
  .balance.danger { color: var(--danger); }
  .coupling { margin-top: .12rem; font-weight: 800; color: var(--success); }
  .coupling.forecast { color: var(--hot); }
  .separation-result { max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .working .rotor { animation: spin var(--rotor-duration) linear infinite; }
  .performed:not(.working) .rotor { animation: spin var(--rotor-duration) linear 12; }
  .burette-station { background: color-mix(in srgb, var(--surface) 90%, var(--cool)); }
  .stand-foot { fill: var(--shadow); opacity: .4; }
  .stand-base { fill: color-mix(in srgb, var(--edge-strong) 76%, var(--surface)); stroke: var(--edge-strong); stroke-width: 1.5; }
  .stand-rod { fill: color-mix(in srgb, var(--edge-strong) 72%, var(--surface)); }
  .boss { fill: var(--edge-strong); }
  .clamp-jaw, .graduation, .stopcock, .burette-tip { fill: none; stroke: var(--edge-strong); stroke-width: 2; stroke-linecap: round; }
  .graduation { stroke-width: .8; }
  .burette-glass { fill: color-mix(in srgb, var(--cool) 8%, transparent); stroke: color-mix(in srgb, var(--cool) 72%, var(--edge-strong)); stroke-width: 1.5; }
  .burette-liquid { fill: color-mix(in srgb, var(--cool) 62%, var(--primary)); opacity: .72; transition: y 180ms linear, height 180ms linear; }
  .burette-drop { fill: var(--cool); animation: drip .75s ease-in infinite; }
  figcaption { display: grid; justify-items: center; margin-top: -0.2rem; color: var(--ink); font-size: 0.55rem; line-height: 1.15; }
  figcaption small { color: var(--dim); }
  .target { margin: 0.12rem 0; padding: 0.09rem 0.28rem; border-radius: 999px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 11%, var(--surface)); font-size: 0.48rem; font-weight: 850; }
  @keyframes grind { to { transform: rotate(-18deg) translateY(-2px); } }
  @keyframes station-steam { 0% { opacity: 0; transform: translateY(4px); } 30% { opacity: calc(.35 + var(--evaporation-intensity) * .55); } 100% { opacity: 0; transform: translateY(-12px); } }
  @keyframes wash-flow { 0% { opacity: 0; stroke-dashoffset: 8; } 20%, 80% { opacity: .85; } 100% { opacity: 0; stroke-dashoffset: -8; } }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes drip { from { transform: translateY(-5px); opacity: 1; } to { transform: translateY(5px); opacity: 0; } }
  @media (prefers-reduced-motion: reduce) { .working .pestle, .working .rotor, .performed .rotor, .burette-drop, .evaporation-steam, .wash-jet { animation: none; } }
</style>
