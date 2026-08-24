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
  }: {
    scene: Scene | null;
    register: string;
    selected: number;
    onselect: (id: number) => void;
    ondropspecies?: (id: number, payload: { key: string; phase: string }) => void;
    pristine?: boolean;
  } = $props();
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
      />
    {/each}
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
    padding: 1.5rem;
    overflow: auto;
  }
  .empty {
    color: var(--dim);
    align-self: center;
  }
  .hint {
    color: var(--dim);
    align-self: center;
    max-width: 16rem;
    font-size: 0.85rem;
  }
</style>
