<script lang="ts">
  import type { ParticleCensus } from "../host/EngineHost";
  import ParticleView from "./ParticleView.svelte";
  import InstrumentTray from "./InstrumentTray.svelte";
  import { t } from "../i18n.svelte";
  import { engineText } from "../engineText";

  let {
    vessel,
    lines,
    particles = undefined,
    boundary = "open",
    busy = false,
    onparticles,
    onclose,
    onaction,
  }: {
    vessel: number;
    lines: string[];
    particles?: ParticleCensus;
    boundary?: string;
    busy?: boolean;
    onparticles: () => void;
    onclose: () => void;
    onaction?: (line: string) => void;
  } = $props();

  const v = $derived(`v${vessel + 1}`);
  // The four classical gas tests (EXP-31): applied to the headspace, and
  // each button is exactly the grammar line — `test v1 pop`.
  const GAS_TESTS = ["pop", "splint", "limewater", "litmus"] as const;
</script>

<section class="inspector" aria-label={t("vessel v{vessel} detail", { vessel: vessel + 1 })}>
  <header>
    <span class="detail-heading">
      <small>{t("selected vessel details")}</small>
      <h2>v{vessel + 1}</h2>
    </span>
    <button onclick={onparticles}>{t("particles")}</button>
    <button onclick={onclose} aria-label={t("close inspector")}>×</button>
  </header>
  {#if onaction}
    <section class="detail-section" aria-label={t("measure selected vessel") }>
      <h3>{t("measure")}</h3>
      <InstrumentTray {vessel} {busy} onmeasure={onaction} />
    </section>
    <section class="detail-section gas-section" aria-label={t("gas tests on {vessel}", { vessel: v })}>
      <h3>{t("gas tests")}</h3>
      <p>{t("Apply a test to the headspace of the selected vessel.")}</p>
      <div class="actions" role="group">
        {#each GAS_TESTS as test (test)}
          <button disabled={busy} onclick={() => onaction(`test ${v} ${test}`)}>{t(test)}</button>
        {/each}
      </div>
    </section>
  {/if}
  <section class="computed-state" aria-label={t("computed state") }>
    <h3>{t("computed state")}</h3>
    <pre>{lines.map(engineText).join("\n")}</pre>
  </section>
  {#if particles}
    <svelte:boundary>
      <ParticleView census={particles} />
      {#snippet failed(error)}
        <p class="fail">{t("the particle view could not be drawn: {error}", { error: String(error) })}</p>
      {/snippet}
    </svelte:boundary>
  {/if}
</section>

<style>
  .inspector {
    border-bottom: 1px solid var(--edge);
    display: flex;
    flex-direction: column;
    /* The instrument and action rows can be taller than their share of a
       narrow journal. Keep them in their own scroll region so they never
       paint over (or steal pointer events from) the notebook below. */
    flex: 0 1 55%;
    min-height: 0;
    max-height: 55%;
    overflow-x: hidden;
    overflow-y: auto;
    overscroll-behavior: contain;
  }
  header {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
  }
  h2 {
    font-size: 0.85rem;
    margin: 0;
    margin-right: auto;
  }
  .detail-heading { min-width: 0; display: flex; flex: 1; flex-direction: column; }
  .detail-heading small { color: var(--dim); font-size: .52rem; font-weight: 750; letter-spacing: .07em; text-transform: uppercase; }
  .detail-section { padding-top: .55rem; border-bottom: 1px solid color-mix(in srgb, var(--edge) 65%, transparent); }
  h3 { margin: 0; padding: 0 1rem .25rem; color: var(--dim); font-size: .58rem; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
  .gas-section { padding-bottom: .55rem; }
  .gas-section p { margin: 0; padding: 0 1rem .25rem; color: var(--dim); font-size: .66rem; line-height: 1.35; }
  button {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.25rem 0.6rem;
    cursor: pointer;
    min-height: 32px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    padding: 0.2rem 1rem 0;
  }
  .actions button {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--ink);
    font: inherit;
    font-size: 0.76rem;
    padding: 0.25rem 0.65rem;
    cursor: pointer;
    min-height: 34px;
  }
  .actions button:hover {
    border-color: var(--hot);
  }
  .fail {
    margin: 0;
    padding: 0.5rem 1rem;
    color: var(--bad);
    font-size: 0.8rem;
  }
  .computed-state { padding-top: .65rem; }
  pre {
    margin: 0;
    padding: 0.35rem 1rem 0.8rem;
    overflow: auto;
    font-size: 0.8rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
