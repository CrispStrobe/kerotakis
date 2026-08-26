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
  <section
    class="help"
    role="dialog"
    aria-modal="true"
    aria-label={t("keyboard shortcuts")}
    onclick={(e) => e.stopPropagation()}
  >
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
    <button onclick={onclose}>{t("close")}</button>
  </section>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 45%);
    display: grid;
    place-items: center;
    z-index: 10;
  }
  .help {
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
