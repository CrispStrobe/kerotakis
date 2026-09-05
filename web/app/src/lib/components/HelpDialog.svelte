<script lang="ts">
  import { t } from "../i18n.svelte";
  let { onclose }: { onclose: () => void } = $props();

  const mod =
    typeof navigator !== "undefined" && /Mac/.test(navigator.platform) ? "⌘" : "Ctrl";
  const keys: [string, string][] = [
    [`${mod} K`, "focus the command bar"],
    ["↑ / ↓", "walk command history"],
    [`${mod} Z`, "undo (replays the bench)"],
    [`${mod} ⇧ Z`, "redo"],
    ["?", "this help"],
    ["Esc", "close panels"],
  ];
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <dialog open
    class="help"
    aria-modal="true"
    aria-label={t("keyboard shortcuts")}
    onclick={(e) => e.stopPropagation()}
  >
    <button class="icon-close corner" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    <h2>{t("Keyboard")}</h2>
    <dl>
      {#each keys as [key, what] (key)}
        <dt><kbd>{key}</kbd></dt>
        <dd>{t(what)}</dd>
      {/each}
    </dl>
    <p class="note">
      {t("Every button and drag also works from the keyboard — vessels are buttons, and everything you do is a command you can read back in the notebook.")}
    </p>
  </dialog>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: var(--scrim);
    display: grid;
    place-items: center;
    /* Above the topbar (20) and the tools panel (40): at 10 this
       modal opened underneath the chrome and lost its heading. */
    z-index: 50;
  }
  .help {
    position: relative;
    margin: 0;
    color: var(--ink);
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 10px;
    padding: 1.2rem 1.4rem;
    max-width: 22rem;
    width: calc(100vw - 2rem);
  }
  h2 {
    margin: 0 0 0.8rem;
    font-size: 0.95rem;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 0.9rem;
    margin: 0;
  }
  dt {
    text-align: right;
  }
  dd {
    margin: 0;
    color: var(--dim);
  }
  kbd {
    background: var(--panel-raised);
    border: 1px solid var(--edge-strong);
    border-radius: 4px;
    padding: 0 0.35rem;
    font: inherit;
    font-size: 0.8rem;
  }
  .note {
    font-size: 0.78rem;
    color: var(--dim);
  }
  button {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.3rem 0.8rem;
    cursor: pointer;
    min-height: 36px;
  }
</style>
