<script lang="ts">
  import { buildTitrateLine } from "../titration";
  import type { ShelfItem } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let {
    vessel,
    shelf,
    busy,
    running,
    onstart,
    onclose,
  }: {
    /** 0-based id of the vessel the burette is clamped over. */
    vessel: number;
    shelf: ShelfItem[];
    busy: boolean;
    /** A titration this burette started is still stepping. */
    running: boolean;
    onstart: (line: string) => void;
    onclose: () => void;
  } = $props();

  let titrant = $state("NaOH");
  let molarity = $state(1);
  let incrementMl = $state(1);
  let targetPh = $state(7);

  const line = $derived(
    buildTitrateLine({ vessel, titrant, molarity, incrementMl, targetPh }),
  );
</script>

<section class="burette" aria-label={t("burette over v{vessel}", { vessel: vessel + 1 })}>
  <svg viewBox="0 0 24 64" class="glassware" aria-hidden="true">
    <!-- The column, the tap, the tip. -->
    <rect x="9" y="2" width="6" height="42" class="col" />
    <rect x="9" y="6" width="6" height={Math.max(2, 30)} class="fill" />
    <path d="M 6 46 L 18 46 L 15 50 L 12 54 L 9 50 Z" class="tap" />
    <rect x="11" y="54" width="2" height="6" class="tip" />
    {#if running}
      <circle class="drop" cx="12" cy="61" r="1.6" />
    {/if}
  </svg>

  <div class="form">
    <strong>{t("burette")} · v{vessel + 1}</strong>
    <label>
      {t("titrant")}
      <select bind:value={titrant}>
        {#each shelf as s (s.key)}
          <option value={s.key}>{s.name}</option>
        {/each}
      </select>
    </label>
    <label>
      {t("concentration")}
      <span><input type="number" step="0.1" min="0.01" bind:value={molarity} /> mol/L</span>
    </label>
    <label>
      {t("per drop")}
      <span><input type="number" step="0.5" min="0.1" bind:value={incrementMl} /> mL</span>
    </label>
    <label>
      {t("until pH")}
      <input type="number" step="0.5" bind:value={targetPh} />
    </label>
    <div class="row">
      <button class="start" disabled={busy || line === null} onclick={() => line && onstart(line)}>
        {running ? t("dripping…") : t("start the drip")}
      </button>
      <button class="close" onclick={onclose}>{t("put away")}</button>
    </div>
    {#if line}
      <code>{line}</code>
    {/if}
  </div>
</section>

<style>
  .burette {
    display: flex;
    gap: 0.8rem;
    align-items: flex-start;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
  }
  .glassware {
    width: 30px;
    flex: none;
  }
  .col {
    fill: none;
    stroke: var(--edge-strong);
    stroke-width: 1.2;
  }
  .fill {
    fill: var(--cool);
    opacity: 0.5;
  }
  .tap {
    fill: var(--edge-strong);
  }
  .tip {
    fill: var(--edge-strong);
  }
  .drop {
    fill: var(--cool);
    animation: drip 0.8s ease-in infinite;
  }
  @keyframes drip {
    from {
      transform: translateY(0);
      opacity: 1;
    }
    to {
      transform: translateY(10px);
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .drop {
      animation: none;
    }
  }
  .form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: center;
    font-size: 0.82rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    color: var(--dim);
  }
  select,
  input {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.25rem 0.4rem;
    min-height: 34px;
  }
  input[type="number"] {
    width: 4.5rem;
  }
  .row {
    display: flex;
    gap: 0.5rem;
  }
  .start {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.3rem 0.8rem;
    cursor: pointer;
    min-height: 36px;
  }
  .close {
    background: none;
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--dim);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.3rem 0.8rem;
    cursor: pointer;
    min-height: 36px;
  }
  code {
    flex-basis: 100%;
    color: var(--dim);
    font-size: 0.72rem;
  }
</style>
