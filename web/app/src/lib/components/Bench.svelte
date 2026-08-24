<script lang="ts">
  import type { Scene } from "../host/EngineHost";
  import Vessel from "./Vessel.svelte";

  let { scene, register }: { scene: Scene | null; register: string } = $props();
</script>

<section class="bench" aria-label="the bench">
  {#if scene}
    {#each scene.vessels as vessel (vessel.id)}
      <Vessel {vessel} {register} />
    {/each}
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
</style>
