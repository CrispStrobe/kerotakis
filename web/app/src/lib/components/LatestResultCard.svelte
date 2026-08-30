<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import type { ResultSummary } from "../resultSummary";
  import { resultCardFilename, resultCardSvg } from "../resultCardImage";
  import { engineText } from "../engineText";

  let { result }: { result: ResultSummary } = $props();

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
</script>

<details class="result-card" open>
  <summary>
    <span class="result-mark" aria-hidden="true">✓</span>
    <span>
      <small>{t("latest computed result")}{result.vessel === undefined ? "" : ` · v${result.vessel + 1}`}</small>
      <strong>{t(result.kind)}</strong>
    </span>
    {#if result.temperatureDeltaK !== undefined}
      <b class:cooler={result.temperatureDeltaK < 0}>
        ΔT {result.temperatureDeltaK > 0 ? "+" : ""}{format(result.temperatureDeltaK)} K
      </b>
    {/if}
  </summary>
  <div class="result-body">
    {#if result.equation}<p class="equation">{result.equation}</p>{/if}
    {#if result.observation}<p class="observation">{result.observation}</p>{/if}
    {#if result.quantities.length > 0}
      <dl>
        {#each result.quantities as quantity}
          <div><dt>{t(quantity.label)}</dt><dd>{format(quantity.value)} {quantity.unit}</dd></div>
        {/each}
      </dl>
    {/if}
    {#if result.boundary}<p class="boundary">{t(result.boundary)}</p>{/if}
    <small class="provenance">{result.provenance ? engineText(result.provenance) : t("from this operation's computed events")}</small>
    <div class="share-actions" aria-label={t("latest computed result")}>
      <button type="button" onclick={exportSvg}>{t("save SVG")}</button>
      <button type="button" onclick={exportPng}>{t("save PNG")}</button>
    </div>
  </div>
</details>

<style>
  .result-card { flex: none; margin: .6rem .65rem 0; border: 1px solid color-mix(in srgb, var(--success) 45%, var(--edge)); border-radius: 14px; color: var(--ink); background: color-mix(in srgb, var(--success) 6%, var(--surface-raised)); overflow: hidden; }
  summary { min-height: 3.25rem; display: grid; grid-template-columns: 32px minmax(0, 1fr) auto; align-items: center; gap: .55rem; padding: .55rem .65rem; cursor: pointer; list-style: none; }
  summary::-webkit-details-marker { display: none; }
  .result-mark { width: 30px; height: 30px; display: grid; place-items: center; border-radius: 10px; color: var(--on-accent); background: var(--success); font-weight: 900; }
  summary span:nth-child(2) { min-width: 0; display: flex; flex-direction: column; }
  summary small { overflow: hidden; color: var(--dim); font-size: .65rem; font-weight: 750; letter-spacing: .08em; text-overflow: ellipsis; text-transform: uppercase; white-space: nowrap; }
  summary strong { overflow: hidden; font-size: .9rem; text-overflow: ellipsis; white-space: nowrap; }
  summary b { padding: .2rem .38rem; border-radius: 999px; color: var(--warning); background: color-mix(in srgb, var(--warning) 10%, var(--surface)); font-size: .72rem; white-space: nowrap; }
  summary b.cooler { color: var(--cool); background: color-mix(in srgb, var(--cool) 10%, var(--surface)); }
  .result-body { display: grid; gap: .45rem; padding: 0 .7rem .65rem 2.85rem; border-top: 1px solid color-mix(in srgb, var(--success) 22%, transparent); }
  p { margin: .55rem 0 0; }
  .equation { overflow-x: auto; font-family: ui-monospace, SFMono-Regular, monospace; font-size: .78rem; font-weight: 700; white-space: nowrap; }
  .observation { color: var(--dim); font-size: .78rem; line-height: 1.35; }
  .boundary { margin: 0; padding: .3rem .4rem; border-left: 3px solid var(--warning); color: var(--dim); background: color-mix(in srgb, var(--warning) 7%, transparent); font-size: .67rem; line-height: 1.3; }
  dl { display: flex; flex-wrap: wrap; gap: .35rem; margin: 0; }
  dl div { padding: .25rem .4rem; border: 1px solid var(--edge); border-radius: 8px; background: var(--surface); }
  dt { color: var(--dim); font-size: .58rem; font-weight: 750; text-transform: uppercase; }
  dd { margin: 0; font-size: .7rem; font-weight: 750; }
  .provenance { color: var(--dim); font-size: .6rem; }
  .share-actions { display: flex; gap: .35rem; }
  .share-actions button { padding: .25rem .5rem; border: 1px solid var(--edge); border-radius: 7px; color: var(--ink); background: var(--surface); font: inherit; font-size: .65rem; cursor: pointer; }
</style>
