<script lang="ts">
  import type { SceneVessel } from "../host/EngineHost";
  import { i18n, t } from "../i18n.svelte";

  let {
    vessel,
    vesselCount,
    onconfirm,
    ontransfer,
    onopenwaste,
    onclose,
  }: {
    vessel: SceneVessel;
    vesselCount: number;
    onconfirm: () => void;
    ontransfer?: () => void;
    onopenwaste: () => void;
    onclose: () => void;
  } = $props();

  const empty = $derived(vessel.mass_g <= 1e-9);
  const onlyVessel = $derived(vesselCount <= 1);
  const volumeMl = $derived((vessel.liquid?.volume_l ?? 0) * 1000);
  const locale = $derived(i18n.locale === "de" ? "de-DE" : "en-GB");
  const amount = $derived(
    [
      volumeMl > 0
        ? t("{amount} mL liquid", {
            amount: volumeMl.toLocaleString(locale, { maximumFractionDigits: 1 }),
          })
        : null,
      vessel.solids.length > 0
        ? t("{count} solid material(s)", { count: vessel.solids.length })
        : null,
      vessel.mass_g > 0
        ? t("{amount} g total", {
            amount: vessel.mass_g.toLocaleString(locale, { maximumFractionDigits: 2 }),
          })
        : null,
    ].filter(Boolean).join(" · "),
  );
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open aria-modal="true" aria-labelledby="remove-vessel-title" onclick={(event) => event.stopPropagation()} onkeydown={(event) => { event.stopPropagation(); if (event.key === "Escape") onclose(); }}>
    <header>
      <span class="mark" aria-hidden="true">×</span>
      <span>
        <small>{t("manage vessel")}</small>
        <h2 id="remove-vessel-title">{t("remove vessel v{vessel}", { vessel: vessel.id + 1 })}</h2>
      </span>
      <button class="close" aria-label={t("close")} onclick={onclose}>×</button>
    </header>

    <section class:ready={empty && !onlyVessel} class:blocked={!empty || onlyVessel}>
      <div class="vessel-mark" aria-hidden="true">⌄</div>
      <div>
        <strong>{t(vessel.label)} · v{vessel.id + 1}</strong>
        {#if empty}
          <p>{onlyVessel
            ? t("Keep at least one vessel on the bench so there is always a workspace.")
            : t("This vessel is empty. Removing it is recorded in the lab history and can be undone.")}</p>
        {:else}
          <p>{t("This vessel still contains material and cannot be removed yet.")}</p>
          <span class="amount">{amount}</span>
        {/if}
      </div>
    </section>

    {#if !empty}
      <div class="decision">
        <strong>{t("Choose what happens to the contents first")}</strong>
        <p>{t("Move recoverable material to another vessel or inspect the waste station. Nothing is discarded automatically.")}</p>
      </div>
    {/if}

    <footer>
      <button class="secondary" onclick={onclose}>{t("keep vessel")}</button>
      {#if !empty && ontransfer && volumeMl > 0}
        <button class="transfer" onclick={ontransfer}>{t("pour all liquid…")}</button>
      {/if}
      {#if !empty}
        <button class="waste" onclick={onopenwaste}>{t("open waste station")}</button>
      {:else if !onlyVessel}
        <button class="remove" onclick={onconfirm}>{t("remove empty vessel")}</button>
      {/if}
    </footer>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 86; display: grid; place-items: center; padding: 1rem; background: var(--scrim); backdrop-filter: blur(10px) saturate(1.12); }
  dialog { position: static; width: min(34rem, 94vw); margin: 0; padding: 0; overflow: hidden; border: 1px solid color-mix(in srgb, var(--warning) 45%, var(--edge)); border-radius: 23px; color: var(--ink); background: var(--surface); box-shadow: 0 28px 80px var(--overlay-shadow); }
  header { display: flex; align-items: center; gap: .75rem; padding: 1rem 1.1rem; background: linear-gradient(110deg, color-mix(in srgb, var(--warning) 14%, var(--surface)), color-mix(in srgb, var(--hot) 8%, var(--surface))); }
  header > span:nth-child(2) { display: grid; gap: .05rem; }
  header small { color: var(--warning); font-size: .58rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.18rem; }
  .mark { width: 42px; height: 42px; display: grid; place-items: center; flex: none; border-radius: 13px; color: var(--on-accent); background: linear-gradient(145deg, var(--warning), var(--hot)); font-size: 1.45rem; font-weight: 850; }
  .close { width: 38px; height: 38px; margin-left: auto; border: 1px solid var(--edge); border-radius: 50%; color: var(--ink); background: var(--surface); cursor: pointer; font: inherit; font-size: 1.2rem; }
  section { display: grid; grid-template-columns: 48px 1fr; align-items: center; gap: .8rem; margin: 1rem 1.1rem .7rem; padding: .85rem; border: 1px solid var(--edge); border-radius: 15px; background: var(--surface-raised); }
  section.ready { border-color: color-mix(in srgb, var(--success) 45%, var(--edge)); background: color-mix(in srgb, var(--success) 7%, var(--surface-raised)); }
  section.blocked { border-color: color-mix(in srgb, var(--warning) 38%, var(--edge)); }
  .vessel-mark { width: 46px; height: 46px; display: grid; place-items: center; border: 2px solid var(--instrument); border-top: 0; border-radius: 4px 4px 15px 15px; color: var(--instrument); font-size: 1.4rem; }
  section strong { font-size: .86rem; }
  section p, .decision p { margin: .18rem 0 0; color: var(--dim); font-size: .72rem; line-height: 1.45; }
  .amount { display: block; margin-top: .38rem; color: var(--instrument); font-size: .68rem; font-weight: 800; }
  .decision { margin: 0 1.1rem .8rem; padding: .75rem .85rem; border-left: 4px solid var(--warning); border-radius: 9px; background: color-mix(in srgb, var(--warning) 7%, var(--surface-raised)); }
  .decision strong { font-size: .76rem; }
  footer { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: .45rem; padding: .85rem 1.1rem 1.1rem; }
  footer button { min-height: 40px; padding: .45rem .75rem; border: 1px solid var(--edge); border-radius: 11px; color: var(--ink); background: var(--surface-raised); font: inherit; font-size: .72rem; font-weight: 750; cursor: pointer; }
  footer .transfer { color: var(--on-accent); border-color: var(--primary); background: var(--primary); }
  footer .waste { color: var(--warning); border-color: color-mix(in srgb, var(--warning) 55%, var(--edge)); background: color-mix(in srgb, var(--warning) 8%, var(--surface)); }
  footer .remove { color: var(--on-accent); border-color: var(--bad); background: var(--bad); }
  @media (max-width: 430px) {
    footer { display: grid; grid-template-columns: 1fr; }
    footer button { width: 100%; }
  }
</style>
