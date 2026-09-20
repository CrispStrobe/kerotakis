<script lang="ts">
  /**
   * GUI-107 — the journal's chrome, in one row instead of two.
   *
   * The pane spent two rows above the log saying who it was. The first was
   * the pane heading — `≡ Laborbuch 2 ›` — and the second was the journal's
   * own control row inside the feed: the view toggle (`≡` observations,
   * `>_` full trace) and the note composer's chevron. Two rows of chrome
   * over a pane that is 18 rem wide on a desktop and the whole screen on a
   * phone, where the only thing worth looking at is the log underneath.
   *
   * So the heading IS the control row. The order left to right is identity
   * (icon, name, how many entries), then what to look at (the two view
   * buttons), then what to add (the composer chevron), then the pane's own
   * collapse — which is the reading order and the tab order both.
   *
   * It is a component rather than markup in `App.svelte` for one reason:
   * the buttons carry the journal's pinned-tip behaviour, and that state
   * belonged to `Feed.svelte`. Moving the buttons into the shell would have
   * meant either copying the tip machinery into a 2800-line file or
   * dropping it, and a `title=` is a tooltip a finger can never reach.
   *
   * `showTrace` and `composing` are `$bindable` because the CONTROL is here
   * and the thing it controls — the log and the composer form — is in
   * `Feed.svelte` below. Nothing else about the journal moved.
   */
  import { t } from "../i18n.svelte";

  let {
    entryCount,
    hiddenCommands,
    collapsed,
    canCompose,
    showTrace = $bindable(false),
    composing = $bindable(false),
    oncollapse,
  }: {
    /** Everything the notebook holds, trace lines included. */
    entryCount: number;
    /** How many of those are commands — the badge on the trace view. */
    hiddenCommands: number;
    collapsed: boolean;
    /** False when the shell passes no note handler: no chevron, no form. */
    canCompose: boolean;
    showTrace?: boolean;
    composing?: boolean;
    oncollapse: () => void;
  } = $props();

  /**
   * The row's one line of tooltip, moved here with the buttons that raise
   * it. `title=` covers a mouse and nothing else: it never reaches a
   * finger, and it is not reliably surfaced on keyboard focus. So the
   * label is state — pointer and focus show it, a tap pins it (a tap is
   * the only "hover" a touch screen has), and the pin times out so it can
   * never sit over the log. `title` stays on every control as the native
   * fallback.
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
</script>

<div class="pane-heading journal-heading">
  <span class="pane-icon" aria-hidden="true">≡</span>
  <strong class="pane-title">{t("lab journal")}</strong>
  <span class="entry-count" title={t("notebook entries")}>{entryCount}</span>
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
  {#if canCompose}
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
  {/if}
  <button
    class="panel-collapse"
    aria-expanded={!collapsed}
    aria-label={collapsed ? t("open lab journal") : t("collapse lab journal")}
    title={collapsed ? t("open lab journal") : t("collapse lab journal")}
    onclick={oncollapse}
  >{collapsed ? "‹" : "›"}</button>
  {#if tip}<p class="tip" aria-hidden="true">{tip}</p>{/if}
</div>

<style>
  /* The shell's pane heading, restated here because Svelte scopes styles
     to the component that writes the markup. The shelf pane still uses the
     shell's copy; this one carries the journal's controls as well, so it
     also owns the rules that keep them on ONE line. */
  .pane-heading {
    position: relative;
    min-height: 44px;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.6rem;
    border-bottom: 1px solid var(--edge);
  }
  .pane-icon {
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    flex: none;
    border-radius: 10px;
    color: var(--discovery);
    background: color-mix(in srgb, var(--discovery) 10%, var(--surface-raised));
    font-size: 0.95rem;
    font-weight: 800;
  }
  /* The one elastic cell. Everything else in the row is a control with a
     fixed hit area, so when the pane narrows it is the WORD that gives way
     — the controls never wrap into the second row this change removed. */
  .pane-title {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    font-size: 0.86rem;
    line-height: 1.2;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .entry-count {
    min-width: 1.5rem;
    flex: none;
    padding: 0.18rem 0.32rem;
    border-radius: 999px;
    color: var(--dim);
    background: var(--surface-raised);
    font-size: 0.66rem;
    text-align: center;
  }
  .journal-view {
    display: flex;
    flex: none;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--edge);
    border-radius: 10px;
    background: color-mix(in srgb, var(--surface) 94%, transparent);
  }
  .icon-btn {
    position: relative;
    min-width: 2rem;
    min-height: 32px;
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    gap: 0.22rem;
    padding: 0 0.28rem;
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
  .composer-toggle {
    border: 1px solid var(--edge);
    border-radius: 9px;
    background: var(--surface-raised);
  }
  .composer-toggle[aria-expanded="true"] { color: var(--primary); border-color: var(--primary); }
  .count { min-width: 1.1rem; padding: 0.04rem 0.22rem; border-radius: 999px; color: var(--dim); background: var(--surface); font-size: 0.52rem; }
  .panel-collapse {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    flex: none;
    padding: 0;
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--dim);
    background: var(--surface-raised);
    font: inherit;
    font-size: 1rem;
    cursor: pointer;
  }
  .panel-collapse:hover { color: var(--primary); border-color: var(--primary); }
  /* Anchored under the row and inset on both sides, so a label can never
     widen the pane: it wraps inside the tip. */
  .tip {
    position: absolute;
    top: calc(100% + 0.2rem);
    left: 0.4rem;
    right: 0.4rem;
    z-index: 6;
    margin: 0;
    padding: 0.3rem 0.45rem;
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--ink);
    background: var(--surface-raised);
    box-shadow: 0 6px 18px var(--shadow);
    font-size: 0.7rem;
    line-height: 1.3;
  }
  @media (max-width: 980px) {
    /* A tab already gives the journal the whole screen here, so the
       collapse chevron is a control that cannot do anything — and the
       width it frees is what lets the rest of the row fit at 320 px. */
    .panel-collapse { display: none; }
  }
</style>
