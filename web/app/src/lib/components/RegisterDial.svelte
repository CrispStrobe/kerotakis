<script lang="ts">
  import { REGISTERS } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let { value, onchange }: { value: string; onchange: (level: string) => void } = $props();
</script>

<!-- The dial is the product: same bench, mid-session switchable detail.
     Three wide buttons said all three levels at once and spent a third of
     the header rail doing it. A dropdown says only where you are standing
     and still offers the other two in one press — and it is the control
     every phone keyboard and screen reader already knows how to drive. -->
<label class="dial">
  <span class="sr-only">{t("detail level")}</span>
  <select
    aria-label={t("detail level")}
    {value}
    onchange={(event) => onchange(event.currentTarget.value)}
  >
    {#each REGISTERS as reg (reg.level)}
      <option value={reg.level}>{reg.level} · {t(reg.label)}</option>
    {/each}
  </select>
</label>

<style>
  .dial {
    display: inline-flex;
    align-items: center;
    min-height: 40px;
  }
  select {
    max-width: 10rem;
    min-height: 40px;
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--primary);
    background: var(--surface-raised);
    font: inherit;
    font-size: 0.78rem;
    font-weight: 650;
    cursor: pointer;
  }
  select:hover {
    border-color: var(--primary);
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
