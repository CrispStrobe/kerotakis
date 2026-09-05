<script lang="ts">
  import { t } from "../i18n.svelte";
  import { missionId, nextUnlockedMission, storyDistricts, type MissionSummary } from "../storyProgress";
  import CaseBoard from "./CaseBoard.svelte";

  let {
    missions,
    completed,
    active = null,
    caseBriefed,
    oncasebriefed,
    onstart,
    onsandbox,
    onexperiments,
    onmap,
    onclose,
  }: {
    missions: MissionSummary[];
    completed: ReadonlySet<string>;
    active?: string | null;
    caseBriefed: boolean;
    oncasebriefed: () => void;
    onstart: (file: string) => void;
    onsandbox: () => void;
    onexperiments: () => void;
    onmap: () => void;
    onclose: () => void;
  } = $props();

  const districts = $derived(storyDistricts(missions, completed));
  let selectedId = $state("discovery-hall");
  const selected = $derived(districts.find((district) => district.id === selectedId) ?? districts[0]);
  const completedCount = $derived(completed.size);
  const nextMission = $derived(nextUnlockedMission(missions, completed, active));
  const continuing = $derived(nextMission !== null && missionId(nextMission.file) === active);
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open class="story-map" aria-labelledby="story-title" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
    <header>
      <div>
        <span class="eyebrow">{t("research campus")}</span>
        <h1 id="story-title">{t("Choose where to investigate")}</h1>
        <p>{t("Finish missions to open new districts. Choose your own route through the campus.")}</p>
      </div>
      {#if nextMission}
        <button class="next-investigation" onclick={() => onstart(nextMission.file)}>
          <small>{continuing ? t("continue investigation") : t("next investigation")}</small>
          <strong>{t(nextMission.name)}</strong><span aria-hidden="true">→</span>
        </button>
      {:else if missions.length > 0}
        <span class="all-complete">✓ {t("all missions complete")}</span>
      {/if}
      <div class="research-score" aria-label={t("{count} missions complete", { count: completedCount })}>
        <strong>{completedCount}</strong><span>{t("discoveries")}</span>
      </div>
      <button class="close" aria-label={t("close research map")} onclick={onclose}>×</button>
    </header>

    <div class="map-and-board">
      <nav class="campus" aria-label={t("campus districts")}>
        <div class="route" aria-hidden="true"></div>
        {#each districts as district, index (district.id)}
          <button
            class="district"
            class:selected={selected?.id === district.id}
            class:locked={!district.unlocked}
            style={`--district:${index}`}
            aria-pressed={selected?.id === district.id}
            onclick={() => (selectedId = district.id)}
          >
            <span class="district-icon" aria-hidden="true">{district.unlocked ? district.icon : "⌁"}</span>
            <span class="district-copy">
              <strong>{t(district.name)}</strong>
              <small>
                {#if district.unlocked}
                  {t("{done} of {total} complete", { done: district.completed, total: district.missions.length })}
                {:else}
                  {district.minimumCompleted === 1
                    ? t("complete one mission to enter")
                    : t("complete {count} missions to enter", { count: district.minimumCompleted })}
                {/if}
              </small>
            </span>
            {#if district.completed === district.missions.length && district.missions.length > 0}
              <span class="district-done" aria-label={t("district complete")}>✓</span>
            {/if}
          </button>
        {/each}
      </nav>

      {#if selected}
        <section class="mission-board" aria-live="polite">
          {#if selected.id === "discovery-hall" && selected.unlocked && selected.missions.length >= 4}
            <CaseBoard
              missions={selected.missions}
              {completed}
              {active}
              briefed={caseBriefed}
              onbriefed={oncasebriefed}
              {onstart}
            />
          {:else if selected.id === "discovery-hall" && selected.missions.length === 0}
            <div class="lock-panel"><span aria-hidden="true">◌</span><h3>{t("Case file is syncing…")}</h3><p>{t("Missions are downloading. The sandbox is ready now.")}</p></div>
          {:else}
            <div class="district-title">
              <span class="large-icon" aria-hidden="true">{selected.icon}</span>
              <div>
                <span class="eyebrow">{selected.unlocked ? t("district open") : t("district locked")}</span>
                <h2>{t(selected.name)}</h2>
                <p>{t(selected.description)}</p>
              </div>
            </div>

            {#if selected.unlocked}
              <div class="mission-list">
                {#each selected.missions as mission, index (mission.file)}
                  {@const done = completed.has(missionId(mission.file))}
                  {@const running = active === missionId(mission.file)}
                  <article class:done class:running style={`--delay:${index * 45}ms`}>
                    <span class="mission-status" aria-hidden="true">{done ? "✓" : running ? "●" : String(index + 1).padStart(2, "0")}</span>
                    <div>
                      <span class="topic">{done ? t("mission complete") : running ? t("mission in progress") : t(mission.topic ?? "more")}</span>
                      <h3>{t(mission.name)}</h3>
                      {#if mission.blurb}<p>{t(mission.blurb)}</p>{/if}
                    </div>
                    <button onclick={() => onstart(mission.file)}>
                      {running ? t("continue mission") : done ? t("replay mission") : t("launch mission")}
                      <span aria-hidden="true">→</span>
                    </button>
                  </article>
                {/each}
              </div>
            {:else}
              <div class="lock-panel">
                <span aria-hidden="true">⌁</span>
                <h3>{t("The route is still being surveyed")}</h3>
                <p>{selected.minimumCompleted === 1
                  ? t("Complete one mission anywhere in the open districts to unlock this route.")
                  : t("Complete {count} missions anywhere in the open districts to unlock this route.", { count: selected.minimumCompleted })}</p>
              </div>
            {/if}
          {/if}
        </section>
      {/if}
    </div>

    <footer>
      <span>{t("Other ways to explore")}</span>
      <button onclick={onexperiments}>⌬ {t("experiment library")}</button>
      <button onclick={onmap}>◎ {t("concept map")}</button>
      <button class="sandbox" onclick={onsandbox}>∞ {t("open full sandbox")}</button>
    </footer>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 80; display: grid; place-items: center; padding: 1rem; background: var(--scrim); backdrop-filter: blur(14px) saturate(1.15); }
  .story-map { width: min(76rem, 100%); height: min(49rem, calc(100dvh - 2rem)); display: grid; grid-template-rows: auto 1fr auto; overflow: hidden; padding: 0; border: 1px solid color-mix(in srgb, var(--primary) 35%, var(--edge)); border-radius: 28px; color: var(--ink); background: var(--surface); box-shadow: 0 34px 100px var(--overlay-shadow); }
  header { position: relative; display: flex; align-items: center; gap: 1.5rem; padding: 1.35rem 4.8rem 1.25rem 1.6rem; border-bottom: 1px solid var(--edge); background: linear-gradient(110deg, color-mix(in srgb, var(--instrument) 13%, var(--surface)), color-mix(in srgb, var(--discovery) 11%, var(--surface))); }
  h1 { margin: .2rem 0 .35rem; font-size: clamp(1.65rem, 3vw, 2.65rem); line-height: 1; letter-spacing: -.045em; }
  header p, .district-title p { max-width: 44rem; margin: 0; color: var(--dim); font-size: .88rem; }
  .eyebrow, .topic, footer > span { color: var(--discovery); font-size: .66rem; font-weight: 850; letter-spacing: .12em; text-transform: uppercase; }
  .next-investigation { max-width: 17rem; display: grid; grid-template-columns: 1fr auto; gap: .1rem .65rem; padding: .55rem .75rem; border: 1px solid color-mix(in srgb, var(--discovery) 35%, var(--edge)); border-radius: 13px; color: var(--ink); background: var(--surface); cursor: pointer; text-align: left; }
  .next-investigation small { grid-column: 1; color: var(--discovery); font-size: .58rem; font-weight: 850; letter-spacing: .08em; text-transform: uppercase; }
  .next-investigation strong { grid-column: 1; overflow: hidden; font-size: .72rem; text-overflow: ellipsis; white-space: nowrap; }
  .next-investigation span { grid-column: 2; grid-row: 1 / 3; align-self: center; color: var(--discovery); }
  .all-complete { color: var(--success); font-size: .72rem; font-weight: 850; }
  .research-score { min-width: 5rem; margin-left: auto; display: grid; justify-items: center; padding: .6rem 1rem; border: 1px solid color-mix(in srgb, var(--discovery) 35%, var(--edge)); border-radius: 16px; background: color-mix(in srgb, var(--surface) 72%, transparent); }
  .research-score strong { color: var(--discovery); font-size: 1.55rem; line-height: 1; }
  .research-score span { color: var(--dim); font-size: .65rem; }
  .close { position: absolute; top: 1rem; right: 1rem; width: 42px; height: 42px; border: 1px solid var(--edge); border-radius: 50%; color: var(--ink); background: var(--surface); cursor: pointer; font-size: 1.4rem; }
  .map-and-board { min-height: 0; display: grid; grid-template-columns: minmax(19rem, .82fr) minmax(28rem, 1.45fr); }
  .campus { position: relative; min-height: 0; display: flex; flex-direction: column; justify-content: space-around; gap: .45rem; overflow: auto; padding: 1.25rem; background: radial-gradient(circle at 10% 20%, color-mix(in srgb, var(--instrument) 18%, transparent), transparent 12rem), linear-gradient(155deg, color-mix(in srgb, var(--primary) 7%, var(--surface-raised)), var(--surface-raised)); }
  .route { position: absolute; inset: 9% auto 9% 3rem; width: 4px; border-radius: 99px; background: repeating-linear-gradient(to bottom, color-mix(in srgb, var(--primary) 45%, transparent) 0 12px, transparent 12px 20px); }
  .district { position: relative; z-index: 1; min-height: 64px; display: grid; grid-template-columns: 48px 1fr auto; align-items: center; gap: .75rem; padding: .65rem .8rem; border: 1px solid var(--edge); border-radius: 16px; color: var(--ink); background: color-mix(in srgb, var(--surface) 92%, transparent); cursor: pointer; text-align: left; transition: transform 180ms ease, border-color 180ms ease, box-shadow 180ms ease; }
  .district:hover, .district.selected { transform: translateX(5px); border-color: var(--primary); box-shadow: 0 10px 24px var(--shadow); }
  .district.selected { background: color-mix(in srgb, var(--primary) 8%, var(--surface)); }
  .district.locked { opacity: .68; filter: saturate(.65); }
  .district-icon { width: 48px; height: 48px; display: grid; place-items: center; border-radius: 15px; color: var(--on-accent); background: linear-gradient(145deg, var(--primary), var(--instrument)); font-size: 1.3rem; box-shadow: 0 5px 13px color-mix(in srgb, var(--primary) 24%, transparent); }
  .locked .district-icon { color: var(--dim); background: var(--surface-raised); box-shadow: none; }
  .district-copy { min-width: 0; display: grid; gap: .15rem; }
  .district-copy strong { font-size: .9rem; }
  .district-copy small { color: var(--dim); font-size: .68rem; }
  .district-done { width: 24px; height: 24px; display: grid; place-items: center; border-radius: 50%; color: var(--on-accent); background: var(--success); font-weight: 900; }
  .mission-board { min-width: 0; overflow: auto; padding: 1.35rem; background: radial-gradient(circle at 100% 0%, color-mix(in srgb, var(--discovery) 11%, transparent), transparent 22rem), var(--surface); }
  .district-title { display: flex; gap: .9rem; padding-bottom: 1rem; }
  .large-icon { flex: 0 0 54px; height: 54px; display: grid; place-items: center; border-radius: 18px; color: var(--action); background: color-mix(in srgb, var(--action) 12%, var(--surface)); font-size: 1.45rem; }
  h2 { margin: .15rem 0 .3rem; font-size: 1.35rem; }
  .mission-list { display: grid; gap: .65rem; }
  article { display: grid; grid-template-columns: 40px 1fr auto; align-items: center; gap: .8rem; padding: .8rem; border: 1px solid var(--edge); border-radius: 16px; background: color-mix(in srgb, var(--surface-raised) 58%, var(--surface)); animation: arrive 300ms both; animation-delay: var(--delay); }
  @keyframes arrive { from { opacity: 0; transform: translateY(7px); } }
  article.done { border-color: color-mix(in srgb, var(--success) 44%, var(--edge)); }
  article.running { border-color: var(--discovery); box-shadow: inset 4px 0 var(--discovery); }
  .mission-status { width: 36px; height: 36px; display: grid; place-items: center; border-radius: 12px; color: var(--primary); background: color-mix(in srgb, var(--primary) 10%, var(--surface)); font-size: .72rem; font-weight: 900; }
  .done .mission-status { color: var(--on-accent); background: var(--success); }
  article h3 { margin: .15rem 0; font-size: .9rem; }
  article p { max-width: 38rem; margin: 0; color: var(--dim); font-size: .7rem; }
  article button, footer button { min-height: 38px; border: 0; border-radius: 11px; color: var(--on-accent); background: var(--primary); cursor: pointer; font-weight: 800; }
  article button { display: flex; align-items: center; gap: 1rem; padding: 0 .75rem; }
  .lock-panel { min-height: 15rem; display: grid; place-items: center; align-content: center; padding: 2rem; border: 1px dashed var(--edge); border-radius: 20px; color: var(--dim); text-align: center; }
  .lock-panel span { color: var(--primary); font-size: 2.2rem; }
  .lock-panel h3 { margin: .6rem 0 .25rem; color: var(--ink); }
  .lock-panel p { max-width: 25rem; margin: 0; font-size: .82rem; }
  footer { display: flex; align-items: center; gap: .55rem; padding: .75rem 1.25rem; border-top: 1px solid var(--edge); background: var(--surface-raised); }
  footer > span { margin-right: auto; color: var(--dim); }
  footer button { padding: 0 .8rem; color: var(--primary); border: 1px solid color-mix(in srgb, var(--primary) 25%, var(--edge)); background: var(--surface); }
  footer .sandbox { color: var(--on-accent); background: var(--instrument); }
  @media (max-width: 760px) {
    .scrim { padding: 0; }
    .story-map { width: 100%; height: 100dvh; border: 0; border-radius: 0; }
    header { padding: 1rem 3.8rem 1rem 1rem; }
    .research-score { display: none; }
    .next-investigation, .all-complete { display: none; }
    .map-and-board { display: block; overflow: auto; }
    .campus { overflow: visible; }
    .mission-board { overflow: visible; }
    article { grid-template-columns: 36px 1fr; }
    article button { grid-column: 1 / -1; justify-content: space-between; }
    footer { overflow-x: auto; }
    footer > span { display: none; }
    footer button { flex: 0 0 auto; }
  }
</style>
