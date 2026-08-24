<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import { quickAmounts as amountsFor } from "../amounts";
  import SpeciesChip from "./SpeciesChip.svelte";

  let {
    items,
    register,
    target,
    onadd,
    kit = null,
  }: {
    items: ShelfItem[];
    register: string;
    target: number;
    onadd: (line: string) => void;
    /** During a lesson: the reagents its own commands use. */
    kit?: string[] | null;
  } = $props();

  let query = $state("");
  /** Kit view on by default while a lesson runs; the sandbox is one tap away. */
  let kitOnly = $state(true);
  const kitActive = $derived(kitOnly && kit !== null && kit.length > 0);
  const visible = $derived(
    kitActive ? items.filter((s) => kit!.includes(s.key)) : items,
  );
  let open = $state<string | null>(null);
  let custom = $state("");

  const filtered = $derived(
    visible.filter((s) => {
      const q = query.trim().toLowerCase();
      if (!q) return true;
      return (
        s.name.toLowerCase().includes(q) ||
        s.formula.toLowerCase().includes(q) ||
        s.key.toLowerCase().includes(q)
      );
    }),
  );

  const quickAmounts = (phase: string) => amountsFor(register, phase);

  function add(item: ShelfItem, amount: string) {
    const a = amount.trim();
    if (!a) return;
    onadd(`add v${target + 1} ${item.key} ${a}`);
    open = null;
    custom = "";
  }
</script>

<section class="shelf" aria-label="reagent shelf">
  {#if kit !== null && kit.length > 0}
    <div class="kit-toggle" role="radiogroup" aria-label="shelf contents">
      <button role="radio" aria-checked={kitOnly} class:on={kitOnly} onclick={() => (kitOnly = true)}>
        the kit ({kit.length})
      </button>
      <button role="radio" aria-checked={!kitOnly} class:on={!kitOnly} onclick={() => (kitOnly = false)}>
        everything
      </button>
    </div>
  {/if}
  <input
    type="search"
    placeholder="find a substance…"
    aria-label="find a substance"
    bind:value={query}
  />
  <ul>
    {#each filtered as item (item.key)}
      <li>
        <button
          class="species"
          aria-expanded={open === item.key}
          draggable="true"
          ondragstart={(e) => {
            e.dataTransfer?.setData(
              "application/x-kero-species",
              JSON.stringify({ key: item.key, phase: item.phase }),
            );
          }}
          onclick={() => (open = open === item.key ? null : item.key)}
        >
          <SpeciesChip {item} />
          <span class="name">{item.name}</span>
          <span class="formula">{item.formula}</span>
        </button>
        {#if open === item.key}
          <div class="amounts" role="group" aria-label={`amount of ${item.name}`}>
            {#each quickAmounts(item.phase) as amount (amount)}
              <button class="amount" onclick={() => add(item, amount)}>{amount}</button>
            {/each}
            {#if register !== "lv1"}
              <form
                class="custom"
                onsubmit={(e) => {
                  e.preventDefault();
                  add(item, custom);
                }}
              >
                <input
                  type="text"
                  placeholder="5g, 0.1mol…"
                  aria-label="custom amount"
                  bind:value={custom}
                />
              </form>
            {/if}
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</section>

<style>
  .shelf {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .kit-toggle {
    display: flex;
    margin: 0.8rem 0.8rem 0;
    border: 1px solid var(--edge);
    border-radius: 999px;
    overflow: hidden;
  }
  .kit-toggle button {
    flex: 1;
    background: none;
    border: 0;
    color: var(--dim);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.35rem;
    cursor: pointer;
    min-height: 36px;
  }
  .kit-toggle button.on {
    background: var(--panel-raised);
    color: var(--ink);
  }
  input[type="search"] {
    margin: 0.8rem;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.45rem 0.6rem;
    min-height: 40px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0.8rem 0.8rem;
    overflow-y: auto;
  }
  .species {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--edge);
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    text-align: left;
    padding: 0.5rem 0.2rem;
    cursor: pointer;
    min-height: 40px;
  }
  .species:hover .name {
    color: var(--hot);
  }
  .name {
    flex: 1;
  }
  .formula {
    color: var(--dim);
  }
  .amounts {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    padding: 0.5rem 0.2rem;
  }
  .amount {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.3rem 0.7rem;
    cursor: pointer;
    min-height: 36px;
  }
  .amount:hover {
    border-color: var(--hot);
  }
  .custom input {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.3rem 0.7rem;
    width: 8rem;
  }
</style>
