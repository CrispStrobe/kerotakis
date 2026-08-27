<script lang="ts">
  import type { Scene } from "../host/EngineHost";
  import type { Effect } from "../magnitudes";
  import Vessel from "./Vessel.svelte";
  import BenchEffect from "./BenchEffect.svelte";
  import StandaloneApparatus from "./StandaloneApparatus.svelte";
  import { t } from "../i18n.svelte";
  import {
    BENCH_ZONES,
    adjacentZone,
    placeVessel,
    zoneFor,
    type BenchLayout,
    type BenchZone,
  } from "../benchLayout";

  let {
    scene,
    register,
    selected,
    onselect,
    ondropspecies,
    pristine = false,
    effects = {},
    titrationPlayback = null,
    onnewvessel,
    onbadge,
    fluidLookup = null,
    transferFrom = null,
    deployedTool = null,
    deployedTarget = null,
    apparatusWorking = false,
    apparatusValues = {},
    layout,
    onmove,
    showZones = true,
    ontogglezones,
    onremove,
  }: {
    scene: Scene | null;
    register: string;
    selected: number;
    onselect: (id: number) => void;
    ondropspecies?: (id: number, payload: { key: string; phase: string }) => void;
    pristine?: boolean;
    effects?: Record<number, Effect[]>;
    titrationPlayback?: { vessel: number; delivered: number; total: number } | null;
    onnewvessel?: (kind: string) => void;
    onbadge?: (vessel: number, badge: { key: string; value: number; confidence: string }) => void;
    fluidLookup?: ((key: string) => import("../fluidScene").FluidSpecies) | null;
    transferFrom?: number | null;
    deployedTool?: string | null;
    deployedTarget?: number | null;
    apparatusWorking?: boolean;
    apparatusValues?: Record<string, number | string>;
    layout: BenchLayout;
    onmove?: (layout: BenchLayout) => void;
    showZones?: boolean;
    ontogglezones?: () => void;
    onremove?: (vessel: number) => void;
  } = $props();

  let choosing = $state(false);
  let dragged = $state<number | null>(null);
  let dropZone = $state<BenchZone | null>(null);
  let moveMessage = $state("");
  let messageTimer: ReturnType<typeof setTimeout> | undefined;
  const VESSEL_KINDS = ["beaker", "flask", "tube", "cylinder", "crucible"];
  const zoneHints: Record<BenchZone, string> = {
    prepare: "set up and measure",
    react: "mix and transform",
    analyse: "measure and compare",
  };
  const spatialEffects = $derived(
    Object.values(effects)
      .flat()
      .filter((effect) => effect.operation && effect.source !== undefined && effect.target !== undefined && Date.now() - effect.at < 3500),
  );

  const vesselsIn = (zone: BenchZone) => scene?.vessels.filter((v) => zoneFor(layout, v.id) === zone) ?? [];
  const latestApparatusEffect = (vessel: number, kind: string) =>
    [...(effects[vessel] ?? [])].reverse().find((effect) => effect.kind === kind);

  function move(vessel: number, zone: BenchZone) {
    onmove?.(placeVessel(layout, vessel, zone));
    onselect(vessel);
    moveMessage = t("vessel v{vessel} moved to {zone}", { vessel: vessel + 1, zone: t(zone) });
    if (messageTimer) clearTimeout(messageTimer);
    messageTimer = setTimeout(() => (moveMessage = ""), 2200);
  }

  function startDrag(event: DragEvent, vessel: number) {
    dragged = vessel;
    event.dataTransfer?.setData("application/x-kero-vessel", String(vessel));
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function acceptDrop(event: DragEvent, zone: BenchZone) {
    if (!event.dataTransfer?.types.includes("application/x-kero-vessel")) return;
    event.preventDefault();
    dropZone = zone;
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  }

  function finishDrop(event: DragEvent, zone: BenchZone) {
    const value = event.dataTransfer?.getData("application/x-kero-vessel") ?? "";
    const vessel = Number(value);
    if (Number.isInteger(vessel) && scene?.vessels.some((v) => v.id === vessel)) move(vessel, zone);
    dragged = null;
    dropZone = null;
  }
</script>

<section class="bench" aria-label={t("the bench")}>
  <div class="lab-backdrop" aria-hidden="true"></div>
  {#if ontogglezones}
    <button class="guide-toggle" aria-pressed={showZones} onclick={ontogglezones}>
      <span aria-hidden="true">{showZones ? "▦" : "☷"}</span>
      {showZones ? t("hide workflow guides") : t("show workflow guides")}
    </button>
  {/if}
  {#if scene}
    {#each spatialEffects as effect (effect.at + ":" + effect.source + ":" + effect.target + ":" + effect.operation)}
      <BenchEffect {effect} />
    {/each}
    <div class="work-zones" class:guides-off={!showZones} aria-label={t("bench work zones")}>
      {#each BENCH_ZONES as zone (zone)}
        {@const zoneVessels = vesselsIn(zone)}
        <section
          class="work-zone"
          class:drop-target={dropZone === zone}
          class:dragging={dragged !== null}
          data-zone={zone}
          aria-label={t("{zone} work zone", { zone: t(zone) })}
          ondragover={(event) => acceptDrop(event, zone)}
          ondragleave={(event) => {
            if (!event.currentTarget.contains(event.relatedTarget as Node | null)) dropZone = null;
          }}
          ondrop={(event) => finishDrop(event, zone)}
        >
          <header>
            <span class="zone-icon" aria-hidden="true">{zone === "prepare" ? "◒" : zone === "react" ? "⚡" : "⌁"}</span>
            <span><strong>{t(zone)}</strong><small>{t(zoneHints[zone])}</small></span>
            <span class="zone-count">{zoneVessels.length}</span>
          </header>
          <div class="zone-deck">
            {#each zoneVessels as vessel (vessel.id)}
              <div
                class="vessel-position"
                class:moving={dragged === vessel.id}
                draggable="true"
                role="group"
                aria-label={t("vessel v{vessel} placement", { vessel: vessel.id + 1 })}
                ondragstart={(event) => startDrag(event, vessel.id)}
                ondragend={() => { dragged = null; dropZone = null; }}
              >
                <span class="connection-port port-in" data-port="in" aria-hidden="true"></span>
                <Vessel
                  {vessel}
                  {register}
                  selected={vessel.id === selected}
                  transferTarget={transferFrom !== null && vessel.id !== transferFrom}
                  {onselect}
                  {ondropspecies}
                  effects={effects[vessel.id] ?? []}
                  titrationPlayback={titrationPlayback?.vessel === vessel.id ? titrationPlayback : null}
                  onbadge={(b) => onbadge?.(vessel.id, b)}
                  {fluidLookup}
                  deployedTool={vessel.id === deployedTarget && !["grind", "centrifuge"].includes(deployedTool ?? "") ? deployedTool : null}
                  {apparatusWorking}
                  {apparatusValues}
                />
                {#if vessel.id === deployedTarget && (deployedTool === "grind" || deployedTool === "centrifuge")}
                  {@const apparatusEffect = latestApparatusEffect(vessel.id, deployedTool)}
                  {#key apparatusEffect?.at}
                    <StandaloneApparatus
                      tool={deployedTool}
                      working={apparatusWorking}
                      performedAt={apparatusEffect?.at}
                      intensity={apparatusEffect?.magnitude ?? 0.5}
                      values={apparatusValues}
                    />
                  {/key}
                {/if}
                <span class="connection-port port-out" data-port="out" aria-hidden="true"></span>
                {#if vessel.id === selected}
                  {@const currentZone = zoneFor(layout, vessel.id)}
                  {@const leftZone = adjacentZone(currentZone, -1)}
                  {@const rightZone = adjacentZone(currentZone, 1)}
                  <div class="placement-controls" role="group" aria-label={t("move vessel v{vessel}", { vessel: vessel.id + 1 })}>
                    <button
                      disabled={leftZone === currentZone}
                      aria-label={t("move vessel v{vessel} to {zone}", { vessel: vessel.id + 1, zone: t(leftZone) })}
                      onclick={() => move(vessel.id, leftZone)}
                    >←</button>
                    <button
                      disabled={rightZone === currentZone}
                      aria-label={t("move vessel v{vessel} to {zone}", { vessel: vessel.id + 1, zone: t(rightZone) })}
                      onclick={() => move(vessel.id, rightZone)}
                    >→</button>
                    {#if onremove}
                      <button
                        class="remove"
                        aria-label={t("remove empty vessel v{vessel}", { vessel: vessel.id + 1 })}
                        title={t("remove empty vessel")}
                        onclick={() => onremove(vessel.id)}
                      >×</button>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
            {#if zone === "prepare" && onnewvessel}
              <div class="add-vessel">
                {#if choosing}
                  {#each VESSEL_KINDS as kind (kind)}
                    <button class="kind" onclick={() => { choosing = false; onnewvessel(kind); }}>{t(kind)}</button>
                  {/each}
                  <button class="kind cancel" aria-label={t("cancel")} onclick={() => (choosing = false)}>×</button>
                {:else}
                  <button class="plus" aria-label={t("add a vessel")} onclick={() => (choosing = true)}>+</button>
                {/if}
              </div>
            {/if}
          </div>
          {#if dropZone === zone}<div class="drop-callout">{t("place vessel here")}</div>{/if}
        </section>
      {/each}
    </div>
    {#if pristine}
      <p class="hint">
        {t("Drag something in from the shelf, type a command below — or pick a lesson.")}
      </p>
    {/if}
    <p class="move-status" aria-live="polite">{moveMessage}</p>
  {:else}
    <p class="empty">{t("The bench is warming up…")}</p>
  {/if}
</section>

<style>
  .bench {
    flex: 1;
    display: block;
    min-height: 24rem;
    padding: 2.7rem 0.75rem 2.6rem;
    overflow: auto;
    position: relative;
    /* The counter the glassware stands on. */
    background:
      radial-gradient(ellipse at 50% 22%, color-mix(in srgb, var(--primary) 9%, transparent), transparent 48%),
      linear-gradient(to bottom, color-mix(in srgb, var(--surface-raised) 88%, var(--surface)) 0 58%, transparent 58%),
      linear-gradient(
        to bottom,
        transparent calc(100% - 2.6rem),
        var(--bench-top, #4a4337) calc(100% - 2.6rem),
        var(--bench-top, #4a4337) calc(100% - 2.2rem),
        var(--bench-front, #3a352c) calc(100% - 2.2rem)
      );
    isolation: isolate;
  }
  .bench::after {
    content: "";
    position: absolute;
    inset: auto 0 0;
    height: 2.2rem;
    z-index: 1;
    pointer-events: none;
    background:
      linear-gradient(90deg, transparent 8%, color-mix(in srgb, var(--edge-strong) 30%, transparent) 8.2% 8.45%, transparent 8.65% 91.3%, color-mix(in srgb, var(--edge-strong) 30%, transparent) 91.55% 91.8%, transparent 92%),
      linear-gradient(to bottom, rgb(255 255 255 / 13%), transparent 30%);
  }
  .lab-backdrop {
    position: absolute;
    inset: 0 0 2.6rem;
    z-index: -1;
    overflow: hidden;
    pointer-events: none;
    background-image:
      radial-gradient(circle, color-mix(in srgb, var(--edge-strong) 16%, transparent) 1px, transparent 1.2px),
      linear-gradient(115deg, color-mix(in srgb, var(--primary) 5%, transparent), transparent 42%, color-mix(in srgb, var(--hot) 5%, transparent));
    background-size: 18px 18px, 100% 100%;
    mask-image: linear-gradient(to bottom, black, transparent 82%);
  }
  .guide-toggle {
    position: absolute;
    z-index: 9;
    top: 0.55rem;
    right: 0.7rem;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    min-height: 30px;
    padding: 0.25rem 0.55rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    background: color-mix(in srgb, var(--surface) 88%, transparent);
    font: inherit;
    font-size: 0.62rem;
    cursor: pointer;
  }
  .guide-toggle:hover { color: var(--primary); border-color: var(--primary); }
  .work-zones {
    position: relative;
    z-index: 2;
    min-height: calc(100% - 1rem);
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.45rem;
  }
  .work-zones.guides-off {
    display: flex;
    flex-wrap: wrap;
    align-content: flex-end;
    align-items: flex-end;
    justify-content: center;
    gap: clamp(0.4rem, 1.4vw, 1.2rem);
    padding: 1rem 1rem 1.8rem;
  }
  .guides-off .work-zone,
  .guides-off .zone-deck { display: contents; }
  .guides-off .work-zone > header,
  .guides-off .drop-callout { display: none; }
  .work-zone {
    position: relative;
    min-width: 0;
    min-height: 15rem;
    display: flex;
    flex-direction: column;
    border: 1px dashed color-mix(in srgb, var(--edge) 62%, transparent);
    border-bottom: 0;
    border-radius: 16px 16px 0 0;
    background: linear-gradient(to bottom, color-mix(in srgb, var(--surface) 24%, transparent), transparent 62%);
    transition: border-color 150ms ease, background-color 150ms ease, box-shadow 150ms ease;
  }
  .work-zone > header {
    min-height: 2.7rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.55rem;
    color: var(--dim);
  }
  .work-zone > header > span:nth-child(2) { min-width: 0; display: flex; flex-direction: column; }
  .work-zone > header strong {
    color: color-mix(in srgb, var(--ink) 74%, transparent);
    font-size: 0.64rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .work-zone > header small { overflow: hidden; color: var(--dim); font-size: 0.58rem; text-overflow: ellipsis; white-space: nowrap; }
  .zone-icon {
    width: 1.75rem;
    height: 1.75rem;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 9px;
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 9%, var(--surface));
  }
  .work-zone[data-zone="react"] .zone-icon { color: var(--hot); background: color-mix(in srgb, var(--hot) 10%, var(--surface)); }
  .work-zone[data-zone="analyse"] .zone-icon { color: var(--instrument); background: color-mix(in srgb, var(--instrument) 10%, var(--surface)); }
  .zone-count {
    min-width: 1.35rem;
    margin-left: auto;
    padding: 0.1rem 0.3rem;
    border-radius: 999px;
    color: var(--dim);
    background: color-mix(in srgb, var(--surface-raised) 78%, transparent);
    font-size: 0.62rem;
    text-align: center;
  }
  .zone-deck {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: center;
    gap: clamp(0.25rem, 1vw, 0.8rem);
    padding: 1rem 0.2rem 1.6rem;
  }
  .work-zone.dragging { border-color: color-mix(in srgb, var(--instrument) 48%, var(--edge)); }
  .work-zone.drop-target {
    border-color: var(--success);
    background: color-mix(in srgb, var(--success) 7%, transparent);
    box-shadow: inset 0 0 0 2px color-mix(in srgb, var(--success) 15%, transparent);
  }
  .drop-callout {
    position: absolute;
    left: 50%;
    bottom: 0.5rem;
    translate: -50% 0;
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    color: white;
    background: var(--success);
    font-size: 0.62rem;
    font-weight: 800;
    white-space: nowrap;
  }
  .vessel-position {
    position: relative;
    z-index: 2;
    cursor: grab;
    transition: opacity 140ms ease, transform 180ms ease;
  }
  .vessel-position:active { cursor: grabbing; }
  .vessel-position.moving { opacity: 0.42; transform: scale(0.96); }
  .connection-port {
    position: absolute;
    top: 52%;
    z-index: 4;
    width: 11px;
    height: 11px;
    border: 2px solid var(--surface);
    border-radius: 50%;
    background: var(--instrument);
    box-shadow: 0 0 0 1px var(--edge-strong), 0 2px 5px var(--shadow);
    opacity: 0.45;
    transition: opacity 140ms ease, transform 140ms ease;
  }
  .port-in { left: -4px; }
  .port-out { right: -4px; }
  .vessel-position:hover .connection-port,
  .vessel-position:has(:global(.vessel.selected)) .connection-port { opacity: 1; transform: scale(1.12); }
  .placement-controls {
    position: absolute;
    z-index: 8;
    left: 50%;
    bottom: -0.75rem;
    translate: -50% 0;
    display: flex;
    align-items: center;
    gap: 0.12rem;
    padding: 0.12rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    background: var(--surface);
    box-shadow: 0 5px 14px var(--shadow);
  }
  .placement-controls button {
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: 50%;
    color: white;
    background: var(--primary);
    cursor: pointer;
  }
  .placement-controls button:disabled { opacity: 0.25; cursor: default; }
  .placement-controls .remove { color: var(--bad); background: color-mix(in srgb, var(--bad) 10%, var(--surface)); }
  .move-status { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
  @media (max-width: 780px) {
    .work-zones { grid-template-columns: 1fr; }
    .work-zone { min-height: 12rem; border-bottom: 1px dashed color-mix(in srgb, var(--edge) 62%, transparent); }
    .bench { padding-top: 2.7rem; }
  }
  .empty {
    color: var(--dim);
    align-self: center;
  }
  .add-vessel {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    align-self: center;
  }
  .plus {
    width: 58px;
    height: 58px;
    border: 2px dashed color-mix(in srgb, var(--primary) 50%, var(--edge));
    border-radius: 18px;
    background: color-mix(in srgb, var(--primary) 7%, var(--surface));
    color: var(--primary);
    font-size: 1.65rem;
    cursor: pointer;
  }
  .plus:hover {
    color: var(--action);
    border-color: var(--action);
    transform: translateY(-2px);
    box-shadow: 0 8px 20px var(--shadow);
  }
  .kind {
    border: 1px solid var(--edge);
    border-radius: 6px;
    background: var(--panel);
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.3rem 0.6rem;
    cursor: pointer;
    min-height: 34px;
  }
  .kind:hover {
    border-color: var(--hot);
  }
  .cancel {
    color: var(--dim);
  }
  .hint {
    color: var(--ink);
    align-self: center;
    max-width: 18rem;
    margin-bottom: 4rem;
    padding: 0.75rem 0.9rem;
    border: 1px solid var(--edge);
    border-radius: 14px;
    background: color-mix(in srgb, var(--surface) 86%, transparent);
    box-shadow: 0 8px 24px var(--shadow);
    font-size: 0.82rem;
  }
  @media (max-height: 680px) {
    .bench { padding-top: 2.7rem; }
  }
</style>
