<script lang="ts">
  import { untrack } from "svelte";
  import {
    checkExpect,
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

  let {
    entries,
    session,
    onclose,
    initial = null,
  }: {
    entries: CodexEntry[];
    session: Session;
    onclose: () => void;
    /** Open directly on one entry (the concept map hands entries over). */
    initial?: CodexEntry | null;
  } = $props();

  let open = $state<CodexEntry | null>(untrack(() => initial));
  let tab = $state<"theory" | "procedure" | "run">("theory");
  let predicted = $state<number | null>(null);
  let result = $state<CheckResult | null>(null);
  let running = $state(false);

  /** The catalog's three doors: everything, by concept, by curriculum. */
  let view = $state<"all" | "concepts" | "curriculum">("all");
  let filter = $state("");
  let concept = $state<string | null>(null);
  const concepts = $derived(conceptIndex(entries));
  const curricula = $derived(curriculumIndex(entries));
  const related = $derived(concept ? relatedConcepts(entries, concept).slice(0, 8) : []);
  const shown = $derived.by(() => {
    let list = entries;
    if (view === "concepts" && concept) {
      list = list.filter((e) => e.concepts?.includes(concept!));
    }
    const q = filter.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (e) =>
          e.id.includes(q) ||
          (e.equation ?? "").toLowerCase().includes(q) ||
          (e.summary ?? "").toLowerCase().includes(q) ||
          e.concepts?.some((c) => c.includes(q)),
      );
    }
    return list;
  });

  function openEntry(e: CodexEntry) {
    open = e;
    tab = "theory";
    predicted = null;
    result = null;
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

  async function runIt() {
    if (!open || running) return;
    running = true;
    result = null;
    try {
      const observed = await session.runExperiment(open.setup.script);
      result = checkExpect(open.expect ?? {}, observed, session.finalStateForCheck());
      // A green check is learner progress: it feeds the concept map.
      if (result.allOk) session.markExperimentDone(open.id);
    } finally {
      running = false;
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

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(e) => e.key === "Escape" && onclose()}>
  <dialog open class="panel" aria-modal="true" aria-label={t("experiments")} onclick={(e) => e.stopPropagation()}>
    {#if !open}
      <header>
        <h2>{t("experiments")}</h2>
        <span class="hint">{t("{count} from the codex — each one computed, checked, and yours to break", { count: entries.length })}</span>
        <button class="close" onclick={onclose}>{t("close")}</button>
      </header>
      <nav class="tabs">
        {#each [["all", "all"], ["concepts", "by concept"], ["curriculum", "by curriculum"]] as [key, label] (key)}
          <button class:on={view === key} onclick={() => (view = key as typeof view)}>{t(label)}</button>
        {/each}
        <input
          class="filter"
          type="search"
          placeholder={t("filter…")}
          bind:value={filter}
          aria-label={t("filter experiments")}
        />
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
                  {st.stage} <small>{st.entries.length}</small>
                </summary>
                <ul class="list">
                  {#each st.entries as e (e.id)}
                    <li>
                      <button class="entry" onclick={() => openEntry(e)}>
                        <strong>{t(e.id.replace(/-/g, " "))}</strong>
                        <span class="eq">{e.equation ?? tEngine(e, "summary")}</span>
                      </button>
                    </li>
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
              </button>
            </li>
          {/each}
          {#if shown.length === 0}
            <li><p class="empty">{t("nothing matches that filter")}</p></li>
          {/if}
        </ul>
      {/if}
    {:else}
      <header>
        <button class="back" onclick={() => (open = null)}>←</button>
        <h2>{t(open.id.replace(/-/g, " "))}</h2>
        <button class="close" onclick={onclose}>{t("close")}</button>
      </header>
      <nav class="tabs">
        {#each [["theory", "theory"], ["procedure", "procedure"], ["run", "predict & run"]] as [key, label] (key)}
          <button class:on={tab === key} onclick={() => (tab = key as typeof tab)}>{t(label)}</button>
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
        <button class="go" disabled={running || session.busy || mustPredict} onclick={() => void runIt()}>
          {running ? t("running…") : t("run it on the bench")}
        </button>
        {#if result}
          <div class="verdict" class:ok={result.allOk}>
            <strong>{result.allOk ? t("the chemistry agrees") : t("not everything checked out")}</strong>
            <ul>
              {#each result.events as e (e.want)}
                <li class:ok={e.seen}>{e.seen ? "✓" : "✗"} {e.want.replace(/_/g, " ")}</li>
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
    background: rgb(0 0 0 / 50%);
    display: grid;
    place-items: center;
    z-index: 10;
    padding: 1rem;
  }
  .panel {
    position: static;
    margin: 0;
    color: var(--ink);
    background: var(--bg);
    border: 1px solid var(--edge);
    border-radius: 12px;
    padding: 1rem;
    width: min(94vw, 640px);
    max-height: 90vh;
    overflow-y: auto;
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
    font-size: 0.8rem;
    padding: 0.25rem 0.7rem;
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
  .tabs {
    display: flex;
    gap: 0.3rem;
    margin: 0.7rem 0;
  }
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
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin: 0.5rem 0;
  }
  .chip {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.2rem 0.7rem;
    cursor: pointer;
  }
  .chip.on {
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
</style>
