<script lang="ts">
  import { t } from "../i18n.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const stations = [
    { icon: "◎", title: "eye protection", detail: "Wear splash goggles whenever materials are on the bench." },
    { icon: "♨", title: "heat and flame", detail: "Keep flammables away from heat. Treat hot glass as hot until measured." },
    { icon: "⌁", title: "fumes and gases", detail: "Do not smell directly. Use wafting only when the lab explicitly offers it." },
    { icon: "⚠", title: "unexpected result", detail: "Stop the operation, leave the vessel where it is, and read the warning before continuing." },
  ];
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open aria-modal="true" aria-labelledby="safety-title" onclick={(event) => event.stopPropagation()}>
    <header>
      <span class="mark" aria-hidden="true">✦</span>
      <span>
        <small>{t("lab wall reference")}</small>
        <h2 id="safety-title">{t("safety station")}</h2>
      </span>
      <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    </header>

    <p class="lead">{t("The simulation can show hazardous chemistry safely, but the same actions in a real laboratory require supervision, protective equipment, and a risk assessment.")}</p>

    <div class="station-grid">
      {#each stations as station (station.title)}
        <article>
          <span class="station-icon" aria-hidden="true">{station.icon}</span>
          <div><h3>{t(station.title)}</h3><p>{t(station.detail)}</p></div>
        </article>
      {/each}
    </div>

    <aside>
      <strong>{t("simulation boundary")}</strong>
      <p>{t("Kerotakis supports learning and planning. It does not replace real laboratory instruction or safety training.")}</p>
    </aside>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 30; display: grid; place-items: center; padding: 1rem; background: var(--scrim); backdrop-filter: blur(5px); }
  dialog { position: static; width: min(92vw, 700px); max-height: 90vh; margin: 0; padding: 0; overflow: auto; border: 1px solid color-mix(in srgb, var(--success) 50%, var(--edge)); border-radius: 22px; color: var(--ink); background: var(--surface); box-shadow: 0 24px 70px var(--overlay-shadow); }
  header { display: flex; align-items: center; gap: .8rem; padding: 1rem 1.1rem; background: linear-gradient(120deg, color-mix(in srgb, var(--success) 17%, var(--surface)), color-mix(in srgb, var(--discovery) 13%, var(--surface))); }
  header > span:nth-child(2) { display: flex; flex-direction: column; }
  .mark { width: 42px; height: 42px; display: grid; place-items: center; flex: none; border-radius: 13px; color: var(--on-accent); background: var(--success); font-size: 1.3rem; transform: rotate(-4deg); }
  header small { color: var(--success); font-size: .58rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  h2 { margin: .08rem 0 0; font-size: 1.15rem; }
  .icon-close { margin-left: auto; }
  .lead { margin: 0; padding: 1rem 1.1rem .2rem; color: var(--dim); font-size: .82rem; line-height: 1.5; }
  .station-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .65rem; padding: .8rem 1.1rem 1rem; }
  article { display: flex; gap: .7rem; padding: .8rem; border: 1px solid var(--edge); border-radius: 15px; background: linear-gradient(145deg, var(--surface-raised), color-mix(in srgb, var(--success) 5%, var(--surface))); }
  .station-icon { width: 36px; height: 36px; display: grid; place-items: center; flex: none; border-radius: 11px; color: var(--success); background: color-mix(in srgb, var(--success) 12%, var(--surface)); font-size: 1.1rem; font-weight: 900; }
  h3 { margin: 0 0 .2rem; font-size: .78rem; }
  article p, aside p { margin: 0; color: var(--dim); font-size: .68rem; line-height: 1.45; }
  aside { margin: 0 1.1rem 1.1rem; padding: .75rem .85rem; border-left: 4px solid var(--discovery); border-radius: 10px; background: color-mix(in srgb, var(--discovery) 8%, var(--surface)); }
  aside strong { display: block; margin-bottom: .2rem; color: var(--discovery); font-size: .62rem; letter-spacing: .08em; text-transform: uppercase; }
  @media (max-width: 560px) { .station-grid { grid-template-columns: 1fr; } }
</style>
