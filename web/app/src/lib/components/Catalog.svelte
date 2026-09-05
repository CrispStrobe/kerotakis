<!--
  ONE catalogue, two tiers.

  There used to be two: `ExperimentCatalog.svelte` for the codex entries and
  `KidsCatalog.svelte` for the KIDS corpus. Two dialogs, two search boxes,
  two close affordances, two ideas of what "run this" means — and only one
  of them actually drove the bench in a way a learner could watch. Folding
  them together is not tidying: it is how the Kids tier gets the real runner
  and the experiment tier gets the Kids tier's honesty about what a card is
  offering, without either of them being reimplemented twice.

  Tier is presentation. Kids cards are wordier about the phenomenon and
  quieter about the equation, experiment rows are the reverse; both open the
  SAME entry panel and both run through `runCatalogEntry`, on the visible
  bench, one step at a time.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import {
    conceptIndex,
    curriculumIndex,
    relatedConcepts,
    scriptKit,
    type CheckResult,
    type CodexEntry,
  } from "../codex";
  import type { Session } from "../session.svelte";
  import KitStrip from "./KitStrip.svelte";
  import { t, tSlug, tEngine, i18n } from "../i18n.svelte";
  import { experimentHasProgress, experimentMatches, experimentProgressLabel, type ExperimentProgressFilter } from "../catalogSearch";
  import {
    canUseFreshVessels,
    runCatalogEntry,
    runGate,
    runnableLines,
    type BenchDecision,
    type RunStep,
  } from "../catalogRunner";
  import {
    codexLearningLabel,
    guidedLearningLabel,
    kidsConnections,
    kidsExperimentMatches,
    kidsText,
    type KidsExperiment,
    type KidsStatus,
  } from "../kidsCatalog";

  type CatalogTier = "kids" | "experiments";

  let {
    tier = "experiments",
    entries,
    kidsEntries = [],
    session,
    capabilityIds = new Set<string>(),
    codexIds = new Set<string>(),
    initial = null,
    kidsInitial = null,
    ontier,
    onlesson,
    onquest,
    oncapability,
    onsandbox,
    onclose,
  }: {
    tier?: CatalogTier;
    entries: CodexEntry[];
    kidsEntries?: KidsExperiment[];
    session: Session;
    capabilityIds?: ReadonlySet<string>;
    codexIds?: ReadonlySet<string>;
    /** Open directly on one entry (the concept map hands entries over). */
    initial?: CodexEntry | null;
    /** Open the KIDS tier filtered to one task id — the concept map's
     * other hand-over. It seeds the shared search rather than pinning it,
     * so the learner can widen it immediately. */
    kidsInitial?: string | null;
    ontier?: (next: CatalogTier) => void;
    onlesson?: (file: string) => void;
    onquest?: (id: string) => void;
    oncapability?: (id: string) => void;
    onsandbox?: (entry: KidsExperiment) => void;
    onclose: () => void;
  } = $props();

  let open = $state<CodexEntry | null>(untrack(() => initial));
  let tab = $state<"theory" | "procedure" | "run">("theory");
  let predicted = $state<number | null>(null);
  let result = $state<CheckResult | null>(null);
  let refusedLine = $state<string | null>(null);

  /** Run state: the panel docks while these are live, so the stage shows. */
  let running = $state(false);
  let step = $state<RunStep | null>(null);
  let asking = $state(false);
  let decision = $state<BenchDecision | null>(null);
  let stopRequested = false;

  /** The catalog's three doors: everything, by concept, by curriculum. */
  let view = $state<"all" | "concepts" | "curriculum">("all");
  /** ONE search box, shared by both tiers — swapping tier keeps the query.
   * Read once, so a deep link seeds it without pinning it. */
  let filter = $state(untrack(() => kidsInitial ?? ""));
  let concept = $state<string | null>(null);
  let progress = $state<ExperimentProgressFilter>("all");
  const concepts = $derived(conceptIndex(entries));
  const curricula = $derived(curriculumIndex(entries));
  const related = $derived(concept ? relatedConcepts(entries, concept).slice(0, 8) : []);
  const shown = $derived.by(() => {
    let list = entries.filter((entry) => experimentHasProgress(entry, session.completedExperiments, progress));
    if (view === "concepts" && concept) {
      // i18n-ok: concept slugs are keys; `concept` comes from a chip, not a box.
      list = list.filter((e) => e.concepts?.includes(concept!));
    }
    const q = filter.trim();
    if (q) {
      // Match canonical and visible localized text. `t()` reads the reactive
      // locale, so changing languages also refilters the catalog.
      list = list.filter((entry) => experimentMatches(entry, q, t));
    }
    return list;
  });

  // ── Kids tier ────────────────────────────────────────────────────────
  const statuses: KidsStatus[] = ["computed", "partial", "boundary", "declined", "unreachable"];
  let status = $state<KidsStatus | null>(null);
  let topic = $state("");
  const topics = $derived([...new Set(kidsEntries.flatMap((entry) => entry.topics))].sort());
  const kidsShown = $derived(kidsEntries.filter((entry) =>
    (!status || entry.status === status) && (!topic || entry.topics.includes(topic))
      && kidsExperimentMatches(entry, filter, i18n.locale),
  ));
  const linksById = $derived(new Map(kidsEntries.map((entry) => [entry.id,
    kidsConnections(entry, capabilityIds, codexIds, session.completedMissions, session.completedExperiments),
  ])));
  const byId = $derived(new Map(entries.map((entry) => [entry.id, entry])));
  /** The codex entry a KIDS card can actually run, if it names one we have. */
  function runnableCodex(entry: KidsExperiment): CodexEntry | null {
    for (const id of entry.codex ?? []) {
      const found = byId.get(id);
      if (found) return found;
    }
    return null;
  }

  function openEntry(e: CodexEntry, at: typeof tab = "theory") {
    open = e;
    tab = at;
    predicted = null;
    result = null;
    refusedLine = null;
    asking = false;
    decision = null;
  }

  function setTier(next: CatalogTier) {
    if (next === tier) return;
    open = null;
    ontier?.(next);
  }

  // The register prose at the dial's level, tolerant of key spelling.
  const theory = $derived.by(() => {
    if (!open) return "";
    const r = open.registers ?? {};
    const lv = session.register;
    // German lives INSIDE the key here — `lv2_de` — because registers are
    // a map keyed by level, not a record of named fields. Every fallback
    // below keeps its German twin ahead of it, so a level translated but
    // not its neighbour still reads German at the level you asked for.
    const de = i18n.locale === "de";
    const pick = (k: string) => (de ? r[`${k}_de`] : undefined) ?? r[k];
    return (
      pick(lv) ??
      pick(lv.replace("lv", "")) ??
      pick("lv2") ??
      pick("2") ??
      Object.values(r)[0] ??
      ""
    );
  });
  /** One option in the reader's language.
   *
   * `options` is an array of plain strings, so it has no `_de` sibling to
   * read the way a named field does; the German is a parallel
   * `options_de` array. Positional, therefore, and the two must stay the
   * same length — a mismatch silently answers a different question than
   * the one the learner was shown, so a short array is treated as absent
   * rather than indexed into.
   */
  function optionText(
    p: { options?: string[]; options_de?: string[] } | null,
    i: number,
  ): string | undefined {
    if (!p || i18n.locale !== "de") return undefined;
    const de = p.options_de;
    if (!de || de.length !== (p.options?.length ?? -1)) return undefined;
    return de[i];
  }

  const prediction = $derived(open?.expect?.predict ?? null);
  const mustPredict = $derived(prediction !== null && predicted === null);
  const stepCount = $derived(open ? runnableLines(open.setup.script).length : 0);
  const canFresh = $derived(open ? canUseFreshVessels(open.setup.script) : false);

  /** Ask before touching a bench that already has the learner's work on it. */
  function requestRun() {
    if (!open || running || session.busy) return;
    if (runGate(session.scene, decision) === "ask") {
      asking = true;
      return;
    }
    void go(decision);
  }

  async function go(chosen: BenchDecision | null) {
    if (!open || running) return;
    const entry = open;
    asking = false;
    decision = chosen;
    running = true;
    result = null;
    refusedLine = null;
    step = null;
    stopRequested = false;
    try {
      const outcome = await runCatalogEntry(session, entry, {
        decision: chosen,
        onstep: (s) => (step = s),
        stopped: () => stopRequested,
      });
      result = outcome.result;
      refusedLine = outcome.refusedAt === null ? null : (outcome.ran.at(-1) ?? null);
    } finally {
      running = false;
      step = null;
      // Consent is per run, not per entry. A replay of an experiment the
      // learner cleared the bench for must ask again, or the second tap
      // is the silent wipe the first one was written to prevent.
      decision = null;
    }
  }

  const kitItems = $derived.by(() => {
    if (!open) return [];
    const keys = scriptKit(open.setup.script);
    return keys
      .map((k) => session.shelf.find((s) => s.key === k))
      .filter((s): s is NonNullable<typeof s> => s != null);
  });

  const diagnosisForPick = $derived.by(() => {
    if (!prediction || predicted === null || predicted === prediction.answer) return null;
    return prediction.diagnosis?.find((d) => d.option === predicted) ?? null;
  });
