<script lang="ts">
  import { t } from "../i18n.svelte";
  import { assemblyFor } from "../apparatusAssembly";

  let { tool, values = {} }: { tool: string; values?: Record<string, number | string> } = $props();
  const assembly = $derived(assemblyFor(tool, values));
</script>

<div class="assembly" aria-label={t("physical setup")}>
  <small>{t("physical setup")}</small>
  <div class="parts">
    {#each assembly.parts as item, index (item.id)}
      {#if index > 0}<span class="connector" aria-hidden="true">—</span>{/if}
      <span class="part" class:attention={item.state === "attention"} title={t(item.label)}>
        <b aria-hidden="true">{item.symbol}</b><span>{t(item.label)}</span>
      </span>
    {/each}
  </div>
</div>

<style>
  .assembly { min-width: 0; align-self: center; }
  .assembly > small { display: block; margin-bottom: .25rem; color: var(--instrument); font-size: .56rem; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
  .parts { display: flex; align-items: stretch; overflow-x: auto; padding: .1rem 0 .2rem; }
  .part { min-width: 4rem; max-width: 7.5rem; display: grid; grid-template-columns: 1.35rem minmax(0, 1fr); align-items: center; gap: .25rem; padding: .28rem .35rem; border: 1px solid color-mix(in srgb, var(--instrument) 30%, var(--edge)); border-radius: 9px; color: var(--ink); background: var(--surface); }
  .part b { width: 1.3rem; height: 1.3rem; display: grid; place-items: center; border-radius: 6px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 10%, var(--surface)); }
  .part span { overflow: hidden; font-size: .56rem; font-weight: 750; line-height: 1.1; text-overflow: ellipsis; }
  .part.attention { border-color: var(--warning); background: color-mix(in srgb, var(--warning) 7%, var(--surface)); }
  .part.attention b { color: var(--warning); }
  .connector { align-self: center; flex: none; width: .75rem; overflow: hidden; color: var(--instrument); text-align: center; }
</style>
