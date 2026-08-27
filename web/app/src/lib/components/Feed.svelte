<script lang="ts">
  import { t } from "../i18n.svelte";
  import type { FeedEntry } from "../session.svelte";
  import Chart from "./Chart.svelte";

  let { entries, onaddnote }: { entries: FeedEntry[]; onaddnote?: (text: string) => void } = $props();
  let note = $state("");

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
<section class="feed" aria-label={t("lab notebook")} aria-live="polite" bind:this={list}>
  {#if onaddnote}
    <form class="note-composer" onsubmit={(event) => {
      event.preventDefault();
      const text = note.trim();
      if (!text) return;
      onaddnote(text);
      note = "";
    }}>
      <textarea rows="2" bind:value={note} placeholder={t("write your own observation…")} aria-label={t("new journal note")}></textarea>
      <button type="submit" disabled={!note.trim()}>{t("add note")}</button>
    </form>
  {/if}
  {#if trimmed > 0}
    <p class="note">{t("…{count} earlier entries not shown (the exports keep them)", { count: trimmed })}</p>
  {/if}
  {#each shown as entry, i (i)}
    {#if entry.kind === "hazard"}
      <div class="hazard" role="alert">
        <span class="chip">{t(entry.severity || "hazard")}</span>
        {entry.text}
      </div>
    {:else if entry.kind === "chart" && entry.chart}
      <svelte:boundary>
        <Chart spec={entry.chart} />
        {#snippet failed(error)}
          <p class="error">{t("the chart {chart} could not be drawn: {error}", { chart: entry.text, error: String(error) })}</p>
        {/snippet}
      </svelte:boundary>
    {:else if entry.kind === "user-note"}
      <article class="user-note">
        <header>
          <strong>{t("my note")}</strong>
          {#if entry.createdAt}<time datetime={entry.createdAt}>{new Date(entry.createdAt).toLocaleString()}</time>{/if}
        </header>
        <p>{entry.text}</p>
      </article>
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
    padding: 0.8rem;
    font-size: 0.88rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .note-composer { display: grid; grid-template-columns: 1fr auto; gap: 0.4rem; margin-bottom: 0.5rem; }
  .note-composer textarea { resize: vertical; min-width: 0; padding: 0.5rem; border: 1px solid var(--edge); border-radius: 9px; color: var(--ink); background: var(--panel-raised); font: inherit; }
  .note-composer button { align-self: stretch; padding: 0.35rem 0.55rem; border: 0; border-radius: 9px; color: white; background: var(--primary); font: inherit; font-size: 0.72rem; font-weight: 750; cursor: pointer; }
  .note-composer button:disabled { opacity: 0.4; cursor: default; }
  .user-note { padding: 0.55rem; border-left: 3px solid var(--discovery); border-radius: 8px; background: color-mix(in srgb, var(--discovery) 7%, var(--surface)); }
  .user-note header { display: flex; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.25rem; color: var(--discovery); font-size: 0.62rem; }
  .user-note time { color: var(--dim); font-weight: 400; }
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
    margin-top: 0.35rem;
    padding: 0.45rem 0.55rem;
    border-radius: 8px;
    color: var(--action);
    background: color-mix(in srgb, var(--action) 7%, transparent);
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
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
