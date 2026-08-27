<script lang="ts">
  /**
   * The map screen (GUI-053): the codex concept graph, layered by
   * prerequisite depth. At lv1 it reads as a skill tree — big nodes,
   * edges only for the concept in hand; at lv3 the full DAG shows.
   * Nothing here is decoration: node fill means the learner ran an
   * entry teaching that concept to a green check on THIS device, and
   * tapping a concept lists the entries that teach it, ready ones first.
   */
  import {
    conceptGraph,
    entryReady,
    metConcepts,
    type CodexEntry,
  } from "../codex";
  import type { Session } from "../session.svelte";
  import { t, tSlug } from "../i18n.svelte";

  let {
    entries,
    session,
    onopenentry,
    onclose,
  }: {
    entries: CodexEntry[];
    session: Session;
    /** Hand the tapped entry to the experiment page. */
    onopenentry: (e: CodexEntry) => void;
    onclose: () => void;
  } = $props();

  const graph = $derived(conceptGraph(entries));
  const met = $derived(metConcepts(entries, session.completedExperiments));
  let picked = $state<string | null>(null);

  // Layout: columns by depth, rows in each column's sorted order.
  const COL_W = 170;
  const ROW_H = 46;
  const layout = $derived.by(() => {
    const rows = new Map<number, number>();
    const at = new Map<string, { x: number; y: number }>();
    for (const n of graph.nodes) {
      const row = rows.get(n.depth) ?? 0;
      rows.set(n.depth, row + 1);
      at.set(n.concept, { x: 20 + n.depth * COL_W, y: 24 + row * ROW_H });
    }
    const width = 40 + (1 + Math.max(0, ...graph.nodes.map((n) => n.depth))) * COL_W;
    const height = 48 + Math.max(1, ...[...rows.values()]) * ROW_H;
    return { at, width, height };
  });

  const fullDag = $derived(session.register === "lv3");
  const shownEdges = $derived(
    fullDag
      ? graph.edges
      : graph.edges.filter((e) => picked !== null && (e.from === picked || e.to === picked)),
  );

  const teaching = $derived.by(() => {
    if (!picked) return [];
    const list = entries.filter((e) => e.concepts?.includes(picked!));
    return [...list].sort(
      (a, b) => Number(entryReady(b, met)) - Number(entryReady(a, met)) || a.id.localeCompare(b.id),
    );
  });

  function edgePath(e: { from: string; to: string }): string {
    const a = layout.at.get(e.from);
    const b = layout.at.get(e.to);
    if (!a || !b) return "";
    const ax = a.x + 128;
    const bx = b.x - 6;
    const mid = (ax + bx) / 2;
    return `M ${ax} ${a.y} C ${mid} ${a.y} ${mid} ${b.y} ${bx} ${b.y}`;
  }
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <dialog open
    class="map"
    aria-modal="true"
    aria-label={t("concept map")}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{t("concept map")}</h2>
      <span class="hint">
        {t("{met} of {total} concepts met — filled means run to a green check here", { met: met.size, total: graph.nodes.length })}
      </span>
      <button class="close" onclick={onclose}>{t("close")}</button>
    </header>
    {#if graph.nodes.length === 0}
      <p class="empty">{t("the codex export has not arrived yet — the map draws itself from it")}</p>
    {:else}
      <div class="scroll">
        <svg width={layout.width} height={layout.height} role="img" aria-label={t("concept graph")}>
          {#each shownEdges as e (e.from + "→" + e.to)}
            <path class="edge" d={edgePath(e)} />
          {/each}
          {#each graph.nodes as n (n.concept)}
            {@const p = layout.at.get(n.concept)!}
            <g transform={`translate(${p.x} ${p.y})`}>
              <foreignObject x="0" y="-16" width="132" height="34">
                <button
                  class="node"
                  class:met={met.has(n.concept)}
                  class:on={picked === n.concept}
                  onclick={() => (picked = picked === n.concept ? null : n.concept)}
                >
                  {t(n.concept.replace(/-/g, " "))}
                  <small>{n.count}</small>
                </button>
              </foreignObject>
            </g>
          {/each}
        </svg>
      </div>
      {#if picked}
        <div class="teach">
          <h3>{t(picked.replace(/-/g, " "))}</h3>
          <ul>
            {#each teaching as e (e.id)}
              <li>
                <button class="entry" onclick={() => onopenentry(e)}>
                  <span class="ready" class:ok={entryReady(e, met)}>
                    {entryReady(e, met) ? t("ready") : t("locked")}
                  </span>
                  {t(e.id.replace(/-/g, " "))}
                  {#if session.completedExperiments.has(e.id)}<span class="done">✓</span>{/if}
                </button>
                {#if !entryReady(e, met)}
                  <span class="needs">
                    {t("needs: {concepts}", { concepts: (e.requires ?? []).filter((r) => !met.has(r)).map(tSlug).join(", ") })}
                  </span>
                {/if}
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    {/if}
  </dialog>
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
  .map {
    position: static;
    margin: 0;
    color: var(--ink);
    background: var(--bg);
    border: 1px solid var(--edge);
    border-radius: 12px;
    padding: 1rem;
    width: min(96vw, 900px);
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 0.7rem;
  }
  h2 {
    margin: 0;
    font-size: 1rem;
  }
  .hint {
    color: var(--dim);
    font-size: 0.76rem;
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
  .empty {
    color: var(--dim);
    font-size: 0.85rem;
  }
  .scroll {
    overflow: auto;
    margin-top: 0.6rem;
    border: 1px solid var(--edge);
    border-radius: 8px;
    background: var(--panel);
    flex: 1;
    min-height: 10rem;
  }
  .edge {
    fill: none;
    stroke: var(--edge-strong);
    stroke-width: 1.2;
    opacity: 0.55;
  }
  .node {
    width: 100%;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.72rem;
    padding: 0.25rem 0.5rem;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .node.met {
    border-color: var(--good);
    background: color-mix(in srgb, var(--good) 18%, var(--panel-raised));
  }
  .node.on {
    border-color: var(--hot);
  }
  .node small {
    color: var(--dim);
  }
  .teach {
    margin-top: 0.6rem;
    max-height: 10rem;
    overflow-y: auto;
  }
  .teach h3 {
    margin: 0 0 0.3rem;
    font-size: 0.85rem;
  }
  .teach ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .entry {
    background: none;
    border: 0;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.25rem 0;
    cursor: pointer;
  }
  .entry:hover {
    color: var(--hot);
  }
  .ready {
    font-size: 0.68rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    padding: 0.05rem 0.45rem;
    color: var(--dim);
    margin-right: 0.35rem;
  }
  .ready.ok {
    border-color: var(--good);
    color: var(--good);
  }
  .done {
    color: var(--good);
  }
  .needs {
    color: var(--dim);
    font-size: 0.72rem;
    margin-left: 0.5rem;
  }
</style>