</script>

<!-- While a script runs the scrim goes transparent and stops swallowing
     pointer events: the whole point is that the learner watches the bench
     react, which they cannot do through a blurred sheet of glass. -->
<div
  class="scrim"
  class:running
  role="presentation"
  onclick={() => !running && onclose()}
  onkeydown={(e) => e.key === "Escape" && !running && onclose()}
>
  <dialog open class="panel" class:running aria-modal={!running} aria-label={tier === "kids" ? t("Kids Lab") : t("experiments")} onclick={(e) => e.stopPropagation()}>
    {#if running}
      <div class="dock" role="status" aria-live="polite">
        <div>
          <span class="dock-kicker">{t("running on the bench")}</span>
          <strong>{open ? t(open.id.replace(/-/g, " ")) : ""}</strong>
          <code>{step?.line ?? ""}</code>
        </div>
        <span class="dock-count">{t("step {step} of {total}", { step: (step?.index ?? 0) + 1, total: step?.total ?? stepCount })}</span>
        <button class="stop" onclick={() => (stopRequested = true)}>{t("stop the run")}</button>
      </div>
    {:else if !open}
      <header>
        <h2 id={tier === "kids" ? "kids-title" : "experiments-title"}>{tier === "kids" ? t("Kids Lab") : t("experiments")}</h2>
        <span class="hint">
          {tier === "kids"
            ? t("sixty experiments for curious kids")
            : t("{count} from the codex — each one computed, checked, and yours to break", { count: entries.length })}
        </span>
        <button class="close" aria-label={t("close")} onclick={onclose}>×</button>
      </header>
      <nav class="tiers" role="group" aria-label={t("catalogue tier")}>
        {#each [["experiments", "experiments"], ["kids", "Kids Lab"]] as const as [key, label] (key)}
          <button class:on={tier === key} aria-pressed={tier === key} onclick={() => setTier(key)}>{t(label)}</button>
        {/each}
        <input
          class="filter"
          type="search"
          placeholder={t("filter…")}
          bind:value={filter}
          aria-label={t("filter experiments")}
        />
      </nav>

      {#if tier === "kids"}
        <section class="filters">
          <div class="chips" role="group" aria-label={t("what the bench can compute")}>
            <button class:on={status === null} aria-pressed={status === null} onclick={() => (status = null)}>{t("all")}</button>
            {#each statuses as value (value)}<button class:on={status === value} aria-pressed={status === value} data-status={value} onclick={() => (status = status === value ? null : value)}>{t(value)}</button>{/each}
          </div>
          <select bind:value={topic} aria-label={t("topic")}><option value="">{t("all topics")}</option>{#each topics as value (value)}<option value={value}>{t(value)}</option>{/each}</select>
          <strong class="tally">{kidsShown.length}/{kidsEntries.length}</strong>
        </section>
        <div class="cards">
          {#each kidsShown as entry (entry.id)}
            {@const links = linksById.get(entry.id)!}
            {@const runnable = runnableCodex(entry)}
            <article data-status={entry.status}>
              <div class="card-head"><span class="kid-id">{entry.id}</span><span class="status">{t(entry.status)}</span><span class="safety">{entry.safety === "home" ? t("home-friendly") : t("school supervision")}</span></div>
              {#if links.linkedLearning > 0}
                <div class="learning-progress" data-progress={links.progress}>
                  <span>{t(`${links.progress} linked learning`)}</span>
                  <strong>{links.completedLearning}/{links.linkedLearning}</strong>
                </div>
              {/if}
              <h2>{kidsText(entry, "title", i18n.locale)}</h2><p>{kidsText(entry, "phenomenon", i18n.locale)}</p>
              <dl><div><dt>{t("ingredients")}</dt><dd>{entry.ingredients.map((value) => t(value.replaceAll("_", " "))).join(" · ")}</dd></div><div><dt>{t("apparatus")}</dt><dd>{entry.apparatus.map((value) => t(value.replaceAll("_", " "))).join(" · ")}</dd></div></dl>
              {#if entry.boundary}<p class="boundary">{kidsText(entry, "boundary", i18n.locale)}</p>{/if}
              {#if links.capabilities.length || links.codex.length || links.lessonCompleted}
                <div class="connections" aria-label={t("related learning and saved progress")}>
                  {#each links.capabilities as id (id)}
                    <button class="related" onclick={() => oncapability?.(id)}>{t("related question")} <span>{id}</span> →</button>
                  {/each}
                  {#each links.codex as id (id)}
                    <button class="related" onclick={() => { const found = byId.get(id); if (found) openEntry(found); }}>{t(codexLearningLabel(links.codexCompleted.includes(id)))} <span>{id.replaceAll("-", " ")}</span> →</button>
                  {/each}
                  {#if links.lessonCompleted}<span class="saved">✓ {t("guided completion saved")}</span>{/if}
                </div>
              {/if}
              <footer>
                <span>{entry.topics.map((value) => t(value)).join(" · ")}</span>
                {#if runnable}<button class="run-here" onclick={() => openEntry(runnable, "run")}>{t("run it on the bench")} →</button>{/if}
                {#if entry.lesson}<button onclick={() => onlesson?.(entry.lesson!)}>{t(guidedLearningLabel(links.lessonCompleted))} →</button>{/if}
                {#if entry.quest}<button onclick={() => onquest?.(entry.quest!)}>{t("start quest")} →</button>{/if}
                {#if !runnable && !entry.lesson && !entry.quest && (entry.status === "computed" || entry.status === "partial")}<button class="sandbox" onclick={() => onsandbox?.(entry)}>{t("explore in Sandbox")} →</button>{/if}
                {#if !runnable && !entry.lesson && !entry.quest && entry.status !== "computed" && entry.status !== "partial"}<span class="no-launch">{t("documented boundary")}</span>{/if}
              </footer>
            </article>
          {:else}<p class="empty">{t("nothing matches that filter")}</p>{/each}
        </div>
      {:else}
        <nav class="tabs">
          {#each [["all", "all"], ["concepts", "by concept"], ["curriculum", "by curriculum"]] as const as [key, label] (key)}
            <button class:on={view === key} aria-pressed={view === key} onclick={() => (view = key as typeof view)}>{t(label)}</button>
          {/each}
          <span class="progress-filters" role="group" aria-label={t("completion status")}>
            {#each [["all", "all"], ["not-tried", "not tried"], ["completed", "completed"]] as const as [value, label] (value)}
              <button class:on={progress === value} aria-pressed={progress === value} onclick={() => (progress = value)}>{t(label)}</button>
            {/each}
          </span>
        </nav>

        {#if view === "concepts"}
          <div class="chips" role="group" aria-label={t("concepts")}>
            {#each concepts as c (c.concept)}
              <button
                class="chip"
                class:on={concept === c.concept}
                onclick={() => (concept = concept === c.concept ? null : c.concept)}
              >
                {t(c.concept.replace(/-/g, " "))} <small>{c.count}</small>
              </button>
            {/each}
            {#if concepts.length === 0}
              <p class="empty">{t("these entries name no concepts yet")}</p>
            {/if}
          </div>
          {#if concept && related.length > 0}
            <p class="meta">
              {t("taught alongside:")}
              {#each related as r, i (r)}
                <button class="link" onclick={() => (concept = r)}>{t(r.replace(/-/g, " "))}</button
                >{i < related.length - 1 ? ", " : ""}
              {/each}
            </p>
          {/if}
        {/if}

        {#if view === "curriculum"}
          {#if curricula.length === 0}
            <p class="empty">{t("no curriculum placements in this export yet")}</p>
          {/if}
          {#each curricula as sys (sys.system)}
            <section class="system">
              <h3>{t(sys.system.replace(/-/g, " "))}</h3>
              {#each sys.stages as st (st.stage)}
                <details>
                  <summary>
                    {t(st.stage)} <small>{st.entries.length}</small>
                  </summary>
                  <ul class="list">
                    {#each st.entries as e (e.id)}
                      {#if experimentHasProgress(e, session.completedExperiments, progress)}
                      <li>
                        <button class="entry" onclick={() => openEntry(e)}>
                          <strong>{t(e.id.replace(/-/g, " "))}</strong>
                          <span class="eq">{e.equation ?? tEngine(e, "summary")}</span>
                          <span class="completion">{session.completedExperiments.has(e.id) ? "✓ " : ""}{t(experimentProgressLabel(e, session.completedExperiments))}</span>
                        </button>
                      </li>
                      {/if}
                    {/each}
                  </ul>
                  {#if st.sources.length > 0}
                    <p class="meta">{t("placed per: {sources}", { sources: st.sources.join("; ") })}</p>
                  {/if}
                </details>
              {/each}
            </section>
          {/each}
        {:else}
          <ul class="list">
            {#each shown as e (e.id)}
              <li>
                <button class="entry" onclick={() => openEntry(e)}>
                  <strong>{t(e.id.replace(/-/g, " "))}</strong>
                  <span class="eq">{e.equation ?? tEngine(e, "summary")}</span>
                  <span class="completion">{session.completedExperiments.has(e.id) ? "✓ " : ""}{t(experimentProgressLabel(e, session.completedExperiments))}</span>
                </button>
              </li>
            {/each}
            {#if shown.length === 0}
              <li><p class="empty">{t("nothing matches that filter")}</p></li>
            {/if}
          </ul>
        {/if}
      {/if}
    {:else}
      <header>
        <button class="back" aria-label={t("back")} onclick={() => (open = null)}>←</button>
        <h2>{t(open.id.replace(/-/g, " "))}</h2>
        <button class="close" aria-label={t("close")} onclick={onclose}>×</button>
      </header>
      <nav class="tabs">
        {#each [["theory", "theory"], ["procedure", "procedure"], ["run", "predict & run"]] as const as [key, label] (key)}
          <button class:on={tab === key} aria-pressed={tab === key} onclick={() => (tab = key as typeof tab)}>{t(label)}</button>
        {/each}
      </nav>

      {#if tab === "theory"}
        {#if open.equation}<p class="equation">{open.equation}</p>{/if}
        <p class="prose">{theory}</p>
        {#if session.register !== "lv1" && (open.concepts?.length ?? 0) > 0}
          <p class="meta">{t("concepts: {concepts}", { concepts: open.concepts!.map(tSlug).join(", ") })}</p>
        {/if}
        {#if session.register === "lv3" && (open.models?.length ?? 0) > 0}
          <p class="meta">{t("models: {models}", { models: open.models!.map(tSlug).join(", ") })}</p>
        {/if}
      {:else if tab === "procedure"}
        {#if (open.apparatus?.length ?? 0) > 0}
          <p class="meta">{t("you will need: {apparatus}", { apparatus: open.apparatus!.map(tSlug).join(", ") })}</p>
        {/if}
        {#if kitItems.length > 0}
          <KitStrip
            items={kitItems}
            register={session.register}
            target={session.selected}
            onadd={(line) => {
              void session.submit(line);
              onclose();
            }}
          />
        {/if}
        <pre class="script">{open.setup.script}</pre>
      {:else}
        {#if prediction}
          <div class="predict">
            <p class="question">{tEngine(prediction, "question")}</p>
            {#each prediction.options as opt, i (i)}
              <button
                class="option"
                class:picked={predicted === i}
                class:right={result !== null && i === prediction.answer}
                class:wrong={result !== null && predicted === i && i !== prediction.answer}
                disabled={result !== null}
                onclick={() => (predicted = i)}
              >
                {optionText(prediction, i) ?? t(opt)}
              </button>
            {/each}
            {#if mustPredict}
              <p class="meta">{t("commit a prediction first — the reveal only teaches if you have.")}</p>
            {/if}
          </div>
        {/if}

        {#if asking}
          <div class="ask" role="group" aria-label={t("the bench is not empty")}>
            <strong>{t("your bench is not empty")}</strong>
            <p>{t("This script writes into the bench you can see. Clear it first, or keep your work and run the experiment in fresh glassware beside it.")}</p>
            <div class="ask-actions">
              <button class="go" onclick={() => void go("clear")}>{t("clear the bench, then run")}</button>
              {#if canFresh}<button class="go" onclick={() => void go("fresh")}>{t("keep my work, run in new vessels")}</button>{/if}
              <button class="go" onclick={() => void go("keep")}>{t("run on this bench as it is")}</button>
              <button class="link" onclick={() => (asking = false)}>{t("cancel")}</button>
            </div>
          </div>
        {:else}
          <button class="go" disabled={running || session.busy || mustPredict} onclick={() => requestRun()}>
            {t("run it on the bench")}
          </button>
          <p class="meta">{t("{count} steps, run one at a time on the bench you can see", { count: stepCount })}</p>
        {/if}

        {#if refusedLine}
          <p class="meta refused">{t("the bench stopped at {line} — the rest of the script did not run", { line: refusedLine })}</p>
        {/if}
        {#if result}
          <div class="verdict" class:ok={result.allOk}>
            <strong>{result.allOk ? t("the chemistry agrees") : t("not everything checked out")}</strong>
            <ul>
              {#each result.events as e (e.want)}
                <li class:ok={e.seen}>{e.seen ? "✓" : "✗"} {t(e.want.replace(/_/g, " "))}</li>
              {/each}
              {#each result.forbidden as f (f.want)}
                <li class:ok={!f.violated}>{f.violated ? `✗ ${t("occurred")}` : `✓ ${t("absent")}`}: {t(f.want.replace(/_/g, " "))}</li>
              {/each}
              {#if result.ph}
                <li class:ok={result.ph.ok}>
                  {result.ph.ok ? "✓" : "✗"} pH {result.ph.value?.toFixed(2) ?? "—"}
                  ({t("expected {range}", { range: `${result.ph.range.min ?? "…"}–${result.ph.range.max ?? "…"}` })})
                </li>
              {/if}
              {#if result.temperature_c}
                <li class:ok={result.temperature_c.ok}>
                  {result.temperature_c.ok ? "✓" : "✗"} {result.temperature_c.value?.toFixed(1) ?? "—"} °C
                  ({t("expected {range}", { range: `${result.temperature_c.range.min ?? "…"}–${result.temperature_c.range.max ?? "…"}` })})
                </li>
              {/if}
            </ul>
            <button class="link" onclick={onclose}>{t("look at the bench")} →</button>
            {#if prediction && predicted !== null}
              {#if predicted === prediction.answer}
                <p class="meta">{t("your prediction held.")}</p>
              {:else}
                <p class="meta">
                  {t("the bench answered {answer}.", { answer: `“${optionText(prediction, prediction.answer) ?? t(prediction.options[prediction.answer] ?? "")}”` })}
                  {#if diagnosisForPick}
                    {tEngine(diagnosisForPick, "reveals")}
                    {#if diagnosisForPick.next}{t("Try: {next}", { next: tEngine(diagnosisForPick, "next") })}{/if}
                  {:else if prediction.misconception}
                    {tEngine(prediction, "misconception")}
                  {/if}
                </p>
              {/if}
            {/if}
          </div>
        {/if}
      {/if}
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
  /* The run is the experiment. While it happens the catalogue is a caption
     at the foot of the screen, not a lid on top of it. */
  .scrim.running {
    background: none;
    backdrop-filter: none;
    pointer-events: none;
    place-items: end center;
    padding: 0 0 1rem;
  }
  .panel {
    position: static;
    margin: 0;
    color: var(--ink);
    background: var(--bg);
    border: 1px solid var(--edge);
    border-radius: 12px;
    padding: 1rem;
    width: min(94vw, 720px);
    max-height: 90vh;
    overflow-y: auto;
  }
  .panel.running {
    pointer-events: auto;
    max-height: none;
    padding: 0.55rem 0.85rem;
    border-color: var(--hot);
  }
  .dock {
    display: flex;
    align-items: center;
    gap: 0.8rem;
  }
  .dock > div {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
  }
  .dock-kicker {
    color: var(--hot);
    font-size: 0.6rem;
    font-weight: 800;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }
  .dock code {
    font-size: 0.78rem;
    color: var(--dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dock-count {
    margin-left: auto;
    color: var(--dim);
    font-size: 0.74rem;
    white-space: nowrap;
  }
  .stop {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.3rem 0.8rem;
    min-height: 36px;
    cursor: pointer;
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
  .close,
  .back {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.9rem;
    line-height: 1;
    padding: 0.3rem 0.6rem;
    cursor: pointer;
  }
  .close {
    margin-left: auto;
  }
  .list {
    list-style: none;
    margin: 0.7rem 0 0;
    padding: 0;
  }
  .entry {
    width: 100%;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--edge);
    color: var(--ink);
    font: inherit;
    padding: 0.5rem 0.2rem;
    cursor: pointer;
  }
  .entry:hover strong {
    color: var(--hot);
  }
  .eq {
    color: var(--dim);
    font-size: 0.78rem;
  }
  .tabs,
  .tiers {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    margin: 0.7rem 0;
  }
  .tiers {
    border-bottom: 1px solid var(--edge);
    padding-bottom: 0.6rem;
  }
  .progress-filters { display: flex; gap: .3rem; padding-left: .3rem; border-left: 1px solid var(--edge); }
  .completion { color: var(--dim); font-size: .68rem; }
  .tabs button,
  .tiers button {
    background: none;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.8rem;
    cursor: pointer;
  }
  .tabs button.on,
  .tiers button.on {
    color: var(--ink);
    border-color: var(--hot);
  }
  .equation {
    text-align: center;
    font-size: 0.95rem;
    margin: 0.4rem 0;
  }
  .prose {
    font-size: 0.88rem;
    line-height: 1.6;
    white-space: pre-wrap;
  }
  .meta {
    color: var(--dim);
    font-size: 0.78rem;
    margin: 0.3rem 0;
  }
  .refused {
    color: var(--bad);
  }
  .script {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 8px;
    padding: 0.6rem 0.8rem;
    font-size: 0.8rem;
    overflow-x: auto;
  }
  .predict {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.6rem;
  }
  .question {
    margin: 0.2rem 0;
    font-size: 0.9rem;
  }
  .option {
    text-align: left;
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.4rem 0.7rem;
    cursor: pointer;
    min-height: 40px;
  }
  .option.picked {
    border-color: var(--hot);
  }
  .option.right {
    border-color: var(--good);
  }
  .option.wrong {
    border-color: var(--bad);
  }
  .ask {
    border: 1px solid var(--hot);
    border-radius: 8px;
    padding: 0.6rem 0.8rem;
    margin: 0.4rem 0;
  }
  .ask p {
    color: var(--dim);
    font-size: 0.8rem;
    line-height: 1.45;
    margin: 0.3rem 0 0.6rem;
  }
  .ask-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
  }
  .go {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 8px;
    color: var(--ink);
    font: inherit;
    padding: 0.4rem 1rem;
    cursor: pointer;
    min-height: 40px;
  }
  .go:disabled {
    opacity: 0.5;
  }
  .verdict {
    margin-top: 0.7rem;
    border: 1px solid var(--bad);
    border-radius: 8px;
    padding: 0.6rem 0.8rem;
    font-size: 0.85rem;
  }
  .verdict.ok {
    border-color: var(--good);
  }
  .verdict ul {
    list-style: none;
    margin: 0.3rem 0;
    padding: 0;
  }
  .verdict li {
    color: var(--bad);
  }
  .verdict li.ok {
    color: var(--good);
  }
  .filter {
    margin-left: auto;
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.8rem;
    min-width: 8rem;
    min-height: 36px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin: 0.5rem 0;
  }
  .chip,
  .chips button {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.2rem 0.7rem;
    cursor: pointer;
  }
  .chip.on,
  .chips button.on {
    border-color: var(--hot);
  }
  .chip small {
    color: var(--dim);
  }
  .link {
    background: none;
    border: 0;
    color: var(--cool);
    font: inherit;
    font-size: inherit;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }
  .empty {
    color: var(--dim);
    font-size: 0.8rem;
  }
  .system h3 {
    font-size: 0.85rem;
    margin: 0.7rem 0 0.2rem;
    text-transform: capitalize;
  }
  details {
    border-bottom: 1px solid var(--edge);
    padding: 0.25rem 0;
  }
  summary {
    cursor: pointer;
    font-size: 0.85rem;
  }
  summary small {
    color: var(--dim);
  }

  /* ── Kids tier: bigger targets, fewer words, same runner ───────────── */
  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }
  .filters select {
    min-height: 40px;
    padding: 0 0.65rem;
    border: 1px solid var(--edge);
    border-radius: 10px;
    color: var(--ink);
    background: var(--panel);
    font: inherit;
  }
  .tally {
    margin-left: auto;
    color: var(--hot);
    font-size: 0.8rem;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    align-content: start;
    gap: 0.7rem;
    margin-top: 0.6rem;
  }
  .cards article {
    display: flex;
    flex-direction: column;
    padding: 0.85rem;
    border: 1px solid var(--edge);
    border-radius: 16px;
    background: var(--panel);
  }
  .cards article[data-status="boundary"],
  .cards article[data-status="declined"],
  .cards article[data-status="unreachable"] {
    border-style: dashed;
  }
  .card-head { display: flex; align-items: center; gap: 0.35rem; }
  .kid-id, .status, .safety, .no-launch {
    padding: 0.18rem 0.4rem;
    border-radius: 999px;
    font-size: 0.55rem;
    font-weight: 850;
    text-transform: uppercase;
  }
  .kid-id { color: var(--bg); background: var(--hot); }
  .status { color: var(--cool); background: var(--panel-raised); }
  .safety { margin-left: auto; color: var(--dim); }
  .cards h2 { margin: 0.55rem 0 0.25rem; font-size: 1rem; }
  .cards article > p { margin: 0; color: var(--dim); font-size: 0.74rem; line-height: 1.45; }
  dl { display: grid; gap: 0.35rem; margin: 0.7rem 0; }
  dl div { display: grid; grid-template-columns: 5rem 1fr; gap: 0.35rem; }
  dt { color: var(--dim); font-size: 0.58rem; font-weight: 800; text-transform: uppercase; }
  dd { margin: 0; font-size: 0.66rem; }
  .boundary { padding: 0.5rem; border-left: 3px solid var(--bad); }
  .connections { display: flex; flex-wrap: wrap; gap: 0.3rem; margin: 0.2rem 0 0.35rem; }
  .connections .related {
    padding: 0.3rem 0.45rem;
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--ink);
    background: var(--panel-raised);
    font: inherit;
    font-size: 0.62rem;
    cursor: pointer;
    text-align: left;
  }
  .connections .related span { color: var(--cool); }
  .saved {
    padding: 0.3rem 0.45rem;
    border-radius: 8px;
    color: var(--good);
    font-size: 0.62rem;
    font-weight: 800;
  }
  .cards article footer { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-top: auto; padding-top: 0.65rem; }
  .cards article footer > span:first-child { flex: 1; color: var(--dim); font-size: 0.6rem; }
  .cards article footer button {
    min-height: 40px;
    border: 1px solid var(--hot);
    border-radius: 9px;
    color: var(--ink);
    background: var(--panel-raised);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.35rem 0.8rem;
    cursor: pointer;
    font-weight: 700;
  }
  .no-launch { color: var(--dim); }
  .learning-progress { display: flex; align-items: center; gap: 0.4rem; margin-top: 0.5rem; color: var(--dim); font-size: 0.62rem; }
  .learning-progress strong { margin-left: auto; }
  .learning-progress[data-progress="all"] { color: var(--good); }
  .learning-progress[data-progress="some"] { color: var(--cool); }

  @media (max-width: 760px) {
    .cards { grid-template-columns: 1fr; }
  }
</style>
