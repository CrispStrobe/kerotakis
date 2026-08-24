<script lang="ts">
  import { REGISTERS } from "../session.svelte";

  let { value, onchange }: { value: string; onchange: (level: string) => void } = $props();
</script>

<!-- The dial is the product: same bench, mid-session switchable detail. -->
<div class="dial" role="radiogroup" aria-label="detail level">
  {#each REGISTERS as reg (reg.level)}
    <button
      role="radio"
      aria-checked={value === reg.level}
      class:active={value === reg.level}
      onclick={() => onchange(reg.level)}
    >
      <span class="lv">{reg.level}</span>
      {reg.label}
    </button>
  {/each}
</div>

<style>
  .dial {
    display: flex;
    border: 1px solid var(--edge);
    border-radius: 999px;
    overflow: hidden;
  }
  button {
    background: none;
    border: 0;
    color: var(--dim);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.45rem 0.9rem;
    cursor: pointer;
    min-height: 36px;
  }
  button.active {
    background: var(--panel-raised);
    color: var(--ink);
  }
  .lv {
    color: var(--hot);
    margin-right: 0.3rem;
    font-size: 0.72rem;
  }
</style>
