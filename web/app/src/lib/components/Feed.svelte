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
  }: {
    entries: FeedEntry[];
    onaddnote?: (text: string) => void;
    oneditnote?: (createdAt: string, text: string) => void;
    onremovenote?: (createdAt: string) => void;
    selectedVessel?: number;
  } = $props();
  let note = $state("");
  let showTrace = $state(false);
  let editing = $state<string | null>(null);
  let editText = $state("");
  /**
   * GUI: the note composer is a chevron until it is asked for.
   *
   * Expanded by default it took a textarea and a button off the top of the
   * journal on every session, and most sessions never write a note at all.
   * Deliberately NOT remembered: the journal opens on the journal, every
   * time, and a learner who wants the composer is one tap from it.
   */
  let composing = $state(false);

  /**
   * The header's one line of tooltip.
   *
   * `title=` covers a mouse and nothing else: it never reaches a finger,
   * and it is not reliably surfaced on keyboard focus. So the label is
   * state — pointer and focus show it, a tap pins it (a tap is the only
   * "hover" a touch screen has), and the pin times out so it can never sit
   * over the log. `title` stays on every control as the native fallback.
   */
  let tip = $state<string | null>(null);
  let pinned: ReturnType<typeof setTimeout> | undefined;
  function hint(text: string | null): void {
    clearTimeout(pinned);
    tip = text;
  }
  function pin(text: string): void {
    hint(text);
    pinned = setTimeout(() => {
      if (tip === text) tip = null;
    }, 2600);
  }
  $effect(() => () => clearTimeout(pinned));

  const visibleEntries = $derived(journalEntries(entries, { showTrace }));
  const shown = $derived(
    visibleEntries.length > JOURNAL_WINDOW ? visibleEntries.slice(-JOURNAL_WINDOW) : visibleEntries,
  );
  const trimmed = $derived(visibleEntries.length - shown.length);
  const hiddenCommands = $derived(entries.filter((entry) => entry.kind === "command").length);

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
  <!-- One row of chrome, and never more: the view toggle and the note
       chevron. Every caption this used to stack is a tooltip, so the
       journal starts at the top of the pane instead of five rows down it.
       What is deliberately NOT here any more:

       * the vessel scope. It filtered the log to one vessel, and the
         journal is the record of the whole bench — a bench where the
         interesting lines are precisely the ones about the OTHER vessel
         you just poured into. A filter whose best case is hiding evidence
         is not worth the row it sits on.
       * the session's status as an icon row. Those notes belong in the
         log, and they went to the header instead — which emptied the
         logbook outright for anyone who had not yet run a command, since
         on a fresh or restored bench they are the only entries there are.
  -->
  <header class="journal-head">
      <div class="journal-view" role="group" aria-label={t("journal view")}>
        <button
          type="button"
          class="icon-btn"
          aria-pressed={!showTrace}
          class:active={!showTrace}
          aria-label={t("observations")}
          title={t("observations")}
          onpointerenter={() => hint(t("observations"))}
          onpointerleave={() => hint(null)}
          onfocus={() => hint(t("observations"))}
          onblur={() => hint(null)}
          onclick={() => {
            showTrace = false;
            pin(t("observations"));
          }}
        ><span aria-hidden="true">≡</span></button>
        <button
          type="button"
          class="icon-btn"
          aria-pressed={showTrace}
          class:active={showTrace}
          aria-label={t("full trace")}
          title={t("full trace")}
          onpointerenter={() => hint(t("full trace"))}
          onpointerleave={() => hint(null)}
          onfocus={() => hint(t("full trace"))}
          onblur={() => hint(null)}
          onclick={() => {
            showTrace = true;
            pin(t("full trace"));
          }}
        >
          <span aria-hidden="true">&gt;_</span>
          {#if hiddenCommands > 0}<span class="count" aria-hidden="true">{hiddenCommands}</span>{/if}
        </button>
      </div>
    {#if onaddnote}
      <div class="note-composer">
        <button
          type="button"
          class="icon-btn composer-toggle"
          aria-expanded={composing}
          aria-controls="journal-note-composer"
          aria-label={t("add note")}
          title={t("add note")}
          onpointerenter={() => hint(t("add note"))}
          onpointerleave={() => hint(null)}
          onfocus={() => hint(t("add note"))}
          onblur={() => hint(null)}
          onclick={() => {
            composing = !composing;
            pin(t("add note"));
          }}
        ><span aria-hidden="true">{composing ? "⌄" : "›"}</span></button>
        {#if composing}
          <form id="journal-note-composer" onsubmit={(event) => {
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
      </div>
    {/if}
    {#if tip}<p class="tip" aria-hidden="true">{tip}</p>{/if}
  </header>
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
  /* Static, not sticky: the tab bar used to float over the log it
     switches between, covering the newest lines as you scrolled. It is a
     heading for the segment below it, so it scrolls with it. */
  .journal-head {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.22rem;
    margin-bottom: 0.35rem;
  }
  .journal-view {
    align-self: flex-start;
    display: flex;
    gap: 2px;
    padding: 3px;
    border: 1px solid var(--edge);
    border-radius: 10px;
    background: color-mix(in srgb, var(--surface) 94%, transparent);
  }
  .icon-btn {
    position: relative;
    min-width: 2.1rem;
    min-height: 30px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.22rem;
    padding: 0 0.3rem;
    border: 0;
    border-radius: 7px;
    color: var(--dim);
    background: transparent;
    font: inherit;
    font-size: 0.74rem;
    font-weight: 750;
    line-height: 1;
    cursor: pointer;
  }
  .icon-btn:hover { color: var(--ink); }
  .journal-view .icon-btn.active { color: var(--primary); background: color-mix(in srgb, var(--primary) 10%, var(--surface-raised)); }
  .count { min-width: 1.1rem; padding: 0.04rem 0.22rem; border-radius: 999px; color: var(--dim); background: var(--surface); font-size: 0.52rem; }
  /* Session bookkeeping, in the log where it belongs but dressed so it
     does not read as chemistry: the icon carries the kind, the sentence
     carries the rest. */
  .status-note { color: var(--dim); font-size: 0.82em; }
  .status-mark { margin-right: 0.35rem; font-weight: 850; }
  .status-note[data-status="bench-live"] .status-mark { color: var(--good); }
  .status-note[data-status="bench-shipped"] .status-mark { color: var(--warn); }
  .status-note[data-status="restored"] .status-mark { color: var(--instrument); }
  .status-note[data-status="restore-failed"] .status-mark { color: var(--bad); }
  /* Anchored to the header and inset on both sides, so a long sentence can
     never widen the pane: it wraps inside the tooltip instead. */
  .tip {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 5;
    margin: 0.15rem 0 0;
    padding: 0.3rem 0.45rem;
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--ink);
    background: var(--surface-raised);
    box-shadow: 0 6px 18px var(--shadow);
    font-size: 0.62rem;
    font-weight: 600;
    line-height: 1.3;
    pointer-events: none;
  }
  .note-composer { display: flex; flex-direction: column; gap: 0.3rem; }
  .composer-toggle { align-self: flex-start; min-width: 1.9rem; }
  .note-composer form { display: grid; grid-template-columns: 1fr auto; gap: 0.4rem; }
  .note-composer textarea { resize: vertical; min-width: 0; padding: 0.5rem; border: 1px solid var(--edge); border-radius: 9px; color: var(--ink); background: var(--panel-raised); font: inherit; }
  .note-composer form button { align-self: stretch; padding: 0.35rem 0.55rem; border: 0; border-radius: 9px; color: var(--on-accent); background: var(--primary); font: inherit; font-size: 0.72rem; font-weight: 750; cursor: pointer; }
  .note-composer form button:disabled { opacity: 0.4; cursor: default; }
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
