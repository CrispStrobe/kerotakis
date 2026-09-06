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
  const current = $derived(locales.find((l) => l.code === i18n.locale)?.name ?? i18n.locale);
</script>

<!-- The code, not the endonym. "Deutsch" is eight characters of header
     rail spent restating what the whole interface already demonstrates,
     and at 390 px it was the widest control in the row. The accessible
     name still carries the language it is naming, and each choice keeps
     its endonym as its title, so a reader who cannot read the current
     interface language can still find their own. -->
<label class="locale">
  <span class="sr-only">{t("Language")}</span>
  <select
    aria-label={`${t("Language")}: ${current}`}
    value={i18n.locale}
    onchange={(event) => i18n.setLocale(event.currentTarget.value as Locale)}
  >
    {#each locales as l (l.code)}
      <option value={l.code} title={l.name} aria-label={l.name}>{l.code.toLocaleUpperCase("en")}</option>
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
    font-weight: 650;
    letter-spacing: 0.04em;
    min-height: 40px;
    padding: 0.25rem 0.4rem;
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
