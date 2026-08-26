<script lang="ts">
  import { t } from "../i18n.svelte";
  let {
    vessel,
    options,
    busy,
    onrun,
    onclose,
  }: {
    vessel: number;
    options: string[];
    busy: boolean;
    onrun: (line: string) => void;
    onclose: () => void;
  } = $props();

  let chosen = $state("");
  const line = $derived(chosen ? `react v${vessel + 1} ${chosen}` : null);
</script>

<section class="react" aria-label={t("curated reaction on v{vessel}", { vessel: vessel + 1 })}>
  <strong>{t("curated reaction")} · v{vessel + 1}</strong>
  <span class="hint">{t("verified family templates the engine can run")}</span>
  <div class="row">
    <select bind:value={chosen}>
      <option value="">{t("choose…")}</option>
      {#each options as name (name)}<option value={name}>{name}</option>{/each}
    </select>
    <button class="run" disabled={busy || line === null} onclick={() => line && onrun(line)}>
      {t("run")}
    </button>
    <button class="close" onclick={onclose}>{t("put away")}</button>
  </div>
  {#if line}<code>{line}</code>{/if}
</section>

<style>
  .react {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
    font-size: 0.82rem;
  }
  .hint {
    color: var(--dim);
    margin-left: 0.6rem;
    font-size: 0.76rem;
  }
  .row {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.35rem;
    align-items: center;
  }
  select {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    padding: 0.25rem 0.4rem;
    min-height: 34px;
  }
  .run {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    padding: 0.3rem 0.8rem;
    cursor: pointer;
    min-height: 36px;
  }
  .close {
    background: none;
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--dim);
    font: inherit;
    padding: 0.3rem 0.7rem;
    cursor: pointer;
    min-height: 36px;
  }
  code {
    display: block;
    margin-top: 0.3rem;
    color: var(--dim);
    font-size: 0.72rem;
  }
</style>
