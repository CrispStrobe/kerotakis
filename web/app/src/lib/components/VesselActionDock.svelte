<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import { vesselQuickActions } from "../directActions";

  let {
    vessel,
    label,
    boundary,
    contents = [],
    volumeMl = 0,
    temperatureC = 25,
    busy,
    onaction,
    onconfigure,
    onpour,
    ondetails,
    onmore,
  }: {
    vessel: number;
    label: string;
    boundary: string;
    contents?: string[];
    volumeMl?: number;
    temperatureC?: number;
    busy: boolean;
    onaction: (line: string) => void;
    onconfigure: (verb: string) => void;
    onpour: () => void;
    ondetails: () => void;
    onmore: () => void;
  } = $props();

  const v = $derived(`v${vessel + 1}`);
  const actions = $derived(vesselQuickActions(vessel, boundary));
  const contentNames = $derived([...new Set(contents.map((name) => t(name)))]);
  const contentLabel = $derived(
    contentNames.length === 0
      ? t("empty")
      : `${contentNames.slice(0, 2).join(", ")}${contentNames.length > 2 ? ` +${contentNames.length - 2}` : ""}`,
  );
  /** Collapsed, the row scrolls; expanded, it wraps and shows everything. */
  let expanded = $state(false);

  // i18n-ok: action ids are wire keys matched against literals below.
  const changeActions = $derived(actions.filter((action) => ["stir", "heat", "cool", "seal", "open"].includes(action.id)));
  /**
   * The three verbs that open a form rather than running immediately.
   *
   * They used to carry a "set…" caption under the label — "rühren
   * einstellen…", "erwärmen einstellen…" — which is a whole extra line of
   * type on a 54px button to say what the ellipsis already says. The
   * ellipsis is the convention for "this opens something"; it is on the
   * label now, and the caption is gone.
   */
  // i18n-ok: action ids are wire keys matched against literals below.
  const opensAForm = (id: string) => ["stir", "heat", "cool"].includes(id);
  // i18n-ok: action ids are wire keys matched against literals below.
  const observeActions = $derived(actions.filter((action) => ["look", "temperature", "ph"].includes(action.id)));
</script>

