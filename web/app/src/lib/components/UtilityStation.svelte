<script lang="ts">
  import { t } from "../i18n.svelte";

  let {
    vessel,
    onwater,
    onequipment,
    onwaste,
    onclose,
    clearable = true,
    disposable = false,
    receipt = null,
  }: {
    vessel: number;
    onwater: () => void;
    onequipment: () => void;
    /** Fired only after the reader has confirmed. */
    onwaste: () => void;
    onclose: () => void;
    /** False when there is nothing on the bench to dispose of. */
    clearable?: boolean;
    /**
     * True when the selected vessel is holding something the engine can
     * pour into the bench's waste container — the `discard vN` verb.
     * False falls the control back to emptying the whole bench, which is
     * all this station could ever do before that verb existed.
     */
    disposable?: boolean;
    /** What the container took last, already worded by the caller. */
    receipt?: string | null;
  } = $props();

  // The waste station read like a control and did nothing: an `<article>`
  // among two buttons, explaining a policy where the other two offered an
  // action. The policy it states is real — nothing is discarded without
  // being asked for — but stating it is not the same as refusing to have
  // a control, and "open waste station" in the remove-vessel dialog led
  // here and then stopped. So it asks, exactly the way the toolbar's own
  // empty control asks, and the press it takes afterwards is the one that
  // does the disposal.
  //
  // What that press means now depends on `disposable`. A vessel with
  // something in it is emptied into the bench's waste container by the
  // engine's own `discard vN` verb — a replayable command, weighed and
  // logged, not a UI deletion. A vessel with nothing in it leaves the
  // station with only the older meaning it ever had: empty the bench.
  let armed = $state(false);
  let timer: ReturnType<typeof setTimeout> | null = null;
  function arm() {
    armed = true;
    if (timer) clearTimeout(timer);
    // Disarms itself: an armed button left on screen becomes a trap.
    timer = setTimeout(() => (armed = false), 8000);
  }
  function disarm() {
    armed = false;
    if (timer) clearTimeout(timer);
    timer = null;
  }
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
      {#if armed}
        <div class="station waste" role="group" aria-label={disposable ? t("empty vessel v{vessel} into the waste container?", { vessel: vessel + 1 }) : t("clear the bench?")}>
          <span class="station-icon" aria-hidden="true">⌫</span>
          <span>
            <strong>{disposable ? t("empty vessel v{vessel} into the waste container?", { vessel: vessel + 1 }) : t("clear the bench?")}</strong>
            <small>{disposable
              ? t("The container keeps what it takes: nothing is destroyed, and it is weighed with everything already in there.")
              : t("empty this bench — the other laboratory is untouched")}</small>
          </span>
          <span class="confirm">
            <button class="yes" onclick={() => { disarm(); onwaste(); }}>{disposable ? t("empty into the waste container") : t("clear the bench")}</button>
            <button class="no" onclick={disarm}>{t("keep it")}</button>
          </span>
        </div>
      {:else}
        <button class="station waste" onclick={arm} disabled={!disposable && !clearable}>
          <span class="station-icon" aria-hidden="true">⌫</span>
          <span><strong>{t("waste station")}</strong><small>{disposable
            ? t("Pour the contents of vessel v{vessel} into the bench's waste container. Nothing goes in until you confirm it here.", { vessel: vessel + 1 })
            : t("Chemical contents are never discarded silently. Empty vessels can be removed at the bench; disposal chemistry remains an explicit operation.")}</small></span>
          <b aria-hidden="true">→</b>
        </button>
      {/if}
      {#if receipt}
        <p class="receipt">{receipt}</p>
      {/if}
    </div>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 82; display: grid; place-items: center; padding: 1rem; background: var(--scrim); backdrop-filter: blur(10px) saturate(1.12); }
  dialog { position: static; width: min(43rem, 94vw); margin: 0; padding: 0; overflow: hidden; border: 1px solid color-mix(in srgb, var(--cool) 48%, var(--edge)); border-radius: 23px; color: var(--ink); background: var(--surface); box-shadow: 0 28px 80px var(--overlay-shadow); }
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
  button.waste:hover:not(:disabled), button.waste:focus-visible:not(:disabled) { border-color: var(--danger); }
  button.waste:disabled { opacity: .5; cursor: default; transform: none; box-shadow: none; }
  .waste b { color: var(--warning); }
  .receipt { margin: 0; padding: .55rem .7rem; border: 1px dashed color-mix(in srgb, var(--warning) 45%, var(--edge)); border-radius: 12px; color: var(--dim); font-size: .68rem; line-height: 1.45; }
  .confirm { display: flex; flex-wrap: wrap; gap: .35rem; }
  .confirm button { min-height: 40px; padding: .4rem .7rem; border: 1px solid var(--edge); border-radius: var(--radius-sm); color: var(--ink); background: var(--surface); font: inherit; font-weight: 650; cursor: pointer; }
  .confirm .yes { color: var(--on-accent); border-color: var(--danger); background: var(--danger); }
</style>
