<script lang="ts">
  import { REGISTERS } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let { value, onchange }: { value: string; onchange: (level: string) => void } = $props();
</script>

<!-- The dial is the product: same bench, mid-session switchable detail. -->
<div class="dial" role="radiogroup" aria-label={t("detail level")}>
  {#each REGISTERS as reg (reg.level)}
    <button
      role="radio"
      aria-checked={value === reg.level}
      class:active={value === reg.level}
      onclick={() => onchange(reg.level)}
    >
      <span class="lv">{reg.level}</span>
      {t(reg.label)}
    </button>
  {/each}
</div>

<style>
  .dial {
    display: flex;
    border: 1px solid var(--edge);
    border-radius: 999px;
    overflow: hidden;
    padding: 3px;
    background: var(--surface-raised);
  }
  button {
    background: none;
    border: 0;
    color: var(--dim);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.4rem 0.75rem;
    cursor: pointer;
    min-height: 36px;
  }
  button.active {
    border-radius: 999px;
    background: var(--surface);
    color: var(--primary);
    box-shadow: 0 2px 8px var(--shadow);
  }
  .lv {
    color: var(--action);
    margin-right: 0.3rem;
    font-size: 0.72rem;
  }
</style>
