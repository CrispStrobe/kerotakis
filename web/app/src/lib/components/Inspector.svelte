<script lang="ts">
  import type { ParticleCensus } from "../host/EngineHost";
  import ParticleView from "./ParticleView.svelte";
  import InstrumentTray from "./InstrumentTray.svelte";
  import { t } from "../i18n.svelte";

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
  // Every button compiles to the same grammar the command bar speaks.
  const actions = $derived([
    { label: "heat 10 kJ", line: `heat ${v} 10kJ` },
    { label: "cool 10 kJ", line: `cool ${v} 10kJ` },
    { label: "stir", line: `stir ${v}` },
    { label: "ignite", line: `ignite ${v}` },
    boundary === "open"
      ? { label: "seal 500 mL", line: `seal ${v} 500mL` }
      : { label: "open", line: `open ${v}` },
  ]);
  // The four classical gas tests (EXP-31): applied to the headspace, and
  // each button is exactly the grammar line — `test v1 pop`.
  const GAS_TESTS = ["pop", "splint", "limewater", "litmus"] as const;
</script>

<section class="inspector" aria-label={t("vessel v{vessel} detail", { vessel: vessel + 1 })}>
  <header>
    <h2>v{vessel + 1}</h2>
    <button onclick={onparticles}>{t("particles")}</button>
    <button onclick={onclose} aria-label={t("close inspector")}>×</button>
  </header>
  {#if onaction}
    <InstrumentTray {vessel} {busy} onmeasure={onaction} />
    <div class="actions" role="group" aria-label={t("act on {vessel}", { vessel: v })}>
      {#each actions as a (a.label)}
        <button disabled={busy} onclick={() => onaction(a.line)}>{t(a.label)}</button>
      {/each}
    </div>
    <div class="actions" role="group" aria-label={t("gas tests on {vessel}", { vessel: v })}>
      <span class="tests-label">{t("test the gas:")}</span>
      {#each GAS_TESTS as test (test)}
        <button disabled={busy} onclick={() => onaction(`test ${v} ${test}`)}>{t(test)}</button>
      {/each}
    </div>
  {/if}
  <pre>{lines.join("\n")}</pre>
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
  .tests-label {
    color: var(--dim);
    font-size: 0.75rem;
    align-self: center;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    padding: 0.5rem 1rem 0;
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
  pre {
    flex: none;
    margin: 0;
    padding: 0.8rem 1rem;
    overflow: auto;
    font-size: 0.8rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
