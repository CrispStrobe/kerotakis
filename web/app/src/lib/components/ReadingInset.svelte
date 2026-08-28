<script lang="ts">
  import { t } from "../i18n.svelte";
  let {
    vessel,
    reading,
    onclose,
  }: {
    vessel: number;
    reading: { key: string; value: number; confidence: string };
    onclose: () => void;
  } = $props();

  const NAMES: Record<string, { label: string; unit: string; digits: number }> = {
    ph: { label: "pH", unit: "", digits: 2 },
    temperature: { label: "temperature", unit: "°C", digits: 1 },
    ionic_strength: { label: "ionic strength", unit: "mol/kgw", digits: 4 },
    pe: { label: "pe", unit: "", digits: 2 },
  };
  const meta = $derived(
    NAMES[reading.key] ?? { label: reading.key.replace(/_/g, " "), unit: "", digits: 2 },
  );
</script>

<button class="inset" data-confidence={reading.confidence} onclick={onclose} aria-label={t("close reading")}>
  <span class="what">{t(meta.label)} · v{vessel + 1}</span>
  <span class="value">
    {reading.value.toFixed(meta.digits)}<small>{meta.unit}</small>
  </span>
  <span class="conf">{t(reading.confidence)}</span>
</button>

<style>
  .inset {
    position: fixed;
    left: 50%;
    bottom: 5.5rem;
    transform: translateX(-50%);
    z-index: 9;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.1rem;
    background: var(--panel);
    border: 2px solid var(--edge-strong);
    border-radius: 14px;
    padding: 0.7rem 1.6rem;
    color: var(--ink);
    font: inherit;
    cursor: pointer;
    box-shadow: 0 6px 24px var(--shadow);
  }
  .what {
    font-size: 0.78rem;
    color: var(--dim);
  }
  .value {
    font-size: 2.2rem;
    font-weight: 600;
    line-height: 1.1;
  }
  .value small {
    font-size: 1rem;
    color: var(--dim);
    margin-left: 0.25rem;
  }
  .conf {
    font-size: 0.7rem;
    color: var(--warn);
    text-transform: lowercase;
  }
</style>
