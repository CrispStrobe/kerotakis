<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import KitStrip from "./KitStrip.svelte";
  import { t } from "../i18n.svelte";
  import { missionHint, missionObjective } from "../missionJournal";
  import { missionTitle } from "../storyProgress";
  import type { OutcomeMissionContract } from "../outcomeMission";

  let {
    name, next, outcome = null, busy, deviation = 0, kit = [], register = "lv2", target = 0,
    cursor = 0, total = 0, evidence = [], onnext, onreturn, onexit, onadd,
  }: {
    name: string; next: string | null;
    outcome?: { contract: OutcomeMissionContract; secured: string[] } | null;
    busy: boolean; deviation?: number;
    kit?: ShelfItem[]; register?: string; target?: number; cursor?: number; total?: number;
    evidence?: string[];
    onnext: () => void; onreturn?: () => void; onexit: () => void;
    onadd?: (line: string) => void;
  } = $props();

  let journalOpen = $state(false);
  let hintOpen = $state(false);
  const secured = $derived(new Set(outcome?.secured ?? []));
  const activeHint = $derived(outcome?.contract.hint ?? (next ? missionHint(next) : ""));
</script>

<div class="lesson" role="region" aria-label={t("lesson {name}", { name: t(missionTitle(name)) })}>
  <div class="mission-identity">
    <span class="mission-mark" aria-hidden="true">◆</span>
    <span><small>{t("mission in progress")}</small><strong>{t(missionTitle(name))}</strong></span>
  </div>
  <div class="mission-progress" aria-label={t("mission progress")}>
    <span class="progress-copy">{outcome
      ? t("{done} of {total} evidence checks", { done: cursor, total })
      : t("{step} of {total} steps", { step: Math.min(cursor + 1, total), total })}</span>
    <span class="track" aria-hidden="true"><span style={`width:${total > 0 ? Math.min(100, cursor / total * 100) : 0}%`}></span></span>
  </div>
  <div class="controls">
    {#if outcome}
      <span class="objective outcome-objective"><small>{t("mission outcome")}</small><strong>{t(outcome.contract.objective)}</strong></span>
      <span class="solver-badge"><i aria-hidden="true"></i>{t("assessed by the solver")}</span>
    {:else if next}
      <span class="objective"><small>{t("current objective")}</small><strong>{t(missionObjective(next))}</strong></span>
      <button class="next" onclick={onnext} disabled={busy}><span>{t("do it")}</span><span aria-hidden="true">→</span></button>
    {/if}
    {#if deviation > 0}
      <span class="deviation">{t(deviation === 1 ? "off the script by {count} step" : "off the script by {count} steps", { count: deviation })} — {t("exploring is allowed")}</span>
      <button onclick={onreturn} disabled={busy}>{t("return to the script")}</button>
    {/if}
    <button class="journal-button" aria-expanded={journalOpen} onclick={() => (journalOpen = !journalOpen)}>▤ {t("mission journal")}</button>
    <button class="leave" onclick={onexit}>{t("leave mission")}</button>
  </div>
  {#if journalOpen}
    <section class="journal" aria-label={t("mission journal")}>
      <div class="instruction">
        <span class="journal-icon" aria-hidden="true">◎</span>
        <div>
          <small>{t(outcome ? "mission goal" : "current lab instruction")}</small>
          {#if outcome}<p class="outcome-brief">{t(outcome.contract.brief)}</p>{:else}<code>{next ?? t("mission complete")}</code>{/if}
        </div>
        {#if next || outcome}<button aria-expanded={hintOpen} onclick={() => (hintOpen = !hintOpen)}>◇ {t(hintOpen ? "hide hint" : "show a hint")}</button>{/if}
      </div>
      {#if hintOpen && (next || outcome)}
        <p class="hint"><strong>{t("hint")}</strong>{t(activeHint)}</p>
      {/if}
      <div class="evidence-ledger">
        <div class="ledger-title"><span><small>{t("evidence ledger")}</small><strong>{t("Results gathered during this mission")}</strong></span><b>{evidence.length}</b></div>
        {#if outcome}
          <ul class="outcome-checks" aria-label={t("outcome evidence checks")}>
            {#each outcome.contract.criteria as criterion (criterion.id)}
              <li class:secured={secured.has(criterion.id)}><span aria-hidden="true">{secured.has(criterion.id) ? "✓" : "○"}</span>{t(criterion.label)}</li>
            {/each}
          </ul>
        {/if}
        {#if evidence.length > 0}
          <ol>{#each evidence as item}<li>{t(item)}</li>{/each}</ol>
        {:else}
          <p>{t("Your engine-backed observations and measurements will collect here.")}</p>
        {/if}
      </div>
    </section>
  {/if}
  {#if kit.length > 0 && onadd}<KitStrip items={kit} {register} {target} {onadd} />{/if}
</div>

<style>
  .lesson {
    display: grid;
    grid-template-columns: auto minmax(8rem, 0.6fr) minmax(18rem, 1fr);
    align-items: center;
    gap: 0.9rem;
    padding: 0.55rem 0.8rem;
    border-bottom: 1px solid color-mix(in srgb, var(--discovery) 32%, var(--edge));
    background: linear-gradient(90deg, color-mix(in srgb, var(--discovery) 10%, var(--panel)), var(--panel) 35%);
    box-shadow: 0 5px 16px var(--shadow);
    font-size: 0.85rem;
    z-index: 10;
  }
  .mission-identity { display: flex; align-items: center; gap: 0.55rem; }
  .mission-identity > span:last-child { display: flex; flex-direction: column; line-height: 1.15; }
  .mission-identity small, .objective small, .journal small { color: var(--discovery); font-size: 0.57rem; font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; }
  .mission-identity strong { max-width: 14rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.78rem; }
  .mission-mark { width: 34px; height: 34px; display: grid; place-items: center; border-radius: 11px; color: white; background: var(--discovery); box-shadow: 0 5px 13px color-mix(in srgb, var(--discovery) 27%, transparent); }
  .mission-progress { display: flex; flex-direction: column; gap: 0.25rem; }
  .progress-copy { color: var(--dim); font-size: 0.65rem; }
  .track { height: 5px; overflow: hidden; border-radius: 999px; background: color-mix(in srgb, var(--edge) 55%, transparent); }
  .track span { display: block; height: 100%; border-radius: inherit; background: linear-gradient(90deg, var(--discovery), var(--primary)); transition: width 300ms ease; }
  .controls { display: flex; align-items: center; justify-content: flex-end; gap: 0.5rem; flex-wrap: wrap; }
  .objective { min-width: 0; display: flex; flex-direction: column; gap: 0.15rem; }
  .objective strong { max-width: 24rem; overflow: hidden; color: var(--ink); font-size: 0.72rem; text-overflow: ellipsis; white-space: nowrap; }
  .outcome-objective strong { max-width: 30rem; }
  .solver-badge { display: inline-flex; align-items: center; gap: .35rem; padding: .3rem .55rem; border-radius: 999px; color: var(--success); background: color-mix(in srgb, var(--success) 10%, var(--panel)); font-size: .62rem; font-weight: 750; white-space: nowrap; }
  .solver-badge i { width: 7px; height: 7px; border-radius: 50%; background: currentColor; box-shadow: 0 0 0 3px color-mix(in srgb, var(--success) 14%, transparent); }
  code { color: var(--ink); font-size: 0.72rem; overflow-wrap: anywhere; }
  button { min-height: 34px; padding: 0.25rem 0.7rem; border: 1px solid var(--edge); border-radius: 10px; color: var(--ink); background: var(--panel-raised); cursor: pointer; font: inherit; font-size: 0.72rem; }
  .next { min-width: 5.5rem; display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; color: white; border-color: var(--discovery); background: var(--discovery); font-weight: 750; }
  .deviation { color: var(--dim); font-size: 0.7rem; }
  .leave { color: var(--dim); }
  .journal-button { color: var(--primary); border-color: color-mix(in srgb, var(--primary) 35%, var(--edge)); }
  .journal { grid-column: 1 / -1; display: grid; grid-template-columns: minmax(17rem, .85fr) minmax(20rem, 1.15fr); gap: .65rem; padding: .75rem; border: 1px solid color-mix(in srgb, var(--discovery) 30%, var(--edge)); border-radius: 16px; background: color-mix(in srgb, var(--panel-raised) 82%, var(--panel)); animation: journal-in 220ms ease both; }
  @keyframes journal-in { from { opacity: 0; transform: translateY(-5px); } }
  .instruction { display: grid; grid-template-columns: 38px 1fr; align-items: center; gap: .6rem; }
  .instruction > button { grid-column: 1 / -1; justify-self: start; }
  .instruction > div { min-width: 0; display: grid; gap: .15rem; }
  .outcome-brief { margin: 0; color: var(--ink); font-size: .72rem; line-height: 1.4; }
  .journal-icon { width: 38px; height: 38px; display: grid; place-items: center; border-radius: 12px; color: var(--action); background: color-mix(in srgb, var(--action) 11%, var(--panel)); }
  .hint { grid-column: 1; margin: 0; padding: .65rem; border-left: 3px solid var(--discovery); border-radius: 8px; color: var(--dim); background: color-mix(in srgb, var(--discovery) 7%, transparent); font-size: .72rem; }
  .hint strong { display: block; margin-bottom: .15rem; color: var(--discovery); text-transform: uppercase; font-size: .6rem; letter-spacing: .08em; }
  .evidence-ledger { grid-column: 2; grid-row: 1 / span 2; min-width: 0; padding-left: .7rem; border-left: 1px solid var(--edge); }
  .ledger-title { display: flex; justify-content: space-between; gap: .5rem; }
  .ledger-title > span { display: grid; }
  .ledger-title b { width: 28px; height: 28px; display: grid; place-items: center; border-radius: 50%; color: var(--success); background: color-mix(in srgb, var(--success) 12%, var(--panel)); }
  .outcome-checks { display: grid; gap: .3rem; margin: .55rem 0 0; padding: 0; list-style: none; }
  .outcome-checks li { display: flex; align-items: center; gap: .4rem; padding: .4rem .5rem; border: 1px dashed var(--edge); border-radius: 9px; color: var(--dim); background: color-mix(in srgb, var(--surface) 70%, transparent); font-size: .68rem; }
  .outcome-checks li.secured { color: var(--success); border-style: solid; border-color: color-mix(in srgb, var(--success) 40%, var(--edge)); background: color-mix(in srgb, var(--success) 8%, var(--surface)); font-weight: 750; }
  .evidence-ledger ol { max-height: 7rem; overflow: auto; margin: .55rem 0 0; padding-left: 1.25rem; }
  .evidence-ledger li, .evidence-ledger > p { margin: .25rem 0; color: var(--dim); font-size: .68rem; line-height: 1.35; }
  .lesson > :global(.kit-strip) { grid-column: 1 / -1; }
  @media (max-width: 820px) {
    .lesson { grid-template-columns: 1fr auto; }
    .mission-progress { display: none; }
    .controls { grid-column: 1 / -1; justify-content: stretch; }
    .objective { flex: 1; }
    .journal { grid-template-columns: 1fr; }
    .evidence-ledger { grid-column: 1; grid-row: auto; padding: .65rem 0 0; border: 0; border-top: 1px solid var(--edge); }
  }
  @media (max-width: 520px) {
    .lesson { gap: 0.4rem; }
    .mission-identity strong { max-width: 11rem; }
    .objective { flex: 1 0 100%; }
    .objective strong { max-width: none; white-space: normal; }
    .deviation { flex-basis: 100%; }
  }
</style>
