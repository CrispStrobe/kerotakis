<script lang="ts">
  /**
   * The map screen (GUI-053): the codex concept graph, layered by
   * prerequisite depth. At lv1 it reads as a skill tree — big nodes,
   * edges only for the concept in hand; at lv3 the full DAG shows.
   * Nothing here is decoration: node fill means the learner ran an
   * entry teaching that concept to a green check on THIS device, and
   * tapping a concept lists everything affiliated with it — catalogue
   * entries, kids tasks, and guided missions — each with the reason it is
   * offered and a button that opens it. A concept the shipped content
   * cannot reach says so rather than showing an empty panel.
   *
   * The panel is TWO surfaces, and the layout says so (owner, German
   * deploy: "there is not enough screen space for the Missions below the
   * map"). The graph used to take a flexible column and the activity list
   * a 14rem strip beneath it, so on anything but a tall desktop the list
   * a learner had just asked for was the part that lost: three links
   * became a row and a half, and the missions sat below the fold of a
   * panel that did not scroll as a whole.
   *
   * So: wide, they are side by side — the graph scrolls in its own box on
   * the left, the activities for the selected concept take a column of
   * their own on the right that never falls below 20rem and scrolls by
   * itself. Narrow, a graph is the wrong shape entirely, and it collapses
   * to the accordion the list wanted to be all along: concepts in
   * prerequisite order, the selected one expanding its activities inline
   * where the finger already is. Either way the header — the tally and
   * the one close affordance — stays put and only the body moves.
   */
  import {
    conceptGraph,
    entryLocked,
    entryReady,
    metConcepts,
    type CodexEntry,
  } from "../codex";
  import { conceptLinks, relationLabel, type ConceptLink } from "../conceptLinks";
  import { kidsText, type KidsExperiment } from "../kidsCatalog";
  import { missionAvailability, type MissionSummary } from "../storyProgress";
  import type { Session } from "../session.svelte";
  import type { LabMode } from "../worldState";
  import { i18n, t, tSlug } from "../i18n.svelte";

  let {
    entries,
    session,
    mode,
    kids = [],
    missions = [],
    onopenentry,
    onopenkids,
    onopenmission,
    onclose,
  }: {
    entries: CodexEntry[];
    session: Session;
    /** Which laboratory this is. Sandbox gates nothing; Story keeps the
     * authored progression. */
    mode: LabMode;
    kids?: KidsExperiment[];
    missions?: MissionSummary[];
    /** Hand the tapped entry to the experiment page. */
    onopenentry: (e: CodexEntry) => void;
    /** Open the catalogue on one guided task. */
    onopenkids?: (id: string) => void;
    /** Start a guided mission by its `.lab` file. */
    onopenmission?: (file: string) => void;
    onclose: () => void;
  } = $props();

  const graph = $derived(conceptGraph(entries));
  const met = $derived(metConcepts(entries, session.completedExperiments));
  let picked = $state<string | null>(null);

  /**
   * One column or two, decided by the width the panel actually has.
   *
   * A media query alone could hide one of two copies of the activity
   * list, but then every button and every measurement inside it would
   * exist twice in the document. The query drives state instead, so
   * exactly one shape is ever mounted.
   */
  const COMPACT = "(max-width: 47.5rem)";
  let compact = $state(false);
  $effect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") return;
    const query = window.matchMedia(COMPACT);
    // Read the list rather than trust the event: `resize` and `change` are
    // two announcements of one fact, they do not always both arrive, and a
    // panel that answered only the second one kept the two-column layout
    // on a window that had already become a phone.
    const sync = () => (compact = query.matches);
    sync();
    query.addEventListener("change", sync);
    window.addEventListener("resize", sync);
    return () => {
      query.removeEventListener("change", sync);
      window.removeEventListener("resize", sync);
    };
  });

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

  const links = $derived.by(() => {
    if (!picked) return [] as ConceptLink[];
    // i18n-ok: concept slugs are keys; `picked` is one, not typed text.
    return conceptLinks(picked, {
      entries,
      kids,
      missions,
      completedExperiments: session.completedExperiments,
      completedMissions: session.completedMissions,
    });
  });

  const teaching = $derived(
    links
      .filter((link): link is Extract<ConceptLink, { kind: "experiment" }> => link.kind === "experiment")
      .sort(
        // Order by the rendered label, not the hidden slug: sorting German
        // entries by their English ids puts them in an order with no visible
        // logic. The locale goes to the collator so umlauts sort as German
        // readers expect rather than after z.
        (a, b) =>
          Number(entryReady(b.entry, met)) - Number(entryReady(a.entry, met)) ||
          tSlug(a.id).localeCompare(tSlug(b.id), i18n.locale),
      ),
  );

  type Elsewhere = Exclude<ConceptLink, { kind: "experiment" }>;

  /** Everything that is not a catalogue entry, evidence links first. */
  const elsewhere = $derived(
    links
      .filter((link): link is Elsewhere => link.kind !== "experiment")
      .sort((a, b) => Number(b.relation === "evidence") - Number(a.relation === "evidence")),
  );

  const linkTitle = (link: Elsewhere): string =>
    link.kind === "kids" ? kidsText(link.kid, "title", i18n.locale) : t(link.mission.name);

  const conceptName = (concept: string): string => t(concept.replace(/-/g, " "));

  /** The concepts an entry builds on that this learner has not met yet.
   * In Story they are the reason it is gated; in Sandbox they are a route
   * through the material, and never a refusal. */
  const pending = (e: CodexEntry): string[] => (e.requires ?? []).filter((r) => !met.has(r));

  const missionAccess = (link: Elsewhere) => link.kind === "mission"
    ? missionAvailability(missions, session.completedMissions, link.mission)
    : null;

  function edgePath(e: { from: string; to: string }): string {
    const a = layout.at.get(e.from);
    const b = layout.at.get(e.to);
    if (!a || !b) return "";
    const ax = a.x + 128;
    const bx = b.x - 6;
    const mid = (ax + bx) / 2;
    return `M ${ax} ${a.y} C ${mid} ${a.y} ${mid} ${b.y} ${bx} ${b.y}`;
  }

  function toggle(concept: string): void {
    picked = picked === concept ? null : concept;
  }
