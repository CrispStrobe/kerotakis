<script lang="ts">
  import type { ParticleCensus } from "../host/EngineHost";
  import { t } from "../i18n.svelte";

  let { census }: { census: ParticleCensus } = $props();

  // Shape carries the meaning, exactly as the terminal glyphs do — the
  // picture must survive being seen without colour.
  const KINDS: Record<string, { describe: string }> = {
    cation: { describe: "positive ion" },
    anion: { describe: "negative ion" },
    neutralsolute: { describe: "uncharged, dissolved" },
    solvent: { describe: "solvent" },
    solid: { describe: "solid" },
    gas: { describe: "gas" },
  };
  const norm = (kind: string) => kind.toLowerCase().replace(/_/g, "");
  const describe = (kind: string) => t(KINDS[norm(kind)]?.describe ?? kind);
  const populationSummary = $derived(
    census.populations
      .map((population) => t("{drawn} of {label}", {
        drawn: population.drawn,
        label: t(population.label),
      }))
      .join(", "),
  );
</script>

<section
  class="particles"
  aria-label={t("particle view: {populations}", { populations: populationSummary })}
>
  {#each census.populations as pop (pop.label)}
    <div class="population">
      <span class="label">{t(pop.label)} <em>({describe(pop.kind)})</em></span>
      <span class="dots" data-kind={norm(pop.kind)}>
        {#each Array.from({ length: Math.min(pop.drawn, 60) }, (_, i) => i) as i (i)}
          <svg viewBox="0 0 10 10" class="dot" aria-hidden="true">
            {#if norm(pop.kind) === "cation"}
              <circle cx="5" cy="5" r="4" class="fill" />
            {:else if norm(pop.kind) === "anion"}
              <circle cx="5" cy="5" r="4" class="hollow" />
            {:else if norm(pop.kind) === "neutralsolute"}
              <circle cx="5" cy="5" r="4" class="hollow" />
              <circle cx="5" cy="5" r="1.8" class="fill" />
            {:else if norm(pop.kind) === "solvent"}
              <circle cx="5" cy="5" r="1.6" class="fill dim" />
            {:else if norm(pop.kind) === "solid"}
              <rect x="1.5" y="1.5" width="7" height="7" class="fill" />
            {:else}
              <circle cx="5" cy="5" r="2.6" class="hollow dim" />
            {/if}
          </svg>
        {/each}
      </span>
    </div>
  {/each}
  {#if census.too_rare.length > 0}
    <p class="note">
      {t("also present, too dilute to draw at this scale:")}
      {census.too_rare.map(([name]) => t(name)).join(", ")}
    </p>
  {/if}
  <p class="note">
    {census.source === "speciation"
      ? t("ratios from solved speciation")
      : t("ratios from the inventory — ion pairs and complexes not resolved")}
    · {t("one shape ≈ {amount} mol", { amount: census.per_glyph.toExponential(1) })}
  </p>
</section>

<style>
  .particles {
    padding: 0.6rem 1rem;
    border-top: 1px solid var(--edge);
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    overflow-y: auto;
  }
  .population {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .label {
    font-size: 0.8rem;
    min-width: 8rem;
  }
  .label em {
    color: var(--dim);
    font-style: normal;
    font-size: 0.72rem;
  }
  .dots {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 2px;
    align-items: center;
  }
  .dot {
    width: 12px;
    height: 12px;
  }
  .fill {
    fill: var(--ink);
  }
  .hollow {
    fill: none;
    stroke: var(--ink);
    stroke-width: 1.2;
  }
  .dim {
    opacity: 0.45;
  }
  .note {
    margin: 0;
    font-size: 0.72rem;
    color: var(--dim);
  }
</style>
