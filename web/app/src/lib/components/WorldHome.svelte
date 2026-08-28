<script lang="ts">
  import { untrack } from "svelte";
  import { t } from "../i18n.svelte";
  import type { LabMode, LabProfile } from "../worldState";

  let {
    mode,
    profile,
    missions,
    experiments,
    canclose = true,
    onenter,
    onmissions,
    onresearch,
    onrename,
    onclose,
  }: {
    mode: LabMode;
    profile: LabProfile;
    missions: number;
    experiments: number;
    canclose?: boolean;
    onenter: (mode: LabMode) => void;
    onmissions: () => void;
    onresearch: () => void;
    onrename: (name: string) => void;
    onclose: () => void;
  } = $props();

  let editing = $state(false);
  let draft = $state(untrack(() => profile.name));

  function saveName() {
    const name = draft.trim();
    if (name) onrename(name);
    else draft = profile.name;
    editing = false;
  }
</script>

<div class="world-scrim" role="presentation">
  <dialog open class="world" aria-labelledby="world-title">
    <header>
      <div class="identity-mark" aria-hidden="true">⚗</div>
      <div class="identity">
        <span>{t("your laboratory")}</span>
        {#if editing}
          <form onsubmit={(event) => { event.preventDefault(); saveName(); }}>
            <input bind:value={draft} maxlength="48" aria-label={t("laboratory name")} />
            <button>{t("save")}</button>
          </form>
        {:else}
          <button class="lab-name" aria-label={t("rename laboratory")} onclick={() => (editing = true)}>
            <h1 id="world-title">{t(profile.name)}</h1><span aria-hidden="true">✎</span>
          </button>
        {/if}
      </div>
      <div class="world-status">
        <small>{t("current save")}</small>
        <strong class:story={mode === "story"}>{mode === "story" ? t("Story") : t("Sandbox")}</strong>
      </div>
      {#if canclose}<button class="close" aria-label={t("close world map")} onclick={onclose}>×</button>{/if}
    </header>

    <main>
      <div class="intro">
        <span class="eyebrow">{t("Kerotakis Research Campus")}</span>
        <h2>{t("Where do you want to work today?")}</h2>
        <p>{t("Explore guided investigations in Story, or enter a fully unlocked laboratory in Sandbox.")}</p>
      </div>

      <div class="campus-map">
        <svg class="routes" viewBox="0 0 1000 420" preserveAspectRatio="none" aria-hidden="true">
          <path d="M 180 220 C 330 40 650 40 820 210" />
          <path d="M 180 220 C 350 390 650 390 820 210" />
          <circle cx="500" cy="74" r="6" /><circle cx="500" cy="356" r="6" />
        </svg>

        <article class="destination story-destination" class:current={mode === "story"}>
          <span class="current-flag">{mode === "story" ? t("active now") : t("separate save")}</span>
          <div class="building story-building" aria-hidden="true"><span>◆</span></div>
          <span class="kicker">{t("Story laboratory")}</span>
          <h3>{t("The Discovery Wing")}</h3>
          <p>{t("Take missions, earn permanent instruments, and follow the chemistry story at your pace.")}</p>
          <div class="destination-meta"><span>{missions > 0 ? t("{count} missions", { count: missions }) : t("missions arriving…")}</span><span>{t("guided progress")}</span></div>
          <button onclick={() => onenter("story")}>
            {mode === "story" ? t("enter Story") : t("switch to Story")}
            <span aria-hidden="true">→</span>
          </button>
        </article>

        <button class="map-node mission-node" onclick={onmissions}>
          <span aria-hidden="true">◎</span><strong>{t("Mission Board")}</strong><small>{t("choose an investigation")}</small>
        </button>

        <button class="map-node research-node" onclick={onresearch}>
          <span aria-hidden="true">⌬</span><strong>{t("Research Library")}</strong><small>{experiments > 0 ? t("{count} computed experiments", { count: experiments }) : t("archive syncing…")}</small>
        </button>

        <article class="destination sandbox-destination" class:current={mode === "sandbox"}>
          <span class="current-flag">{mode === "sandbox" ? t("active now") : t("separate save")}</span>
          <div class="building sandbox-building" aria-hidden="true"><span>∞</span></div>
          <span class="kicker">{t("Sandbox hangar")}</span>
          <h3>{t("The Open Bench")}</h3>
          <p>{t("Every reagent and instrument is available. Build freely without changing Story progress.")}</p>
          <div class="destination-meta"><span>{t("everything unlocked")}</span><span>{t("free exploration")}</span></div>
          <button onclick={() => onenter("sandbox")}>
            {mode === "sandbox" ? t("enter Sandbox") : t("switch to Sandbox")}
            <span aria-hidden="true">→</span>
          </button>
        </article>
      </div>
    </main>

    <footer>
      <span aria-hidden="true">✓</span>
      <p><strong>{t("Your saves stay separate.")}</strong> {t("Clearing or experimenting in Sandbox never changes your Story laboratory.")}</p>
    </footer>
  </dialog>
</div>

<style>
  .world-scrim { position: fixed; inset: 0; z-index: 95; display: grid; place-items: center; padding: 1rem; background: rgb(8 28 45 / 68%); backdrop-filter: blur(14px) saturate(1.1); }
  .world { width: min(74rem, 100%); max-height: calc(100dvh - 2rem); overflow: auto; border: 1px solid color-mix(in srgb, var(--primary) 40%, var(--edge)); border-radius: 30px; color: var(--ink); background: radial-gradient(circle at 50% -20%, color-mix(in srgb, var(--primary) 20%, transparent), transparent 31rem), var(--surface); box-shadow: 0 35px 110px rgb(2 17 31 / 55%); }
  header { position: sticky; top: 0; z-index: 5; min-height: 5rem; display: flex; align-items: center; gap: .8rem; padding: .85rem clamp(1rem, 3vw, 2rem); border-bottom: 1px solid var(--edge); background: color-mix(in srgb, var(--surface) 92%, transparent); backdrop-filter: blur(14px); }
  .identity-mark { width: 48px; height: 48px; display: grid; place-items: center; flex: none; border-radius: 15px; color: var(--on-accent); background: linear-gradient(145deg, var(--primary), var(--instrument)); box-shadow: 0 8px 20px color-mix(in srgb, var(--primary) 25%, transparent); font-size: 1.45rem; }
  .identity { min-width: 0; }
  .identity > span, .world-status small, .eyebrow, .kicker, .current-flag { color: var(--dim); font-size: .62rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  .lab-name { display: flex; align-items: center; gap: .45rem; padding: 0; border: 0; color: var(--ink); background: transparent; cursor: pointer; }
  h1 { margin: .08rem 0 0; overflow: hidden; font-size: 1.15rem; text-overflow: ellipsis; white-space: nowrap; }
  .identity form { display: flex; gap: .35rem; }
  .identity input { min-height: 36px; border: 1px solid var(--primary); border-radius: 9px; color: var(--ink); background: var(--surface); font: inherit; padding: 0 .55rem; }
  .identity form button { border: 0; border-radius: 9px; color: var(--on-accent); background: var(--primary); font-weight: 750; }
  .world-status { margin-left: auto; display: flex; flex-direction: column; align-items: end; }
  .world-status strong { color: var(--instrument); }
  .world-status strong.story { color: var(--discovery); }
  .close { width: 40px; height: 40px; flex: none; border: 1px solid var(--edge); border-radius: 50%; color: var(--ink); background: var(--surface-raised); cursor: pointer; font-size: 1.3rem; }
  main { padding: clamp(1.2rem, 3vw, 2.2rem); }
  .intro { max-width: 42rem; margin: 0 auto 1.2rem; text-align: center; }
  .eyebrow { color: var(--discovery); }
  .intro h2 { margin: .3rem 0 .55rem; font-size: clamp(1.6rem, 3.4vw, 2.7rem); line-height: 1; letter-spacing: -.04em; }
  .intro p { margin: 0; color: var(--dim); }
  .campus-map { position: relative; min-height: 29rem; display: grid; grid-template-columns: minmax(15rem, 1fr) minmax(10rem, .65fr) minmax(15rem, 1fr); grid-template-rows: 1fr 1fr; gap: 1rem; align-items: center; }
  .routes { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; }
  .routes path { fill: none; stroke: color-mix(in srgb, var(--primary) 30%, var(--edge)); stroke-width: 3; stroke-dasharray: 9 7; }
  .routes circle { fill: var(--discovery); }
  .destination { position: relative; z-index: 2; min-height: 23rem; display: flex; flex-direction: column; padding: 1.1rem; border: 1px solid var(--edge); border-radius: 22px; background: color-mix(in srgb, var(--surface) 94%, var(--primary) 6%); box-shadow: 0 12px 28px var(--shadow); }
  .story-destination { grid-row: 1 / 3; }
  .sandbox-destination { grid-column: 3; grid-row: 1 / 3; background: color-mix(in srgb, var(--surface) 94%, var(--instrument) 6%); }
  .destination.current { border-color: var(--discovery); box-shadow: 0 0 0 3px color-mix(in srgb, var(--discovery) 13%, transparent), 0 18px 34px var(--shadow); }
  .sandbox-destination.current { border-color: var(--instrument); box-shadow: 0 0 0 3px color-mix(in srgb, var(--instrument) 13%, transparent), 0 18px 34px var(--shadow); }
  .current-flag { align-self: end; color: var(--discovery); }
  .sandbox-destination .current-flag { color: var(--instrument); }
  .building { height: 7.5rem; display: grid; place-items: center; margin: .55rem 0 1rem; border-radius: 18px; color: var(--on-accent); background: linear-gradient(145deg, color-mix(in srgb, var(--discovery) 88%, white), color-mix(in srgb, var(--primary) 75%, black)); box-shadow: inset 0 -18px 28px rgb(0 0 0 / 10%); font-size: 3rem; }
  .sandbox-building { background: linear-gradient(145deg, color-mix(in srgb, var(--instrument) 88%, white), color-mix(in srgb, var(--primary) 65%, black)); }
  .kicker { color: var(--discovery); }
  .sandbox-destination .kicker { color: var(--instrument); }
  .destination h3 { margin: .18rem 0 .4rem; font-size: 1.3rem; }
  .destination p { margin: 0; color: var(--dim); font-size: .82rem; line-height: 1.45; }
  .destination-meta { display: flex; flex-wrap: wrap; gap: .35rem; margin: .8rem 0; }
  .destination-meta span { padding: .25rem .45rem; border-radius: 999px; color: var(--dim); background: var(--surface-raised); font-size: .62rem; font-weight: 700; }
  .destination > button { min-height: 44px; display: flex; align-items: center; justify-content: space-between; margin-top: auto; padding: 0 .8rem; border: 0; border-radius: 12px; color: var(--on-accent); background: var(--discovery); cursor: pointer; font-weight: 800; }
  .sandbox-destination > button { background: var(--instrument); }
  .map-node { position: relative; z-index: 3; min-height: 8rem; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: .7rem; border: 1px solid var(--edge); border-radius: 18px; color: var(--ink); background: var(--surface); box-shadow: 0 10px 24px var(--shadow); cursor: pointer; text-align: center; }
  .map-node > span { width: 38px; height: 38px; display: grid; place-items: center; margin-bottom: .35rem; border-radius: 12px; color: var(--primary); background: color-mix(in srgb, var(--primary) 10%, var(--surface)); font-size: 1.2rem; }
  .map-node small { color: var(--dim); font-size: .65rem; }
  .map-node:hover, .destination > button:hover { transform: translateY(-2px); filter: brightness(1.04); }
  footer { display: flex; align-items: center; justify-content: center; gap: .6rem; padding: .8rem 1.2rem; border-top: 1px solid var(--edge); color: var(--dim); background: var(--surface-raised); }
  footer > span { color: var(--success); font-weight: 900; }
  footer p { margin: 0; font-size: .75rem; }
  @media (max-width: 760px) {
    .world-scrim { padding: 0; }
    .world { width: 100%; height: 100dvh; max-height: none; border: 0; border-radius: 0; }
    .campus-map { display: flex; flex-direction: column; }
    .destination, .map-node { width: 100%; min-height: auto; }
    .building { height: 5rem; }
    .routes { display: none; }
    .world-status { display: none; }
  }
</style>
