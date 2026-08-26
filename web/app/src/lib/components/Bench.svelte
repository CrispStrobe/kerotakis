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
    padding: clamp(1rem, 3vw, 2.5rem) 1.5rem 0;
    overflow: auto;
    position: relative;
    /* The counter the glassware stands on. */
    background:
      radial-gradient(circle at 50% 18%, color-mix(in srgb, var(--primary) 7%, transparent), transparent 42%),
      linear-gradient(to bottom, color-mix(in srgb, var(--surface-raised) 48%, transparent), transparent 38%),
      linear-gradient(
        to bottom,
        transparent calc(100% - 2.6rem),
        var(--bench-top, #4a4337) calc(100% - 2.6rem),
        var(--bench-top, #4a4337) calc(100% - 2.2rem),
        var(--bench-front, #3a352c) calc(100% - 2.2rem)
      );
  }
  .bench > :global(.vessel) {
    margin-bottom: 1.9rem;
    position: relative;
    z-index: 2;
  }
  .work-zones {
    position: absolute;
    inset: 0.75rem 0.75rem 3.1rem;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    pointer-events: none;
    z-index: 0;
  }
  .work-zones span {
    padding: 0.2rem 0.55rem;
    border-right: 1px dashed color-mix(in srgb, var(--edge) 70%, transparent);
    color: color-mix(in srgb, var(--dim) 72%, transparent);
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .work-zones span:last-child {
    border-right: 0;
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
</style>
