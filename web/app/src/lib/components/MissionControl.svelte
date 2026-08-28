<script lang="ts">
  import { t } from "../i18n.svelte";
  import type { CodexEntry } from "../codex";
  import type { LabMode } from "../worldState";

  type Mission = { file: string; name: string; blurb?: string; topic?: string };

  let {
    missions,
    experiments,
    mode,
    active = null,
    cursor = 0,
    total = 0,
    onstart,
    onsandbox,
    onexperiments,
    onmap,
    onclose,
  }: {
    missions: Mission[];
    experiments: CodexEntry[];
    mode: LabMode;
    active?: string | null;
    cursor?: number;
    total?: number;
    onstart: (file: string) => void;
    onsandbox: () => void;
    onexperiments: () => void;
    onmap: () => void;
    onclose: () => void;
  } = $props();

  const topicIcon = (topic?: string) => {
    if (topic?.includes("acid")) return "⚗";
    if (topic?.includes("heat") || topic?.includes("fire")) return "♨";
    if (topic?.includes("electric")) return "ϟ";
    if (topic?.includes("water")) return "◉";
    if (topic?.includes("gas")) return "◎";
    if (topic?.includes("separation")) return "◇";
    return "✦";
  };
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(e) => e.key === "Escape" && onclose()}>
  <dialog open class="mission-control" aria-labelledby="mission-title" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
    <header>
      <div class="eyebrow"><span aria-hidden="true">✦</span>{t("Mission Control")}</div>
      <h1 id="mission-title">{t("Choose your path")}</h1>
      <p>{t("Follow a guided investigation or open the whole laboratory.")}</p>
      <button class="close" aria-label={t("close Mission Control")} onclick={onclose}>×</button>
    </header>

    <div class="paths">
      <article class="sandbox-card" class:current={mode === "sandbox"}>
        <div class="card-icon" aria-hidden="true">∞</div>
        <div>
          <span class="card-kicker">{t("Sandbox lab")}</span>
          <h2>{t("Your laboratory, your rules")}</h2>
          <p>{t("Everything is unlocked. Build, test, and break your own ideas.")}</p>
        </div>
        <button onclick={onsandbox}>{mode === "sandbox" ? t("you are here") : t("exit to sandbox")}</button>
      </article>

      <article class="library-card">
        <div class="card-icon" aria-hidden="true">⌬</div>
        <div>
          <span class="card-kicker">{t("Experiment library")}</span>
          <h2>{t("{count} experiments", { count: experiments.length })}</h2>
          <p>{t("Predict first, run real chemistry, then compare the evidence.")}</p>
        </div>
        <div class="library-actions">
          <button onclick={onexperiments}>{t("browse experiments")}</button>
          <button class="map-button" onclick={onmap}>{t("concept map")}</button>
        </div>
      </article>

      {#if active}
        <article class="active-mission">
          <div class="pulse" aria-hidden="true"></div>
          <div>
            <span class="card-kicker">{t("mission in progress")}</span>
            <h2>{t(active)}</h2>
            <p>{t("{step} of {total} steps", { step: Math.min(cursor + 1, total), total })}</p>
          </div>
          <button onclick={onclose}>{t("continue mission")}</button>
          <div class="progress" aria-hidden="true"><span style={`width:${total > 0 ? Math.min(100, cursor / total * 100) : 0}%`}></span></div>
        </article>
      {/if}
    </div>

    <div class="mission-heading">
      <div><span>{t("Story missions")}</span><h2>{t("Available missions")}</h2></div>
      <span class="mission-count">{missions.length}</span>
    </div>

    {#if missions.length > 0}
      <div class="mission-grid">
        {#each missions as mission, i (mission.file)}
          <article class="mission-card" style={`--delay:${Math.min(i, 8) * 35}ms`}>
            <div class="mission-number">{String(i + 1).padStart(2, "0")}</div>
            <div class="topic-icon" aria-hidden="true">{topicIcon(mission.topic)}</div>
            <span class="topic">{t(mission.topic ?? "more")}</span>
            <h3>{t(mission.name)}</h3>
            {#if mission.blurb}<p>{t(mission.blurb)}</p>{/if}
            <button onclick={() => onstart(mission.file)}>
              <span>{t("launch mission")}</span><span aria-hidden="true">→</span>
            </button>
          </article>
        {/each}
      </div>
    {:else}
      <div class="no-missions">
        <span aria-hidden="true">⌁</span>
        <p>{t("Missions are downloading. The sandbox is ready now.")}</p>
      </div>
    {/if}
  </dialog>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 80;
    display: grid;
    place-items: center;
    padding: 1rem;
    background: rgb(7 24 42 / 58%);
    backdrop-filter: blur(12px) saturate(1.1);
  }
  .mission-control {
    width: min(68rem, 100%);
    max-height: min(48rem, calc(100dvh - 2rem));
    overflow: auto;
    border: 1px solid color-mix(in srgb, var(--primary) 32%, var(--edge));
    border-radius: 28px;
    color: var(--ink);
    background:
      radial-gradient(circle at 88% 4%, color-mix(in srgb, var(--discovery) 18%, transparent), transparent 21rem),
      radial-gradient(circle at 12% 0%, color-mix(in srgb, var(--instrument) 14%, transparent), transparent 19rem),
      var(--surface);
    box-shadow: 0 30px 90px rgb(4 19 34 / 42%);
  }
  header {
    position: relative;
    padding: clamp(1.5rem, 4vw, 2.6rem);
    border-bottom: 1px solid color-mix(in srgb, var(--edge) 72%, transparent);
  }
  .eyebrow,
  .card-kicker,
  .mission-heading span,
  .topic {
    color: var(--discovery);
    font-size: 0.7rem;
    font-weight: 800;
    letter-spacing: 0.11em;
    text-transform: uppercase;
  }
  .eyebrow { display: flex; align-items: center; gap: 0.4rem; }
  h1 { margin: 0.35rem 0 0; font-size: clamp(1.8rem, 4vw, 3rem); line-height: 1; letter-spacing: -0.04em; }
  header p { max-width: 38rem; margin: 0.7rem 0 0; color: var(--dim); font-size: 1rem; }
  .close {
    position: absolute;
    top: 1.2rem;
    right: 1.2rem;
    width: 42px;
    height: 42px;
    border: 1px solid var(--edge);
    border-radius: 50%;
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 84%, transparent);
    cursor: pointer;
    font-size: 1.45rem;
  }
  .paths {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
    gap: 0.8rem;
    padding: 1rem clamp(1rem, 4vw, 2.6rem) 0;
  }
  .sandbox-card,
  .library-card,
  .active-mission {
    position: relative;
    min-height: 10.5rem;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.8rem;
    padding: 1rem;
    overflow: hidden;
    border: 1px solid var(--edge);
    border-radius: 18px;
    background: color-mix(in srgb, var(--surface-raised) 72%, transparent);
  }
  .sandbox-card.current { border-color: var(--instrument); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--instrument) 22%, transparent); }
  .library-card { border-color: color-mix(in srgb, var(--primary) 40%, var(--edge)); }
  .library-card .card-icon { color: var(--primary); background: color-mix(in srgb, var(--primary) 12%, var(--surface)); }
  .card-icon {
    width: 46px;
    height: 46px;
    display: grid;
    place-items: center;
    border-radius: 14px;
    color: var(--instrument);
    background: color-mix(in srgb, var(--instrument) 12%, var(--surface));
    font-size: 1.55rem;
    font-weight: 800;
  }
  .paths h2 { margin: 0.15rem 0 0.35rem; font-size: 1.05rem; }
  .paths p { margin: 0; color: var(--dim); font-size: 0.8rem; }
  .paths button,
  .mission-card button {
    border: 0;
    border-radius: 11px;
    color: var(--on-accent);
    background: var(--primary);
    cursor: pointer;
    font-weight: 750;
  }
  .paths > article > button { grid-column: 1 / -1; min-height: 38px; }
  .library-actions { grid-column: 1 / -1; display: grid; grid-template-columns: 1fr auto; gap: 0.4rem; }
  .library-actions button { min-height: 38px; }
  .library-actions .map-button { color: var(--primary); border: 1px solid color-mix(in srgb, var(--primary) 35%, var(--edge)); background: var(--surface); }
  .active-mission { border-color: color-mix(in srgb, var(--discovery) 55%, var(--edge)); }
  .active-mission .pulse { width: 13px; height: 13px; margin: 0.35rem; border-radius: 50%; background: var(--discovery); box-shadow: 0 0 0 7px color-mix(in srgb, var(--discovery) 13%, transparent); }
  .progress { position: absolute; inset: auto 0 0; height: 4px; background: color-mix(in srgb, var(--edge) 45%, transparent); }
  .progress span { display: block; height: 100%; background: var(--discovery); transition: width 300ms ease; }
  .mission-heading { display: flex; align-items: end; justify-content: space-between; padding: 1.6rem clamp(1rem, 4vw, 2.6rem) 0.7rem; }
  .mission-heading h2 { margin: 0.1rem 0 0; font-size: 1.25rem; }
  .mission-count { min-width: 2rem; padding: 0.25rem 0.5rem; border-radius: 999px; color: var(--dim) !important; background: var(--surface-raised); text-align: center; }
  .mission-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr)); gap: 0.75rem; padding: 0  clamp(1rem, 4vw, 2.6rem) 2.6rem; }
  .mission-card {
    position: relative;
    min-height: 14rem;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    overflow: hidden;
    border: 1px solid var(--edge);
    border-radius: 18px;
    background: color-mix(in srgb, var(--surface) 86%, var(--surface-raised));
    animation: arrive 360ms both;
    animation-delay: var(--delay);
  }
  @keyframes arrive { from { opacity: 0; transform: translateY(8px); } }
  .mission-card:hover { border-color: var(--primary); transform: translateY(-2px); box-shadow: 0 12px 26px var(--shadow); }
  .mission-number { position: absolute; top: 0.3rem; right: 0.65rem; color: color-mix(in srgb, var(--primary) 11%, transparent); font-size: 3.5rem; font-weight: 900; line-height: 1; }
  .topic-icon { width: 38px; height: 38px; display: grid; place-items: center; margin-bottom: 1rem; border-radius: 12px; color: var(--action); background: color-mix(in srgb, var(--action) 10%, var(--surface)); font-size: 1.1rem; }
  .topic { color: var(--dim); font-size: 0.62rem; }
  .mission-card h3 { position: relative; margin: 0.3rem 0; font-size: 0.95rem; line-height: 1.25; }
  .mission-card p { margin: 0 0 1rem; color: var(--dim); font-size: 0.75rem; }
  .mission-card button { min-height: 40px; display: flex; align-items: center; justify-content: space-between; margin-top: auto; padding: 0 0.75rem; }
  .mission-card button:hover,
  .paths button:hover { filter: brightness(1.08); transform: translateY(-1px); box-shadow: 0 7px 16px color-mix(in srgb, var(--primary) 24%, transparent); }
  .no-missions { margin: 0 clamp(1rem, 4vw, 2.6rem) 2.6rem; padding: 2rem; border: 1px dashed var(--edge); border-radius: 18px; color: var(--dim); text-align: center; }
  .no-missions span { font-size: 2rem; }
  @media (max-width: 560px) {
    .scrim { padding: 0; }
    .mission-control { max-height: 100dvh; height: 100dvh; border: 0; border-radius: 0; }
    .paths { grid-template-columns: 1fr; }
    .mission-grid { grid-template-columns: 1fr; }
  }
</style>
