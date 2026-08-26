<script lang="ts">
  import { i18n, type Locale, t } from "../i18n.svelte";
</script>

<label class="locale">
  <span class="sr-only">{t("Language")}</span>
  <select
    aria-label={t("Language")}
    value={i18n.locale}
    onchange={(event) => i18n.setLocale(event.currentTarget.value as Locale)}
  >
    <option value="en">{t("English")}</option>
    <option value="de">{t("German")}</option>
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
