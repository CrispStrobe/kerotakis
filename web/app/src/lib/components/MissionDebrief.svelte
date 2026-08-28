<script lang="ts">
  import { t } from "../i18n.svelte";
  import type { MissionDebrief as Debrief } from "../session.svelte";
  import { missionTitle } from "../storyProgress";
  import { equipmentRewardAt } from "../catalogProgress";
  import ToolIcon from "./ToolIcon.svelte";

  let { debrief, onmap, onplace, onclose }: { debrief: Debrief; onmap: () => void; onplace: (verb: string) => void; onclose: () => void } = $props();
  const reward = $derived(debrief.firstCompletion ? equipmentRewardAt(debrief.completedTotal) : null);

  const outcome = $derived(
    !debrief.firstCompletion
      ? "This mission was already complete. Your new run remains in the lab notebook."
      : debrief.completedTotal === 1
        ? "Matter Gardens and Energy Yard are now open."
        : debrief.completedTotal === 3
          ? "Electron Works is now open."
          : debrief.completedTotal === 4
            ? "Systems Dock is now open."
            : "Your discovery is now part of the Story research record.",
  );
</script>

<div class="debrief-anchor" aria-live="polite">
  <aside class="debrief" role="status" aria-label={t("mission debrief")}>
    <div class="success-mark" aria-hidden="true">✓</div>
    <div class="title">
      <span>{t(debrief.firstCompletion ? "discovery recorded" : "mission replay complete")}</span>
      <h2>{t(missionTitle(debrief.id))}</h2>
      <p>{t(outcome)}</p>
    </div>
    <button class="close" aria-label={t("close mission debrief")} onclick={onclose}>×</button>

    <div class="summary">
      <div><strong>{debrief.evidence.length}</strong><span>{t("evidence items")}</span></div>
      <div><strong>{debrief.completedTotal}</strong><span>{t("missions complete")}</span></div>
    </div>

    {#if reward}
      <div class="reward">
        <span class="reward-icon" aria-hidden="true"><ToolIcon name={reward.verb} /></span>
        <span><small>{t("new permanent equipment")}</small><strong>{t(reward.title)}</strong><p>{t(reward.description)}</p></span>
        <button onclick={() => onplace(reward.verb)}>{t("place on bench")} <span aria-hidden="true">→</span></button>
      </div>
    {/if}

    {#if debrief.firstCompletion}
      <div class="resupply"><span aria-hidden="true">↻</span><p><strong>{t("stockroom replenished")}</strong>{t("Permanent supplies are ready for the next investigation.")}</p></div>
    {/if}

    {#if debrief.evidence.length > 0}
      <details>
        <summary>{t("review the evidence")}</summary>
        <ul>{#each debrief.evidence as item}<li>{t(item)}</li>{/each}</ul>
      </details>
    {/if}

    <div class="actions">
      <button onclick={onclose}>{t("keep experimenting")}</button>
      <button class="map" onclick={onmap}>{t("return to research map")} <span aria-hidden="true">→</span></button>
    </div>
  </aside>
</div>

<style>
  .debrief-anchor { position: fixed; inset: 0; z-index: 70; pointer-events: none; display: grid; align-items: end; justify-items: center; padding: 1rem; }
  .debrief { pointer-events: auto; position: relative; width: min(36rem, calc(100vw - 2rem)); display: grid; grid-template-columns: 54px 1fr; gap: .9rem; padding: 1rem; border: 1px solid color-mix(in srgb, var(--success) 55%, var(--edge)); border-radius: 22px; color: var(--ink); background: color-mix(in srgb, var(--surface) 94%, transparent); backdrop-filter: blur(16px); box-shadow: 0 22px 65px rgb(3 24 39 / 38%); animation: debrief-in 420ms cubic-bezier(.2,.8,.2,1) both; }
  @keyframes debrief-in { from { opacity: 0; transform: translateY(30px) scale(.97); } }
  .success-mark { width: 54px; height: 54px; display: grid; place-items: center; border-radius: 18px; color: var(--on-accent); background: linear-gradient(145deg, var(--success), var(--instrument)); font-size: 1.55rem; font-weight: 900; box-shadow: 0 8px 22px color-mix(in srgb, var(--success) 30%, transparent); }
  .title span { color: var(--success); font-size: .64rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  h2 { margin: .15rem 2.2rem .25rem 0; font-size: 1.25rem; }
  p { margin: 0; color: var(--dim); font-size: .78rem; }
  .close { position: absolute; top: .7rem; right: .7rem; width: 34px; height: 34px; border: 1px solid var(--edge); border-radius: 50%; color: var(--ink); background: var(--surface); cursor: pointer; font-size: 1.15rem; }
  .summary { grid-column: 1 / -1; display: grid; grid-template-columns: 1fr 1fr; gap: .55rem; }
  .summary div { display: flex; align-items: baseline; gap: .45rem; padding: .6rem .75rem; border-radius: 12px; background: var(--surface-raised); }
  .summary strong { color: var(--success); font-size: 1.15rem; }
  .summary span { color: var(--dim); font-size: .7rem; }
  .reward { grid-column: 1 / -1; display: grid; grid-template-columns: 46px 1fr auto; align-items: center; gap: .7rem; padding: .7rem; overflow: hidden; border: 1px solid color-mix(in srgb, var(--instrument) 42%, var(--edge)); border-radius: 15px; background: linear-gradient(120deg, color-mix(in srgb, var(--instrument) 12%, var(--surface)), color-mix(in srgb, var(--discovery) 8%, var(--surface))); animation: reward-in 520ms 160ms both; }
  @keyframes reward-in { from { opacity: 0; transform: translateX(14px); } }
  .reward-icon { width: 46px; height: 46px; display: grid; place-items: center; border-radius: 14px; color: var(--on-accent); background: var(--instrument); }
  .reward-icon :global(svg) { width: 32px; height: 32px; }
  .reward > span:nth-child(2) { min-width: 0; display: grid; }
  .reward small { color: var(--instrument); font-size: .58rem; font-weight: 850; letter-spacing: .09em; text-transform: uppercase; }
  .reward strong { font-size: .85rem; }
  .reward p { font-size: .67rem; }
  .reward button { min-height: 36px; padding: 0 .7rem; border: 0; border-radius: 10px; color: var(--on-accent); background: var(--instrument); cursor: pointer; font-weight: 800; }
  .resupply { grid-column: 1 / -1; display: flex; align-items: center; gap: .6rem; padding: .55rem .7rem; border-radius: 12px; color: var(--ink); background: color-mix(in srgb, var(--success) 8%, var(--surface-raised)); }
  .resupply > span { width: 28px; height: 28px; display: grid; place-items: center; flex: none; border-radius: 50%; color: var(--on-accent); background: var(--success); font-weight: 900; }
  .resupply p { display: grid; font-size: .68rem; }
  .resupply strong { color: var(--success); font-size: .62rem; letter-spacing: .06em; text-transform: uppercase; }
  details { grid-column: 1 / -1; padding: .55rem .7rem; border: 1px solid var(--edge); border-radius: 12px; font-size: .72rem; }
  summary { color: var(--primary); cursor: pointer; font-weight: 750; }
  ul { max-height: 7rem; overflow: auto; margin: .5rem 0 0; padding-left: 1.2rem; color: var(--dim); }
  li { margin: .2rem 0; }
  .actions { grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: .5rem; }
  .actions button { min-height: 38px; padding: 0 .8rem; border: 1px solid var(--edge); border-radius: 11px; color: var(--ink); background: var(--surface-raised); cursor: pointer; font-weight: 750; }
  .actions .map { display: flex; align-items: center; gap: .8rem; color: var(--on-accent); border-color: var(--primary); background: var(--primary); }
  @media (max-width: 520px) {
    .debrief-anchor { padding: .5rem; }
    .debrief { width: calc(100vw - 1rem); grid-template-columns: 44px 1fr; border-radius: 18px; }
    .success-mark { width: 44px; height: 44px; border-radius: 14px; }
    .actions { display: grid; grid-template-columns: 1fr 1fr; }
    .actions button { justify-content: center; }
    .reward { grid-template-columns: 42px 1fr; }
    .reward button { grid-column: 1 / -1; }
  }
</style>
