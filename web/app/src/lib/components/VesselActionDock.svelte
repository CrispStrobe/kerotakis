<script lang="ts">
  import { t } from "../i18n.svelte";
  import { vesselQuickActions } from "../directActions";

  let {
    vessel,
    label,
    boundary,
    busy,
    onaction,
    onpour,
    ondetails,
    onmore,
  }: {
    vessel: number;
    label: string;
    boundary: string;
    busy: boolean;
    onaction: (line: string) => void;
    onpour: () => void;
    ondetails: () => void;
    onmore: () => void;
  } = $props();

  const v = $derived(`v${vessel + 1}`);
  const actions = $derived(vesselQuickActions(vessel, boundary));
</script>

<section class="dock" aria-label={t("quick actions for vessel v{vessel}", { vessel: vessel + 1 })}>
  <div class="selection">
    <span class="selection-dot" aria-hidden="true"></span>
    <span><small>{t("selected")}</small><strong>{t(label)} · {v}</strong></span>
  </div>
  <div class="actions">
    <button class="pour" disabled={busy} onclick={onpour} title={t("pour from {vessel}", { vessel: v })}>
      <span class="icon" aria-hidden="true">↗</span>
      <span>{t("pour")}</span>
    </button>
    {#each actions as action (action.label)}
      <button class={action.tone} disabled={busy} onclick={() => onaction(action.line)} title={t("run {action} on {vessel}", { action: t(action.label), vessel: v })}>
        <span class="icon" aria-hidden="true">{action.icon}</span>
        <span>{t(action.label)}</span>
      </button>
    {/each}
  </div>
  <div class="more-actions">
    <button onclick={ondetails}>{t("details")}</button>
    <button class="more" onclick={onmore}>{t("more tools")}</button>
  </div>
</section>

<style>
  .dock {
    min-height: 74px;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0.65rem;
    border-top: 1px solid var(--edge);
    background: color-mix(in srgb, var(--surface) 94%, var(--primary) 6%);
  }
  .selection {
    min-width: 8.5rem;
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0 0.45rem;
  }
  .selection > span:last-child {
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
  .pour .icon { color: var(--action); }
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
    .selection > span:last-child,
    .more-actions {
      display: none;
    }
    .actions button {
      min-width: 52px;
      min-height: 52px;
    }
  }
</style>
