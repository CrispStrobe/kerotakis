<script lang="ts">
  import { ELEMENTS, elementsInFormula, type ElementInfo } from "../elements";
  import type { ShelfItem } from "../session.svelte";
  import SpeciesChip from "./SpeciesChip.svelte";
  import { t } from "../i18n.svelte";

  let {
    shelf,
    register,
    onadd,
    onclose,
  }: {
    shelf: ShelfItem[];
    register: string;
    onadd: (item: ShelfItem) => void;
    onclose: () => void;
  } = $props();

  let picked = $state<ElementInfo | null>(null);

  // Grid placement: main body rows 1-7 by period/group; the f-block sits
  // below with a gap row.
  function place(e: ElementInfo): { col: number; row: number } {
    if (e.block === "f" || (e.category === "lanthanide" && e.z !== 71) || (e.category === "actinide" && e.z !== 103)) {
      if (e.z >= 57 && e.z <= 70) return { col: 4 + (e.z - 57), row: 9 };
      if (e.z >= 89 && e.z <= 102) return { col: 4 + (e.z - 89), row: 10 };
    }
    if (e.z === 71) return { col: 3, row: 6 };
    if (e.z === 103) return { col: 3, row: 7 };
    return { col: e.group, row: e.period };
  }

  /** The lab connection: shelf species containing the picked element. */
  const inLab = $derived(
    picked
      ? shelf.filter((s) => elementsInFormula(s.formula).includes(picked!.symbol))
      : [],
  );
  const flames = $derived(
    [...new Set(inLab.map((s) => s.flame).filter(Boolean))] as string[],
  );
  const CATEGORY_WORDS: Record<string, string> = {
    alkali: "alkali metal",
    alkaline: "alkaline-earth metal",
    transition: "transition metal",
    post: "post-transition metal",
    metalloid: "metalloid",
    nonmetal: "nonmetal",
    halogen: "halogen",
    noble: "noble gas",
    lanthanide: "lanthanide",
    actinide: "actinide",
    unknown: "properties not yet established",
  };
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(e) => e.key === "Escape" && onclose()}>
  <section
    class="table-panel"
    role="dialog"
    aria-modal="true"
    aria-label={t("periodic table")}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{t("the elements")}</h2>
      <span class="hint">{t("tap one to see what the lab has of it")}</span>
      <button class="close" onclick={onclose}>{t("close")}</button>
    </header>

    <div class="grid" role="listbox" aria-label={t("elements")}>
      {#each ELEMENTS as e (e.z)}
        {@const p = place(e)}
        <button
          class={`el cat-${e.category}`}
          class:picked={picked?.z === e.z}
          style={`grid-column:${p.col};grid-row:${p.row}`}
          role="option"
          aria-selected={picked?.z === e.z}
          title={`${t(e.name)} (${e.z})`}
          onclick={() => (picked = e)}
        >
          <span class="z">{e.z}</span>
          <span class="sym">{e.symbol}</span>
        </button>
      {/each}
    </div>

    {#if picked}
      <aside class="detail">
        <h3>{t(picked.name)} <small>{picked.symbol} · {picked.z}</small></h3>
        <p class="cat">{t(CATEGORY_WORDS[picked.category] ?? picked.category)}</p>
        {#if register !== "lv1"}
          <p class="facts">
            {t("period {period} · group {group} · {block}-block", { period: picked.period, group: picked.group, block: picked.block })}
          </p>
        {/if}
        {#if flames.length > 0}
          <p class="facts">{t("flame test: {flames}", { flames: flames.map((f) => t(f)).join(", ") })}</p>
        {/if}
        {#if inLab.length > 0}
          <p class="facts">{t("on the shelf, containing {symbol}:", { symbol: picked.symbol })}</p>
          <ul class="species">
            {#each inLab as item (item.key)}
              <li>
                <button class="add" onclick={() => onadd(item)}>
                  <SpeciesChip {item} />
                  <span>{t(item.name)}</span>
                  <span class="formula">{item.formula}</span>
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="facts none">
            {t("nothing on the shelf contains {symbol} yet — the registry grows by provenance-carrying tranches, not by wishful entries.", { symbol: picked.symbol })}
          </p>
        {/if}
      </aside>
    {/if}
  </section>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 50%);
    display: grid;
    place-items: center;
    z-index: 10;
    padding: 1rem;
  }
  .table-panel {
    background: var(--bg);
    border: 1px solid var(--edge);
    border-radius: 12px;
    padding: 1rem;
    max-width: min(96vw, 1000px);
    max-height: 92vh;
    overflow: auto;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 0.8rem;
    margin-bottom: 0.7rem;
  }
  h2 {
    margin: 0;
    font-size: 1rem;
  }
  .hint {
    color: var(--dim);
    font-size: 0.78rem;
  }
  .close {
    margin-left: auto;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.7rem;
    cursor: pointer;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(18, minmax(30px, 1fr));
    gap: 2px;
  }
  .el {
    aspect-ratio: 1;
    border: 1px solid var(--edge);
    border-radius: 4px;
    background: var(--panel);
    color: var(--ink);
    font: inherit;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1px;
    min-width: 0;
  }
  .el:hover,
  .el.picked {
    outline: 2px solid var(--hot);
    outline-offset: -1px;
  }
  .z {
    font-size: 0.5rem;
    color: var(--dim);
    line-height: 1;
  }
  .sym {
    font-size: clamp(0.6rem, 1.6vw, 0.85rem);
    font-weight: 600;
    line-height: 1.1;
  }
  /* Category tints on the border-left, plus a subtle wash — category is
     also written in words in the detail, never colour alone. */
  .cat-alkali { border-left: 3px solid #d98a4a; }
  .cat-alkaline { border-left: 3px solid #c9a227; }
  .cat-transition { border-left: 3px solid #6fa8c7; }
  .cat-post { border-left: 3px solid #8ba8a0; }
  .cat-metalloid { border-left: 3px solid #a487c9; }
  .cat-nonmetal { border-left: 3px solid #86b06a; }
  .cat-halogen { border-left: 3px solid #5ea5a5; }
  .cat-noble { border-left: 3px solid #b06a86; }
  .cat-lanthanide { border-left: 3px solid #b65a3a; }
  .cat-actinide { border-left: 3px solid #7d6a5a; }
  .cat-unknown { border-left: 3px dashed var(--edge-strong); }
  .detail {
    margin-top: 0.8rem;
    border-top: 1px solid var(--edge);
    padding-top: 0.6rem;
  }
  h3 {
    margin: 0;
    font-size: 0.95rem;
  }
  h3 small {
    color: var(--dim);
    font-weight: 400;
  }
  .cat {
    margin: 0.15rem 0;
    color: var(--warn);
    font-size: 0.82rem;
  }
  .facts {
    margin: 0.15rem 0;
    color: var(--dim);
    font-size: 0.8rem;
  }
  .none {
    font-style: italic;
  }
  .species {
    list-style: none;
    margin: 0.3rem 0 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }
  .add {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.2rem 0.6rem 0.2rem 0.3rem;
    cursor: pointer;
    min-height: 34px;
  }
  .add:hover {
    border-color: var(--hot);
  }
  .formula {
    color: var(--dim);
  }
</style>
