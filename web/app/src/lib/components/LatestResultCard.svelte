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

  let { result, onclose }: { result: ResultSummary; onclose: () => void } = $props();

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
      provenance: result.provenance
        ? engineText(result.provenance)
        : t("from this operation's computed events"),
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

<details class="result-card" open>
  <summary>
    <span class="result-mark" aria-hidden="true">✓</span>
    <span>
      <small>{t("latest computed result")}{result.vessel === undefined ? "" : ` · v${result.vessel + 1}`}</small>
      {#if result.reactionClass}
        <strong class="badge" data-confidence="computed">{t(result.reactionClass)}</strong>
      {:else}
        <strong class="operation">{t(result.kind)}</strong>
      {/if}
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
    <button
      class="icon-close"
      type="button"
      aria-label={t("close")}
      title={t("close")}
      onclick={(event) => { event.preventDefault(); event.stopPropagation(); onclose(); }}
    >×</button>
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
    <small class="provenance">{result.provenance ? engineText(result.provenance) : t("from this operation's computed events")}</small>
    <div class="share-actions" aria-label={t("latest computed result")}>
      <button type="button" onclick={exportSvg}>{t("save SVG")}</button>
      <button type="button" onclick={exportPng}>{t("save PNG")}</button>
    </div>
  </div>
</details>

<style>
  .result-card { flex: none; margin: .6rem .65rem 0; border: 1px solid color-mix(in srgb, var(--success) 45%, var(--edge)); border-radius: 14px; color: var(--ink); background: color-mix(in srgb, var(--success) 6%, var(--surface-raised)); overflow: hidden; }
  summary { min-height: 3.25rem; display: grid; grid-template-columns: 32px minmax(0, 1fr) auto auto; align-items: center; gap: .55rem; padding: .55rem .65rem; cursor: pointer; list-style: none; }
  summary::-webkit-details-marker { display: none; }
  .result-mark { width: 30px; height: 30px; display: grid; place-items: center; border-radius: 10px; color: var(--on-accent); background: var(--success); font-weight: 900; }
  summary span:nth-child(2) { min-width: 0; display: flex; flex-direction: column; align-items: flex-start; }
  summary small { overflow: hidden; color: var(--dim); font-size: .65rem; font-weight: 750; letter-spacing: .08em; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
  /* GUI-023 supplies the border STYLE from data-confidence; the width and
     colour are the card's, so an unclassified tag is visibly a different
     kind of claim without a second colour vocabulary. */
  .badge { max-width: 100%; overflow: hidden; padding: .05rem .3rem; border: 1px solid color-mix(in srgb, var(--ink) 30%, transparent); border-radius: 7px; font-size: .9rem; text-overflow: ellipsis; white-space: nowrap; }
  .operation { max-width: 100%; overflow: hidden; font-size: .9rem; text-overflow: ellipsis; white-space: nowrap; }
  summary b { padding: .2rem .38rem; border: 1px solid transparent; border-radius: 999px; color: var(--warning); background: color-mix(in srgb, var(--warning) 10%, var(--surface)); font-size: .72rem; white-space: nowrap; }
  summary b[data-confidence] { border-color: color-mix(in srgb, var(--warning) 55%, transparent); }
  summary b.cooler { color: var(--cool); background: color-mix(in srgb, var(--cool) 10%, var(--surface)); }
  summary b.cooler[data-confidence] { border-color: color-mix(in srgb, var(--cool) 55%, transparent); }
  .result-body { display: grid; gap: .45rem; padding: 0 .7rem .65rem 2.85rem; border-top: 1px solid color-mix(in srgb, var(--success) 22%, transparent); }
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
  .provenance { color: var(--dim); font-size: .6rem; }
  .share-actions { display: flex; gap: .35rem; }
  .share-actions button { padding: .25rem .5rem; border: 1px solid var(--edge); border-radius: 7px; color: var(--ink); background: var(--surface); font: inherit; font-size: .65rem; cursor: pointer; }
</style>
