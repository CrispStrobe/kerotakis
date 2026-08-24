<script lang="ts">
  import type { ParticleCensus } from "../host/EngineHost";
  import ParticleView from "./ParticleView.svelte";

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
</script>

<section class="inspector" aria-label={`vessel v${vessel + 1} detail`}>
  <header>
    <h2>v{vessel + 1}</h2>
    <button onclick={onparticles}>particles</button>
    <button onclick={onclose} aria-label="close inspector">×</button>
  </header>
  {#if onaction}
    <div class="actions" role="group" aria-label={`act on ${v}`}>
      {#each actions as a (a.label)}
        <button disabled={busy} onclick={() => onaction(a.line)}>{a.label}</button>
      {/each}
    </div>
  {/if}
  <pre>{lines.join("\n")}</pre>
  {#if particles}
    <svelte:boundary>
      <ParticleView census={particles} />
      {#snippet failed(error)}
        <p class="fail">the particle view could not be drawn: {String(error)}</p>
      {/snippet}
    </svelte:boundary>
  {/if}
</section>

<style>
  .inspector {
    border-bottom: 1px solid var(--edge);
    display: flex;
    flex-direction: column;
    max-height: 45%;
  }
  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--edge);
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
    margin: 0;
    padding: 0.8rem 1rem;
    overflow: auto;
    font-size: 0.8rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
