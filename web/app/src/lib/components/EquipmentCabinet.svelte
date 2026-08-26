<script lang="ts">
  import { APPARATUS } from "../apparatus";
  import type { TwoVesselAction } from "../directActions";
  import { t } from "../i18n.svelte";
  import ToolIcon from "./ToolIcon.svelte";

  const TRANSFER_TOOLS: { verb: TwoVesselAction; title: string; blurb: string }[] = [
    { verb: "filter", title: "filter", blurb: "separate solids from liquid" },
    { verb: "decant", title: "decant", blurb: "pour off a chosen fraction" },
    { verb: "drain", title: "drain", blurb: "move the lower liquid layer" },
    { verb: "cell", title: "voltmeter", blurb: "connect two half-cells" },
    { verb: "distil", title: "still", blurb: "separate by volatility" },
  ];

  let {
    target,
    targetLabel,
    buretteOut,
    apparatusOut,
    transferVerb,
    reactAvailable,
    onburette,
    onapparatus,
    ontransfer,
  }: {
    target: number;
    targetLabel: string;
    buretteOut: boolean;
    apparatusOut: string | null;
    transferVerb: TwoVesselAction | null;
    reactAvailable: boolean;
    onburette: () => void;
    onapparatus: (verb: string) => void;
    ontransfer: (verb: TwoVesselAction) => void;
  } = $props();
</script>

