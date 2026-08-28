<script lang="ts">
  import { t } from "../i18n.svelte";

  let {
    vessel,
    onwater,
    onequipment,
    onclose,
  }: {
    vessel: number;
    onwater: () => void;
    onequipment: () => void;
    onclose: () => void;
  } = $props();
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open aria-modal="true" aria-labelledby="utility-title" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
    <header>
      <span class="mark" aria-hidden="true">⌁</span>
      <span><small>{t("lab wall utility")}</small><h2 id="utility-title">{t("utility station")}</h2></span>
      <button class="close" aria-label={t("close")} onclick={onclose}>×</button>
    </header>
    <p class="lead">{t("Connect supplies to the selected workspace: vessel v{vessel}.", { vessel: vessel + 1 })}</p>

    <div class="stations">
      <button class="station water" onclick={onwater}>
        <span class="station-icon" aria-hidden="true">●</span>
        <span><strong>{t("water supply")}</strong><small>{t("Choose a measured amount of water for the selected vessel.")}</small></span>
        <b aria-hidden="true">→</b>
      </button>
      <button class="station power" onclick={onequipment}>
        <span class="station-icon" aria-hidden="true">ϟ</span>
        <span><strong>{t("power and apparatus")}</strong><small>{t("Open powered instruments, probes, heaters, and separators.")}</small></span>
        <b aria-hidden="true">→</b>
      </button>
      <article class="station waste">
        <span class="station-icon" aria-hidden="true">⌫</span>
        <span><strong>{t("waste station")}</strong><small>{t("Chemical contents are never discarded silently. Empty vessels can be removed at the bench; disposal chemistry remains an explicit operation.")}</small></span>
      </article>
    </div>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 82; display: grid; place-items: center; padding: 1rem; background: rgb(5 25 45 / 64%); backdrop-filter: blur(10px) saturate(1.12); }
  dialog { position: static; width: min(43rem, 94vw); margin: 0; padding: 0; overflow: hidden; border: 1px solid color-mix(in srgb, var(--cool) 48%, var(--edge)); border-radius: 23px; color: var(--ink); background: var(--surface); box-shadow: 0 28px 80px rgb(2 18 34 / 44%); }
  header { display: flex; align-items: center; gap: .75rem; padding: 1rem 1.1rem; background: linear-gradient(110deg, color-mix(in srgb, var(--cool) 17%, var(--surface)), color-mix(in srgb, var(--instrument) 10%, var(--surface))); }
  header > span:nth-child(2) { display: grid; gap: .05rem; }
  header small { color: var(--instrument); font-size: .58rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.18rem; }
  .mark { width: 42px; height: 42px; display: grid; place-items: center; border-radius: 13px; color: var(--on-accent); background: linear-gradient(145deg, var(--cool), var(--instrument)); font-size: 1.35rem; }
  .close { width: 38px; height: 38px; margin-left: auto; border: 1px solid var(--edge); border-radius: 50%; color: var(--ink); background: var(--surface); cursor: pointer; font: inherit; font-size: 1.2rem; }
  .lead { margin: 0; padding: 1rem 1.1rem .25rem; color: var(--dim); font-size: .8rem; }
  .stations { display: grid; gap: .65rem; padding: .8rem 1.1rem 1.15rem; }
  .station { width: 100%; min-height: 68px; display: grid; grid-template-columns: 42px minmax(0, 1fr) auto; align-items: center; gap: .75rem; padding: .7rem .8rem; border: 1px solid var(--edge); border-radius: 15px; color: var(--ink); background: var(--surface-raised); font: inherit; text-align: left; }
  button.station { cursor: pointer; transition: transform 150ms ease, border-color 150ms ease, box-shadow 150ms ease; }
  button.station:hover, button.station:focus-visible { transform: translateX(3px); border-color: var(--primary); box-shadow: 0 8px 20px var(--shadow); }
  .station > span:nth-child(2) { min-width: 0; display: grid; gap: .16rem; }
  .station strong { font-size: .78rem; }
  .station small { color: var(--dim); font-size: .67rem; line-height: 1.4; }
  .station b { color: var(--primary); }
  .station-icon { width: 42px; height: 42px; display: grid; place-items: center; border-radius: 13px; color: var(--on-accent); background: var(--instrument); font-size: 1.15rem; }
  .water .station-icon { background: var(--cool); }
  .power .station-icon { color: var(--ink); background: var(--action); }
  .waste { background: color-mix(in srgb, var(--warning) 6%, var(--surface-raised)); }
  .waste .station-icon { color: var(--warning); background: color-mix(in srgb, var(--warning) 13%, var(--surface)); }
</style>
