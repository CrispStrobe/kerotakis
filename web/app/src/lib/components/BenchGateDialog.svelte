<!--
  The bench a curated script needs, asked for rather than taken.

  Lessons and the 500 reviewed corpus prompts both name their glassware
  absolutely (`v1`, `v2`) and were written against the bench the engine
  hands over: one empty vessel. Started on a bench that still holds an
  earlier run, they pour into someone else's chemistry and then report the
  result as their own — the electrode lesson measuring the sherbet
  lesson's pH.

  Emptying glassware is never this app's decision to take quietly: the
  disposal station weighs and logs it, the remove-vessel dialog refuses a
  non-empty vessel and offers to pour it out instead. So the precondition
  is put to the learner, in the same three-option shape the catalogue's
  own run gate uses. It is only shown when the bench really does hold
  something, so the ordinary path — start a lesson on a clean bench — is
  still one press.
-->
<script lang="ts">
  import { t } from "../i18n.svelte";

  let {
    title,
    occupied,
    onclear,
    onkeep,
    oncancel,
  }: {
    /** What is waiting to start, in the reader's language. */
    title: string;
    /** Whether there is chemistry in the glassware, or only spare vessels. */
    occupied: boolean;
    onclear: () => void;
    onkeep: () => void;
    oncancel: () => void;
  } = $props();
</script>

<div class="scrim" role="presentation" onclick={oncancel} onkeydown={(event) => event.key === "Escape" && oncancel()}>
  <dialog
    open
    aria-modal="true"
    aria-labelledby="bench-gate-title"
    onclick={(event) => event.stopPropagation()}
    onkeydown={(event) => { event.stopPropagation(); if (event.key === "Escape") oncancel(); }}
  >
    <header>
      <span class="mark" aria-hidden="true">⌫</span>
      <span>
        <small>{t("before it starts")}</small>
        <h2 id="bench-gate-title">{t("your bench is not empty")}</h2>
      </span>
    </header>

    <section>
      <p class="what">{title}</p>
      <p>{t("It begins in v1 and numbers the rest of its glassware itself.")}</p>
      <p>
        {occupied
          ? t("Your bench still holds material from earlier work, so its readings would be your work and this one mixed together.")
          : t("Your bench carries more glassware than this expects, so the vessels it asks for would not be the ones it gets.")}
      </p>
    </section>

    <div class="decision">
      <strong>{t("Nothing is emptied unless you say so")}</strong>
      <p>{t("Clearing empties this laboratory and closes its journal; the note stays in the feed. Keeping your bench starts anyway, and the feed records that the readings are not this experiment's alone.")}</p>
    </div>

    <footer>
      <button class="secondary" onclick={oncancel}>{t("cancel")}</button>
      <button class="keep" onclick={onkeep}>{t("start on this bench as it is")}</button>
      <button class="clear" onclick={onclear}>{t("clear the bench, then start")}</button>
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
  .mark { width: 42px; height: 42px; display: grid; place-items: center; flex: none; border-radius: 13px; color: var(--on-accent); background: linear-gradient(145deg, var(--warning), var(--hot)); font-size: 1.3rem; font-weight: 850; }
  section { margin: 1rem 1.1rem .7rem; padding: .85rem; border: 1px solid color-mix(in srgb, var(--warning) 38%, var(--edge)); border-radius: 15px; background: var(--surface-raised); }
  section p { margin: .18rem 0 0; color: var(--dim); font-size: .72rem; line-height: 1.45; }
  section .what { margin: 0 0 .3rem; color: var(--ink); font-size: .86rem; font-weight: 750; }
  .decision { margin: 0 1.1rem .8rem; padding: .75rem .85rem; border-left: 4px solid var(--warning); border-radius: 9px; background: color-mix(in srgb, var(--warning) 7%, var(--surface-raised)); }
  .decision strong { font-size: .76rem; }
  .decision p { margin: .18rem 0 0; color: var(--dim); font-size: .72rem; line-height: 1.45; }
  footer { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: .45rem; padding: .85rem 1.1rem 1.1rem; }
  footer button { min-height: 40px; padding: .45rem .75rem; border: 1px solid var(--edge); border-radius: 11px; color: var(--ink); background: var(--surface-raised); font: inherit; font-size: .72rem; font-weight: 750; cursor: pointer; }
  footer .keep { color: var(--warning); border-color: color-mix(in srgb, var(--warning) 55%, var(--edge)); background: color-mix(in srgb, var(--warning) 8%, var(--surface)); }
  footer .clear { color: var(--on-accent); border-color: var(--primary); background: var(--primary); }
  @media (max-width: 430px) {
    footer { display: grid; grid-template-columns: 1fr; }
    footer button { width: 100%; }
  }
</style>
