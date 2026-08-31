<script lang="ts">
  /**
   * GUI-095 — balancing practice, generated rather than authored.
   *
   * The question is any equation the codex or this session produced with
   * its coefficients taken off; the marking is `balancing.ts` over the
   * composition matrix the engine returns from `balance`. There is no
   * question bank here and no answer key: the engine solves the skeleton,
   * and the learner's own numbers are checked against the matrix, which is
   * what lets a *correct multiple* be told apart from a mistake.
   */
  import { onMount } from "svelte";
  import type { Session } from "../session.svelte";
  import type { BalanceReport } from "../host/EngineHost";
  import { t, tSlug } from "../i18n.svelte";
  import {
    balancingSources,
    blankEquation,
    markBalance,
    markMessage,
    nextSource,
    writeEquation,
    type BalanceMark,
    type BalancingSource,
  } from "../balancing";

  let {
    session,
    entries,
    onclose,
  }: {
    session: Session;
    entries: { id: string; equation?: string | null }[];
    onclose: () => void;
  } = $props();

  const sources = $derived(balancingSources(entries, session.benchEquations));

  let asked = $state<string[]>([]);
  let source = $state<BalancingSource | null>(null);
  let report = $state<BalanceReport | null>(null);
  let refusal = $state<string | null>(null);
  let answers = $state<string[]>([]);
  let mark = $state<BalanceMark | null>(null);
  let revealed = $state(false);
  let loading = $state(false);
  /** Rounds marked correct, and how many were asked — an honest tally. */
  let right = $state(0);
  let done = $state(0);

  async function draw() {
    mark = null;
    revealed = false;
    refusal = null;
    report = null;
    loading = true;
    // An equation whose answer is all ones asks nothing — every blank is
    // already 1. It is still a legitimate question, so it is kept as a
    // fallback rather than dropped, but a candidate with real coefficients
    // is preferred while there is one within the budget.
    let trivial: { candidate: BalancingSource; reply: BalanceReport } | null = null;
    const take = (candidate: BalancingSource, reply: BalanceReport) => {
      source = candidate;
      report = reply;
      answers = reply.species.map(() => "");
    };
    try {
      // The engine is the only authority on whether a string is an
      // equation, so a refusal skips to the next candidate rather than
      // being shown as a broken question. Bounded, so a codex full of
      // prose cannot spin here.
      for (let attempt = 0; attempt < 12; attempt += 1) {
        const candidate = nextSource(sources, asked);
        if (!candidate) break;
        asked = [...asked, candidate.id];
        const reply = await session.balance(candidate.equation);
        if (!reply.ok) continue;
        if (reply.coefficients.some((coefficient) => coefficient !== 1)) {
          take(candidate, reply);
          return;
        }
        trivial ??= { candidate, reply };
      }
      if (trivial) {
        take(trivial.candidate, trivial.reply);
        return;
      }
      refusal = t("no equation on the bench or in the catalogue can be balanced yet");
    } finally {
      loading = false;
    }
  }

  function check() {
    if (!report) return;
    const numbers = answers.map((value) => {
      const text = value.trim();
      return /^\d+$/.test(text) ? Number(text) : Number.NaN;
    });
    const result = markBalance(report, numbers);
    if (mark === null && result.verdict !== "incomplete") {
      done += 1;
      if (result.verdict === "correct") right += 1;
    }
    mark = result;
  }

  const message = $derived(mark ? markMessage(mark) : null);
  const verdictWord = $derived(
    mark === null
      ? ""
      : mark.verdict === "correct"
        ? t("correct")
        : mark.verdict === "multiple"
          ? t("balanced, but not simplest")
          : mark.verdict === "unbalanced"
            ? t("not balanced")
            : t("incomplete"),
  );

  // Once, on open. Deliberately not an `$effect`: `draw` reads and then
  // writes the same state an effect would track, which is a loop waiting
  // to happen rather than a subscription worth having.
  onMount(() => void draw());
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <dialog open
    class="drill"
    aria-modal="true"
    aria-label={t("balancing practice")}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{t("Balance it")}</h2>
      <p class="sub">{t("generated from equations the engine can solve — never from a question bank")}</p>
      <button class="close" onclick={onclose} aria-label={t("close balancing practice")}>×</button>
    </header>

    <div class="body">
      {#if refusal}
        <p class="empty">{refusal}</p>
      {:else if report === null}
        <p class="empty">{t("drawing a question…")}</p>
      {:else}
        <p class="origin">
          {source?.origin === "bench" ? t("from this session's own bench") : t("from the experiment catalogue")}
          {#if source && source.origin === "codex"}<span class="slug">{tSlug(source.id)}</span>{/if}
        </p>
        <p class="skeleton">{blankEquation(report)}</p>

        <form
          class="answer"
          onsubmit={(e) => {
            e.preventDefault();
            check();
          }}
        >
          {#each report.species as species, index (species + index)}
            {#if index === report.reactants}
              <span class="arrow" aria-hidden="true">{report.reversible ? "⇌" : "→"}</span>
            {:else if index > 0}
              <span class="plus" aria-hidden="true">+</span>
            {/if}
            <label class="slot">
              <span class="sr">{t("coefficient for {species}", { species })}</span>
              <input
                bind:value={answers[index]}
                inputmode="numeric"
                autocomplete="off"
                spellcheck="false"
                maxlength="4"
                onfocus={(e) => e.currentTarget.select()}
              />
              <b>{species}</b>
            </label>
          {/each}
          <button class="go" type="submit">{t("check")}</button>
        </form>

        {#if mark && message}
          <p class="mark" data-verdict={mark.verdict} role="status">
            <strong>{verdictWord}</strong>
            {t(message.key, message.vars)}
          </p>
          {#if mark.verdict === "multiple"}
            <p class="hint">
              {t("divide every coefficient by {factor} and check again", {
                factor: String(mark.factor ?? 1),
              })}
            </p>
          {/if}
          {#if mark.family}
            <p class="hint">
              {t("this skeleton is under-determined: more than one independent reaction fits it, so several answers are right")}
            </p>
          {/if}
        {/if}

        {#if revealed}
          <p class="reveal">
            <span class="note-label">{t("the engine's answer")}</span>
            {writeEquation(report, report.coefficients)}
          </p>
        {/if}
      {/if}
    </div>

    <footer>
      <span class="tally">{t("{right} of {done} right", { right, done })}</span>
      {#if report}
        <button
          class="secondary"
          onclick={() => (revealed = true)}
          disabled={revealed}
        >{t("show the answer")}</button>
      {/if}
      <button class="secondary" onclick={() => void draw()} disabled={loading}>
        {t("next equation")}
      </button>
    </footer>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 10; display: grid; place-items: center; background: var(--scrim); }
  .drill { position: static; display: flex; flex-direction: column; overflow: hidden; width: min(40rem, calc(100vw - 2rem)); max-height: calc(100vh - 3rem); margin: 0; border: 1px solid var(--edge); border-radius: 10px; color: var(--ink); background: var(--panel); }
  header { display: flex; align-items: baseline; gap: .7rem; padding: .9rem 1.1rem .6rem; border-bottom: 1px solid var(--edge); }
  h2 { margin: 0; font-size: 1rem; }
  .sub { margin: 0; color: var(--dim); font-size: .78rem; }
  .close { margin-left: auto; border: 0; color: var(--dim); background: none; font-size: 1.3rem; cursor: pointer; }
  .body { display: grid; gap: .7rem; overflow-y: auto; padding: 1rem 1.1rem; }
  .empty { margin: 0; color: var(--dim); font-size: .85rem; }
  .origin { display: flex; flex-wrap: wrap; gap: .4rem; margin: 0; color: var(--dim); font-size: .68rem; font-weight: 750; letter-spacing: .06em; text-transform: uppercase; }
  .slug { color: var(--ink); text-transform: none; letter-spacing: 0; }
  .skeleton { overflow-x: auto; margin: 0; padding: .5rem .6rem; border: 1px dashed var(--edge-strong); border-radius: 9px; background: var(--surface); font-family: ui-monospace, SFMono-Regular, monospace; font-size: .9rem; white-space: nowrap; }
  .answer { display: flex; flex-wrap: wrap; align-items: center; gap: .35rem; margin: 0; }
  .arrow, .plus { color: var(--dim); font-size: 1rem; }
  .slot { display: flex; align-items: center; gap: .25rem; }
  .slot input { width: 2.6rem; padding: .3rem; border: 1px solid var(--edge-strong); border-radius: 7px; color: var(--ink); background: var(--surface); font: inherit; font-size: .85rem; text-align: center; }
  .slot b { font-family: ui-monospace, SFMono-Regular, monospace; font-size: .85rem; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .go { margin-left: .3rem; padding: .32rem .8rem; border: 0; border-radius: 8px; color: var(--on-accent); background: var(--accent); font: inherit; font-size: .82rem; font-weight: 700; cursor: pointer; }
  .mark { display: flex; flex-wrap: wrap; gap: .4rem; margin: 0; padding: .45rem .55rem; border-left: 4px solid var(--edge-strong); border-radius: 0 8px 8px 0; background: var(--surface); font-size: .82rem; line-height: 1.4; }
  .mark[data-verdict="correct"] { border-left-color: var(--success); background: color-mix(in srgb, var(--success) 9%, var(--surface)); }
  .mark[data-verdict="multiple"] { border-left-color: var(--warning); background: color-mix(in srgb, var(--warning) 9%, var(--surface)); }
  .mark[data-verdict="unbalanced"] { border-left-color: var(--danger); background: color-mix(in srgb, var(--danger) 9%, var(--surface)); }
  .hint { margin: 0; color: var(--dim); font-size: .74rem; line-height: 1.4; }
  .reveal { margin: 0; padding: .45rem .55rem; border: 1px solid var(--edge); border-radius: 8px; background: var(--surface); font-family: ui-monospace, SFMono-Regular, monospace; font-size: .82rem; }
  .note-label { display: block; margin-bottom: .2rem; color: var(--dim); font-family: system-ui, sans-serif; font-size: .6rem; font-weight: 750; letter-spacing: .07em; text-transform: uppercase; }
  footer { display: flex; align-items: center; gap: .5rem; padding: .7rem 1.1rem; border-top: 1px solid var(--edge); }
  .tally { margin-right: auto; color: var(--dim); font-size: .74rem; }
  .secondary { padding: .32rem .7rem; border: 1px solid var(--edge-strong); border-radius: 8px; color: var(--ink); background: var(--surface); font: inherit; font-size: .78rem; cursor: pointer; }
  .secondary:disabled { opacity: .5; cursor: default; }
</style>
