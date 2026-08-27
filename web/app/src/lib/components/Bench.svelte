<script lang="ts">
  import type { Scene } from "../host/EngineHost";
  import type { Effect } from "../magnitudes";
  import Vessel from "./Vessel.svelte";
  import BenchEffect from "./BenchEffect.svelte";
  import StandaloneApparatus from "./StandaloneApparatus.svelte";
  import { t } from "../i18n.svelte";
  import {
    BENCH_ZONES,
    apparatusPositionFor,
    positionFor,
    positionApparatus,
    positionVessel,
    zoneAt,
    zoneFor,
    type BenchLayout,
    type BenchZone,
  } from "../benchLayout";

  let {
    scene,
    room = "discovery",
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
    onopenperiodic,
    onopencabinet,
    onopensafety,
    onopenutilities,
    missionName = null,
    missionDone = 0,
    missionTotal = 0,
    missionEvidence = false,
    onopenmission,
    onremove,
  }: {
    scene: Scene | null;
    room?: "discovery" | "research" | "orbital";
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
    onopenperiodic?: () => void;
    onopencabinet?: () => void;
    onopensafety?: () => void;
    onopenutilities?: () => void;
    missionName?: string | null;
    missionDone?: number;
    missionTotal?: number;
    missionEvidence?: boolean;
    onopenmission?: () => void;
    onremove?: (vessel: number) => void;
  } = $props();

  let choosing = $state(false);
  let workSurface = $state<HTMLDivElement | null>(null);
  let dragged = $state<number | null>(null);
  let dropZone = $state<BenchZone | null>(null);
  let dragPreview = $state<{ vessel: number; x: number; y: number } | null>(null);
  let pointerDrag: { vessel: number; pointer: number; startX: number; startY: number; moved: boolean } | null = null;
  let apparatusPreview = $state<{ tool: string; x: number; y: number } | null>(null);
  let apparatusPointer = $state<{ tool: string; pointer: number; startX: number; startY: number; moved: boolean } | null>(null);
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

  const latestApparatusEffect = (vessel: number, kind: string) =>
    [...(effects[vessel] ?? [])].reverse().find((effect) => effect.kind === kind);

  const apparatusName = (tool: string) =>
    tool === "grind" ? "mortar" : tool === "centrifuge" ? "mini centrifuge" : "burette and stand";

  const placement = (vessel: number) =>
    dragPreview?.vessel === vessel ? dragPreview : positionFor(layout, vessel);

  /** Put a freestanding machine in the clear lane above or below its target.
   * It remains visually aligned with the vessel without becoming vessel
   * contents or covering neighbouring glassware. */
  function apparatusPlacement(target: number) {
    const anchor = placement(target);
    return {
      zone: anchor.zone,
      x: anchor.x,
      y: anchor.y >= 0.5
        ? Math.max(0.12, anchor.y - 0.48)
        : Math.min(0.88, anchor.y + 0.48),
    };
  }

  function machinePlacement(tool: string, target: number) {
    if (apparatusPreview?.tool === tool) {
      return { zone: zoneAt(apparatusPreview.x), x: apparatusPreview.x, y: apparatusPreview.y };
    }
    return apparatusPositionFor(layout, tool, apparatusPlacement(target));
  }

  function surfacePosition(clientX: number, clientY: number) {
    if (!workSurface) return null;
    const rect = workSurface.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;
    return { x: (clientX - rect.left) / rect.width, y: (clientY - rect.top) / rect.height };
  }

  function placeAt(vessel: number, x: number, y: number) {
    const next = positionVessel(layout, vessel, x, y);
    onmove?.(next);
    onselect(vessel);
    moveMessage = t("vessel v{vessel} moved to {zone}", {
      vessel: vessel + 1,
      zone: t(zoneFor(next, vessel)),
    });
    if (messageTimer) clearTimeout(messageTimer);
    messageTimer = setTimeout(() => (moveMessage = ""), 2200);
  }

  function nudge(vessel: number, dx: number, dy: number) {
    const current = positionFor(layout, vessel);
    placeAt(vessel, current.x + dx, current.y + dy);
  }

  function startPointer(event: PointerEvent, vessel: number) {
    if (event.button !== 0 || (event.target as HTMLElement).closest(".placement-controls")) return;
    pointerDrag = { vessel, pointer: event.pointerId, startX: event.clientX, startY: event.clientY, moved: false };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    onselect(vessel);
  }

  function trackPointer(event: PointerEvent) {
    if (!pointerDrag || pointerDrag.pointer !== event.pointerId) return;
    if (!pointerDrag.moved && Math.hypot(event.clientX - pointerDrag.startX, event.clientY - pointerDrag.startY) < 6) return;
    pointerDrag.moved = true;
    const p = surfacePosition(event.clientX, event.clientY);
    if (!p) return;
    event.preventDefault();
    dragged = pointerDrag.vessel;
    dragPreview = { vessel: pointerDrag.vessel, ...p };
    dropZone = p.x < 1 / 3 ? "prepare" : p.x > 2 / 3 ? "analyse" : "react";
  }

  function finishPointer(event: PointerEvent) {
    if (!pointerDrag || pointerDrag.pointer !== event.pointerId) return;
    if (pointerDrag.moved && dragPreview) {
      event.preventDefault();
      placeAt(pointerDrag.vessel, dragPreview.x, dragPreview.y);
    }
    pointerDrag = null;
    dragPreview = null;
    dragged = null;
    dropZone = null;
  }

  function placeApparatusAt(tool: string, target: number, x: number, y: number) {
    const next = positionApparatus(layout, tool, x, y);
    onmove?.(next);
    onselect(target);
    moveMessage = t("{tool} moved on the bench", { tool: t(apparatusName(tool)) });
    if (messageTimer) clearTimeout(messageTimer);
    messageTimer = setTimeout(() => (moveMessage = ""), 2200);
  }

  function startApparatusPointer(event: PointerEvent, tool: string, target: number) {
    if (event.button !== 0) return;
    event.stopPropagation();
    apparatusPointer = { tool, pointer: event.pointerId, startX: event.clientX, startY: event.clientY, moved: false };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    onselect(target);
  }

  function trackApparatusPointer(event: PointerEvent) {
    if (!apparatusPointer || apparatusPointer.pointer !== event.pointerId) return;
    if (!apparatusPointer.moved && Math.hypot(event.clientX - apparatusPointer.startX, event.clientY - apparatusPointer.startY) < 6) return;
    apparatusPointer.moved = true;
    const p = surfacePosition(event.clientX, event.clientY);
    if (!p) return;
    event.preventDefault();
    const previewLayout = positionApparatus(layout, apparatusPointer.tool, p.x, p.y);
    const placed = previewLayout.apparatus[apparatusPointer.tool]!;
    apparatusPreview = { tool: apparatusPointer.tool, x: placed.x, y: placed.y };
  }

  function finishApparatusPointer(event: PointerEvent, target: number) {
    if (!apparatusPointer || apparatusPointer.pointer !== event.pointerId) return;
    if (apparatusPointer.moved && apparatusPreview) {
      event.preventDefault();
      placeApparatusAt(apparatusPointer.tool, target, apparatusPreview.x, apparatusPreview.y);
    }
    apparatusPointer = null;
    apparatusPreview = null;
  }

  function nudgeApparatus(tool: string, target: number, dx: number, dy: number) {
    const current = machinePlacement(tool, target);
    placeApparatusAt(tool, target, current.x + dx, current.y + dy);
  }

  function apparatusKeydown(event: KeyboardEvent, tool: string, target: number) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onselect(target);
      return;
    }
    const movement: Record<string, [number, number]> = {
      ArrowLeft: [-0.04, 0], ArrowRight: [0.04, 0], ArrowUp: [0, -0.05], ArrowDown: [0, 0.05],
    };
    const delta = movement[event.key];
    if (!delta) return;
    event.preventDefault();
    nudgeApparatus(tool, target, delta[0], delta[1]);
  }
