<script lang="ts">
  import { t } from "../i18n.svelte";
  let {
    position,
    total,
    busy,
    onjump,
  }: {
    position: number;
    total: number;
    busy: boolean;
    onjump: (to: number) => void;
  } = $props();
</script>

{#if total > 0}
  <label class="timeline">
    <span class="count">{position}/{total}</span>
    <input
      type="range"
      min="0"
      max={total}
      step="1"
      value={position}
      disabled={busy}
      aria-label={t("timeline: step {position} of {total}", { position, total })}
      onchange={(e) => onjump(Number(e.currentTarget.value))}
    />
  </label>
{/if}

<style>
  .timeline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.78rem;
    color: var(--dim);
  }
  input[type="range"] {
    width: 9rem;
    accent-color: var(--hot);
  }
</style>
