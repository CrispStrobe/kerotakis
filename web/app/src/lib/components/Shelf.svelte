<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import { quickAmounts as amountsFor } from "../amounts";
  import SpeciesChip from "./SpeciesChip.svelte";
  import { t } from "../i18n.svelte";
  import { reagentAccess } from "../catalogProgress";
  import type { LabMode } from "../worldState";

  let {
    items,
    register,
    target,
    onadd,
    kit = null,
    mode = "sandbox",
    completed = 0,
  }: {
    items: ShelfItem[];
    register: string;
    target: number;
    onadd: (line: string) => void;
    /** During a lesson: the reagents its own commands use. */
    kit?: string[] | null;
    mode?: LabMode;
    completed?: number;
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

  /** One tap narrows to a phase; the chips only exist when useful. */
  let phase = $state<string | null>(null);
  const phases = $derived([...new Set(visible.map((s) => s.phase))].sort());

  const filtered = $derived(
    visible.filter((s) => {
      if (phase && s.phase !== phase) return false;
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

<section class="shelf" aria-label={t("reagent shelf")}>
  {#if kit !== null && kit.length > 0}
    <div class="kit-toggle" role="radiogroup" aria-label={t("shelf contents")}>
      <button role="radio" aria-checked={kitOnly} class:on={kitOnly} onclick={() => (kitOnly = true)}>
        {t("the kit ({count})", { count: kit.length })}
      </button>
      <button role="radio" aria-checked={!kitOnly} class:on={!kitOnly} onclick={() => (kitOnly = false)}>
        {t("everything")}
      </button>
    </div>
  {/if}
  <input
    type="search"
    placeholder={t("find a substance…")}
    aria-label={t("find a substance")}
    bind:value={query}
  />
  {#if phases.length > 1}
    <div class="phases" role="radiogroup" aria-label={t("phase filter")}>
      {#each phases as p (p)}
        <button
          role="radio"
          aria-checked={phase === p}
          class:on={phase === p}
          onclick={() => (phase = phase === p ? null : p)}
        >
          {t(p)}
        </button>
      {/each}
    </div>
  {/if}
  <ul>
    {#each filtered as item (item.key)}
      {@const access = reagentAccess(mode, completed, item, kit?.includes(item.key) ?? false)}
      <li>
        <button
          class="species"
          class:locked={!access.available}
          aria-expanded={open === item.key}
          aria-disabled={!access.available}
          draggable={access.available}
          ondragstart={(e) => {
            if (!access.available) return;
            e.dataTransfer?.setData(
              "application/x-kero-species",
              JSON.stringify({ key: item.key, phase: item.phase }),
            );
          }}
          onclick={() => (open = open === item.key ? null : item.key)}
        >
          <SpeciesChip {item} />
          <span class="name">{t(item.name)}</span>
          <span class="formula">{item.formula}</span>
          {#if access.loaned}<span class="loan">{t("mission kit")}</span>{/if}
          {#if !access.available}<span class="lock" aria-hidden="true">⌁</span>{/if}
        </button>
        {#if open === item.key}
          {#if access.available}
            <div class="amounts" role="group" aria-label={t("amount of {name}", { name: t(item.name) })}>
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
                    aria-label={t("custom amount")}
                    bind:value={custom}
                  />
                </form>
              {/if}
            </div>
          {:else}
            <p class="stock-lock">{access.minimumCompleted === 1
              ? t("Permanent stock unlocks after one completed mission. Mission kits loan required materials.")
              : t("Permanent stock unlocks after {count} completed missions. Mission kits loan required materials.", { count: access.minimumCompleted })}</p>
          {/if}
        {/if}
      </li>
    {/each}
    {#if filtered.length === 0}
      <li class="none">{t("nothing on the shelf matches")}</li>
    {/if}
  </ul>
  <p class="tally">
    {filtered.length === items.length
      ? t("{count} substances — every one computed, none painted on", { count: items.length })
      : t("{shown} of {total} substances", { shown: filtered.length, total: items.length })}
  </p>
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
  .phases {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin: 0.4rem 0.8rem 0;
  }
  .phases button {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    font: inherit;
    font-size: 0.72rem;
    padding: 0.15rem 0.6rem;
    cursor: pointer;
  }
  .phases button.on {
    color: var(--ink);
    border-color: var(--hot);
  }
  .none {
    color: var(--dim);
    font-size: 0.8rem;
    padding: 0.6rem 0.8rem;
    list-style: none;
  }
  .tally {
    margin: 0;
    padding: 0.35rem 0.8rem 0.6rem;
    color: var(--dim);
    font-size: 0.7rem;
    border-top: 1px solid var(--edge);
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
    margin: 0.65rem;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 11px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.55rem 0.7rem;
    min-height: 44px;
  }
  input[type="search"]:focus {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 13%, transparent);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0.65rem 0.8rem;
    overflow-y: auto;
  }
  .species {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.32rem;
    background: color-mix(in srgb, var(--surface-raised) 76%, transparent);
    border: 1px solid transparent;
    border-radius: 11px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    text-align: left;
    padding: 0.55rem 0.6rem;
    cursor: pointer;
    min-height: 40px;
  }
  .species:hover .name {
    color: var(--primary);
  }
  .species:hover,
  .species[aria-expanded="true"] {
    border-color: color-mix(in srgb, var(--primary) 45%, var(--edge));
    background: color-mix(in srgb, var(--primary) 8%, var(--surface-raised));
  }
  .species.locked { opacity: .62; border-color: color-mix(in srgb, var(--edge) 75%, transparent); cursor: pointer; filter: saturate(.55); }
  .species.locked:hover .name { color: var(--ink); }
  .lock { color: var(--dim); font-weight: 900; }
  .loan { padding: .12rem .28rem; border-radius: 6px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 11%, var(--surface)); font-size: .48rem; font-weight: 850; text-transform: uppercase; }
  .stock-lock { margin: 0 .2rem .5rem; padding: .5rem; border-left: 3px solid var(--instrument); border-radius: 7px; color: var(--dim); background: color-mix(in srgb, var(--instrument) 7%, transparent); font-size: .68rem; line-height: 1.35; }
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
    border-color: var(--action);
    color: var(--action);
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