<section class="equipment-cabinet" aria-label={t("equipment") }>
  <div class="target-card">
    <span class="target-orbit" aria-hidden="true"><span></span></span>
    <span><small>{t("active work area")}</small><strong>{t(targetLabel)} · v{target + 1}</strong></span>
  </div>

  <div class="cabinet-intro">
    <span>{t("Instrument wall")}</span>
    <p>{t("Choose a tool and it appears at the selected work area.")}</p>
  </div>

  <div class="equipment-group">
    <h2><span>{t("measure and transform")}</span><small>{APPARATUS.length + 1}</small></h2>
    <div class="equipment-grid">
      <button class="equipment-card feature" class:deployed={buretteOut} aria-pressed={buretteOut} onclick={onburette}>
        <span class="equipment-icon"><ToolIcon name="burette" /></span>
        <span class="equipment-copy"><strong>{t("burette")}</strong><small>{t("controlled addition")}</small></span>
        {#if buretteOut}<span class="deployed-label">{t("on bench")}</span>{/if}
      </button>
      {#each APPARATUS as item (item.verb)}
        <button class="equipment-card" class:deployed={apparatusOut === item.verb} aria-pressed={apparatusOut === item.verb} onclick={() => onapparatus(item.verb)}>
          <span class="equipment-icon"><ToolIcon name={item.verb} /></span>
          <span class="equipment-copy"><strong>{t(item.title)}</strong><small>{t(item.blurb)}</small></span>
          {#if apparatusOut === item.verb}<span class="deployed-label">{t("on bench")}</span>{/if}
        </button>
      {/each}
    </div>
  </div>

  <div class="equipment-group">
    <h2><span>{t("transfer and separation")}</span><small>{TRANSFER_TOOLS.length + 1}</small></h2>
    <div class="equipment-grid">
      {#each TRANSFER_TOOLS as item (item.verb)}
        <button class="equipment-card" class:deployed={transferVerb === item.verb} aria-pressed={transferVerb === item.verb} onclick={() => ontransfer(item.verb)}>
          <span class="equipment-icon"><ToolIcon name={item.verb} /></span>
          <span class="equipment-copy"><strong>{t(item.title)}</strong><small>{t(item.blurb)}</small></span>
          {#if transferVerb === item.verb}<span class="deployed-label">{t("select source")}</span>{/if}
        </button>
      {/each}
      <button class="equipment-card" class:deployed={apparatusOut === "transport"} aria-pressed={apparatusOut === "transport"} onclick={() => onapparatus("transport")}>
        <span class="equipment-icon"><ToolIcon name="transport" /></span>
        <span class="equipment-copy"><strong>{t("column train")}</strong><small>{t("move solution through connected cells")}</small></span>
        {#if apparatusOut === "transport"}<span class="deployed-label">{t("on bench")}</span>{/if}
      </button>
    </div>
  </div>

  {#if reactAvailable}
    <div class="equipment-group">
      <h2><span>{t("reaction studio")}</span><small>1</small></h2>
      <button class="equipment-card wide" class:deployed={apparatusOut === "react"} aria-pressed={apparatusOut === "react"} onclick={() => onapparatus("react")}>
        <span class="equipment-icon"><ToolIcon name="react" /></span>
        <span class="equipment-copy"><strong>{t("curated reaction")}</strong><small>{t("choose a verified reaction family")}</small></span>
        {#if apparatusOut === "react"}<span class="deployed-label">{t("on bench")}</span>{/if}
      </button>
    </div>
  {/if}
</section>

<style>
  .equipment-cabinet { min-height: 0; overflow-y: auto; padding: 0.65rem; }
  .target-card { display: flex; align-items: center; gap: 0.6rem; margin-bottom: 0.75rem; padding: 0.65rem; border: 1px solid color-mix(in srgb, var(--instrument) 30%, var(--edge)); border-radius: 13px; background: color-mix(in srgb, var(--instrument) 8%, var(--surface-raised)); }
  .target-card > span:last-child { min-width: 0; display: flex; flex-direction: column; }
  .target-card small { color: var(--instrument); font-size: 0.57rem; font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; }
  .target-card strong { overflow: hidden; font-size: 0.75rem; text-overflow: ellipsis; white-space: nowrap; }
  .target-orbit { width: 31px; height: 31px; display: grid; place-items: center; flex: none; border: 1px solid color-mix(in srgb, var(--instrument) 45%, transparent); border-radius: 50%; }
  .target-orbit span { width: 9px; height: 9px; border: 2px solid var(--surface); border-radius: 50%; background: var(--instrument); box-shadow: 0 0 0 2px var(--instrument); }
  .cabinet-intro { margin: 0.2rem 0.2rem 1rem; }
  .cabinet-intro > span { color: var(--ink); font-size: 0.85rem; font-weight: 800; }
  .cabinet-intro p { margin: 0.15rem 0 0; color: var(--dim); font-size: 0.67rem; line-height: 1.35; }
  .equipment-group { margin-bottom: 1.15rem; }
  .equipment-group h2 { display: flex; align-items: center; justify-content: space-between; margin: 0 0 0.45rem; color: var(--dim); font-size: 0.62rem; letter-spacing: 0.07em; text-transform: uppercase; }
  .equipment-group h2 small { min-width: 1.35rem; padding: 0.12rem 0.3rem; border-radius: 999px; background: var(--surface-raised); text-align: center; }
  .equipment-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0.42rem; }
  .equipment-card { position: relative; min-width: 0; min-height: 100px; display: flex; flex-direction: column; align-items: flex-start; gap: 0.48rem; padding: 0.58rem; overflow: hidden; border: 1px solid var(--edge); border-radius: 13px; color: var(--ink); background: linear-gradient(145deg, var(--surface), color-mix(in srgb, var(--surface-raised) 76%, var(--surface))); cursor: pointer; text-align: left; }
  .equipment-card:hover { border-color: var(--action); transform: translateY(-2px); box-shadow: 0 7px 16px var(--shadow); }
  .equipment-card.deployed { border-color: var(--action); background: color-mix(in srgb, var(--action) 9%, var(--surface)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--action) 20%, transparent); }
  .equipment-card.feature { grid-column: 1 / -1; min-height: 82px; flex-direction: row; align-items: center; background: linear-gradient(135deg, color-mix(in srgb, var(--primary) 10%, var(--surface)), color-mix(in srgb, var(--instrument) 9%, var(--surface))); }
  .equipment-card.wide { width: 100%; min-height: 82px; flex-direction: row; align-items: center; }
  .equipment-icon { width: 36px; height: 36px; display: grid; place-items: center; flex: none; border-radius: 11px; color: var(--action); background: color-mix(in srgb, var(--action) 10%, var(--surface)); }
  .equipment-icon :global(svg) { width: 26px; height: 26px; margin: 0; }
  .equipment-copy { min-width: 0; display: flex; flex-direction: column; gap: 0.15rem; }
  .equipment-copy strong { font-size: 0.72rem; line-height: 1.15; }
  .equipment-copy small { color: var(--dim); font-size: 0.61rem; line-height: 1.25; }
  .deployed-label { position: absolute; top: 0.35rem; right: 0.35rem; padding: 0.14rem 0.32rem; border-radius: 999px; color: white; background: var(--action); font-size: 0.48rem; font-weight: 800; letter-spacing: 0.04em; text-transform: uppercase; }
</style>
