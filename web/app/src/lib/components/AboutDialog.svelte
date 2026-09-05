<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import {
    BUILD,
    COPYRIGHT,
    NOTICE_SECTIONS,
    REPO,
    THIRD_PARTY_LICENSES,
    builtAt,
    commitUrl,
    hardReload,
  } from "../about";

  let { onclose }: { onclose: () => void } = $props();

  let reloading = $state(false);
  const when = $derived(builtAt(i18n.locale));
  const commit = commitUrl();

  async function refresh() {
    // No confirmation: nothing is lost. Notes and preferences are in
    // localStorage, which this deliberately does not touch.
    reloading = true;
    await hardReload();
  }
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <dialog open
    class="about"
    aria-modal="true"
    aria-label={t("about Kerotakis")}
    onclick={(e) => e.stopPropagation()}
  >
    <button class="icon-close about-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    <h2>Kerotakis</h2>
    <p class="tagline">{t("a virtual chemistry laboratory that computes real chemistry")}</p>

    <dl class="build">
      <dt>{t("build")}</dt>
      <dd class="mono">
        {#if commit}
          <a href={commit} target="_blank" rel="noreferrer noopener">{BUILD.commit}</a>
        {:else}
          <!-- "unknown" is a real answer: a source tarball has no .git.
               Saying so beats inventing a version that would send someone
               reading the wrong code. -->
          {t("unknown — this build carries no commit stamp")}
        {/if}
      </dd>
      {#if BUILD.ref}
        <dt>{t("tag")}</dt>
        <dd class="mono">{BUILD.ref}</dd>
      {/if}
      {#if when}
        <dt>{t("built")}</dt>
        <dd>{when}</dd>
      {/if}
      <dt>{t("copyright")}</dt>
      <dd>{COPYRIGHT}</dd>
      <dt>{t("licence")}</dt>
      <dd>
        <a
          href={`${THIRD_PARTY_LICENSES}#kerotakis-license`}
          target="_blank"
          rel="noreferrer noopener"
        >
          AGPL-3.0-or-later + {t("section 7 app-store permission")}
        </a>
      </dd>
      <dt>{t("source")}</dt>
      <dd><a href={REPO} target="_blank" rel="noreferrer noopener">{t("on GitHub")}</a></dd>
    </dl>

    <button class="refresh" onclick={refresh} disabled={reloading}>
      {reloading ? t("reloading…") : t("reload, discarding cached files")}
    </button>
    <p class="note">
      {t("Clears the cached app so a new version is fetched. Your notes and settings are kept.")}
    </p>
    <a
      class="legal-link"
      href={`${THIRD_PARTY_LICENSES}?lang=${encodeURIComponent(i18n.locale)}`}
      target="_blank"
      rel="noreferrer noopener"
    >{t("open complete open-source licences")}</a>
    <p class="note">
      {t("The complete dependency licences are bundled with this app and work offline.")}
    </p>

    <h3>{t("Your freedoms and warranty")}</h3>
    <p class="note legal-notice">
      {t("Kerotakis comes with no warranty, to the extent permitted by law. You may use, share and modify it under AGPL-3.0-or-later. Open the licence above for the exact terms and the corresponding source.")}
    </p>

    <h3>{t("Privacy")}</h3>
    <p class="note">
      {t("Kerotakis has no account, analytics, telemetry, advertising or tracking. Native builds make no app-originated network requests; the web host may record ordinary requests used to download the app. Your lab work remains on this device.")}
    </p>

    <h3>{t("Components, authors and acknowledgements")}</h3>
    <p class="note">
      {t("Reproduced from NOTICE, which is the authoritative list.")}
      <a href={`${REPO}/blob/main/NOTICE`} target="_blank" rel="noreferrer noopener"
        >{t("read it in full")}</a
      >
    </p>
    {#each NOTICE_SECTIONS as section (section.title)}
      <section class="group">
        <h4>{t(section.title)}</h4>
        <ul>
          {#each section.entries as entry (entry)}
            <!-- Verbatim, and deliberately not translated: these are
                 licence statements, and a translated licence claim is a
                 different claim. -->
            <li>{entry}</li>
          {/each}
        </ul>
      </section>
    {/each}
  </dialog>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: var(--scrim);
    display: grid;
    place-items: center;
    z-index: 60;
  }
  .about-close {
    position: sticky;
    top: 0;
    z-index: 3;
    float: right;
  }
  .about {
    /* Tablets: never taller than the viewport, and scroll inside rather
       than pushing the close button off-screen. */
    max-height: min(86vh, 44rem);
    max-width: min(92vw, 40rem);
    overflow-y: auto;
    border: 1px solid var(--edge);
    border-radius: 0.75rem;
    padding: 1.1rem 1.25rem 1.25rem;
    background: var(--panel);
    color: var(--ink);
  }
  h2 {
    margin: 0;
    font-size: 1.25rem;
  }
  .tagline {
    margin: 0.15rem 0 0.9rem;
    color: var(--dim);
  }
  h3 {
    margin: 1.4rem 0 0.2rem;
    font-size: 1rem;
  }
  h4 {
    margin: 0.9rem 0 0.3rem;
    font-size: 0.9rem;
    color: var(--dim);
  }
  .build {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.28rem 0.9rem;
    margin: 0 0 1rem;
  }
  .build dt {
    color: var(--dim);
  }
  .build dd {
    margin: 0;
  }
  .mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .refresh {
    /* A tablet target, not a mouse one. */
    min-height: 2.75rem;
    padding: 0 1rem;
    width: 100%;
  }
  .note {
    margin: 0.35rem 0 0;
    color: var(--dim);
    font-size: 0.85rem;
  }
  .legal-notice {
    padding-left: 0.7rem;
    border-left: 0.2rem solid var(--primary);
  }
  .legal-link {
    display: grid;
    place-items: center;
    min-height: 2.75rem;
    margin-top: 0.75rem;
    padding: 0 1rem;
    border: 1px solid var(--edge);
    border-radius: 0.6rem;
    font-weight: 700;
  }
  ul {
    margin: 0;
    padding-left: 1.1rem;
  }
  li {
    margin: 0.3rem 0;
    font-size: 0.85rem;
    line-height: 1.45;
  }
</style>
