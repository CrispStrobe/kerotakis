<script lang="ts">
  /**
   * GUI-090 / GUI-091 — the newest result as a card rather than another
   * line in the feed.
   *
   * Every field is projected from the accepted command's own typed events
   * and the scenes it was computed between (`resultSummary.ts`); nothing
   * here asks the engine a second question. The feed underneath stays the
   * notebook and the transcript — this is what you look at while you work.
   *
   * `<details>` on purpose: with no JavaScript the disclosure still opens,
   * and the card is one element the register rule can drop entirely, which
   * is how it "degrades to the current feed" at lv3 (App.svelte decides).
   *
   * GUI-023's confidence encoding rides on `data-confidence`, so a
   * temperature that came out of a fitted excess-enthalpy model is dashed
   * rather than solid. The class badge is drawn ONLY where the engine
   * classified a reaction; where it did not, the operation is named without
   * a bordered badge, because a badge that says "mixing" is not a reaction
   * class and must not be dressed as one.
   */
  import { i18n, t } from "../i18n.svelte";
  import { engineText } from "../engineText";
  import type { ResultSummary } from "../resultSummary";
  import { resultCardFilename, resultCardSvg } from "../resultCardImage";
  import { RESULT_CARD_OPEN_KEY, loadResultCardOpen, saveResultCardOpen } from "../resultSummary";

  let {
    result,
    onclose,
    onexport,
    onprovenance,
  }: {
    result: ResultSummary;
    onclose: () => void;
    /**
     * GUI-052: open the provenance drawer on this result.
     *
     * Optional, and the icon is drawn only when it is supplied — the shell
     * passes it only where the engine actually recorded routing, so the
     * card never offers a door onto an empty room. The card knows nothing
     * about what is behind it; the drawer is the shell's to mount.
     */
    onprovenance?: () => void;
    /**
     * The card's exporter, handed UP to the shell.
     *
     * The two big "save SVG" / "save PNG" buttons used to sit in the card
     * body, on every result, for the one session in a hundred that hands a
     * card in. They are one icon in the header now — but the shell may want
     * to offer the same thing from its own overflow menu, and it cannot
     * reach a function inside a component. So the card passes its exporter
     * out while it is mounted, and passes `null` when it goes away, which
     * is what stops a shell menu item from ever calling a stale card.
     */
    onexport?: (run: ((format: "svg" | "png") => void) | null) => void;
  } = $props();

  /**
   * GUI-108 — the disclosure, remembered.
   *
   * The card is capped so it can no longer squeeze the log to nothing, but
   * a reader who wants the summary and not the detail should be able to
   * say so once rather than on every command. `<details>` is the control
   * they already have; this is what makes the answer stick.
   *
   * Read through `localStorage` guarded twice: the property itself throws
   * in a private window, and there is no `localStorage` at all when the
   * card is rendered on the server — which is how every component test in
   * this app renders it.
   */
  const viewStorage = (): Storage | null => {
    try {
      return typeof localStorage === "undefined" ? null : localStorage;
    } catch {
      return null;
    }
  };
  let expanded = $state(loadResultCardOpen(viewStorage(), RESULT_CARD_OPEN_KEY));
  $effect(() => {
    saveResultCardOpen(viewStorage(), RESULT_CARD_OPEN_KEY, expanded);
  });

  /** The export menu is closed until asked for, and closes behind itself. */
  let exporting = $state(false);
  // A new result is a new card. Leaving the menu open across one would hang
  // it over a card whose numbers it no longer belongs to.
  $effect(() => {
    void result;
    exporting = false;
  });
  // Escape closes it, because a small menu with no way out but a second
  // exact tap is a trap on a phone.
  $effect(() => {
    if (!exporting) return;
    const close = (event: KeyboardEvent) => {
      if (event.key === "Escape") exporting = false;
    };
    window.addEventListener("keydown", close);
    return () => window.removeEventListener("keydown", close);
  });

  const provenanceText = $derived(
    result.provenance ? engineText(result.provenance) : t("from this operation's computed events"),
  );

  function format(value: number): string {
    return new Intl.NumberFormat(i18n.locale === "de" ? "de-DE" : "en-GB", {
      maximumSignificantDigits: 4,
    }).format(value);
  }

  function cardSvg(): string {
    const localized = {
      ...result,
      kind: t(result.kind),
      quantities: result.quantities.map((quantity) => ({ ...quantity, label: t(quantity.label) })),
    };
    return resultCardSvg(localized, {
      title: t("latest computed result"),
      vessel: result.vessel === undefined ? undefined : `v${result.vessel + 1}`,
      equation: t("latest reaction equation"),
      observation: t("observation"),
      results: t("latest computed result"),
      // The image keeps the provenance sentence the card stopped printing.
      provenance: provenanceText,
      emptyEquation: "—",
      emptyObservation: "—",
    }, format);
  }

  function save(href: string, extension: "svg" | "png") {
    const anchor = document.createElement("a");
    anchor.href = href;
    anchor.download = resultCardFilename(result, extension);
    anchor.click();
  }

  /** One entry point, so the header menu and the shell run the same code. */
  function exportCard(format: "svg" | "png"): void {
    exporting = false;
    if (format === "svg") exportSvg();
    else exportPng();
  }

  $effect(() => {
    onexport?.(exportCard);
    return () => onexport?.(null);
  });

  function exportSvg() {
    const url = URL.createObjectURL(new Blob([cardSvg()], { type: "image/svg+xml;charset=utf-8" }));
    save(url, "svg");
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
  }

  function exportPng() {
    const url = URL.createObjectURL(new Blob([cardSvg()], { type: "image/svg+xml;charset=utf-8" }));
    const image = new Image();
    image.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = 1600;
      canvas.height = 1060;
      canvas.getContext("2d")?.drawImage(image, 0, 0, canvas.width, canvas.height);
      URL.revokeObjectURL(url);
      save(canvas.toDataURL("image/png"), "png");
    };
    image.onerror = () => URL.revokeObjectURL(url);
    image.src = url;
  }

  /** Kelvin is the engine's unit; the bench reads in degrees Celsius. */
  function celsius(kelvin: number): string {
    return format(Math.round((kelvin - 273.15) * 10) / 10);
  }