</script>

<section class="bench" class:mission-active={Boolean(missionName && onopenmission)} data-room={room} aria-label={t("the bench")}>
  <div class="lab-backdrop" aria-hidden="true"></div>
  {#if onopenperiodic}
    <button
      class="wall-poster"
      aria-label={t("open periodic table")}
      onclick={onopenperiodic}
    >
      <span class="poster-grid" aria-hidden="true">
        {#each ["H", "He", "Li", "C", "N", "O", "Na", "Mg", "Cl", "Fe", "Cu", "Ag"] as symbol, index}
          <span class:hot-cell={index === 0 || index === 5} class:metal-cell={index > 5}>{symbol}</span>
        {/each}
      </span>
      <span class="poster-copy">
        <strong>{t("periodic table")}</strong>
        <small>{t("tap to explore")}</small>
      </span>
    </button>
  {/if}
  {#if onopencabinet}
    <button class="wall-cabinet" aria-label={t("open instrument cabinet")} onclick={onopencabinet}>
      <span class="cabinet-drawing" aria-hidden="true">
        <span></span><span></span><span></span><span></span>
      </span>
      <span class="poster-copy">
        <strong>{t("Instrument wall")}</strong>
        <small>{t("choose a tool")}</small>
      </span>
    </button>
  {/if}
  {#if ontogglezones}
    <button class="guide-toggle" aria-pressed={showZones} onclick={ontogglezones}>
      <span aria-hidden="true">{showZones ? "▦" : "☷"}</span>
      {showZones ? t("hide workflow guides") : t("show workflow guides")}
    </button>
  {/if}
  {#if onopensafety}
    <button class="wall-safety" aria-label={t("open safety station")} onclick={onopensafety}>
      <span class="safety-mark" aria-hidden="true">✦</span>
      <span class="poster-copy"><strong>{t("safety station")}</strong><small>{t("tap for the real-lab rules")}</small></span>
    </button>
  {/if}
  {#if onopenutilities}
    <button class="wall-utilities" aria-label={t("open utility station")} title={t("utility station")} onclick={onopenutilities}>
      <span aria-hidden="true">⌁</span><i aria-hidden="true"></i>
    </button>
  {/if}
  {#if missionName && onopenmission}
    <button
      class="mission-briefing"
      aria-label={t("open mission journal for {name}", { name: missionName })}
      onclick={onopenmission}
    >
      <span class="briefing-mark" aria-hidden="true">◆</span>
      <span class="briefing-copy">
        <small>{t("mission briefing")}</small>
        <strong>{missionName}</strong>
      </span>
      <span class="briefing-progress">
        <b>{missionDone}/{missionTotal}</b>
        <small>{t(missionEvidence ? "evidence" : "steps")}</small>
      </span>
      <span class="briefing-open" aria-hidden="true">▤</span>
    </button>
  {/if}
  {#if scene}
    {#each spatialEffects as effect (effect.at + ":" + effect.source + ":" + effect.target + ":" + effect.operation)}
      <BenchEffect {effect} />
    {/each}
    <div
      class="work-surface"
      bind:this={workSurface}
      role="group"
      aria-label={t("free-positioned laboratory bench")}
    >
      {#if showZones}
        <div class="zone-guides" aria-label={t("bench work zones")}>
          {#each BENCH_ZONES as zone (zone)}
            {@const zoneCount = scene.vessels.filter((v) => zoneFor(layout, v.id) === zone).length}
            <section
              class="work-zone"
              class:drop-target={dropZone === zone}
              data-zone={zone}
              aria-label={t("{zone} work zone", { zone: t(zone) })}
            >
              <header>
                <span class="zone-icon" aria-hidden="true">{zone === "prepare" ? "◒" : zone === "react" ? "⚡" : "⌁"}</span>
                <span><strong>{t(zone)}</strong><small>{t(zoneHints[zone])}</small></span>
                <span class="zone-count">{zoneCount}</span>
              </header>
            </section>
          {/each}
        </div>
      {/if}
      {#each scene.vessels as vessel (vessel.id)}
        {@const p = placement(vessel.id)}
        <section
          class="vessel-position"
          class:moving={dragged === vessel.id}
          style={`left:${p.x * 100}%;top:${p.y * 100}%`}
          aria-label={t("vessel v{vessel} placement", { vessel: vessel.id + 1 })}
          onpointerdown={(event) => startPointer(event, vessel.id)}
          onpointermove={trackPointer}
          onpointerup={finishPointer}
          onpointercancel={finishPointer}
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
            titrationPlayback={deployedTool !== "burette" && titrationPlayback?.vessel === vessel.id ? titrationPlayback : null}
            onbadge={(b) => onbadge?.(vessel.id, b)}
            {fluidLookup}
            deployedTool={vessel.id === deployedTarget && !["grind", "centrifuge", "burette"].includes(deployedTool ?? "") ? deployedTool : null}
            {apparatusWorking}
            {apparatusValues}
          />
          <span class="connection-port port-out" data-port="out" aria-hidden="true"></span>
          {#if vessel.id === selected}
            <div class="placement-controls" role="group" aria-label={t("move vessel v{vessel}", { vessel: vessel.id + 1 })}>
              <button aria-label={t("move vessel v{vessel} left", { vessel: vessel.id + 1 })} onclick={() => nudge(vessel.id, -0.05, 0)}>←</button>
              <button aria-label={t("move vessel v{vessel} up", { vessel: vessel.id + 1 })} onclick={() => nudge(vessel.id, 0, -0.06)}>↑</button>
              <button aria-label={t("move vessel v{vessel} down", { vessel: vessel.id + 1 })} onclick={() => nudge(vessel.id, 0, 0.06)}>↓</button>
              <button aria-label={t("move vessel v{vessel} right", { vessel: vessel.id + 1 })} onclick={() => nudge(vessel.id, 0.05, 0)}>→</button>
              {#if onremove}
                <button class="remove" aria-label={t("remove empty vessel v{vessel}", { vessel: vessel.id + 1 })} title={t("remove empty vessel")} onclick={() => onremove(vessel.id)}>×</button>
              {/if}
            </div>
          {/if}
        </section>
      {/each}
      {#if deployedTarget !== null && deployedTool && ["grind", "centrifuge", "burette"].includes(deployedTool)}
        {@const machinePosition = machinePlacement(deployedTool, deployedTarget)}
        {@const targetPosition = placement(deployedTarget)}
        {@const apparatusEffect = latestApparatusEffect(deployedTarget, deployedTool)}
        <svg
          class="apparatus-target-link"
          class:working={apparatusWorking}
          viewBox="0 0 100 100"
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          <line
            x1={machinePosition.x * 100}
            y1={machinePosition.y * 100}
            x2={targetPosition.x * 100}
            y2={targetPosition.y * 100}
          />
          <circle cx={machinePosition.x * 100} cy={machinePosition.y * 100} r="0.65" />
          <circle cx={targetPosition.x * 100} cy={targetPosition.y * 100} r="0.65" />
        </svg>
        <div
          class="apparatus-position"
          class:moving={apparatusPointer?.tool === deployedTool}
          style={`left:${machinePosition.x * 100}%;top:${machinePosition.y * 100}%`}
          role="button"
          tabindex="0"
          title={t("drag or use arrow keys to move")}
          aria-label={`${t("{tool} workstation for vessel v{vessel}", { tool: t(apparatusName(deployedTool)), vessel: deployedTarget + 1 })}. ${t("drag or use arrow keys to move")}`}
          onpointerdown={(event) => startApparatusPointer(event, deployedTool, deployedTarget)}
          onpointermove={trackApparatusPointer}
          onpointerup={(event) => finishApparatusPointer(event, deployedTarget)}
          onpointercancel={(event) => finishApparatusPointer(event, deployedTarget)}
          onkeydown={(event) => apparatusKeydown(event, deployedTool, deployedTarget)}
        >
          <span class="apparatus-grip" aria-hidden="true">⠿</span>
          {#key apparatusEffect?.at}
            <StandaloneApparatus
              tool={deployedTool}
              target={deployedTarget}
              working={apparatusWorking || (deployedTool === "burette" && titrationPlayback !== null)}
              performedAt={apparatusEffect?.at}
              intensity={apparatusEffect?.magnitude ?? 0.5}
              values={deployedTool === "burette"
                ? {
                    ...apparatusValues,
                    delivered: titrationPlayback?.delivered ?? 0,
                    total: titrationPlayback?.total ?? 0,
                  }
                : apparatusValues}
            />
          {/key}
        </div>
      {/if}
      {#if onnewvessel}
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
      {#if dragged !== null}<div class="drop-callout">{t("place vessel here")}</div>{/if}
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
  .bench.mission-active { padding-top: 6.8rem; }
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
  .bench[data-room="discovery"] {
    background:
      radial-gradient(circle at 12% 17%, color-mix(in srgb, var(--action) 16%, transparent), transparent 13rem),
      radial-gradient(circle at 88% 12%, color-mix(in srgb, var(--instrument) 17%, transparent), transparent 14rem),
      linear-gradient(to bottom, color-mix(in srgb, #fff4bd 36%, var(--surface-raised)) 0 58%, transparent 58%),
      linear-gradient(to bottom, transparent calc(100% - 2.6rem), #2c9ba2 calc(100% - 2.6rem), #2c9ba2 calc(100% - 2.2rem), #176775 calc(100% - 2.2rem));
  }
  .bench[data-room="research"] {
    background:
      radial-gradient(ellipse at 50% 22%, color-mix(in srgb, #9eb2bd 18%, transparent), transparent 48%),
      linear-gradient(to bottom, color-mix(in srgb, #e9eef1 48%, var(--surface-raised)) 0 58%, transparent 58%),
      linear-gradient(to bottom, transparent calc(100% - 2.6rem), #8297a3 calc(100% - 2.6rem), #8297a3 calc(100% - 2.2rem), #526570 calc(100% - 2.2rem));
  }
  .bench[data-room="orbital"] {
    background:
      radial-gradient(ellipse at 50% 5%, color-mix(in srgb, #25b9f1 20%, transparent), transparent 42%),
      linear-gradient(115deg, transparent 19%, color-mix(in srgb, #22baf4 12%, transparent) 19.3% 21.5%, transparent 21.8% 78%, color-mix(in srgb, #22baf4 12%, transparent) 78.3% 80.5%, transparent 80.8%),
      linear-gradient(to bottom, color-mix(in srgb, #e8f8ff 48%, var(--surface-raised)) 0 58%, transparent 58%),
      linear-gradient(to bottom, transparent calc(100% - 2.6rem), #2aaee4 calc(100% - 2.6rem), #2aaee4 calc(100% - 2.2rem), #155d85 calc(100% - 2.2rem));
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
  .wall-poster {
    position: absolute;
    z-index: 8;
    top: 0.45rem;
    left: 0.7rem;
    display: flex;
    align-items: center;
    gap: 0.55rem;
    max-width: min(14rem, 48%);
    min-height: 34px;
    padding: 0.3rem 0.55rem;
    border: 1px solid color-mix(in srgb, var(--primary) 42%, var(--edge));
    border-radius: 11px;
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    box-shadow: 0 4px 12px color-mix(in srgb, var(--shadow) 72%, transparent);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
  }
  .wall-poster:hover,
  .wall-poster:focus-visible {
    border-color: var(--primary);
    box-shadow: 0 7px 18px var(--shadow);
    transform: translateY(-1px);
  }
  .poster-grid {
    width: 3.7rem;
    flex: none;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 2px;
  }
  .poster-grid span {
    display: grid;
    place-items: center;
    aspect-ratio: 1;
    border-radius: 2px;
    color: color-mix(in srgb, var(--primary) 86%, var(--ink));
    background: color-mix(in srgb, var(--primary) 15%, var(--surface));
    font-size: 0.35rem;
    font-weight: 850;
  }
  .poster-grid .hot-cell {
    color: color-mix(in srgb, var(--hot) 88%, var(--ink));
    background: color-mix(in srgb, var(--hot) 18%, var(--surface));
  }
  .poster-grid .metal-cell {
    color: color-mix(in srgb, var(--instrument) 90%, var(--ink));
    background: color-mix(in srgb, var(--instrument) 17%, var(--surface));
  }
  .poster-copy { min-width: 0; display: flex; flex-direction: column; }
  .poster-copy strong { font-size: 0.67rem; line-height: 1.1; }
  .poster-copy small { color: var(--dim); font-size: 0.52rem; white-space: nowrap; }
  .wall-cabinet {
    position: absolute;
    z-index: 8;
    top: 0.45rem;
    left: 50%;
    translate: -50% 0;
    display: flex;
    align-items: center;
    gap: .5rem;
    min-height: 34px;
    padding: .3rem .55rem;
    border: 1px solid color-mix(in srgb, var(--action) 40%, var(--edge));
    border-radius: 10px;
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    box-shadow: 0 4px 12px color-mix(in srgb, var(--shadow) 72%, transparent);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
  }
  .wall-cabinet:hover,
  .wall-cabinet:focus-visible { border-color: var(--action); box-shadow: 0 7px 18px var(--shadow); transform: translateY(-1px); }
  .cabinet-drawing {
    width: 34px;
    height: 25px;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 2px;
    padding: 3px;
    border: 2px solid color-mix(in srgb, var(--action) 65%, var(--edge-strong));
    border-radius: 5px;
    background: color-mix(in srgb, var(--action) 9%, var(--surface));
  }
  .cabinet-drawing span { border-radius: 2px; background: color-mix(in srgb, var(--action) 45%, var(--surface)); }
  .guide-toggle {
    position: absolute;
    z-index: 9;
    right: 0.7rem;
    top: 2.9rem;
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
  .wall-safety {
    position: absolute;
    z-index: 8;
    top: .45rem;
    right: .7rem;
    display: flex;
    align-items: center;
    gap: .48rem;
    min-height: 34px;
    padding: .3rem .55rem;
    border: 1px solid color-mix(in srgb, var(--success) 48%, var(--edge));
    border-radius: 10px;
    color: var(--ink);
    background: color-mix(in srgb, var(--success) 8%, var(--surface));
    box-shadow: 0 4px 12px color-mix(in srgb, var(--shadow) 72%, transparent);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: transform 140ms ease, box-shadow 140ms ease, border-color 140ms ease;
  }
  .wall-safety:hover, .wall-safety:focus-visible { border-color: var(--success); box-shadow: 0 7px 18px var(--shadow); transform: translateY(-1px); }
  .safety-mark { width: 29px; height: 25px; display: grid; place-items: center; flex: none; border-radius: 7px; color: white; background: var(--success); font-weight: 900; }
  .wall-utilities { position: absolute; z-index: 9; top: .45rem; right: 10.3rem; width: 36px; height: 34px; display: grid; place-items: center; border: 1px solid color-mix(in srgb, var(--cool) 52%, var(--edge)); border-radius: 10px; color: var(--instrument); background: color-mix(in srgb, var(--cool) 11%, var(--surface)); box-shadow: 0 4px 12px color-mix(in srgb, var(--shadow) 72%, transparent); cursor: pointer; font: inherit; font-size: 1.05rem; }
  .wall-utilities i { position: absolute; right: 5px; bottom: 5px; width: 6px; height: 6px; border-radius: 50%; background: var(--action); box-shadow: 0 0 0 2px var(--surface); }
  .wall-utilities:hover, .wall-utilities:focus-visible { border-color: var(--cool); transform: translateY(-1px); }
  .mission-briefing {
    position: absolute;
    z-index: 8;
    top: 3.05rem;
    left: 50%;
    translate: -50% 0;
    width: min(23rem, 48%);
    min-height: 52px;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto auto;
    align-items: center;
    gap: .55rem;
    padding: .42rem .55rem;
    border: 1px solid color-mix(in srgb, var(--discovery) 52%, var(--edge));
    border-radius: 13px;
    color: var(--ink);
    background:
      linear-gradient(100deg, color-mix(in srgb, var(--discovery) 15%, var(--surface)), color-mix(in srgb, var(--action) 8%, var(--surface)));
    box-shadow: 0 6px 17px color-mix(in srgb, var(--discovery) 18%, var(--shadow));
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
  }
  .mission-briefing:hover,
  .mission-briefing:focus-visible {
    border-color: var(--discovery);
    box-shadow: 0 9px 23px color-mix(in srgb, var(--discovery) 25%, var(--shadow));
    transform: translateY(-1px);
  }
  .briefing-mark {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 11px;
    color: white;
    background: linear-gradient(145deg, var(--discovery), color-mix(in srgb, var(--discovery) 55%, var(--action)));
    box-shadow: 0 5px 12px color-mix(in srgb, var(--discovery) 28%, transparent);
  }
  .briefing-copy { min-width: 0; display: grid; gap: .08rem; }
  .briefing-copy small { color: var(--discovery); font-size: .5rem; font-weight: 850; letter-spacing: .1em; text-transform: uppercase; }
  .briefing-copy strong { overflow: hidden; font-size: .68rem; text-overflow: ellipsis; white-space: nowrap; }
  .briefing-progress { display: grid; justify-items: end; line-height: 1; }
  .briefing-progress b { color: var(--discovery); font-size: .7rem; }
  .briefing-progress small { margin-top: .2rem; color: var(--dim); font-size: .48rem; text-transform: uppercase; }
  .briefing-open { color: var(--discovery); font-size: 1rem; }
  .work-surface {
    position: relative;
    z-index: 2;
    width: 100%;
    min-width: 42rem;
    min-height: max(29rem, calc(100% - 1rem));
    overflow: hidden;
    border-radius: 16px 16px 0 0;
  }
  .zone-guides {
    position: absolute;
    inset: 0;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.45rem;
    pointer-events: none;
  }
  .work-zone {
    position: relative;
    min-width: 0;
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
  .work-zone.drop-target {
    border-color: var(--success);
    background: color-mix(in srgb, var(--success) 7%, transparent);
    box-shadow: inset 0 0 0 2px color-mix(in srgb, var(--success) 15%, transparent);
  }
  .drop-callout {
    position: absolute;
    left: 50%;
    bottom: 0.65rem;
    z-index: 10;
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
    position: absolute;
    z-index: 3;
    translate: -50% -50%;
    cursor: grab;
    touch-action: none;
    user-select: none;
    transition: opacity 140ms ease, transform 180ms ease;
  }
  .vessel-position:has(:global(.vessel.selected)) { z-index: 6; }
  .vessel-position:active { cursor: grabbing; }
  .vessel-position.moving { opacity: 0.42; transform: scale(0.96); }
  .apparatus-position {
    position: absolute;
    z-index: 5;
    width: 112px;
    translate: -50% -50%;
    cursor: grab;
    touch-action: none;
    user-select: none;
  }
  .apparatus-position:focus-visible {
    outline: 3px solid color-mix(in srgb, var(--primary) 72%, white);
    outline-offset: 4px;
    border-radius: 16px;
  }
  .apparatus-position:active { cursor: grabbing; }
  .apparatus-position.moving { opacity: 0.62; }
  .apparatus-grip {
    position: absolute;
    z-index: 2;
    top: 0.34rem;
    right: 0.38rem;
    display: grid;
    width: 1.25rem;
    height: 1.25rem;
    place-items: center;
    border: 1px solid color-mix(in srgb, var(--instrument) 36%, var(--edge));
    border-radius: 7px;
    color: var(--instrument);
    background: color-mix(in srgb, var(--surface) 88%, var(--instrument));
    box-shadow: 0 2px 5px var(--shadow);
    font-size: 0.85rem;
    line-height: 1;
  }
  .apparatus-target-link {
    position: absolute;
    inset: 0;
    z-index: 2;
    width: 100%;
    height: 100%;
    overflow: visible;
    pointer-events: none;
  }
  .apparatus-target-link line {
    fill: none;
    stroke: color-mix(in srgb, var(--instrument) 68%, var(--edge-strong));
    stroke-width: 2.2;
    stroke-dasharray: 3 7;
    stroke-linecap: round;
    opacity: 0.62;
    vector-effect: non-scaling-stroke;
  }
  .apparatus-target-link circle {
    fill: var(--surface);
    stroke: var(--instrument);
    stroke-width: 2;
    vector-effect: non-scaling-stroke;
  }
  .apparatus-target-link.working line { animation: route-pulse 0.75s linear infinite; }
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
    top: -0.65rem;
    right: -0.55rem;
    display: flex;
    align-items: center;
    gap: 0.1rem;
    padding: 0.14rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    background: var(--surface);
    box-shadow: 0 5px 14px var(--shadow);
  }
  .placement-controls button {
    width: 22px;
    height: 22px;
    border: 0;
    border-radius: 50%;
    color: white;
    background: var(--primary);
    cursor: pointer;
    font-size: 0.7rem;
    line-height: 1;
  }
  @keyframes route-pulse { to { stroke-dashoffset: -10; } }
  .placement-controls button:disabled { opacity: 0.25; cursor: default; }
  .placement-controls .remove { color: var(--bad); background: color-mix(in srgb, var(--bad) 10%, var(--surface)); }
  .move-status { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
  @media (max-width: 780px) {
    .bench { padding-top: 2.7rem; }
    .bench.mission-active { padding-top: 6.6rem; }
    .poster-copy { display: none; }
    .wall-poster, .wall-cabinet, .wall-safety { max-width: none; padding-inline: 0.4rem; }
    .wall-utilities { right: 3.75rem; }
    .mission-briefing { width: min(20rem, 58%); }
    .briefing-copy small, .briefing-progress small { display: none; }
  }
  .empty {
    color: var(--dim);
    align-self: center;
  }
  .add-vessel {
    position: absolute;
    z-index: 7;
    left: 0.8rem;
    bottom: 0.9rem;
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
    position: absolute;
    z-index: 7;
    right: 1rem;
    bottom: 3.4rem;
    color: var(--ink);
    max-width: 18rem;
    margin: 0;
    padding: 0.75rem 0.9rem;
    border: 1px solid var(--edge);
    border-radius: 14px;
    background: color-mix(in srgb, var(--surface) 86%, transparent);
    box-shadow: 0 8px 24px var(--shadow);
    font-size: 0.82rem;
  }
  @media (max-width: 640px) {
    .hint { right: 0.65rem; bottom: 3rem; max-width: min(17rem, calc(100% - 5rem)); font-size: 0.72rem; }
  }
  @media (max-height: 680px) {
    .bench { padding-top: 2.7rem; }
    .bench.mission-active { padding-top: 6.3rem; }
  }
  @media (prefers-reduced-motion: reduce) {
    .apparatus-target-link.working line { animation: none; }
  }
</style>
