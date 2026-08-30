<script lang="ts">
  import { onMount } from "svelte";
  import {
    ELEMENTS,
    LAB_ELEMENTS,
    contentRoutesForElement,
    elementCapability,
    elementsMatchingSearch,
    shelfItemsContainingElement,
    type ElementExperimentIndexEntry,
    type ElementCoverageReport,
    type ElementInfo,
    type ElementLessonIndexEntry,
  } from "../elements";
  import type { ShelfItem } from "../session.svelte";
  import SpeciesChip from "./SpeciesChip.svelte";
  import { t } from "../i18n.svelte";

  let {
    shelf,
    register,
    coverage = null,
    lessons = [],
    experiments = [],
    onadd,
    onlesson,
    onexperiment,
    onclose,
  }: {
    shelf: ShelfItem[];
    register: string;
    coverage?: ElementCoverageReport | null;
    lessons?: ElementLessonIndexEntry[];
    experiments?: ElementExperimentIndexEntry[];
    onadd: (item: ShelfItem) => void;
    onlesson?: (file: string) => void;
    onexperiment?: (id: string) => void;
    onclose: () => void;
  } = $props();

  let picked = $state<ElementInfo | null>(null);
  let fullTable = $state(false);
  let query = $state("");
  const visibleElements = $derived(elementsMatchingSearch(
    query,
    fullTable ? ELEMENTS : LAB_ELEMENTS,
    shelf,
    (value) => t(value),
  ));

  onMount(() => {
    try {
      fullTable = localStorage.getItem("kerotakis-periodic-table") === "full";
    } catch {
      // Storage is optional; the approachable table remains the default.
    }
  });

  function setFullTable(value: boolean) {
    fullTable = value;
    if (!value && picked && !LAB_ELEMENTS.some((element) => element.z === picked!.z)) {
      picked = null;
    }
    try {
      localStorage.setItem("kerotakis-periodic-table", value ? "full" : "lab");
    } catch {
      // A privacy-restricted host may reject storage; the view still works.
    }
  }

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
  function installedExamples(symbol: string): ShelfItem[] {
    const generated = coverage?.elements.find((entry) => entry.symbol === symbol);
    if (!generated) return shelfItemsContainingElement(symbol, shelf);
    const keys = new Set(generated.examples.map((example) => example.shelf_key));
    return shelf.filter((item) => keys.has(item.key));
  }
  const inLab = $derived(
    picked
      ? installedExamples(picked.symbol)
      : [],
  );
  const flames = $derived(
    [...new Set(inLab.map((s) => s.flame).filter(Boolean))] as string[],
  );
  const routes = $derived(picked
    ? contentRoutesForElement(picked.symbol, shelf, lessons, experiments)
    : []);
  const capability = $derived(routes.some((route) => route.kind === "lesson")
    ? "lesson_backed"
    : routes.length > 0
      ? "reacting"
      : coverage?.elements.find((entry) => entry.symbol === picked?.symbol)?.capability
        ?? elementCapability(inLab, routes));
  const CAPABILITY_WORDS = {
    identity_only: "identity only",
    add_observe: "add and observe",
    property_backed: "property backed",
    reacting: "reacting",
    lesson_backed: "lesson backed",
  } as const;
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
  <dialog open
    class="table-panel"
    aria-modal="true"
    aria-label={t("periodic table")}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{t("the elements")}</h2>
      <span class="hint">{t("tap one to see what the lab has of it")}</span>
      <button
        class="mode"
        aria-pressed={fullTable}
        onclick={() => setFullTable(!fullTable)}
      >
        {fullTable ? t("show lab table") : t("show all 118 elements")}
      </button>
      <button class="close" onclick={onclose}>{t("close")}</button>
    </header>

    <label class="search">
      <span>{t("search elements and lab materials")}</span>
      <input
        type="search"
        bind:value={query}
        placeholder={t("symbol, element, formula or material")}
        autocomplete="off"
      />
    </label>

    <div class="grid" role="listbox" aria-label={t("elements")}>
      {#each visibleElements as e (e.z)}
        {@const p = place(e)}
        {@const count = installedExamples(e.symbol).length}
        <button
          class={`el cat-${e.category}`}
          class:picked={picked?.z === e.z}
          class:unsupported={count === 0}
          style={`grid-column:${p.col};grid-row:${p.row}`}
          role="option"
          aria-selected={picked?.z === e.z}
          aria-label={`${t(e.name)}, ${e.symbol}, ${t("{count} shelf examples", { count })}`}
          title={`${t(e.name)} (${e.z}) · ${t("{count} shelf examples", { count })}`}
          onclick={() => (picked = e)}
        >
          <span class="z">{e.z}</span>
          <span class="sym">{e.symbol}</span>
          <span class="coverage" aria-hidden="true">{count}</span>
        </button>
      {/each}
    </div>
    {#if visibleElements.length === 0}
      <p class="empty-search" role="status">{t("No elements or installed lab materials match that search.")}</p>
    {/if}

    {#if picked}
      <aside class="detail">
        <h3>{t(picked.name)} <small>{picked.symbol} · {picked.z}</small></h3>
        <p class="cat">{t(CATEGORY_WORDS[picked.category] ?? picked.category)}</p>
        <p class="capability">{t("coverage: {level}", { level: t(CAPABILITY_WORDS[capability]) })}</p>
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
        {#if routes.length > 0}
          <p class="facts">{t("things you can run with this element:")}</p>
          <ul class="routes">
            {#each routes as route (route.kind + route.key)}
              <li>
                <button
                  onclick={() => route.kind === "lesson"
                    ? onlesson?.(route.key)
                    : onexperiment?.(route.key)}
                  disabled={route.kind === "lesson" ? !onlesson : !onexperiment}
                >
                  <strong>{route.kind === "lesson" ? t("lesson") : t("experiment")}</strong>
                  <span>{t(route.label)}</span>
                  <small>{t("needs: {materials}", { materials: route.requiredShelfKeys.join(", ") })}</small>
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="facts none">{t("No installed reaction or lesson is linked to this element yet.")}</p>
        {/if}
      </aside>
    {/if}
  </dialog>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: var(--scrim);
    display: grid;
    place-items: center;
    z-index: 10;
    padding: 1rem;
  }
  .table-panel {
    position: static;
    margin: 0;
    color: var(--ink);
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
  .mode {
    margin-left: auto;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.72rem;
    padding: 0.25rem 0.65rem;
    cursor: pointer;
  }
  .mode + .close {
    margin-left: 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(18, minmax(30px, 1fr));
    gap: 2px;
  }
  .search {
    display: grid;
    gap: 0.25rem;
    margin: 0 0 0.65rem;
    color: var(--dim);
    font-size: 0.72rem;
  }
  .search input {
    width: min(100%, 28rem);
    border: 1px solid var(--edge);
    border-radius: 6px;
    background: var(--panel);
    color: var(--ink);
    font: inherit;
    padding: 0.45rem 0.55rem;
  }
  .empty-search { color: var(--dim); font-size: 0.8rem; }
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
    position: relative;
  }
  .el.unsupported {
    opacity: 0.58;
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
  .coverage {
    position: absolute;
    right: 2px;
    bottom: 1px;
    color: var(--dim);
    font-size: 0.45rem;
    line-height: 1;
  }
  /* Category tints on the border-left, plus a subtle wash — category is
     also written in words in the detail, never colour alone. */
  .cat-alkali { border-left: 3px solid var(--element-alkali); }
  .cat-alkaline { border-left: 3px solid var(--element-alkaline); }
  .cat-transition { border-left: 3px solid var(--element-transition); }
  .cat-post { border-left: 3px solid var(--element-post); }
  .cat-metalloid { border-left: 3px solid var(--element-metalloid); }
  .cat-nonmetal { border-left: 3px solid var(--element-nonmetal); }
  .cat-halogen { border-left: 3px solid var(--element-halogen); }
  .cat-noble { border-left: 3px solid var(--element-noble); }
  .cat-lanthanide { border-left: 3px solid var(--element-lanthanide); }
  .cat-actinide { border-left: 3px solid var(--element-actinide); }
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
  .capability {
    display: inline-block;
    margin: 0.15rem 0;
    border: 1px solid var(--edge);
    border-radius: 999px;
    padding: 0.1rem 0.45rem;
    color: var(--dim);
    font-size: 0.7rem;
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
  .routes {
    list-style: none;
    margin: 0.35rem 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 0.35rem;
  }
  .routes button {
    width: 100%;
    min-height: 44px;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.15rem 0.45rem;
    text-align: left;
    border: 1px solid var(--edge);
    border-radius: 7px;
    background: var(--panel);
    color: var(--ink);
    padding: 0.45rem;
    cursor: pointer;
  }
  .routes small { grid-column: 1 / -1; color: var(--dim); }
  @media (max-width: 640px) {
    .table-panel { padding: 0.65rem; }
    header { align-items: stretch; flex-wrap: wrap; }
    .hint { flex-basis: 100%; }
    .grid { grid-template-columns: repeat(18, minmax(24px, 1fr)); overflow-x: auto; }
    .el { min-width: 24px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .table-panel, .el, .routes button { scroll-behavior: auto; transition: none; }
  }
</style>
