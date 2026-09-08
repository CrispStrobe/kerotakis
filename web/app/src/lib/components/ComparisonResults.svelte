<script lang="ts">
  import { t } from "../i18n.svelte";
  import type { ComparisonRow } from "../comparisonResults";
  let { rows }: { rows: readonly ComparisonRow[] } = $props();
  const label = (value: string) => t(value.replaceAll("_", " "));
  const stepLabel = (value: string) => value.startsWith("after:") ? t("after step {step}", { step: value.slice(6) }) : t(value);
</script>
{#if rows.length > 0}
  <section class="comparison" aria-label={t("trial comparison")}>
    <strong>{t("compare the trials")}</strong>
    {#each rows as row, i (i)}
      <div class="relation" data-holds={row.holds === null ? "unknown" : row.holds}>
        <span class="claim">{row.holds === true ? "✓" : row.holds === false ? "✗" : "—"} {t(row.kind)}</span>
        <div class="cells">{#each row.cells as cell, j (j)}<span class="cell"><small>{cell.sample.vessel ?? stepLabel(cell.sample.step)} · {label(cell.sample.metric)}</small><b>{cell.value === null ? t("not available") : `${Number(cell.value.toPrecision(5))} ${cell.unit}`}</b></span>{/each}</div>
      </div>
    {/each}
  </section>
{/if}
<style>
  .comparison { display: grid; gap: .45rem; margin-top: .65rem; padding-top: .65rem; border-top: 1px solid var(--edge); }
  .comparison > strong { font-size: .72rem; letter-spacing: .04em; text-transform: uppercase; }
  .relation { display: grid; grid-template-columns: minmax(6rem, auto) 1fr; gap: .45rem; align-items: stretch; }
  .claim { display: flex; align-items: center; gap: .3rem; padding: .45rem .55rem; border-radius: 9px; background: var(--surface-raised); color: var(--dim); font-size: .7rem; font-weight: 800; }
  .relation[data-holds="true"] .claim { color: var(--success); } .relation[data-holds="false"] .claim { color: var(--warning); }
  .cells { display: grid; grid-auto-flow: column; grid-auto-columns: minmax(5.5rem, 1fr); gap: .35rem; overflow-x: auto; }
  .cell { display: grid; align-content: center; min-height: 42px; padding: .35rem .5rem; border: 1px solid var(--edge); border-radius: 9px; white-space: nowrap; }
  small { color: var(--dim); font-size: .56rem; } b { font-size: .75rem; }
  @media (max-width: 520px) { .relation { grid-template-columns: 1fr; } }
</style>
