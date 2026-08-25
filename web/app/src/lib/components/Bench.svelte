<script lang="ts">
  import type { Scene } from "../host/EngineHost";
  import Vessel from "./Vessel.svelte";

  let {
    scene,
    register,
    selected,
    onselect,
    ondropspecies,
    pristine = false,
    effects = {},
    onnewvessel,
    onbadge,
    fluidLookup = null,
  }: {
    scene: Scene | null;
    register: string;
    selected: number;
    onselect: (id: number) => void;
    ondropspecies?: (id: number, payload: { key: string; phase: string }) => void;
    pristine?: boolean;
    effects?: Record<number, { kind: string; at: number }[]>;
    onnewvessel?: (kind: string) => void;
    onbadge?: (vessel: number, badge: { key: string; value: number; confidence: string }) => void;
    fluidLookup?: ((key: string) => import("../fluidScene").FluidSpecies) | null;
  } = $props();

  let choosing = $state(false);
  const VESSEL_KINDS = ["beaker", "flask", "tube", "cylinder", "crucible"];
</script>

<section class="bench" aria-label="the bench">
  {#if scene}
    {#each scene.vessels as vessel (vessel.id)}
      <Vessel
        {vessel}
        {register}
        selected={vessel.id === selected}
        {onselect}
        {ondropspecies}
        effects={effects[vessel.id] ?? []}
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
              {kind}
            </button>
          {/each}
          <button class="kind cancel" onclick={() => (choosing = false)}>×</button>
        {:else}
          <button class="plus" aria-label="add a vessel" onclick={() => (choosing = true)}>
            +
          </button>
        {/if}
      </div>
    {/if}
    {#if pristine}
      <p class="hint">
        Drag something in from the shelf, type a command below — or pick a
        lesson.
      </p>
    {/if}
  {:else}
    <p class="empty">The bench is warming up…</p>
  {/if}
</section>

<style>
  .bench {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: center;
    gap: 1.5rem;
    padding: 1.5rem 1.5rem 0;
    overflow: auto;
    position: relative;
    /* The counter the glassware stands on. */
    background:
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
    width: 44px;
    height: 44px;
    border: 1px dashed var(--edge-strong);
    border-radius: 10px;
    background: none;
    color: var(--dim);
    font-size: 1.4rem;
    cursor: pointer;
  }
  .plus:hover {
    color: var(--hot);
    border-color: var(--hot);
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
    color: var(--dim);
    align-self: center;
    max-width: 16rem;
    font-size: 0.85rem;
  }
</style>
