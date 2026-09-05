<script lang="ts">
  import { t } from "../i18n.svelte";
  let {
    vessel,
    options,
    busy,
    onrun,
    onclose,
  }: {
    vessel: number;
    options: string[];
    busy: boolean;
    onrun: (line: string) => void;
    onclose: () => void;
  } = $props();

  let chosen = $state("");
  const line = $derived(chosen ? `react v${vessel + 1} ${chosen}` : null);

  /**
   * What each curated reaction actually does, and its equation.
   *
   * The engine's grammar endpoint sends NAMES only — `curated::ORG_REACTIONS`
   * keeps the equation, the boundary and the provenance on the Rust side and
   * the wire drops all three — so the picker offered five bare identifiers and
   * nothing to choose between them. Authored here, keyed by the engine's own
   * name, until the grammar payload carries them. The equations are copied
   * from that table verbatim and are formulas, not prose: they are the same
   * in every language and are deliberately NOT translated. The blurbs are,
   * and reach the dictionary through `t()` as variables.
   */
  const DETAILS: Record<string, { blurb: string; equation: string }> = {
    esterification: {
      blurb: "acid and alcohol reach an ester–water equilibrium; the requested extent is driven to completion",
      equation: "CH3COOH + C2H5OH ⇌ CH3COOC2H5 + H2O",
    },
    saponification: {
      blurb: "alkali splits an ester into a carboxylate salt and an alcohol",
      equation: "CH3COOC2H5 + NaOH → NaOAc + C2H5OH",
    },
    "alcohol-oxidation": {
      blurb: "ethanol and oxygen give acetic acid — the overall equation of vinegar, not its pathway",
      equation: "C2H5OH + O2 → CH3COOH + H2O",
    },
    respiration: {
      blurb: "the overall equation of aerobic respiration; no cell, no enzyme and no reaction heat is modelled",
      equation: "C6H12O6 + 6 O2 → 6 CO2 + 6 H2O",
    },
    haloalkane: {
      blurb: "picks the SN1, SN2, E1 or E2 mechanism from substrate, nucleophile and temperature",
      equation: "",
    },
  };
  const detail = $derived(DETAILS[chosen] ?? null);
  // Through a function, not a repeated `DETAILS[name] ? DETAILS[name].blurb`:
  // under `noUncheckedIndexedAccess` the second index access is a fresh
  // `| undefined` and svelte-check is right to say so.
  const blurbOf = (name: string): string => {
    const known = DETAILS[name];
    return known ? t(known.blurb) : "";
  };
</script>

<section class="react" aria-label={t("curated reaction on v{vessel}", { vessel: vessel + 1 })}>
  <button class="icon-close corner" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
  <strong>{t("curated reaction")} · v{vessel + 1}</strong>
  <span class="hint">{t("verified family templates the engine can run")}</span>
  <div class="row">
    <select bind:value={chosen}>
      <option value="">{t("choose…")}</option>
      {#each options as name (name)}<option value={name} title={blurbOf(name)}>{t(name)}</option>{/each}
    </select>
    <button class="run" disabled={busy || line === null} onclick={() => line && onrun(line)}>
      {t("run")}
    </button>
  </div>
  {#if detail}
    <p class="detail">{t(detail.blurb)}</p>
    {#if detail.equation}<p class="equation"><small>{t("equation")}</small> <code>{detail.equation}</code></p>{/if}
  {/if}
  {#if line}<code>{line}</code>{/if}
</section>

<style>
  .react {
    position: relative;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
    font-size: 0.82rem;
  }
  .hint {
    color: var(--dim);
    margin-left: 0.6rem;
    font-size: 0.76rem;
  }
  .row {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.35rem;
    align-items: center;
  }
  select {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    padding: 0.25rem 0.4rem;
    min-height: 34px;
  }
  .run {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    padding: 0.3rem 0.8rem;
    cursor: pointer;
    min-height: 36px;
  }
  code {
    display: block;
    margin-top: 0.3rem;
    color: var(--dim);
    font-size: 0.72rem;
  }
  .detail {
    margin: 0.35rem 0 0;
    color: var(--dim);
    font-size: 0.76rem;
    line-height: 1.35;
  }
  .equation {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    margin: 0.15rem 0 0;
  }
  .equation small {
    color: var(--dim);
    font-size: 0.62rem;
    font-weight: 750;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .equation code {
    margin-top: 0;
    color: var(--ink);
    font-size: 0.76rem;
  }
</style>