</script>

<details class="result-card" bind:open={expanded}>
  <!-- The provenance sentence was a visible line of its own under every
       card, saying the same thing every time. It is the tooltip on the
       claim it backs now, and the exported image still writes it out in
       full — the card stops repeating it, nobody loses it. -->
  <summary title={provenanceText}>
    <span class="result-mark" aria-hidden="true">✓</span>
    <span class="headline">
      <!-- GUI-120 — "Neuestes berechnetes Ergebnis" was a visible eyebrow
           over the name, on every card, saying what the card's own frame
           and tick already say. It was also the SECOND LINE of this
           column, and so half the header's height. It is the accessible
           name of the disclosure now: a screen reader still hears it
           first, and the pixels go to the one string that differs between
           two cards — what the bench just did. -->
      <span class="sr-only">{t("latest computed result")}</span>
      {#if result.reactionClass}
        <strong class="badge" data-confidence="computed">{t(result.reactionClass)}</strong>
      {:else}
        <strong class="operation">{t(result.kind)}</strong>
      {/if}
      <!-- Which vessel is the one fact in the old eyebrow that is not
           deducible from anything else on the card, so it stays visible. -->
      {#if result.vessel !== undefined}<small class="vessel">v{result.vessel + 1}</small>{/if}
    </span>
    {#if result.temperature}
      <b
        class:cooler={result.temperature.deltaK < 0}
        data-confidence={result.temperature.confidence}
      >
        ΔT {result.temperature.deltaK > 0 ? "+" : ""}{format(result.temperature.deltaK)} K
      </b>
    {/if}
    <!-- The card had no way out: it stayed until the next command replaced
         it, over the feed it was summarizing. The button is the bench's one
         close affordance, and it stops the click from also toggling the
         disclosure it sits inside. -->
    <span class="header-actions">
      {#if onprovenance}
        <button
          class="icon-provenance"
          type="button"
          aria-label={t("where this answer came from")}
          title={t("where this answer came from")}
          onclick={(event) => { event.preventDefault(); event.stopPropagation(); onprovenance?.(); }}
        >⌖</button>
      {/if}
      <button
        class="icon-export"
        type="button"
        aria-expanded={exporting}
        aria-haspopup="true"
        aria-controls="result-card-export"
        aria-label={t("export the result card")}
        title={t("export the result card")}
        onclick={(event) => { event.preventDefault(); event.stopPropagation(); exporting = !exporting; }}
      >⤓</button>
      <button
        class="icon-close"
        type="button"
        aria-label={t("close")}
        title={t("close")}
        onclick={(event) => { event.preventDefault(); event.stopPropagation(); onclose(); }}
      >×</button>
      {#if exporting}
        <span id="result-card-export" class="export-menu" role="group" aria-label={t("export the result card")}>
          <button type="button" onclick={(event) => { event.preventDefault(); event.stopPropagation(); exportCard("svg"); }}>{t("save SVG")}</button>
          <button type="button" onclick={(event) => { event.preventDefault(); event.stopPropagation(); exportCard("png"); }}>{t("save PNG")}</button>
        </span>
      {/if}
    </span>
  </summary>
  <div class="result-body">
    {#if result.equation}<p class="equation">{result.equation}</p>{/if}
    {#if result.reactants.length > 0}
      <ul class="reactants" aria-label={t("reactants")}>
        {#each result.reactants as reactant (reactant)}
          <li>{reactant}</li>
        {/each}
      </ul>
    {/if}
    {#if result.observation}<p class="observation">{result.observation}</p>{/if}
    {#if result.temperature}
      <p class="thermal" data-confidence={result.temperature.confidence}>
        <span class="from">{celsius(result.temperature.beforeK)} °C</span>
        <span aria-hidden="true">→</span>
        <span class="to">{celsius(result.temperature.afterK)} °C</span>
        <b class:cooler={result.temperature.deltaK < 0}>
          {result.temperature.deltaK > 0 ? "+" : ""}{format(result.temperature.deltaK)} K
        </b>
        <small>{t(result.temperature.confidence)}</small>
      </p>
    {/if}
    {#if result.quantities.length > 0}
      <dl>
        {#each result.quantities as quantity}
          <div><dt>{t(quantity.label)}</dt><dd>{format(quantity.value)} {quantity.unit}</dd></div>
        {/each}
      </dl>
    {/if}
    {#if result.note}<p class="note">{engineText(result.note)}</p>{/if}
    {#if result.boundary}<p class="boundary">{t(result.boundary)}</p>{/if}
    {#if result.safety}
      <p class="safety" role="alert">
        <span class="chip">{t(result.safety.severity || "hazard")}</span>
        {engineText(result.safety.hazard)}{result.safety.realWorld ? ` — ${engineText(result.safety.realWorld)}` : ""}
      </p>
    {/if}
  </div>
</details>

<style>
  /* GUI-108 — the card takes what is left of the journal pane, never what
     is under it.

     `flex: none` is what the owner met as "the card overlays the text in
     Laborbuch". It is not an overlay in the painting sense — the card has
     no `position` and no stacking context worth the name, which is why it
     never appears in `overlayStacking.test.ts` — but in a flex column it
     was the one item that would not give way. The feed beside it is
     `flex: 1; min-height: 0`, so an `<details open>` with an equation, a
     reactant list, an observation, a thermal row and a quantity table
     simply took the pane and squeezed the log it was summarising to
     nothing. A reader after a step wants BOTH, and got one.

     `Inspector.svelte` learned this exactly one component earlier and its
     comment still says so: a block that can be taller than its share of a
     narrow journal keeps its own scroll region. So does this one. The cap
     is `vh` rather than a percentage because `<details>` cannot be made a
     flex or grid container without risking the closed state, and the pane
     is very nearly viewport height in every layout the app offers. */
  /* Two bounds, and the reason for each was measured rather than chosen.

     `.result-body` used to carry `max-height: 30vh` — on a 1000 px window
     that is 300 px of a 371 px journal pane, so the card was allowed to be
     nearly the whole of the thing it sits above, which is the owner's
     complaint in one declaration. `flex-shrink` never rescues it: the log
     is `flex: 1`, that is `1 1 0%`, so its basis is zero and it takes only
     what the card leaves.

     `max-height: 50%` on the card moved the log from 51 px to 98 px, so it
     DOES resolve — my first reading of that was wrong, and adding
     `flex-basis: 50%` on top changed nothing because the ceiling was
     already doing the work. `max-height: 8rem` on the body is the second
     bound, and it matters because the card's summary is about 119 px on
     its own: a rule on the body is the only part of the card a rule on
     the body can reach.

     What neither bound can do is make the log half the pane. The check now
     reports the decomposition and it reads, at 371 px of pane:

         journal chrome   88 px   fixed
         card summary    104 px   fixed
         card body        44 px   at this cap
         log             135 px

     Half for the log would leave the card 124 px — barely its own header
     — so the check asks for a third. That trade was deliberate and it was
     a squeeze: the body scrolled in about two lines, and the room was not
     really the body's to give. **104 px of summary is where this pane
     went**, and a more compact summary was the change that would buy both
     the card and the log something. That was GUI-108's sequel.

     GUI-120 is it, and the first thing it found is that the 104 px was
     not a wrapped header. It is `min-height: 3.25rem` read at a 32 px
     root: the check above runs inside the 200% TEXT ZOOM the audit before
     it injects and never removes, so every number in that table is a
     zoomed number. The header is sized in px now and the eyebrow line is
     gone, which takes it to 44 px — the audited touch floor, and the
     summary is a real press target. At the same 371 px of pane the four
     numbers read:

         journal chrome   88 px   fixed
         card summary     44 px   was 104
         card body        88 px   was 44
         log             149 px   was 135

     so the cap comes down from 40% to 36% and the log and the body both
     gain. What the zoomed measurement had been hiding is in the header
     rule below. */
  .result-card { display: flex; flex-direction: column; max-height: 36%; flex: 0 1 auto; min-height: 0; margin: .6rem .65rem 0; border: 1px solid color-mix(in srgb, var(--success) 45%, var(--edge)); border-radius: 14px; color: var(--ink); background: color-mix(in srgb, var(--success) 6%, var(--surface-raised)); overflow: hidden; }
  /* GUI-120 — the header, measured rather than guessed.
     ...............................................................
     GUI-108 left the sequel named at this rule: "104 px of summary is
     where this pane went". The 104 px is real and it is not a wrap.
     `min-height: 3.25rem` is 104 px because the check that reported it
     runs under the 200% TEXT ZOOM the audit above it injects and never
     removes (`#ux-text-zoom` in `tools/test-ux-quality.mjs`), and 3.25rem
     of a 32 px root is exactly 104. Every number in GUI-108's table is a
     zoomed number. Measured in a replica of the pane, nothing in this
     header wraps at ANY width from 160 px to 400 px — the row stays one
     row, because four explicit grid tracks cannot wrap.

     What went wrong instead is worse than wrapping, and it is what the
     four tracks did to the second one. The journal is 288 px wide (the
     `min(18rem, 23vw)` rule near the end of `App.svelte`, not the
     `min(24rem, 34vw)` earlier in it), so the summary has 263 px. The
     three RIGID tracks — a 32 px tick, an 84 px ΔT and 90 px of icons,
     plus 26 px of `.55rem` gaps — take 232 of them. `minmax(0, 1fr)`
     then gives the name 10 px. Under the audit's text zoom it gives the
     name ZERO: at 200% the ΔT badge alone is 165 px, and the operation
     name and the reaction class — the card's whole answer to "what just
     happened" — are not painted at all.

     So three things change, and each is a rule that used to grow with
     the type and now does not:

     1. The chrome is in px. `min-height`, the gaps and the padding are
        the card's furniture, not its prose: at 200% they doubled and
        took the pane with them. 44 px is the audited touch floor and the
        summary is a real press target (it toggles the disclosure), so
        that is the floor it keeps — at every zoom.
     2. The eyebrow is gone from the flow (see the markup), so this
        column is one line instead of two.
     3. ΔT moves to a second grid row and is hidden while the card is
        OPEN, because the body two rules below already states it exactly,
        with the before and after temperatures the header cannot fit. A
        closed card has no body, so there it is shown — and a closed card
        is only its header, so the second row costs the log nothing. */
  summary {
    flex: none;
    min-height: 44px;
    display: grid;
    grid-template-columns: 20px minmax(0, 1fr) auto;
    grid-template-areas: "mark headline actions" "delta delta delta";
    align-items: center;
    align-content: center;
    column-gap: 6px;
    row-gap: 0;
    padding: 6px 8px;
    cursor: pointer;
    list-style: none;
  }
  /* Off the screen, not out of the document: the disclosure keeps the name
     it always had for anyone who cannot see the frame it sits in. */
  .sr-only { position: absolute; width: 1px; height: 1px; margin: -1px; padding: 0; border: 0; clip-path: inset(50%); overflow: hidden; white-space: nowrap; }
  .headline { grid-area: headline; min-width: 0; display: flex; align-items: baseline; gap: .3rem; }
  .vessel { flex: none; color: var(--dim); font-size: .68rem; font-weight: 750; letter-spacing: .04em; }
  /* Anchored so the menu hangs off the icon rather than widening the card. */
  .header-actions { grid-area: actions; position: relative; display: flex; align-items: center; gap: 3px; }
  .icon-export {
    width: 28px;
    height: 28px;
    flex: none;
    display: grid;
    place-items: center;
    padding: 0;
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--dim);
    background: var(--surface);
    font: inherit;
    font-size: .95rem;
    font-weight: 800;
    line-height: 1;
    cursor: pointer;
  }
  /* Same 28px chrome as the export icon beside it: two icons in one header
     that do not match read as two different kinds of control. */
  .icon-provenance {
    width: 28px;
    height: 28px;
    flex: none;
    display: grid;
    place-items: center;
    padding: 0;
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--dim);
    background: var(--surface);
    font: inherit;
    font-size: .95rem;
    font-weight: 800;
    line-height: 1;
    cursor: pointer;
  }
  .icon-provenance:hover { color: var(--primary); border-color: var(--primary); }
  .icon-export:hover,
  .icon-export[aria-expanded="true"] { color: var(--primary); border-color: var(--primary); }
  .export-menu {
    position: absolute;
    top: calc(100% + .25rem);
    right: 0;
    z-index: 6;
    display: flex;
    flex-direction: column;
    gap: .15rem;
    padding: .2rem;
    border: 1px solid var(--edge);
    border-radius: 9px;
    background: var(--surface-raised);
    box-shadow: 0 8px 22px var(--shadow);
  }
  .export-menu button { min-height: 32px; padding: .25rem .55rem; border: 0; border-radius: 7px; color: var(--ink); background: transparent; font: inherit; font-size: .68rem; font-weight: 700; white-space: nowrap; cursor: pointer; }
  .export-menu button:hover { background: color-mix(in srgb, var(--primary) 12%, transparent); }
  summary::-webkit-details-marker { display: none; }
  /* 20px rather than 30px, and in px so it does not double: it is a tick
     on a green card, and every pixel it gives back is a pixel of the name
     beside it. */
  .result-mark { grid-area: mark; width: 20px; height: 20px; display: grid; place-items: center; border-radius: 7px; color: var(--on-accent); background: var(--success); font-size: .7rem; font-weight: 900; }
  /* GUI-023 supplies the border STYLE from data-confidence; the width and
     colour are the card's, so an unclassified tag is visibly a different
     kind of claim without a second colour vocabulary. */
  .badge { min-width: 0; max-width: 100%; overflow: hidden; padding: .05rem .3rem; border: 1px solid color-mix(in srgb, var(--ink) 30%, transparent); border-radius: 7px; font-size: .9rem; text-overflow: ellipsis; white-space: nowrap; }
  .operation { min-width: 0; max-width: 100%; overflow: hidden; font-size: .9rem; text-overflow: ellipsis; white-space: nowrap; }
  summary b { grid-area: delta; justify-self: start; margin-top: 3px; padding: .2rem .38rem; border: 1px solid transparent; border-radius: 999px; color: var(--warning); background: color-mix(in srgb, var(--warning) 10%, var(--surface)); font-size: .72rem; white-space: nowrap; }
  /* Open, the thermal row in the body says this and more; a header that
     repeated it was spending a third of its width on a duplicate. */
  .result-card[open] summary b { display: none; }
  summary b[data-confidence] { border-color: color-mix(in srgb, var(--warning) 55%, transparent); }
  summary b.cooler { color: var(--cool); background: color-mix(in srgb, var(--cool) 10%, var(--surface)); }
  summary b.cooler[data-confidence] { border-color: color-mix(in srgb, var(--cool) 55%, transparent); }
  /* The scroll region. `overscroll-behavior` so that reaching the bottom of
     a long result does not then start scrolling the log underneath — the
     journal is the thing the reader is trying to keep. */
  /* GUI-120 — the cap was clipping the scroll region rather than sizing it.
     Chrome wraps everything after the `<summary>` in `::details-content`,
     so `.result-body` is a grandchild of the card and its `flex: 1 1 auto;
     min-height: 0` was being read by a block box that is not a flex
     container. The card's flex line therefore never shrank the body: at
     GUI-108's cap the card was 146 px of content box holding 180 px, and
     the bottom 34 px of the body — the end of its own scrollbar — was
     clipped and unreachable. Giving the pseudo-element the flex rules the
     body was written for puts the body back in the line it was meant to
     be in. Browsers without `::details-content` never had the bug: there
     the body IS the flex child, and both rules say the same thing. */
  .result-card::details-content { flex: 1 1 auto; min-height: 0; display: flex; flex-direction: column; }
  /* The left indent aligned the body under the headline, and in `rem` it
     was 91 px of a 285 px card at 200% text zoom — a third of the body's
     width, spent on an indent. In px it still aligns (8 padding + 20 mark
     + 6 gap) and it stops growing. */
  .result-body { flex: 1 1 auto; min-height: 0; max-height: 8rem; display: grid; gap: .45rem; padding: 0 .7rem .65rem 34px; overflow-y: auto; overscroll-behavior: contain; border-top: 1px solid color-mix(in srgb, var(--success) 22%, transparent); }
  p { margin: .55rem 0 0; }
  .equation { overflow-x: auto; font-family: ui-monospace, SFMono-Regular, monospace; font-size: .78rem; font-weight: 700; white-space: nowrap; }
  .reactants { display: flex; flex-wrap: wrap; gap: .3rem; margin: 0; padding: 0; list-style: none; }
  .reactants li { padding: .15rem .4rem; border: 1px solid var(--edge); border-radius: 999px; background: var(--surface); font-family: ui-monospace, SFMono-Regular, monospace; font-size: .7rem; }
  .observation { color: var(--dim); font-size: .78rem; line-height: 1.35; }
  .thermal { display: flex; flex-wrap: wrap; align-items: center; gap: .35rem; margin: 0; padding: .28rem .42rem; border: 1px solid var(--edge-strong); border-radius: 9px; background: var(--surface); font-size: .78rem; }
  .thermal .from { color: var(--dim); }
  .thermal .to { font-weight: 800; }
  .thermal b { padding: .1rem .34rem; border-radius: 999px; color: var(--warning); background: color-mix(in srgb, var(--warning) 12%, transparent); font-size: .72rem; }
  .thermal b.cooler { color: var(--cool); background: color-mix(in srgb, var(--cool) 12%, transparent); }
  .thermal small { color: var(--dim); font-size: .6rem; text-transform: lowercase; }
  .note { margin: 0; color: var(--dim); font-size: .7rem; line-height: 1.35; }
  .boundary { margin: 0; padding: .3rem .4rem; border-left: 3px solid var(--warning); color: var(--dim); background: color-mix(in srgb, var(--warning) 7%, transparent); font-size: .67rem; line-height: 1.3; }
  .safety { margin: 0; padding: .32rem .42rem; border-left: 3px solid var(--danger); color: var(--ink); background: color-mix(in srgb, var(--danger) 10%, transparent); font-size: .7rem; line-height: 1.35; }
  .safety .chip { margin-right: .3rem; padding: .05rem .3rem; border-radius: 5px; color: var(--on-accent); background: var(--danger); font-size: .58rem; font-weight: 800; text-transform: uppercase; }
  dl { display: flex; flex-wrap: wrap; gap: .35rem; margin: 0; }
  dl div { padding: .25rem .4rem; border: 1px solid var(--edge); border-radius: 8px; background: var(--surface); }
  dt { color: var(--dim); font-size: .58rem; font-weight: 750; text-transform: uppercase; }
  dd { margin: 0; font-size: .7rem; font-weight: 750; }
</style>
