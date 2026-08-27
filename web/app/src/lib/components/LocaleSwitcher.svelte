<script lang="ts">
  /**
   * The languages this build ships, named as they name themselves.
   *
   * Built from `availableLocales()` rather than listed here, so adding a
   * language is still just adding `locales/<code>.json` — a hardcoded
   * <option> list is exactly the third place that gets forgotten.
   */
  import { availableLocales, i18n, type Locale, t } from "../i18n.svelte";

  const locales = availableLocales();
</script>

<label class="locale">
  <span class="sr-only">{t("Language")}</span>
  <select
    aria-label={t("Language")}
    value={i18n.locale}
    onchange={(event) => i18n.setLocale(event.currentTarget.value as Locale)}
  >
    {#each locales as l (l.code)}
      <option value={l.code}>{l.name}</option>
    {/each}
  </select>
</label>

<style>
  .locale select {
    background: var(--surface-raised);
    border: 1px solid var(--edge);
    border-radius: 8px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    min-height: 40px;
    padding: 0.25rem 0.45rem;
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