</script>

{#snippet activities()}
  {#if picked}
    <h3>{conceptName(picked)}</h3>
    {#if links.length === 0}
      <p class="none">{t("no activity teaches this concept yet")}</p>
    {/if}
    <ul>
      {#each teaching as link (link.id)}
        {@const e = link.entry}
        {@const gated = entryLocked(e, met, mode)}
        {@const waiting = pending(e)}
        <li>
          <button class="entry" onclick={() => onopenentry(e)}>
            <span class="ready" class:ok={!gated} class:locked={gated}>
              {gated ? t("locked") : t("ready")}
            </span>
            {tSlug(e.id)}
            {#if link.done}<span class="done">✓</span>{/if}
          </button>
          <span class="why">{t(relationLabel(link.relation))}</span>
          {#if waiting.length > 0}
            <span class="needs">
              {gated
                ? t("needs: {concepts}", { concepts: waiting.map(tSlug).join(", ") })
                : t("builds on: {concepts}", { concepts: waiting.map(tSlug).join(", ") })}
            </span>
          {/if}
        </li>
      {/each}
      {#each elsewhere as link (link.kind + ":" + link.id)}
        {@const access = missionAccess(link)}
        <li>
          <button
            class="entry"
            data-mission-unlocked={access?.unlocked}
            onclick={() => {
              if (link.kind === "kids") onopenkids?.(link.id);
              else if (access?.unlocked) onopenmission?.(link.id);
            }}
            disabled={link.kind === "kids" ? onopenkids === undefined : onopenmission === undefined || !access?.unlocked}
          >
            <span class="ready kind">{link.kind === "kids" ? t("experiment") : t("mission")}</span>
            {linkTitle(link)}
            {#if link.done}<span class="done">✓</span>{/if}
          </button>
          <span class="why">{t(relationLabel(link.relation))}</span>
          {#if access && !access.unlocked}
            <span class="needs">{access.remaining === 1
              ? t("complete one more mission to unlock")
              : t("complete {count} more missions to unlock", { count: access.remaining })}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
{/snippet}

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <dialog open
    class="map"
    class:compact
    aria-modal="true"
    aria-label={t("concept map")}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{t("concept map")}</h2>
      <span class="hint">
        {t("{met} of {total} concepts met — filled means run to a green check here", { met: met.size, total: graph.nodes.length })}
      </span>
      <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    </header>
    {#if graph.nodes.length === 0}
      <p class="empty">{t("the codex export has not arrived yet — the map draws itself from it")}</p>
    {:else if compact}
      <!-- One column: a laid-out graph does not survive a phone, so the
           layering becomes an indent and the activities open inline under
           the concept the finger just chose. -->
      <div class="body stacked">
        <ul class="concepts">
          {#each graph.nodes as n (n.concept)}
            <li>
              <button
                class="node concept-row"
                class:met={met.has(n.concept)}
                class:on={picked === n.concept}
                style={`--depth: ${n.depth}`}
                aria-expanded={picked === n.concept}
                onclick={() => toggle(n.concept)}
              >
                <span class="concept-name">{conceptName(n.concept)}</span>
                <small>{n.count}</small>
              </button>
              {#if picked === n.concept}
                <div class="teach inline">
                  {@render activities()}
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {:else}
      <div class="body split">
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
                    onclick={() => toggle(n.concept)}
                  >
                    {conceptName(n.concept)}
                    <small>{n.count}</small>
                  </button>
                </foreignObject>
              </g>
            {/each}
          </svg>
        </div>
        <aside class="teach">
          {#if picked}
            {@render activities()}
          {:else}
            <p class="none">{t("choose a concept to see what teaches it")}</p>
          {/if}
        </aside>
      </div>
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
    /* Above the topbar (20) and the tools panel (40): at 10 this
       modal opened underneath the chrome and lost its heading. */
    z-index: 50;
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
    /* Of the scrim's own content box, so the panel can never be wider
       than the room it is centred in. */
    width: min(100%, 1040px);
    /* The scrim's own padding is part of the height budget: 92vh plus
       2rem of padding is taller than the screen it is centred in, and the
       header went off the top before anything inside could scroll. */
    max-height: calc(100vh - 2rem);
    max-height: calc(100dvh - 2rem);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 0.7rem;
    /* Fixed: the tally and the close stay whatever the body does. */
    flex: 0 0 auto;
  }
  h2 {
    margin: 0;
    font-size: 1rem;
  }
  .hint {
    color: var(--dim);
    font-size: 0.76rem;
  }
  .icon-close {
    margin-left: auto;
  }
  .empty {
    color: var(--dim);
    font-size: 0.85rem;
  }
  .body {
    flex: 1 1 auto;
    /* Without this a flex child refuses to shrink below its content and
       the panel grows past the viewport instead of scrolling. */
    min-height: 0;
    margin-top: 0.6rem;
  }
  .body.split {
    display: grid;
    /* The activity list is the half the owner could not read. It gets a
       floor, and the graph takes what is left. */
    grid-template-columns: minmax(0, 1fr) minmax(20rem, 24rem);
    /* An implicit `auto` row sizes to its tallest content and takes the
       panel with it; pinned to the body's own height, the two columns
       scroll instead. */
    grid-template-rows: minmax(0, 1fr);
    gap: 0.6rem;
  }
  .body.stacked {
    overflow-y: auto;
  }
  .scroll {
    overflow: auto;
    border: 1px solid var(--edge);
    border-radius: 8px;
    background: var(--panel);
    min-height: 0;
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
  .concepts {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .concept-row {
    /* Depth was the graph's x axis; here it is the indent, capped so a
       deep concept still gets most of a narrow row for its name. */
    margin-left: calc(min(var(--depth), 4) * 0.6rem);
    width: auto;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    /* A row is a touch target before it is a label. */
    min-height: 44px;
    text-align: start;
    border-radius: 10px;
    font-size: 0.82rem;
  }
  .concept-name {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .teach {
    min-height: 0;
  }
  .body.split .teach {
    overflow-y: auto;
    border: 1px solid var(--edge);
    border-radius: 8px;
    background: var(--panel);
    padding: 0.5rem 0.6rem;
  }
  .teach.inline {
    margin: 0.3rem 0 0.5rem;
    padding: 0.4rem 0.6rem;
    border-left: 2px solid var(--edge-strong);
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
  .teach li {
    padding: 0.15rem 0;
  }
  .entry {
    background: none;
    border: 0;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.25rem 0;
    cursor: pointer;
    text-align: start;
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
    white-space: nowrap;
  }
  .ready.ok {
    border-color: var(--good);
    color: var(--good);
  }
  .done {
    color: var(--good);
  }
  .needs, .why {
    color: var(--dim);
    font-size: 0.72rem;
    margin-left: 0.5rem;
  }
  .ready.kind {
    border-color: var(--hot);
    color: var(--hot);
  }
  .entry:disabled {
    color: var(--dim);
    cursor: default;
  }
  .none {
    color: var(--dim);
    font-size: 0.8rem;
    margin: 0.2rem 0;
  }
</style>
