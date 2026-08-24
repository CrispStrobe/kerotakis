<script lang="ts">
  import type { ShelfItem } from "../session.svelte";

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

  let query = $state("");
  let open = $state<string | null>(null);
  let custom = $state("");

  const filtered = $derived(
    items.filter((s) => {
      const q = query.trim().toLowerCase();
      if (!q) return true;
      return (
        s.name.toLowerCase().includes(q) ||
        s.formula.toLowerCase().includes(q) ||
        s.key.toLowerCase().includes(q)
      );
    }),
  );

  // Register-aware quick amounts: lv1 speaks kitchen units the grammar
  // already parses; lv2/lv3 speak the lab's. Free text always available.
  function quickAmounts(phase: string): string[] {
    if (register === "lv1") {
      return phase === "liquid" ? ["1cup", "100mL"] : ["1pinch", "1g"];
    }
    return phase === "liquid" ? ["10mL", "100mL", "1mol"] : ["1g", "0.01mol", "0.1mol"];
  }

  function add(item: ShelfItem, amount: string) {
    const a = amount.trim();
    if (!a) return;
    onadd(`add v${target + 1} ${item.key} ${a}`);
    open = null;
    custom = "";
  }
</script>

<section class="shelf" aria-label="reagent shelf">
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
          onclick={() => (open = open === item.key ? null : item.key)}
        >
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
    justify-content: space-between;
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
