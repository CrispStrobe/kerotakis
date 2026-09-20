<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import type { FeedEntry } from "../session.svelte";
  import Chart from "./Chart.svelte";
  import { engineText } from "../engineText";
  import {
    JOURNAL_WINDOW,
    displayText,
    entryVessel,
    journalEntries,
    statusIcon,
  } from "../journalFeed";

  let {
    entries,
    onaddnote,
    oneditnote,
    onremovenote,
    selectedVessel = 0,
    showTrace = $bindable(false),
    composing = $bindable(false),
  }: {
    entries: FeedEntry[];
    onaddnote?: (text: string) => void;
    oneditnote?: (createdAt: string, text: string) => void;
    onremovenote?: (createdAt: string) => void;
    selectedVessel?: number;
    /**
     * Which view the log is drawn in. GUI-107 moved the two buttons that
     * set it up into the pane heading, so the control and the thing it
     * controls now live in different components; the state stays here,
     * where the filtering it drives lives, and the heading binds to it.
     */
    showTrace?: boolean;
    /**
     * GUI: the note composer is a chevron until it is asked for.
     *
     * Expanded by default it took a textarea and a button off the top of
     * the journal on every session, and most sessions never write a note
     * at all. Deliberately NOT remembered: the journal opens on the
     * journal, every time, and a learner who wants the composer is one tap
     * from it. Bindable for the same reason as `showTrace` — the chevron
     * is in the heading, the form is here.
     */
    composing?: boolean;
  } = $props();
  let note = $state("");
  let editing = $state<string | null>(null);
  let editText = $state("");

  const visibleEntries = $derived(journalEntries(entries, { showTrace }));
  const shown = $derived(
    visibleEntries.length > JOURNAL_WINDOW ? visibleEntries.slice(-JOURNAL_WINDOW) : visibleEntries,
  );
  const trimmed = $derived(visibleEntries.length - shown.length);

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
  <!-- No row of chrome at all, since GUI-107: the view toggle and the note
       chevron went up into the pane heading (`JournalHeader.svelte`), which
       was already drawing `≡ Laborbuch 2 ›` one row above them. Two rows of
       chrome over a log is one row too many on a pane this narrow.

       What is deliberately NOT here, and did not move either:

       * the vessel scope. It filtered the log to one vessel, and the
         journal is the record of the whole bench — a bench where the
         interesting lines are precisely the ones about the OTHER vessel
         you just poured into. A filter whose best case is hiding evidence
         is not worth the row it sits on.
       * the session's status as an icon row. Those notes belong in the
         log, and they went to the header instead — which emptied the
         logbook outright for anyone who had not yet run a command, since
         on a fresh or restored bench they are the only entries there are.

       The composer FORM stays: the chevron that opens it is chrome, but the
       textarea is part of the notebook and belongs beside the entries it
       is about to join. -->
  {#if onaddnote && composing}
    <form id="journal-note-composer" class="note-composer" onsubmit={(event) => {
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
    {#if entry.kind === "nudge"}
      <p class="nudge">💡 {entry.text}</p>
    {:else if entry.kind === "claim"}
      <p class="claim">🏅 {entry.text}</p>
    {:else if entry.kind === "hazard"}
      <div class="hazard" role="alert">
        <span class="chip">{t(entry.severity || "hazard")}</span>
        {#if entry.hazardText && entry.realWorld}
          {engineText(entry.hazardText)} — {engineText(entry.realWorld)}
        {:else}
          {engineText(entry.text)}
        {/if}
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
          {#if entry.createdAt}<time datetime={entry.createdAt}>{new Date(entry.createdAt).toLocaleString(i18n.locale === "de" ? "de-DE" : "en-GB")}</time>{/if}
          {#if entry.createdAt && oneditnote}
            <button aria-label={t("edit note")} onclick={() => { editing = entry.createdAt!; editText = entry.text; }}>✎</button>
          {/if}
          {#if entry.createdAt && onremovenote}
            <button class="delete-note" aria-label={t("delete note")} onclick={() => onremovenote(entry.createdAt!)}>×</button>
          {/if}
        </header>
        {#if editing === entry.createdAt}
          <form class="note-editor" onsubmit={(event) => {
            event.preventDefault();
            if (!entry.createdAt || !editText.trim()) return;
            oneditnote?.(entry.createdAt, editText);
            editing = null;
          }}>
            <textarea rows="3" bind:value={editText} aria-label={t("edit note")}></textarea>
            <span><button type="submit" disabled={!editText.trim()}>{t("save")}</button><button type="button" onclick={() => (editing = null)}>{t("cancel")}</button></span>
          </form>
        {:else}
          <p>{entry.text}</p>
        {/if}
      </article>
    {:else}
      {@const vesselId = entryVessel(entry)}
      {@const status = statusIcon(entry)}
      <p class={entry.kind} class:status-note={status !== undefined} data-status={entry.status}>
        {#if entry.kind === "command"}<span class="prompt">kero&gt;</span>{/if}
        {#if status}<span class="status-mark" aria-hidden="true">{status}</span>{/if}
        <!-- The chip is a marker, not a filter: the vessel you have
             selected is marked so its lines are findable in a long log,
             and every other vessel's lines stay exactly where they are. -->
        {#if vesselId !== null}<span class="vessel-chip" class:current={vesselId === selectedVessel}>v{vesselId + 1}</span>{/if}
        {engineText(displayText(entry))}
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
  /* Session bookkeeping, in the log where it belongs but dressed so it
     does not read as chemistry: the icon carries the kind, the sentence
     carries the rest. */
  .status-note { color: var(--dim); font-size: 0.82em; }
  .status-mark { margin-right: 0.35rem; font-weight: 850; }
  .status-note[data-status="bench-live"] .status-mark { color: var(--good); }
  .status-note[data-status="bench-shipped"] .status-mark { color: var(--warn); }
  .status-note[data-status="restored"] .status-mark { color: var(--instrument); }
  .status-note[data-status="restore-failed"] .status-mark { color: var(--bad); }
  .status-note[data-status="cabinet-silent"] .status-mark { color: var(--bad); }
  .status-note[data-status="cabinet-answered"] .status-mark { color: var(--good); }
  /* The composer form, where the chevron in the pane heading puts it:
     at the top of the log, immediately above the entries the note is
     about to join. */
  .note-composer { display: grid; grid-template-columns: 1fr auto; gap: 0.4rem; margin-bottom: 0.35rem; }
  .note-composer textarea { resize: vertical; min-width: 0; padding: 0.5rem; border: 1px solid var(--edge); border-radius: 9px; color: var(--ink); background: var(--panel-raised); font: inherit; }
  .note-composer button { align-self: stretch; padding: 0.35rem 0.55rem; border: 0; border-radius: 9px; color: var(--on-accent); background: var(--primary); font: inherit; font-size: 0.72rem; font-weight: 750; cursor: pointer; }
  .note-composer button:disabled { opacity: 0.4; cursor: default; }
  .user-note { padding: 0.55rem; border-left: 3px solid var(--discovery); border-radius: 8px; background: color-mix(in srgb, var(--discovery) 7%, var(--surface)); }
  .user-note header { display: flex; align-items: center; gap: 0.35rem; margin-bottom: 0.25rem; color: var(--discovery); font-size: 0.62rem; }
  .user-note header time { margin-left: auto; }
  .user-note header button { width: 25px; height: 25px; display: grid; place-items: center; padding: 0; border: 1px solid var(--edge); border-radius: 7px; color: var(--dim); background: var(--surface); font: inherit; cursor: pointer; }
  .user-note header button:hover { color: var(--primary); border-color: var(--primary); }
  .user-note header .delete-note:hover { color: var(--bad); border-color: var(--bad); }
  .user-note time { color: var(--dim); font-weight: 400; }
  .note-editor { display: grid; gap: .35rem; }
  .note-editor textarea { width: 100%; box-sizing: border-box; resize: vertical; padding: .45rem; border: 1px solid var(--edge); border-radius: 8px; color: var(--ink); background: var(--surface); font: inherit; }
  .note-editor span { display: flex; gap: .35rem; justify-content: flex-end; }
  .note-editor button { padding: .25rem .5rem; border: 1px solid var(--edge); border-radius: 7px; color: var(--ink); background: var(--surface); font: inherit; font-size: .65rem; cursor: pointer; }
  .note-editor button[type="submit"] { color: var(--on-accent); border-color: var(--primary); background: var(--primary); }
  .note-editor button:disabled { opacity: .4; cursor: default; }
  p {
    margin: 0;
    white-space: pre-wrap;
    /* A German compound is longer than a phone is wide. `hyphens: auto`
       breaks it where the language allows (the document carries `lang`, set
       by i18n.svelte.ts), and `overflow-wrap: anywhere` is the fallback for
       the formulas and engine tokens no hyphenation dictionary knows. */
    hyphens: auto;
    overflow-wrap: anywhere;
  }
  .vessel-chip.current { border-color: var(--instrument); background: color-mix(in srgb, var(--instrument) 18%, var(--surface)); font-weight: 900; }
  .vessel-chip { display: inline-flex; align-items: center; justify-content: center; min-width: 1.65rem; margin-right: .35rem; padding: .06rem .3rem; border: 1px solid color-mix(in srgb, var(--instrument) 34%, var(--edge)); border-radius: 999px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 8%, var(--surface)); font-size: .62rem; font-weight: 850; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
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
    hyphens: auto;
    overflow-wrap: anywhere;
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
  .nudge {
    border-left: 3px solid var(--cool);
    padding-left: 0.5rem;
    color: var(--ink);
    font-style: italic;
  }
  .claim {
    border-left: 3px solid var(--good);
    padding-left: 0.5rem;
    color: var(--good);
  }
</style>