<section class="dock" aria-label={t("quick actions for vessel v{vessel}", { vessel: vessel + 1 })}>
  <div class="selection">
    <span class="selection-dot" aria-hidden="true"></span>
    <span class="selection-copy">
      <small>{t("selected target")}</small>
      <strong>{t(label)} · {v}</strong>
      <span class="vitals">{volumeMl.toLocaleString(i18n.locale === "de" ? "de-DE" : "en-GB", { maximumFractionDigits: 1 })} mL · {temperatureC.toLocaleString(i18n.locale === "de" ? "de-DE" : "en-GB", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} °C</span>
      <span class="contents" title={contentLabel}>{contentLabel}</span>
    </span>
  </div>
  <div class="actions" class:expanded>
    <div class="action-group">
      <small>{t("change vessel")}</small>
      <div>
        <!-- A pouring glyph, so "pour" reads as one of the row rather than
             as the odd button out beside ↻, ↑ and ❄. -->
        <button class="pour" disabled={busy} onclick={onpour} title={t("pour from {vessel}", { vessel: v })}>
          <span class="icon" aria-hidden="true">⤵</span>
          <span>{t("pour")}</span>
        </button>
        {#each changeActions as action (action.label)}
          <button
            class={action.tone}
            disabled={busy}
            onclick={() => opensAForm(action.id) ? onconfigure(action.id) : onaction(action.line)}
            title={opensAForm(action.id)
              ? t("configure {apparatus}", { apparatus: t(action.label) })
              : t("run {action} on {vessel}", { action: t(action.label), vessel: v })}
          >
            <span class="icon" aria-hidden="true">{action.icon}</span>
            <span>{opensAForm(action.id) ? t("{verb}…", { verb: t(action.label) }) : t(action.label)}</span>
          </button>
        {/each}
      </div>
    </div>
    <div class="action-group">
      <small>{t("observe and measure")}</small>
      <div>
        {#each observeActions as action (action.label)}
          <button class={action.tone} disabled={busy} onclick={() => onaction(action.line)} title={t("run {action} on {vessel}", { action: t(action.label), vessel: v })}>
            <span class="icon" aria-hidden="true">{action.icon}</span>
            <span>{t(action.label)}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
  <div class="more-actions">
    <button
      class="expand"
      aria-expanded={expanded}
      onclick={() => (expanded = !expanded)}
      title={expanded ? t("show one row") : t("show every action")}
    >{expanded ? t("fewer") : t("show all")}</button>
    <button onclick={ondetails} title={t("Open measurement tools for {vessel}", { vessel: v })}>
      <span aria-hidden="true">⌁</span>{t("measurement tools")}
    </button>
    <button class="more" onclick={onmore} title={t("Open the equipment cabinet")}>
      <span aria-hidden="true">▦</span>{t("equipment cabinet")}
    </button>
  </div>
</section>

<style>
  .dock {
    min-height: 74px;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0.5rem;
    padding: 0.55rem 0.65rem;
    border: 1px solid color-mix(in srgb, var(--primary) 25%, var(--edge));
    border-radius: 16px;
    background: color-mix(in srgb, var(--surface) 91%, var(--primary) 9%);
    box-shadow: 0 8px 24px var(--shadow);
  }
  .selection {
    min-width: 10.5rem;
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0 0.45rem;
  }
  .selection-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.15;
  }
  .selection small {
    color: var(--dim);
    font-size: 0.62rem;
    letter-spacing: 0.07em;
    text-transform: uppercase;
  }
  .selection strong {
    max-width: 9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.76rem;
  }
  .vitals { margin-top: 0.14rem; color: var(--instrument); font-size: 0.59rem; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .contents { max-width: 10rem; overflow: hidden; color: var(--dim); font-size: 0.57rem; text-overflow: ellipsis; white-space: nowrap; }
  .selection-dot {
    width: 10px;
    height: 10px;
    flex: none;
    border: 2px solid var(--surface);
    border-radius: 50%;
    background: var(--action);
    box-shadow: 0 0 0 2px var(--action);
  }
  .actions {
    min-width: 0;
    display: flex;
    flex: 1;
    gap: 0.35rem;
    overflow-x: auto;
    padding: 0.2rem;
    scrollbar-width: thin;
  }
  .action-group { display: grid; gap: 0.15rem; flex: none; }
  /* Expanded: wrap instead of scroll, so the last button is not sliced
     mid-word with nothing to say more exists. */
  .actions.expanded {
    flex-wrap: wrap;
    overflow-x: visible;
  }
  .actions.expanded .action-group { flex: 1 1 auto; }
  .actions.expanded .action-group > div { flex-wrap: wrap; }
  .expand { white-space: nowrap; }
  .action-group > small { padding-left: 0.25rem; color: var(--dim); font-size: 0.53rem; font-weight: 750; letter-spacing: 0.06em; text-transform: uppercase; }
  .action-group > div { display: flex; gap: 0.35rem; }
  .actions button {
    min-width: 58px;
    min-height: 54px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.15rem;
    flex: none;
    border: 1px solid var(--edge);
    border-radius: 12px;
    color: var(--ink);
    background: var(--surface);
    cursor: pointer;
    font-size: 0.66rem;
    font-weight: 650;
  }
  .pour {
    color: var(--on-accent);
    border-color: var(--action);
    background: linear-gradient(145deg, var(--action), color-mix(in srgb, var(--action) 72%, var(--primary)));
  }
  .pour .icon { color: var(--on-accent); }
  .actions button:hover:not(:disabled) {
    border-color: currentColor;
    transform: translateY(-2px);
    box-shadow: 0 5px 13px var(--shadow);
  }
  .actions button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .icon {
    min-height: 20px;
    display: grid;
    place-items: center;
    color: var(--primary);
    font-size: 1rem;
    font-weight: 800;
  }
  .action .icon { color: var(--action); }
  .instrument .icon { color: var(--instrument); }
  .discovery .icon { color: var(--discovery); }
  .more-actions {
    display: grid;
    gap: 0.3rem;
  }
  .more-actions button {
    min-height: 28px;
    padding: 0.2rem 0.55rem;
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--dim);
    background: var(--surface);
    cursor: pointer;
    font-size: 0.66rem;
    white-space: nowrap;
  }
  .more-actions button { display: flex; align-items: center; gap: .35rem; }
  .more-actions button > span { color: var(--instrument); font-size: .82rem; font-weight: 850; }
  .more-actions button:hover {
    color: var(--primary);
    border-color: var(--primary);
  }
  @media (max-width: 640px) {
    .dock {
      min-height: 68px;
      gap: 0.35rem;
      padding: 0.4rem;
    }
    .selection {
      min-width: 2rem;
      padding: 0.25rem;
    }
    .selection-copy,
    .more-actions {
      display: none;
    }
    .actions button {
      min-width: 52px;
      min-height: 52px;
    }
  }
</style>
