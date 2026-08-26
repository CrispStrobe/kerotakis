<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import KitStrip from "./KitStrip.svelte";
  import { t } from "../i18n.svelte";

  let {
    name, next, busy, deviation = 0, kit = [], register = "lv2", target = 0,
    cursor = 0, total = 0, onnext, onreturn, onexit, onadd,
  }: {
    name: string; next: string | null; busy: boolean; deviation?: number;
    kit?: ShelfItem[]; register?: string; target?: number; cursor?: number; total?: number;
    onnext: () => void; onreturn?: () => void; onexit: () => void;
    onadd?: (line: string) => void;
  } = $props();
</script>

<div class="lesson" role="region" aria-label={t("lesson {name}", { name: t(name) })}>
  <div class="mission-identity">
    <span class="mission-mark" aria-hidden="true">◆</span>
    <span><small>{t("mission in progress")}</small><strong>{t(name)}</strong></span>
  </div>
  <div class="mission-progress" aria-label={t("mission progress")}>
    <span class="progress-copy">{t("{step} of {total} steps", { step: Math.min(cursor + 1, total), total })}</span>
    <span class="track" aria-hidden="true"><span style={`width:${total > 0 ? Math.min(100, cursor / total * 100) : 0}%`}></span></span>
  </div>
  <div class="controls">
    {#if next}
      <span class="objective"><small>{t("current objective")}</small><code>{next}</code></span>
      <button class="next" onclick={onnext} disabled={busy}><span>{t("do it")}</span><span aria-hidden="true">→</span></button>
    {/if}
    {#if deviation > 0}
      <span class="deviation">{t(deviation === 1 ? "off the script by {count} step" : "off the script by {count} steps", { count: deviation })} — {t("exploring is allowed")}</span>
      <button onclick={onreturn} disabled={busy}>{t("return to the script")}</button>
    {/if}
    <button class="leave" onclick={onexit}>{t("leave lesson")}</button>
  </div>
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
  .mission-identity small, .objective small { color: var(--discovery); font-size: 0.57rem; font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; }
  .mission-identity strong { max-width: 14rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.78rem; }
  .mission-mark { width: 34px; height: 34px; display: grid; place-items: center; border-radius: 11px; color: white; background: var(--discovery); box-shadow: 0 5px 13px color-mix(in srgb, var(--discovery) 27%, transparent); }
  .mission-progress { display: flex; flex-direction: column; gap: 0.25rem; }
  .progress-copy { color: var(--dim); font-size: 0.65rem; }
  .track { height: 5px; overflow: hidden; border-radius: 999px; background: color-mix(in srgb, var(--edge) 55%, transparent); }
  .track span { display: block; height: 100%; border-radius: inherit; background: linear-gradient(90deg, var(--discovery), var(--primary)); transition: width 300ms ease; }
  .controls { display: flex; align-items: center; justify-content: flex-end; gap: 0.5rem; flex-wrap: wrap; }
  .objective { min-width: 0; display: flex; flex-direction: column; gap: 0.15rem; }
  code { max-width: 24rem; overflow: hidden; color: var(--ink); font-size: 0.72rem; text-overflow: ellipsis; white-space: nowrap; }
  button { min-height: 34px; padding: 0.25rem 0.7rem; border: 1px solid var(--edge); border-radius: 10px; color: var(--ink); background: var(--panel-raised); cursor: pointer; font: inherit; font-size: 0.72rem; }
  .next { min-width: 5.5rem; display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; color: white; border-color: var(--discovery); background: var(--discovery); font-weight: 750; }
  .deviation { color: var(--dim); font-size: 0.7rem; }
  .leave { color: var(--dim); }
  .lesson > :global(.kit-strip) { grid-column: 1 / -1; }
  @media (max-width: 820px) {
    .lesson { grid-template-columns: 1fr auto; }
    .mission-progress { display: none; }
    .controls { grid-column: 1 / -1; justify-content: stretch; }
    .objective { flex: 1; }
  }
  @media (max-width: 520px) {
    .lesson { gap: 0.4rem; }
    .mission-identity strong { max-width: 11rem; }
    .deviation { flex-basis: 100%; }
  }
</style>
