<script lang="ts">
  import type { FeedEntry } from "../session.svelte";

  let { entries }: { entries: FeedEntry[] } = $props();

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
  {#each entries as entry, i (i)}
    <p class={entry.kind}>
      {#if entry.kind === "command"}<span class="prompt">kero&gt;</span>{/if}
      {entry.text}
    </p>
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
</style>
