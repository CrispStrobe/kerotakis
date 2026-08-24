<script lang="ts">
  import type { ParticleCensus } from "../host/EngineHost";
  import ParticleView from "./ParticleView.svelte";

  let {
    vessel,
    lines,
    particles = undefined,
    onparticles,
    onclose,
  }: {
    vessel: number;
    lines: string[];
    particles?: ParticleCensus;
    onparticles: () => void;
    onclose: () => void;
  } = $props();
</script>

<section class="inspector" aria-label={`vessel v${vessel + 1} detail`}>
  <header>
    <h2>v{vessel + 1}</h2>
    <button onclick={onparticles}>particles</button>
    <button onclick={onclose} aria-label="close inspector">×</button>
  </header>
  <pre>{lines.join("\n")}</pre>
  {#if particles}
    <ParticleView census={particles} />
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
