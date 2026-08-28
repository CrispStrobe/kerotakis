<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import type { FeedEntry } from "../session.svelte";
  import Chart from "./Chart.svelte";
  import { engineText } from "../engineText";

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
  let scope = $state<"all" | "selected">("all");
  let editing = $state<string | null>(null);
  let editText = $state("");

  // Render only the tail of a very long session: the exports keep every
  // entry, the DOM does not have to (low-end budget). 400 entries is far
  // beyond what a screen shows and well within what a Chromebook lays out.
  const WINDOW = 400;
  function entryVessel(entry: FeedEntry): number | null {
    if (!["command", "line", "error", "refusal"].includes(entry.kind)) return null;
    const match = entry.text.match(entry.kind === "command" ? /\bv(\d+)\b/i : /^\s*v(\d+)\s*:/i);
    return match ? Number(match[1]) - 1 : null;
  }
  function displayText(entry: FeedEntry): string {
    return entry.kind === "line" ? entry.text.replace(/^\s*v\d+\s*:\s*/i, "") : entry.text;
  }
  const visibleEntries = $derived(
    entries.filter((entry) => {
      if (!showTrace && entry.kind === "command") return false;
      const vessel = entryVessel(entry);
      return scope === "all" || vessel === null || vessel === selectedVessel;
    }),
  );
  const shown = $derived(visibleEntries.length > WINDOW ? visibleEntries.slice(-WINDOW) : visibleEntries);
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
  <div class="journal-view" role="group" aria-label={t("journal view") }>
    <button aria-pressed={!showTrace} class:active={!showTrace} onclick={() => (showTrace = false)}>
      {t("observations")}
    </button>
    <button aria-pressed={showTrace} class:active={showTrace} onclick={() => (showTrace = true)}>
      {t("full trace")}
      {#if hiddenCommands > 0}<span>{hiddenCommands}</span>{/if}
    </button>
  </div>
  <div class="journal-scope" role="group" aria-label={t("journal scope") }>
    <span>{t("show entries for")}</span>
    <button aria-pressed={scope === "all"} class:active={scope === "all"} onclick={() => (scope = "all")}>{t("whole lab")}</button>
    <button aria-pressed={scope === "selected"} class:active={scope === "selected"} onclick={() => (scope = "selected")}>
      {t("selected v{vessel}", { vessel: selectedVessel + 1 })}
    </button>
  </div>
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
      <p class={entry.kind}>
        {#if entry.kind === "command"}<span class="prompt">kero&gt;</span>{/if}
        {#if vesselId !== null}<span class="vessel-chip">v{vesselId + 1}</span>{/if}
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
  .journal-view { position: sticky; top: 0; z-index: 2; display: grid; grid-template-columns: 1fr 1fr; gap: 2px; padding: 3px; border: 1px solid var(--edge); border-radius: 10px; background: color-mix(in srgb, var(--surface) 94%, transparent); backdrop-filter: blur(10px); }
  .journal-view button { min-height: 30px; display: flex; align-items: center; justify-content: center; gap: .35rem; border: 0; border-radius: 7px; color: var(--dim); background: transparent; font: inherit; font-size: .66rem; font-weight: 750; cursor: pointer; }
  .journal-view button.active { color: var(--primary); background: color-mix(in srgb, var(--primary) 10%, var(--surface-raised)); }
  .journal-view span { min-width: 1.2rem; padding: .08rem .25rem; border-radius: 999px; color: var(--dim); background: var(--surface); font-size: .52rem; }
  .journal-scope { display: grid; grid-template-columns: 1fr auto auto; align-items: center; gap: .25rem; padding: .28rem .35rem; color: var(--dim); font-size: .58rem; }
  .journal-scope > span { padding-left: .2rem; font-weight: 700; }
  .journal-scope button { min-height: 27px; padding: .2rem .45rem; border: 1px solid transparent; border-radius: 999px; color: var(--dim); background: transparent; font: inherit; font-weight: 750; cursor: pointer; }
  .journal-scope button.active { color: var(--instrument); border-color: color-mix(in srgb, var(--instrument) 35%, var(--edge)); background: color-mix(in srgb, var(--instrument) 8%, var(--surface)); }
  .note-composer { display: grid; grid-template-columns: 1fr auto; gap: 0.4rem; margin-bottom: 0.5rem; }
  .note-composer textarea { resize: vertical; min-width: 0; padding: 0.5rem; border: 1px solid var(--edge); border-radius: 9px; color: var(--ink); background: var(--panel-raised); font: inherit; }
  .note-composer button { align-self: stretch; padding: 0.35rem 0.55rem; border: 0; border-radius: 9px; color: white; background: var(--primary); font: inherit; font-size: 0.72rem; font-weight: 750; cursor: pointer; }
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
  .note-editor button[type="submit"] { color: white; border-color: var(--primary); background: var(--primary); }
  .note-editor button:disabled { opacity: .4; cursor: default; }
  p {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }
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
