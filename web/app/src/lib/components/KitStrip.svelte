<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import { quickAmounts } from "../amounts";
  import SpeciesChip from "./SpeciesChip.svelte";

  let {
    items,
    register,
    target,
    onadd,
  }: {
    items: ShelfItem[];
    register: string;
    target: number;
    onadd: (line: string) => void;
  } = $props();

  let open = $state<string | null>(null);

  function add(item: ShelfItem, amount: string) {
    onadd(`add v${target + 1} ${item.key} ${amount}`);
    open = null;
  }
</script>

<div class="kit-strip" role="group" aria-label="kit reagents">
  {#each items as item (item.key)}
    <div class="kit-item" class:expanded={open === item.key}>
      <button
        class="kit-chip"
        aria-expanded={open === item.key}
        onclick={() => (open = open === item.key ? null : item.key)}
        title={item.name}
      >
        <SpeciesChip {item} />
        <span class="label">{item.name}</span>
      </button>
      {#if open === item.key}
        <div class="amounts" role="group" aria-label={`amount of ${item.name}`}>
          {#each quickAmounts(register, item.phase) as amount (amount)}
            <button class="amount" onclick={() => add(item, amount)}>{amount}</button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .kit-strip {
    display: flex;
    gap: 0.3rem;
    overflow-x: auto;
    padding: 0.25rem 0;
    align-items: flex-start;
  }
  .kit-item {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  .kit-chip {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem 0.15rem 0.25rem;
    cursor: pointer;
    white-space: nowrap;
    min-height: 28px;
  }
  .kit-chip:hover {
    border-color: var(--hot);
  }
  .expanded .kit-chip {
    border-color: var(--hot);
  }
  .label {
    max-width: 8rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .amounts {
    display: flex;
    gap: 0.2rem;
    margin-top: 0.2rem;
    padding-left: 0.25rem;
  }
  .amount {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    cursor: pointer;
    white-space: nowrap;
    min-height: 24px;
  }
  .amount:hover {
    border-color: var(--hot);
  }
</style>
