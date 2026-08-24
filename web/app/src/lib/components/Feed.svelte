<script lang="ts">
  import type { FeedEntry } from "../session.svelte";
  import Chart from "./Chart.svelte";

  let { entries }: { entries: FeedEntry[] } = $props();

  // Render only the tail of a very long session: the exports keep every
  // entry, the DOM does not have to (low-end budget). 400 entries is far
  // beyond what a screen shows and well within what a Chromebook lays out.
  const WINDOW = 400;
  const shown = $derived(entries.length > WINDOW ? entries.slice(-WINDOW) : entries);
  const trimmed = $derived(entries.length - shown.length);

  let list: HTMLElement | undefined = $state();
  $effect(() => {
    // Track length so new entries keep the latest line in view.
    void entries.length;
    list?.scrollTo({ top: list.scrollHeight });
  });
</script>

<!-- The feed is the notebook and the screen-reader surface: everything the
     bench does is a legible line here, announced as it happens. -->
<section class="feed" aria-label="lab notebook" aria-live="polite" bind:this={list}>
  {#if trimmed > 0}
    <p class="note">…{trimmed} earlier entries not shown (the exports keep them)</p>
  {/if}
  {#each shown as entry, i (i)}
    {#if entry.kind === "hazard"}
      <div class="hazard" role="alert">
        <span class="chip">{entry.severity || "hazard"}</span>
        {entry.text}
      </div>
    {:else if entry.kind === "chart" && entry.chart}
      <Chart spec={entry.chart} />
    {:else}
      <p class={entry.kind}>
        {#if entry.kind === "command"}<span class="prompt">kero&gt;</span>{/if}
        {entry.text}
      </p>
    {/if}
  {/each}
</section>

<style>
  .feed {
    overflow-y: auto;
    padding: 1rem;
    font-size: 0.88rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  p {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .prompt {
    color: var(--hot);
    margin-right: 0.4rem;
  }
  .command {
    color: var(--hot);
  }
  .note {
    color: var(--dim);
  }
  .error {
    color: var(--bad);
  }
  .refusal {
    color: var(--warn);
    border-left: 3px solid var(--warn);
    padding-left: 0.6rem;
  }
  .hazard {
    border: 1px solid var(--warn);
    border-left-width: 4px;
    border-radius: 6px;
    padding: 0.5rem 0.7rem;
    margin: 0.2rem 0;
    background: var(--panel-raised);
  }
  .hazard .chip {
    display: inline-block;
    background: var(--warn);
    color: var(--bg);
    border-radius: 999px;
    font-size: 0.7rem;
    padding: 0 0.5rem;
    margin-right: 0.5rem;
    text-transform: lowercase;
  }
</style>
