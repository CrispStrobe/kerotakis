<!--
  ONE catalogue. One card. One row of filters.

  There used to be two catalogues in two components; then there was one
  component still drawing two tiers, and a tier is exactly the thing a
  learner cannot filter by, cannot search across, and did not ask for. The
  two shapes disagreed about everything a card is: one was a row with an
  equation, the other a wide card with a phenomenon; one had status chips
  and a topic menu, the other concept and curriculum tabs and a completion
  filter; and the second tier wore a name that told a reader it was for
  younger children, which is a judgement about the READER made by the file
  the content happened to ship in.

  So the tiers are gone. `lib/catalogEntry.ts` maps both corpora into one
  view model — deriving the level, the age, the duration and the topics
  that only one side carried — and everything below draws exactly one kind
  of card from it. What used to be the tier is now the LEVEL chip, which is
  a claim about the experiment rather than about the person reading it, and
  which composes with every other filter instead of partitioning the
  library before they are applied.

  The filters are one horizontally scrolling rail (the shelf's pattern),
  because two rows of chips on a phone pushed the experiments themselves
  below the fold: the filter must never be taller than the thing it
  filters. The concept and curriculum doors became two selects in that same
  rail — the same browsing, minus a mode switch that hid the cards.

  Every card offers the same primary action, and which action it is follows
  the CONTENT: an entry with a script runs on the visible bench, one whose
  procedure was written as a guided lesson opens that lesson. Both go
  through `runCatalogEntry`, one step at a time, on the bench you can see.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import {
    conceptIndex,
    relatedConcepts,
    scriptKit,
    type CheckResult,
    type CodexEntry,
  } from "../codex";
  import type { Session } from "../session.svelte";
  import KitStrip from "./KitStrip.svelte";
  import { t, tSlug, tEngine, i18n } from "../i18n.svelte";
  import { available } from "../catalogProgress";
  import {
    CATALOG_DURATIONS,
    CATALOG_LEVELS,
    catalogEntries,
    durationLabel,
    filterCatalogEntries,
    levelCounts,
    levelLabel,
    NO_CATALOG_FILTERS,
    presentPlacements,
    presentTopics,
    runTargetLabel,
    slugWords,
    topicLabel,
    type CatalogEntry,
    type CatalogFilters,
    type CatalogLevel,
  } from "../catalogEntry";
  import {
    canUseFreshVessels,
    loadRunMode,
    runCatalogEntry,
    runGate,
    runnableLines,
    saveRunMode,
    type BenchDecision,
    type RunMode,
    type RunStep,
    type StepReport,
    type StepVerdict,
  } from "../catalogRunner";
  import {
    codexLearningLabel,
    guidedLearningLabel,
    kidsConnections,
    type KidsExperiment,
  } from "../kidsCatalog";

  let {
    entries,
    kidsEntries = [],
    session,
    capabilityIds = new Set<string>(),
    codexIds = new Set<string>(),
    initial = null,
    kidsInitial = null,
    initialLevel = null,
    onlesson,
    onquest,
    oncapability,
    onsandbox,
    onclose,
  }: {
    entries: CodexEntry[];
    kidsEntries?: KidsExperiment[];
    session: Session;
    capabilityIds?: ReadonlySet<string>;
    codexIds?: ReadonlySet<string>;
    /** Open directly on one entry (the concept map hands entries over). */
    initial?: CodexEntry | null;
    /** Open on one entry by its guided-task id — the concept map's other
     * hand-over. It seeds the shared search rather than pinning it, so the
     * learner can widen it immediately. */
    kidsInitial?: string | null;
    /** A door that used to open a tier now opens the list pre-filtered. */
    initialLevel?: CatalogLevel | null;
    onlesson?: (file: string) => void;
    onquest?: (id: string) => void;
    oncapability?: (id: string) => void;
    onsandbox?: (entry: KidsExperiment) => void;
    onclose: () => void;
  } = $props();

  /** Materials the learner can actually reach right now. */
  const shelfKeys = $derived(new Set(
    session.shelf.filter((item) => available(session.catalog, item.key)).map((item) => item.key),
  ));

  /**
   * Both corpora as one list.
   *
   * Rebuilt when the locale changes, because the titles and the one-line
   * hooks ARE the search index: a reader typing German has to match the
   * German they can see, which is the defect `catalogSearch` was written
   * for and which a cached English index would quietly bring back.
   */
  const all = $derived(catalogEntries(entries, kidsEntries, {
    locale: i18n.locale,
    translate: t,
    completed: session.completedExperiments,
    completedMissions: session.completedMissions,
    shelfKeys,
  }));
  const byId = $derived(new Map(all.map((entry) => [entry.id, entry])));

  let filters = $state<CatalogFilters>({
    ...NO_CATALOG_FILTERS,
    level: untrack(() => initialLevel),
    query: untrack(() => kidsInitial ?? ""),
  });
  const shown = $derived(filterCatalogEntries(all, filters));
  const counts = $derived(levelCounts(all));
  const topics = $derived(presentTopics(all)
    .map((topic) => ({ topic, label: t(topicLabel(topic)) }))
    .sort((a, b) => a.label.localeCompare(b.label, i18n.locale)));
  const concepts = $derived(conceptIndex(entries));
  const placements = $derived(presentPlacements(all, i18n.locale));
  const related = $derived(filters.concept ? relatedConcepts(entries, filters.concept).slice(0, 6) : []);

  /**
   * The open entry, held by id rather than by value.
   *
   * The list is rebuilt whenever the locale or the learner's progress
   * changes, so a captured object would go stale the moment either did —
   * the panel would keep showing the German title of an English session,
   * or a completion that has since been recorded.
   */
  let openId = $state<string | null>(untrack(() => initial?.id ?? null));
  const open = $derived(openId === null ? null : byId.get(openId) ?? null);
  let tab = $state<"theory" | "procedure" | "run">("theory");
  let predicted = $state<number | null>(null);
  let result = $state<CheckResult | null>(null);
  let refusedLine = $state<string | null>(null);

  /** Linked learning, for the entries that name any. */
  const linksById = $derived(new Map(all
    .filter((entry) => entry.guided !== null)
    .map((entry) => [entry.id, kidsConnections(
      entry.guided!, capabilityIds, codexIds, session.completedMissions, session.completedExperiments,
    )])));

  /** Run state: the panel docks while these are live, so the stage shows. */
  let running = $state(false);
  let step = $state<RunStep | null>(null);
  let asking = $state(false);
  let decision = $state<BenchDecision | null>(null);
  let stopRequested = false;

  /**
   * Who says "next".
   *
   * Automatic is a demonstration; step by step is the experiment. A learner
   * who wants to watch one line land, read what it did, and only then let
   * the next one go had no way to ask for that — the run was a single
   * gesture whatever it contained. The choice is theirs, it is made before
   * the run rather than during it, and it is remembered, because a learner
   * who works this way works this way every time.
   */
  let runMode = $state<RunMode>(untrack(() => loadRunMode()));
  /** The step just finished, while the runner waits for an answer. */
  let stepReport = $state<StepReport | null>(null);
  let awaiting = $state(false);
  /**
   * Whether THIS run is a stepped one.
   *
   * Separate from `runMode` so the strip's buttons can stay mounted for
   * the whole run and merely go inert between steps. Unmounting them on
   * each answer would throw keyboard focus back to the body every time,
   * and "next step" is the one control a learner presses over and over.
   */
  let stepping = $state(false);
  /** True after a run the learner ended early: nothing was recorded. */
  let halted = $state(false);
  let answerStep: ((verdict: StepVerdict) => void) | null = null;

  function setRunMode(mode: RunMode) {
    runMode = mode;
    saveRunMode(mode);
  }

  /** Hand the runner the learner's answer, exactly once. */
  function answer(verdict: StepVerdict) {
    const resolve = answerStep;
    answerStep = null;
    awaiting = false;
    resolve?.(verdict);
  }

  /**
   * Stop, whichever mode is running.
   *
   * The automatic runner is polled between steps, so a flag reaches it;
   * a stepped one is parked on a promise, so it has to be answered. Doing
   * both is what makes one button honest in both modes.
   */
  function stopRun() {
    stopRequested = true;
    answer("stop");
  }

  function openEntry(entry: CatalogEntry, at: typeof tab = "theory") {
    openId = entry.id;
    tab = entry.script ? at : "theory";
    predicted = null;
    result = null;
    refusedLine = null;
    asking = false;
    decision = null;
  }

  /** The card's primary action, which follows the content and not the corpus. */
  function act(entry: CatalogEntry) {
    switch (entry.run.kind) {
      case "script":
        openEntry(entry, "run");
        return;
      case "lesson":
        onlesson?.(entry.run.file);
        return;
      case "quest":
        onquest?.(entry.run.id);
        return;
      case "sandbox":
        if (entry.guided) onsandbox?.(entry.guided);
        return;
      default:
        openEntry(entry);
    }
  }

  function clearFilters() {
    filters = { ...NO_CATALOG_FILTERS };
  }

  const filtering = $derived(
    filters.level !== null || filters.topic !== null || filters.duration !== null
    || filters.shelfOnly || filters.progress !== "all"
    || filters.concept !== null || filters.curriculum !== null || filters.query.trim() !== "",
  );

  // The register prose at the dial's level, tolerant of key spelling.
  const theory = $derived.by(() => {
    const script = open?.script;
    if (!script) return "";
    const r = script.registers ?? {};
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

  const prediction = $derived(open?.script?.expect?.predict ?? null);
  const mustPredict = $derived(prediction !== null && predicted === null);
  const stepCount = $derived(open?.script ? runnableLines(open.script.setup.script).length : 0);
  const canFresh = $derived(open?.script ? canUseFreshVessels(open.script.setup.script) : false);

  /** Ask before touching a bench that already has the learner's work on it. */
  function requestRun() {
    if (!open?.script || running || session.busy) return;
    if (runGate(session.scene, decision) === "ask") {
      asking = true;
      return;
    }
    void go(decision);
  }

  async function go(chosen: BenchDecision | null) {
    const script = open?.script;
    if (!script || running) return;
    asking = false;
    decision = chosen;
    running = true;
    result = null;
    refusedLine = null;
    step = null;
    stepReport = null;
    halted = false;
    stepping = runMode === "step";
    stopRequested = false;
    try {
      const outcome = await runCatalogEntry(session, script, {
        decision: chosen,
        onstep: (s) => {
          step = s;
          // The previous step's account belongs to the previous step.
          stepReport = null;
        },
        stopped: () => stopRequested,
        // Presence is the mode: handing the runner a gate is what makes it
        // wait, so an automatic run passes nothing rather than a flag the
        // runner would have to interpret.
        onstepdone: runMode === "step"
          ? (report) => new Promise<StepVerdict>((resolve) => {
              stepReport = report;
              awaiting = true;
              answerStep = resolve;
            })
          : undefined,
      });
      result = outcome.result;
      halted = outcome.halted;
      refusedLine = outcome.refusedAt === null ? null : (outcome.ran.at(-1) ?? null);
    } finally {
      running = false;
      step = null;
      stepReport = null;
      awaiting = false;
      stepping = false;
      answerStep = null;
      // Consent is per run, not per entry. A replay of an experiment the
      // learner cleared the bench for must ask again, or the second tap
      // is the silent wipe the first one was written to prevent.
      decision = null;
    }
  }

  const kitItems = $derived.by(() => {
    const script = open?.script;
    if (!script) return [];
    return scriptKit(script.setup.script)
      .map((k) => session.shelf.find((s) => s.key === k))
      .filter((s): s is NonNullable<typeof s> => s != null);
  });

  const diagnosisForPick = $derived.by(() => {
    if (!prediction || predicted === null || predicted === prediction.answer) return null;
    return prediction.diagnosis?.find((d) => d.option === predicted) ?? null;
  });

  /** Slugs as a card says them, in the reader's language. */
  const words = (values: readonly string[]) =>
    values.map((value) => t(slugWords(value))).join(" · ");
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
  <dialog open class="panel" class:running aria-modal={!running} aria-label={t("experiments")} onclick={(e) => e.stopPropagation()}>
    {#if running}
      <div class="dock" class:waiting={awaiting} role="status" aria-live="polite">
        <div>
          <span class="dock-kicker">{awaiting ? t("your turn") : t("running on the bench")}</span>
          <strong>{open?.title ?? ""}</strong>
          <code>{step?.line ?? ""}</code>
          <!-- What that line DID, in the bench's own words. The feed is
               already the record a learner reads when they type a command
               themselves, so quoting its tail here is the same account, not
               a second one written for the catalogue. -->
          {#if awaiting && stepReport}
            {#if stepReport.produced.length > 0}
              <ul class="dock-produced">
                {#each stepReport.produced as line, i (i)}
                  <li data-kind={line.kind}>{line.text}</li>
                {/each}
              </ul>
            {:else}
              <span class="dock-quiet">{t("the bench reported nothing for that step")}</span>
            {/if}
          {/if}
        </div>
        <span class="dock-count">{t("step {step} of {total}", { step: (step?.index ?? 0) + 1, total: step?.total ?? stepCount })}</span>
        {#if stepping}
          <!-- Mounted for the whole run, inert between steps: a control
               that disappears after every press takes the keyboard's focus
               with it, and this is the press a learner repeats. -->
          <button class="go dock-next" disabled={!awaiting} onclick={() => answer("next")}>{t("next step")}</button>
          <button class="stop" disabled={!awaiting} onclick={() => answer("rest")}>{t("run the rest for me")}</button>
        {/if}
        <button class="stop" onclick={stopRun}>{t("stop the run")}</button>
      </div>
    {:else if !open}
      <header>
        <h2 id="catalog-title">{t("experiments")}</h2>
        <span class="hint">{t("{shown} of {total} — each one computed, checked, and yours to break", { shown: shown.length, total: all.length })}</span>
        <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
      </header>

      <!-- ONE row. The search box keeps its place beside the chips and the
           chips scroll sideways past it, so no filter ever wraps the list
           off the screen. -->
      <div class="filter-row">
        <input
          class="filter"
          type="search"
          placeholder={t("filter…")}
          bind:value={filters.query}
          aria-label={t("filter experiments")}
        />
        <div class="filter-rail">
          <div class="chips levels" role="group" aria-label={t("level")}>
            <button class:on={filters.level === null} aria-pressed={filters.level === null} onclick={() => (filters.level = null)}>{t("all")}</button>
            {#each CATALOG_LEVELS as level (level)}
              <button
                data-level={level}
                class:on={filters.level === level}
                aria-pressed={filters.level === level}
                onclick={() => (filters.level = filters.level === level ? null : level)}
              >{t(levelLabel(level))} <small>{counts[level]}</small></button>
            {/each}
          </div>
          <select bind:value={filters.topic} aria-label={t("topic")}>
            <option value={null}>{t("all topics")}</option>
            {#each topics as item (item.topic)}<option value={item.topic}>{item.label}</option>{/each}
          </select>
          <div class="chips durations" role="group" aria-label={t("how long it takes")}>
            <button class:on={filters.duration === null} aria-pressed={filters.duration === null} onclick={() => (filters.duration = null)}>{t("any length")}</button>
            {#each CATALOG_DURATIONS as band (band)}
              <button
                data-duration={band}
                class:on={filters.duration === band}
                aria-pressed={filters.duration === band}
                onclick={() => (filters.duration = filters.duration === band ? null : band)}
              >{t(durationLabel(band))}</button>
            {/each}
          </div>
          <div class="chips shelf-filter" role="group" aria-label={t("what you can reach")}>
            <button
              class="shelf-only"
              class:on={filters.shelfOnly}
              aria-pressed={filters.shelfOnly}
              onclick={() => (filters.shelfOnly = !filters.shelfOnly)}
            >{t("only what is on my shelf")}</button>
          </div>
          <div class="chips progress-filters" role="group" aria-label={t("completion status")}>
            {#each [["all", "all"], ["not-tried", "not tried"], ["completed", "completed"]] as const as [value, label] (value)}
              <button class:on={filters.progress === value} aria-pressed={filters.progress === value} onclick={() => (filters.progress = value)}>{t(label)}</button>
            {/each}
          </div>
          <select bind:value={filters.concept} aria-label={t("concept")}>
            <option value={null}>{t("all concepts")}</option>
            {#each concepts as item (item.concept)}<option value={item.concept}>{tSlug(item.concept)} ({item.count})</option>{/each}
          </select>
          <select bind:value={filters.curriculum} aria-label={t("curriculum")}>
            <option value={null}>{t("any curriculum")}</option>
            {#each placements as placement (placement.key)}
              <option value={placement.key}>{t(slugWords(placement.system))} — {t(placement.stage)}</option>
            {/each}
          </select>
          {#if filtering}
            <button class="chip clear" onclick={clearFilters}>{t("clear filters")}</button>
          {/if}
        </div>
      </div>

      {#if filters.concept && related.length > 0}
        <p class="meta">
          {t("taught alongside:")}
          {#each related as r, i (r)}
            <button class="link" onclick={() => (filters.concept = r)}>{tSlug(r)}</button
            >{i < related.length - 1 ? ", " : ""}
          {/each}
        </p>
      {/if}

      <div class="cards">
        {#each shown as item (item.id)}
          {@const links = linksById.get(item.id) ?? null}
          <article data-id={item.id} data-level={item.level} data-status={item.status} data-run={item.run.kind}>
            <div class="card-head">
              <span class="level">{t(levelLabel(item.level))}</span>
              <span class="age">{t("from age {age}", { age: item.ageMin })}</span>
              <span class="minutes">{t("about {count} min", { count: item.minutes })}</span>
              <span class="completion">{item.done ? "✓ " : ""}{t(item.done ? "completed" : "not tried")}</span>
            </div>
            <h2>{item.title}</h2>
            <p class="hook">{item.hook}</p>
            <dl>
              <div><dt>{t("what you need")}</dt><dd>{item.needs.length > 0 ? words(item.needs) : t("nothing from the shelf")}</dd></div>
              <div><dt>{t("apparatus")}</dt><dd>{item.apparatus.length > 0 ? words(item.apparatus) : t("the bench as it stands")}</dd></div>
            </dl>
            {#if item.boundary}<p class="boundary">{item.boundary}</p>{/if}
            {#if links && links.linkedLearning > 0}
              <div class="learning-progress" data-progress={links.progress}>
                <span>{t(`${links.progress} linked learning`)}</span>
                <strong>{links.completedLearning}/{links.linkedLearning}</strong>
              </div>
            {/if}
            {#if links && (links.capabilities.length > 0 || links.codex.length > 0 || links.lessonCompleted)}
              <div class="connections" aria-label={t("related learning and saved progress")}>
                {#each links.capabilities as id (id)}
                  <button class="related" onclick={() => oncapability?.(id)}>{t("related question")} <span>{id}</span> →</button>
                {/each}
                {#each links.codex as id (id)}
                  <button class="related" onclick={() => { const found = byId.get(id); if (found) openEntry(found); }}>{t(codexLearningLabel(links.codexCompleted.includes(id)))} <span>{t(slugWords(id))}</span> →</button>
                {/each}
                {#if links.lessonCompleted}<span class="saved">✓ {t("guided completion saved")}</span>{/if}
              </div>
            {/if}
            <footer>
              <span class="topics">{item.topics.map((topic) => t(topicLabel(topic))).join(" · ")}</span>
              {#if item.run.kind === "boundary"}
                <span class="no-launch">{t("documented boundary")}</span>
              {:else}
                <button class="run-here" onclick={() => act(item)}>{t(runTargetLabel(item.run, item.done))} →</button>
              {/if}
              <!-- An entry can offer more than one door — a script AND the
                   guided lesson written around it. The primary button is
                   whichever the content makes first; the others stay
                   reachable rather than being hidden by the choice. -->
              {#if item.lesson && item.run.kind !== "lesson"}
                <button class="also" onclick={() => onlesson?.(item.lesson!)}>{t(guidedLearningLabel(links?.lessonCompleted ?? false))} →</button>
              {/if}
              {#if item.quest && item.run.kind !== "quest"}
                <button class="also" onclick={() => onquest?.(item.quest!)}>{t("start quest")} →</button>
              {/if}
              <button class="details" aria-label={t("what this experiment covers")} title={t("what this experiment covers")} onclick={() => openEntry(item)}>i</button>
            </footer>
          </article>
        {:else}
          <p class="empty">{t("nothing matches that filter")}</p>
        {/each}
      </div>
    {:else}
      <header>
        <button class="back" aria-label={t("back")} onclick={() => (openId = null)}>←</button>
        <h2>{open.title}</h2>
        <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
      </header>
      <p class="entry-meta">
        <span>{t(levelLabel(open.level))}</span>
        <span>{t("from age {age}", { age: open.ageMin })}</span>
        <span>{t("about {count} min", { count: open.minutes })}</span>
        <!-- Only where the content actually declares them. The codex export
             carries no supervision note and no computability verdict, and
             deriving one from an age band would be inventing a claim the
             entry never made. -->
        {#if open.guided}
          <span>{t(open.status)}</span>
          <span>{t(open.safety === "home" ? "home-friendly" : "school supervision")}</span>
        {/if}
        <span>{open.topics.map((topic) => t(topicLabel(topic))).join(" · ")}</span>
      </p>

      {#if open.script}
        <nav class="tabs">
          {#each [["theory", "theory"], ["procedure", "procedure"], ["run", "predict & run"]] as const as [key, label] (key)}
            <button class:on={tab === key} aria-pressed={tab === key} onclick={() => (tab = key as typeof tab)}>{t(label)}</button>
          {/each}
        </nav>
      {/if}

      {#if !open.script}
        <!-- No script of its own: the honest page is what it is about, what
             it needs, where the model stops, and the door that does exist. -->
        <p class="prose">{open.hook}</p>
        {#if open.boundary}<p class="boundary">{open.boundary}</p>{/if}
        <p class="meta">{t("you will need: {apparatus}", { apparatus: [...open.needs, ...open.apparatus].map((value) => t(slugWords(value))).join(", ") })}</p>
        {#if open.run.kind !== "boundary"}
          <button class="go" onclick={() => act(open)}>{t(runTargetLabel(open.run, open.done))}</button>
        {:else}
          <p class="meta">{t("documented boundary")}</p>
        {/if}
      {:else if tab === "theory"}
        {#if open.equation}<p class="equation">{open.equation}</p>{/if}
        <p class="prose">{theory}</p>
        {#if session.register !== "lv1" && open.concepts.length > 0}
          <p class="meta">{t("concepts: {concepts}", { concepts: open.concepts.map(tSlug).join(", ") })}</p>
        {/if}
        {#if session.register === "lv3" && (open.script.models?.length ?? 0) > 0}
          <p class="meta">{t("models: {models}", { models: open.script.models!.map(tSlug).join(", ") })}</p>
        {/if}
      {:else if tab === "procedure"}
        {#if open.apparatus.length > 0}
          <p class="meta">{t("you will need: {apparatus}", { apparatus: open.apparatus.map(tSlug).join(", ") })}</p>
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
        <pre class="script">{open.script.setup.script}</pre>
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
          <!-- The pace is chosen BEFORE the run, because during it there is
               nothing left to decide: a script already halfway through at
               420ms a line cannot be un-watched. -->
          <div class="pace" role="group" aria-label={t("how to run it")}>
            <span class="pace-label">{t("pace")}</span>
            {#each [["auto", "straight through"], ["step", "step by step"]] as const as [key, label] (key)}
              <button
                type="button"
                class="chip"
                class:on={runMode === key}
                aria-pressed={runMode === key}
                onclick={() => setRunMode(key)}
              >{t(label)}</button>
            {/each}
          </div>
          <button class="go" disabled={running || session.busy || mustPredict} onclick={() => requestRun()}>
            {t("run it on the bench")}
          </button>
          <p class="meta">
            {runMode === "step"
              ? t("{count} steps — you decide when each one goes", { count: stepCount })
              : t("{count} steps, run one at a time on the bench you can see", { count: stepCount })}
          </p>
        {/if}

        {#if refusedLine}
          <p class="meta refused">{t("the bench stopped at {line} — the rest of the script did not run", { line: refusedLine })}</p>
        {/if}
        {#if halted}
          <!-- The checker answers about the bench, and a bench can satisfy
               it after one line. Saying so is the difference between a
               verdict and a claim that the experiment was done. -->
          <p class="meta refused">{t("you stopped the run — the rest did not happen, and nothing was recorded")}</p>
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
    width: min(94vw, 860px);
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
  /* Waiting is a state, not a pause in the same state: the strip grows to
     hold the account of the step and says whose turn it is. */
  .dock.waiting {
    align-items: flex-start;
    flex-wrap: wrap;
  }
  .dock-produced {
    list-style: none;
    margin: 0.2rem 0 0;
    padding: 0;
    max-height: 5.5rem;
    overflow-y: auto;
    font-size: 0.76rem;
  }
  .dock-produced li {
    color: var(--ink);
  }
  .dock-produced li[data-kind="error"],
  .dock-produced li[data-kind="refusal"],
  .dock-produced li[data-kind="hazard"] {
    color: var(--bad);
  }
  .dock-produced li[data-kind="note"],
  .dock-produced li[data-kind="nudge"] {
    color: var(--dim);
  }
  .dock-quiet {
    color: var(--dim);
    font-size: 0.76rem;
  }
  .dock-next {
    min-height: 36px;
    padding: 0.3rem 0.9rem;
    font-size: 0.78rem;
  }
  .pace {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin-bottom: 0.5rem;
  }
  .pace-label {
    color: var(--dim);
    font-size: 0.62rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .pace .chip {
    min-height: 36px;
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
  .stop:disabled {
    opacity: 0.5;
    cursor: default;
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
  header :global(.icon-close) {
    margin-left: auto;
  }
  .entry-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin: 0.4rem 0 0;
    color: var(--dim);
    font-size: 0.7rem;
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    margin: 0.7rem 0;
  }
  .completion { color: var(--dim); font-size: .68rem; }
  .tabs button {
    background: none;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.8rem;
    cursor: pointer;
  }
  .tabs button.on {
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

  /* ── One row of filters ────────────────────────────────────────────── */
  .filter-row {
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    gap: 0.5rem;
    /* The overflow has to happen in the rail, so nothing here may report a
       min-content width up to the dialog. */
    min-width: 0;
    margin: 0.6rem 0 0.2rem;
    padding-bottom: 0.3rem;
    border-bottom: 1px solid var(--edge);
  }
  .filter {
    flex: none;
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.8rem;
    width: 9rem;
    min-height: 36px;
  }
  /* The rail scrolls; the groups inside it never wrap. `flex: none` on the
     chips matters as much as `nowrap` — without it they shrink to fit and
     a long German label becomes an ellipsis instead of scrolling. */
  .filter-rail {
    display: flex;
    align-items: center;
    flex: 1;
    flex-wrap: nowrap;
    min-width: 0;
    gap: 0.5rem;
    overflow-x: auto;
    overscroll-behavior-x: contain;
    scrollbar-width: thin;
    padding-bottom: 0.2rem;
  }
  .filter-rail > * {
    flex: none;
  }
  .filter-rail button {
    flex: none;
    white-space: nowrap;
  }
  .chips {
    display: flex;
    flex-wrap: nowrap;
    gap: 0.25rem;
    margin: 0;
  }
  .chip,
  .chips button {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.75rem;
    padding: 0.2rem 0.7rem;
    min-height: 36px;
    cursor: pointer;
  }
  .chip.on,
  .chips button.on {
    border-color: var(--hot);
  }
  .chips small {
    color: var(--dim);
  }
  .filter-rail select {
    min-height: 36px;
    max-width: 12rem;
    padding: 0 0.55rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    background: var(--panel);
    font: inherit;
    font-size: 0.75rem;
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

  /* ── One card, whatever the entry is ───────────────────────────────── */
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
  /* The only visual difference between cards is a claim about the
     CONTENT: a documented boundary is not an experiment you can run, and
     saying so with a dashed edge is cheaper than a paragraph. */
  .cards article[data-run="boundary"] {
    border-style: dashed;
  }
  .card-head { display: flex; align-items: center; flex-wrap: wrap; gap: 0.35rem; }
  .level, .age, .minutes, .no-launch {
    padding: 0.18rem 0.4rem;
    border-radius: 999px;
    font-size: 0.55rem;
    font-weight: 850;
    text-transform: uppercase;
  }
  .level { color: var(--bg); background: var(--hot); }
  .age, .minutes { color: var(--cool); background: var(--panel-raised); }
  .card-head .completion { margin-left: auto; }
  .cards h2 { margin: 0.55rem 0 0.25rem; font-size: 1rem; }
  .hook { margin: 0; color: var(--dim); font-size: 0.74rem; line-height: 1.45; }
  dl { display: grid; gap: 0.35rem; margin: 0.7rem 0; }
  dl div { display: grid; grid-template-columns: 5rem 1fr; gap: 0.35rem; }
  dt { color: var(--dim); font-size: 0.58rem; font-weight: 800; text-transform: uppercase; }
  dd { margin: 0; font-size: 0.66rem; }
  .boundary { padding: 0.5rem; border-left: 3px solid var(--bad); font-size: 0.7rem; line-height: 1.45; }
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
  .topics { flex: 1; color: var(--dim); font-size: 0.6rem; }
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
  .cards article footer button.also {
    border-color: var(--edge);
    font-weight: 600;
    font-size: 0.74rem;
  }
  .cards article footer button.details {
    min-width: 40px;
    border-color: var(--edge);
    font-style: italic;
  }
  .no-launch { color: var(--dim); }
  .learning-progress { display: flex; align-items: center; gap: 0.4rem; margin-top: 0.5rem; color: var(--dim); font-size: 0.62rem; }
  .learning-progress strong { margin-left: auto; }
  .learning-progress[data-progress="all"] { color: var(--good); }
  .learning-progress[data-progress="some"] { color: var(--cool); }

  @media (max-width: 760px) {
    .cards { grid-template-columns: 1fr; }
    .filter { width: 6.5rem; }
  }
</style>
