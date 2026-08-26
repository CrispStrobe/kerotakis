<script lang="ts">
  import type { Scene } from "../host/EngineHost";
  import type { Effect } from "../magnitudes";
  import Vessel from "./Vessel.svelte";
  import { t } from "../i18n.svelte";

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
  } = $props();

  let choosing = $state(false);
  const VESSEL_KINDS = ["beaker", "flask", "tube", "cylinder", "crucible"];
</script>

<section class="bench" aria-label={t("the bench")}>
  <div class="lab-backdrop" aria-hidden="true">
    <span class="light light-a"></span>
    <span class="light light-b"></span>
    <span class="service-rail"></span>
    <span class="socket socket-a"></span>
    <span class="socket socket-b"></span>
    <span class="shelf-line"></span>
  </div>
  <div class="work-zones" aria-label={t("bench work zones")}>
    <span>{t("prepare")}</span>
    <span>{t("react")}</span>
    <span>{t("analyse")}</span>
  </div>
  {#if scene}
    {#each scene.vessels as vessel (vessel.id)}
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
      />
    {/each}
    {#if onnewvessel}
      <div class="add-vessel">
        {#if choosing}
          {#each VESSEL_KINDS as kind (kind)}
            <button
              class="kind"
              onclick={() => {
                choosing = false;
                onnewvessel(kind);
              }}
            >
              {t(kind)}
            </button>
          {/each}
          <button class="kind cancel" onclick={() => (choosing = false)}>×</button>
        {:else}
          <button class="plus" aria-label={t("add a vessel")} onclick={() => (choosing = true)}>
            +
          </button>
        {/if}
      </div>
    {/if}
    {#if pristine}
      <p class="hint">
        {t("Drag something in from the shelf, type a command below — or pick a lesson.")}
      </p>
    {/if}
  {:else}
    <p class="empty">{t("The bench is warming up…")}</p>
  {/if}
</section>

<style>
  .bench {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: center;
    gap: clamp(1rem, 3vw, 2.25rem);
    padding: clamp(4.5rem, 12vh, 7.5rem) 1.5rem 0;
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
    background-image: radial-gradient(circle, color-mix(in srgb, var(--edge-strong) 16%, transparent) 1px, transparent 1.2px);
    background-size: 18px 18px;
    mask-image: linear-gradient(to bottom, black, transparent 72%);
  }
  .light {
    position: absolute;
    top: 0.8rem;
    width: min(24vw, 15rem);
    height: 0.45rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 38%, white);
    box-shadow: 0 0 18px color-mix(in srgb, var(--primary) 30%, transparent), 0 22px 55px color-mix(in srgb, var(--primary) 12%, transparent);
  }
  .light-a { left: 15%; }
  .light-b { right: 15%; }
  .service-rail {
    position: absolute;
    top: 3.25rem;
    left: 6%;
    right: 6%;
    height: 0.48rem;
    border: 1px solid color-mix(in srgb, var(--edge-strong) 50%, var(--edge));
    border-radius: 999px;
    background: color-mix(in srgb, var(--surface) 55%, var(--edge));
    box-shadow: 0 4px 8px var(--shadow);
  }
  .socket {
    position: absolute;
    top: 2.65rem;
    width: 1.7rem;
    height: 1.7rem;
    border: 3px solid var(--surface);
    border-radius: 8px;
    background: var(--edge-strong);
    box-shadow: 0 3px 9px var(--shadow);
  }
  .socket::after { content: ""; position: absolute; inset: 0.42rem; border-radius: 50%; background: var(--surface); }
  .socket-a { left: 22%; }
  .socket-b { right: 22%; }
  .shelf-line {
    position: absolute;
    top: 5.2rem;
    left: 12%;
    right: 12%;
    height: 0.55rem;
    border-radius: 0 0 8px 8px;
    background: color-mix(in srgb, var(--edge-strong) 42%, var(--surface));
    box-shadow: 0 7px 12px var(--shadow);
  }
  .bench > :global(.vessel) {
    margin-bottom: 1.9rem;
    position: relative;
    z-index: 2;
  }
  .work-zones {
    position: absolute;
    inset: 6.5rem 0.75rem 3.1rem;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    pointer-events: none;
    z-index: 0;
  }
  .work-zones span {
    margin: 0 0.28rem;
    padding: 0.45rem 0.65rem;
    border: 1px dashed color-mix(in srgb, var(--edge) 62%, transparent);
    border-bottom: 0;
    border-radius: 16px 16px 0 0;
    color: color-mix(in srgb, var(--dim) 72%, transparent);
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .work-zones span:last-child {
    border-right: 1px dashed color-mix(in srgb, var(--edge) 62%, transparent);
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
    .bench { padding-top: 3.6rem; }
    .work-zones { inset-block-start: 4.8rem; }
    .shelf-line { display: none; }
  }
</style>
