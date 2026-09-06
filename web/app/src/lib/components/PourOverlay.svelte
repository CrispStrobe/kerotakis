<script lang="ts">
  /**
   * The pour chooser, on the vessel it pours out of.
   *
   * This was a banner between the top bar and the stage: "dekantieren —
   * gieße 25% 50% 75% 100% · von v1 — jetzt das Ziel antippen". It named
   * two vessels that are drawn a few centimetres below it, it pushed the
   * stage down by its own height at the moment the stage mattered most,
   * and on a phone it wrapped onto three lines.
   *
   * Here the chooser is anchored to the source vessel — the fractions in
   * one row beside it, the prompt under them — and the eligible targets
   * are already breathing on the stage (`Vessel.svelte`'s
   * `.transfer-target`). Nothing says "von v1" because the chooser is ON
   * v1.
   *
   * Before a source is tapped there is nothing to anchor to, so the
   * chooser sits at the head of the stage as one compact chip rather than
   * a full-width bar: the same information, taking a corner instead of a
   * row.
   *
   * `role="status"` and an `aria-label` naming the verb, because a
   * learner using a screen reader gets the prompt read to them; the
   * breathing outline on the target vessels is not a message.
   */
  import { t } from "../i18n.svelte";
  import { POUR_FRACTIONS, anchorSide, choosesFraction, clampAnchor, transferPrompt, type TransferDraft } from "../pour";

  let {
    draft,
    eligible,
    x = null,
    y = null,
    onfraction,
    oncancel,
  }: {
    draft: TransferDraft;
    /** How many vessels may be tapped next. */
    eligible: number;
    /** Where the source vessel stands, 0–1 across the work surface. */
    x?: number | null;
    y?: number | null;
    onfraction: (fraction: number) => void;
    oncancel: () => void;
  } = $props();

  const anchored = $derived(draft.from !== null && x !== null && y !== null);
  const side = $derived(anchored ? anchorSide(y!) : "above");
  const prompt = $derived(transferPrompt(draft, eligible));
  const percent = (fraction: number) => `${Math.round(fraction * 100)}%`;
</script>

<div
  class="pour-overlay"
  class:anchored
  class:below={anchored && side === "below"}
  role="status"
  aria-label={t("{verb} — {prompt}", { verb: t(draft.verb), prompt: t(prompt) })}
  style={anchored ? `--pour-x:${clampAnchor(x!) * 100}%;--pour-y:${y! * 100}%` : undefined}
>
  <div class="row">
    <strong>{t(draft.verb)}</strong>
    {#if choosesFraction(draft.verb) && draft.from !== null}
      <div class="fractions" role="group" aria-label={t("how much to pour")}>
        {#each POUR_FRACTIONS as fraction (fraction)}
          <button
            type="button"
            class:on={draft.fraction === fraction}
            aria-pressed={draft.fraction === fraction}
            onclick={() => onfraction(fraction)}
          >{percent(fraction)}</button>
        {/each}
      </div>
    {/if}
    <button class="cancel" type="button" aria-label={t("cancel")} title={t("cancel")} onclick={oncancel}>×</button>
  </div>
  <small class:waiting={eligible === 0}>{t(prompt)}</small>
</div>

<style>
  /* Unanchored: a chip in the stage's own corner, not a band across it.
     Anchored: centred over the source vessel and lifted clear of the
     glass, which is 140 units tall in its own viewBox and about 9 rem on
     the bench. */
  .pour-overlay {
    position: absolute;
    z-index: 6;
    top: 0.5rem;
    left: 0.5rem;
    max-width: min(14rem, calc(100% - 1rem));
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 45%, var(--edge));
    border-radius: 12px;
    background: color-mix(in srgb, var(--surface) 93%, var(--instrument));
    box-shadow: 0 8px 22px var(--shadow);
    font-size: 0.78rem;
  }
  /* The anchor arrives as custom properties rather than as inline
     `left`/`top`, so a rule below can still move the overlay — an inline
     declaration would outrank one. */
  .pour-overlay.anchored {
    top: var(--pour-y);
    left: var(--pour-x);
    translate: -50% -50%;
    /* Clear of the glass, which stands about 9 rem tall on the bench. */
    margin-top: -8rem;
  }
  .pour-overlay.anchored.below { margin-top: 8rem; }
  /* Wraps rather than widens: the chooser is 14 rem at most, and a verb
     plus four chips plus a close is more than that in German. */
  .row { display: flex; flex-wrap: wrap; align-items: center; gap: 0.3rem; }
  .row strong { flex: none; font-size: 0.72rem; }
  .fractions { display: flex; gap: 0.15rem; }
  /* 30 px rather than 44: these sit ON the bench beside the glassware, and
     four 44 px chips are wider than the vessel they belong to. The row is
     one of two ways to pour — the vessel dock's "pour" is the other, and
     it keeps its 48 px targets. */
  .fractions button {
    min-width: 2.5rem;
    min-height: 30px;
    padding: 0.1rem 0.3rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    background: var(--surface-raised);
    font: inherit;
    font-size: 0.7rem;
    cursor: pointer;
  }
  .fractions button.on {
    border-color: var(--instrument);
    color: var(--on-accent);
    background: var(--instrument);
    font-weight: 800;
  }
  .cancel {
    flex: none;
    width: 1.7rem;
    min-height: 30px;
    margin-left: auto;
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--dim);
    background: transparent;
    font: inherit;
    font-size: 0.9rem;
    line-height: 1;
    cursor: pointer;
  }
  small { color: var(--dim); font-size: 0.63rem; line-height: 1.3; }
  small.waiting { color: var(--warning); font-weight: 700; }
  @media (max-width: 30rem) {
    /* The bench does not narrow with the phone — the work surface keeps a
       42 rem minimum and scrolls — so the chooser stays with its vessel
       and only its chips get smaller. */
    .fractions button { min-width: 2.1rem; font-size: 0.64rem; }
  }
</style>
