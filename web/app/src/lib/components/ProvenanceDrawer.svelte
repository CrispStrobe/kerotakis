<script lang="ts">
  /**
   * GUI-052 — where this answer came from.
   *
   * The engine has always known which solver answered, on what data, within
   * what bounds, and what it declined; none of it reached a reader. This is
   * that record, and only that record. Every sentence with chemistry in it
   * is the ENGINE's own, printed verbatim — the labels around them are the
   * interface's and go through `t()`. The drawer never explains a result;
   * it says who computed it and who did not.
   *
   * The model is built by `provenance.ts`, which is pure and unit-tested
   * against the wire shapes. This file is layout.
   *
   * Registers (GUI-090's rule): lv1 gets the one line a learner can act on
   * — who answered, on what data. lv2 and lv3 get the whole chain, the
   * datasets with their literature sources, the validity notes and the
   * refusals. The honesty items are shown at EVERY register: a gap in the
   * model is not an advanced topic.
   */
  import { t } from "../i18n.svelte";
  import { engineText } from "../engineText";
  import type { ProvenanceReport, ProvenanceRoute } from "../provenance";

  let {
    report,
    register = "lv1",
    onclose,
  }: {
    report: ProvenanceReport;
    register?: string;
    onclose: () => void;
  } = $props();

  const detailed = $derived(register !== "lv1");

  /**
   * `NotModelledCause`, as words. The kebab id is what a reader groups by
   * and is not English, so it cannot be handed to the dictionary directly
   * — `t("no-solver")` would render the tag. Each label is the one-line
   * meaning the variant's own doc comment gives it in `ops.rs`.
   */
  const CAUSE_LABELS: Record<string, string> = {
    "no-solution": "no aqueous solution has been characterised",
    "nothing-to-act-on": "the vessel does not hold what this needs",
    "no-solver": "no wired solver covers this state",
    "rate-not-modelled": "the chemistry is right and its speed is not modelled",
    "model-boundary": "a model was asked outside the range it is fitted for",
    "no-reviewed-datum": "the registry carries no reviewed value for this",
    "phase-not-in-registry": "this lab's registry is missing a phase the databases know",
    "not-speciated": "no shipped database can speciate this substance",
    "no-transport-path": "no modelled route between where the matter is and where the question looks",
    "boundary-mismatch": "the vessel's boundary cannot do what this needs",
    "not-parameterised": "no curated row for this, and the general case is not derivable",
    "not-in-any-database": "no shipped thermodynamic database defines this at all",
    "unclassified": "recorded before the bench classified its refusals",
  };
  const causeLabel = (cause: string) => t(CAUSE_LABELS[cause] ?? "unclassified reason");

  /** One word per outcome, plus the solver's own sentence where it gave one. */
  function outcomeWord(route: ProvenanceRoute): string {
    if (route.outcome === "answered") return t("answered");
    if (route.outcome === "failed") return t("failed");
    return route.reason ? t("declined") : t("not applicable");
  }

  const answered = $derived(report.routes.filter((route) => route.outcome === "answered").length);
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(event) => event.key === "Escape" && onclose()}
>
  <dialog
    open
    class="provenance-drawer"
    aria-modal="true"
    aria-labelledby="provenance-drawer-title"
    onclick={(event) => event.stopPropagation()}
  >
    <header>
      <span class="mark" aria-hidden="true">⌖</span>
      <span class="titles">
        <small>{t("provenance")}{report.vessel === undefined ? "" : ` · v${report.vessel + 1}`}</small>
        <h2 id="provenance-drawer-title">{t("where this answer came from")}</h2>
      </span>
      <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    </header>

    <div class="body">
      <!-- lv1's whole answer, and the opening line at every other register:
           who answered, on what data, and how many were asked and had
           nothing to add. -->
      <p class="headline">
        {#if report.headline}
          {report.headline.dataset
            ? t("answered by {solver} using {dataset}", {
                solver: report.headline.solver,
                dataset: report.headline.dataset,
              })
            : t("answered by {solver}", { solver: report.headline.solver })}
          {#if report.headline.declined > 0}
            <span class="aside">
              {t("{declined} of {asked} solvers had nothing to add", {
                declined: report.headline.declined,
                asked: report.routes.length,
              })}
            </span>
          {/if}
        {:else if report.routes.length > 0}
          {t("no solver answered this step")}
        {:else}
          {t("the bench recorded no routing for this step")}
        {/if}
      </p>

      <!-- A refusal is shown at every register. It is the one thing a
           reader must not have to open a detail view to find out. -->
      {#if report.gaps.length > 0}
        <section aria-labelledby="provenance-gaps">
          <h3 id="provenance-gaps">{t("what the bench did not model")}</h3>
          <ul class="gaps">
            {#each report.gaps as gap, index (`${gap.cause}-${index}`)}
              <li>
                <p>{engineText(gap.what)}</p>
                <small>{causeLabel(gap.cause)}</small>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if detailed}
        <section aria-labelledby="provenance-chain">
          <h3 id="provenance-chain">
            {t("the solver chain")}
            <span class="tally">{t("{answered} of {asked} answered", { answered, asked: report.routes.length })}</span>
          </h3>
          {#if report.routes.length === 0}
            <p class="empty">{t("the bench recorded no routing for this step")}</p>
          {:else}
            <ol class="chain">
              {#each report.routes as route, index (`${route.solver}-${index}`)}
                <li data-outcome={route.outcome}>
                  <span class="route-head">
                    <code>{route.solver}</code>
                    <span class="kind" data-confidence={route.kind}>{t(route.kind)}</span>
                    <b>{outcomeWord(route)}</b>
                  </span>
                  {#if route.outcome === "answered" && route.eventCount > 0}
                    <small>{t("{count} events", { count: route.eventCount })}</small>
                  {/if}
                  {#if route.reason}
                    <p class="reason">{engineText(route.reason)}</p>
                  {/if}
                </li>
              {/each}
            </ol>
          {/if}
        </section>

        {#if report.sources.length > 0}
          <section aria-labelledby="provenance-data">
            <h3 id="provenance-data">{t("data and models")}</h3>
            <ul class="sources">
              {#each report.sources as source, index (`${source.engine}-${index}`)}
                <li>
                  {#if source.engine}<strong>{source.engine}</strong>{/if}
                  {#if source.dataset}<code>{source.dataset}</code>{/if}
                  {#if source.model}<p class="model">{engineText(source.model)}</p>{/if}
                  {#if source.routing}<p class="routing">{engineText(source.routing)}</p>{/if}
                  {#if source.datasetSources.length > 0}
                    <details>
                      <summary>{t("the dataset's own sources")}</summary>
                      <ul class="citations">
                        {#each source.datasetSources as citation (citation)}
                          <li>{citation}</li>
                        {/each}
                      </ul>
                    </details>
                  {/if}
                </li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if report.validity.length > 0}
          <section aria-labelledby="provenance-validity">
            <h3 id="provenance-validity">{t("where the numbers hold")}</h3>
            <ul class="validity">
              {#each report.validity as bound, index (`${bound.instrument ?? "reading"}-${index}`)}
                <li>
                  {#if bound.instrument}<code>{bound.instrument}</code>{/if}
                  <p>{engineText(bound.note)}</p>
                </li>
              {/each}
            </ul>
          </section>
        {/if}
      {:else}
        <p class="more">{t("switch to a higher register to see the whole chain")}</p>
      {/if}
    </div>
  </dialog>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: grid;
    justify-items: end;
    align-items: stretch;
    padding: 0;
    background: var(--scrim);
  }
  /* Right-hand drawer: full height, its own scroll region, and never wider
     than the viewport it hangs in. `100dvh` rather than `100vh` so a phone's
     collapsing address bar does not leave the close button off-screen. */
  .provenance-drawer {
    position: relative;
    width: min(27rem, 100vw);
    max-width: 100vw;
    height: 100%;
    max-height: 100dvh;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 0;
    border-left: 1px solid var(--edge);
    color: var(--ink);
    background: var(--surface);
    box-shadow: -24px 0 70px var(--overlay-shadow);
  }
  header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: calc(env(safe-area-inset-top) + 0.7rem) 0.85rem 0.7rem;
    border-bottom: 1px solid var(--edge);
  }
  .mark { width: 28px; height: 28px; flex: none; display: grid; place-items: center; border-radius: 9px; color: var(--on-accent); background: var(--action); font-weight: 900; }
  .titles { min-width: 0; display: flex; flex: 1; flex-direction: column; }
  .titles small { overflow: hidden; color: var(--dim); font-size: .55rem; font-weight: 800; letter-spacing: .1em; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
  h2 { margin: 0; font-size: .95rem; overflow-wrap: anywhere; }
  /* `min-height: 0` is what makes the drawer's height budget bite: without
     it the flex child grows to its content and the panel scrolls the page
     instead of itself. */
  .body { min-height: 0; padding: 0 .85rem calc(env(safe-area-inset-bottom) + 1rem); overflow-y: auto; overscroll-behavior: contain; }
  h3 { display: flex; flex-wrap: wrap; align-items: baseline; gap: .4rem; margin: 1rem 0 .35rem; color: var(--dim); font-size: .58rem; font-weight: 850; letter-spacing: .09em; text-transform: uppercase; }
  .tally { color: var(--dim); font-size: .55rem; font-weight: 700; letter-spacing: .04em; text-transform: none; }
  .headline { margin: .85rem 0 0; font-size: .84rem; line-height: 1.45; overflow-wrap: anywhere; }
  .headline .aside { display: block; margin-top: .2rem; color: var(--dim); font-size: .7rem; }
  .more, .empty { margin: .6rem 0 0; color: var(--dim); font-size: .72rem; line-height: 1.4; }
  ul, ol { margin: 0; padding: 0; list-style: none; }
  .chain li,
  .sources li,
  .validity li,
  .gaps li {
    min-height: 44px;
    display: flex;
    flex-direction: column;
    gap: .18rem;
    justify-content: center;
    padding: .45rem .55rem;
    margin-top: .3rem;
    border: 1px solid var(--edge);
    border-radius: 10px;
    background: var(--surface-raised);
    overflow-wrap: anywhere;
  }
  .route-head { display: flex; flex-wrap: wrap; align-items: center; gap: .35rem; }
  code { font-family: ui-monospace, SFMono-Regular, monospace; font-size: .7rem; }
  .kind { padding: .05rem .3rem; border: 1px solid color-mix(in srgb, var(--ink) 28%, transparent); border-radius: 6px; color: var(--dim); font-size: .58rem; font-weight: 750; text-transform: uppercase; }
  .route-head b { font-size: .68rem; font-weight: 800; text-transform: uppercase; letter-spacing: .05em; }
  [data-outcome="answered"] .route-head b { color: var(--success); }
  [data-outcome="declined"] .route-head b { color: var(--dim); }
  [data-outcome="failed"] .route-head b { color: var(--danger); }
  [data-outcome="answered"] { border-color: color-mix(in srgb, var(--success) 38%, var(--edge)); }
  [data-outcome="failed"] { border-color: color-mix(in srgb, var(--danger) 45%, var(--edge)); }
  .reason, .model, .routing, .validity p, .gaps p { margin: 0; font-size: .72rem; line-height: 1.4; }
  .reason, .model, .routing { color: var(--dim); }
  .gaps li { border-left: 3px solid var(--warning); }
  .gaps small, .chain small { color: var(--dim); font-size: .62rem; }
  .sources strong { font-size: .76rem; }
  details summary { min-height: 30px; display: flex; align-items: center; color: var(--dim); font-size: .62rem; font-weight: 750; cursor: pointer; }
  .citations li { min-height: 0; display: block; margin: 0; padding: .1rem 0 .1rem .6rem; border: 0; border-left: 2px solid var(--edge); border-radius: 0; background: none; color: var(--dim); font-size: .66rem; line-height: 1.35; }
  /* Full-bleed on a phone. The scrim already carries no padding, so there
     is nothing to zero here — a padded scrim under a 100vw panel is what
     widens the page and trips the overflow gate. */
  @media (max-width: 30rem) {
    .provenance-drawer { width: 100vw; border-left: 0; }
  }
</style>
