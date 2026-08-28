<script lang="ts">
  import { t } from "../i18n.svelte";
  import { missionId, type MissionSummary } from "../storyProgress";
  import { contaminatedSampleLeads, contaminatedSampleProgress } from "../storyChapter";

  let {
    missions,
    completed,
    active = null,
    briefed,
    onbriefed,
    onstart,
  }: {
    missions: MissionSummary[];
    completed: ReadonlySet<string>;
    active?: string | null;
    briefed: boolean;
    onbriefed: () => void;
    onstart: (file: string) => void;
  } = $props();

  const leads = $derived(contaminatedSampleLeads(missions, completed));
  const progress = $derived(contaminatedSampleProgress(leads));
  const core = $derived(leads.filter((lead) => !lead.optional));
  const optional = $derived(leads.find((lead) => lead.optional) ?? null);
</script>

<section class="case-board" aria-labelledby="case-title">
  <header class="case-header">
    <div class="sample" aria-hidden="true"><span class="cap"></span><span class="liquid"><i></i><b></b></span></div>
    <div class="case-title">
      <span>{t("case 01 · field sample")}</span>
      <h2 id="case-title">{t("The contaminated sample")}</h2>
      <p>{t("A cloudy water sample arrived without a trustworthy label. Build an evidence trail before the campus reopens its supply line.")}</p>
    </div>
    <div class="case-progress" aria-label={t("{done} of {total} core leads complete", { done: progress.done, total: progress.total })}>
      <strong>{progress.done}/{progress.total}</strong><span>{t("core leads")}</span>
      <div aria-hidden="true">{#each core as lead}<i class:done={lead.done}></i>{/each}</div>
    </div>
  </header>

  {#if !briefed}
    <div class="briefing">
      <div class="contact" aria-hidden="true"><span>AK</span><i></i></div>
      <div class="message">
        <span>{t("Dr Ada Keller · campus chemist")}</span>
        <h3>{t("We need evidence, not a guess.")}</h3>
        <p>{t("Choose any of the three leads first. Each uses the same real laboratory, and your results stay on the bench when you return to this board.")}</p>
        <div class="brief-facts">
          <span>◆ {t("three routes, any order")}</span>
          <span>⌬ {t("one optional safety lead")}</span>
          <span>∞ {t("Sandbox stays separate")}</span>
        </div>
        <button onclick={onbriefed}>{t("open the case file")} <span aria-hidden="true">→</span></button>
      </div>
    </div>
  {:else}
    {#if progress.complete}
      <div class="case-complete" role="status"><span aria-hidden="true">✓</span><p><strong>{t("core evidence assembled")}</strong>{t("The sample now has a defensible evidence trail. The optional safety audit remains available.")}</p></div>
    {/if}

    <div class="lead-heading"><div><span>{t("case leads")}</span><h3>{t("Choose the next investigation")}</h3></div><small>{t("any order")}</small></div>
    <div class="lead-grid">
      {#each core as lead, index (lead.id)}
        {@const running = active === missionId(lead.mission.file)}
        <article class:done={lead.done} class:running style={`--delay:${index * 55}ms`}>
          <div class="lead-number" aria-hidden="true">{lead.done ? "✓" : String(index + 1).padStart(2, "0")}</div>
          <span class="lead-state">{lead.done ? t("evidence secured") : running ? t("investigation active") : t("open lead")}</span>
          <h4>{t(lead.objective)}</h4>
          <p>{t(lead.evidence)}</p>
          {#if lead.outcomeAssessed}<span class="assessment"><i aria-hidden="true"></i>{t("solver-assessed outcome")}</span>{/if}
          <small>{t(lead.mission.name)}</small>
          <button onclick={() => onstart(lead.mission.file)}>
            {running ? t("continue investigation") : lead.done ? t("review investigation") : t("investigate")}
            <span aria-hidden="true">→</span>
          </button>
        </article>
      {/each}
    </div>

    {#if optional}
      {@const running = active === missionId(optional.mission.file)}
      <article class="optional" class:done={optional.done} class:running>
        <span class="optional-icon" aria-hidden="true">⚠</span>
        <div><span>{t("optional discovery")}</span><h4>{t(optional.objective)}</h4><p>{t(optional.evidence)}</p></div>
        <button onclick={() => onstart(optional.mission.file)}>{running ? t("continue") : optional.done ? t("review") : t("inspect")} <span aria-hidden="true">→</span></button>
      </article>
    {/if}
  {/if}
</section>

<style>
  .case-board { display: grid; gap: 1rem; }
  .case-header { display: grid; grid-template-columns: 72px 1fr auto; align-items: center; gap: 1rem; padding: 1rem; overflow: hidden; border: 1px solid color-mix(in srgb, var(--discovery) 38%, var(--edge)); border-radius: 20px; background: radial-gradient(circle at 8% 0%, color-mix(in srgb, var(--action) 15%, transparent), transparent 10rem), linear-gradient(125deg, color-mix(in srgb, var(--discovery) 8%, var(--surface)), var(--surface)); }
  .sample { position: relative; width: 58px; height: 72px; margin: auto; filter: drop-shadow(0 8px 8px rgb(10 38 48 / 20%)); }
  .sample .cap { position: absolute; z-index: 2; top: 0; left: 13px; width: 32px; height: 13px; border: 3px solid color-mix(in srgb, var(--primary) 65%, white); border-radius: 7px 7px 3px 3px; background: var(--primary); }
  .sample .liquid { position: absolute; inset: 9px 4px 0; overflow: hidden; border: 3px solid color-mix(in srgb, var(--primary) 38%, white); border-radius: 10px 10px 17px 17px; background: color-mix(in srgb, var(--surface) 72%, transparent); }
  .sample .liquid::before { content: ""; position: absolute; inset: 29px -5px -5px; background: linear-gradient(#8fd9cc, #4ba88e 62%, #81704d); animation: sample-wave 3.4s ease-in-out infinite alternate; }
  .sample i, .sample b { position: absolute; z-index: 1; width: 7px; height: 7px; border-radius: 50%; background: rgb(255 255 255 / 58%); animation: bubble 2.4s ease-in infinite; }
  .sample i { left: 10px; bottom: 7px; }.sample b { right: 9px; bottom: 15px; animation-delay: -1.1s; }
  @keyframes sample-wave { to { transform: translateY(-3px) rotate(2deg); } }
  @keyframes bubble { to { transform: translateY(-26px) scale(.6); opacity: 0; } }
  .case-title > span, .lead-state, .lead-heading span, .optional div > span, .message > span { color: var(--discovery); font-size: .61rem; font-weight: 850; letter-spacing: .1em; text-transform: uppercase; }
  .case-title h2 { margin: .2rem 0 .35rem; font-size: 1.35rem; }
  .case-title p { max-width: 40rem; margin: 0; color: var(--dim); font-size: .78rem; line-height: 1.4; }
  .case-progress { min-width: 5rem; display: grid; justify-items: center; gap: .15rem; }
  .case-progress strong { color: var(--discovery); font-size: 1.35rem; }.case-progress > span { color: var(--dim); font-size: .58rem; }
  .case-progress div { display: flex; gap: .25rem; }.case-progress i { width: 18px; height: 5px; border-radius: 99px; background: var(--edge); }.case-progress i.done { background: var(--success); }
  .briefing { display: grid; grid-template-columns: 76px 1fr; gap: 1rem; padding: 1.15rem; border: 1px solid var(--edge); border-radius: 20px; background: color-mix(in srgb, var(--surface-raised) 70%, var(--surface)); animation: arrive 380ms both; }
  .contact { position: relative; width: 68px; height: 68px; display: grid; place-items: center; border-radius: 22px; color: var(--on-accent); background: linear-gradient(145deg, var(--primary), var(--instrument)); font-size: 1.1rem; font-weight: 900; }.contact i { position: absolute; right: -2px; bottom: -2px; width: 18px; height: 18px; border: 3px solid var(--surface); border-radius: 50%; background: var(--success); }
  .message h3 { margin: .25rem 0 .35rem; font-size: 1.15rem; }.message p { margin: 0; color: var(--dim); font-size: .82rem; line-height: 1.45; }
  .brief-facts { display: flex; flex-wrap: wrap; gap: .35rem; margin: .75rem 0; }.brief-facts span { padding: .25rem .45rem; border-radius: 999px; color: var(--dim); background: var(--surface); font-size: .62rem; }
  button { min-height: 38px; border: 0; border-radius: 11px; color: var(--on-accent); background: var(--primary); cursor: pointer; font-weight: 800; }.message button { min-width: 13rem; display: flex; align-items: center; justify-content: space-between; padding: 0 .8rem; }
  .case-complete { display: flex; align-items: center; gap: .7rem; padding: .7rem .85rem; border: 1px solid color-mix(in srgb, var(--success) 35%, var(--edge)); border-radius: 15px; background: color-mix(in srgb, var(--success) 8%, var(--surface)); }.case-complete > span { width: 34px; height: 34px; display: grid; place-items: center; flex: none; border-radius: 12px; color: var(--on-accent); background: var(--success); font-weight: 900; }.case-complete p { display: grid; margin: 0; color: var(--dim); font-size: .68rem; }.case-complete strong { color: var(--success); font-size: .66rem; letter-spacing: .06em; text-transform: uppercase; }
  .lead-heading { display: flex; align-items: end; justify-content: space-between; }.lead-heading h3 { margin: .15rem 0 0; font-size: 1.05rem; }.lead-heading small { color: var(--dim); font-size: .65rem; }
  .lead-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: .65rem; }
  .lead-grid article { min-height: 13.5rem; display: flex; flex-direction: column; padding: .8rem; border: 1px solid var(--edge); border-radius: 16px; background: color-mix(in srgb, var(--surface-raised) 60%, var(--surface)); animation: arrive 330ms both; animation-delay: var(--delay); }.lead-grid article:hover { transform: translateY(-2px); border-color: var(--primary); box-shadow: 0 10px 22px var(--shadow); }.lead-grid article.done { border-color: color-mix(in srgb, var(--success) 42%, var(--edge)); }.lead-grid article.running { border-color: var(--discovery); box-shadow: inset 0 4px var(--discovery); }
  @keyframes arrive { from { opacity: 0; transform: translateY(8px); } }
  .lead-number { width: 34px; height: 34px; display: grid; place-items: center; margin-bottom: .75rem; border-radius: 11px; color: var(--primary); background: color-mix(in srgb, var(--primary) 10%, var(--surface)); font-size: .7rem; font-weight: 900; }.done .lead-number { color: var(--on-accent); background: var(--success); }
  .lead-grid h4, .optional h4 { margin: .25rem 0; font-size: .88rem; }.lead-grid p, .optional p { margin: 0; color: var(--dim); font-size: .68rem; line-height: 1.35; }.lead-grid article > small { margin-top: .55rem; color: var(--dim); font-size: .58rem; }.lead-grid button { display: flex; align-items: center; justify-content: space-between; margin-top: auto; padding: 0 .65rem; }
  .assessment { display: inline-flex; align-items: center; align-self: flex-start; gap: .35rem; margin-top: .55rem; padding: .25rem .45rem; border-radius: 999px; color: var(--success); background: color-mix(in srgb, var(--success) 10%, var(--surface)); font-size: .58rem; font-weight: 800; }.assessment i { width: 7px; height: 7px; border-radius: 50%; background: currentColor; }
  .optional { display: grid; grid-template-columns: 42px 1fr auto; align-items: center; gap: .7rem; padding: .7rem; border: 1px dashed color-mix(in srgb, var(--warning) 55%, var(--edge)); border-radius: 15px; background: color-mix(in srgb, var(--warning) 6%, var(--surface)); }.optional-icon { width: 40px; height: 40px; display: grid; place-items: center; border-radius: 12px; color: var(--warning); background: color-mix(in srgb, var(--warning) 12%, var(--surface)); }.optional button { min-width: 7rem; padding: 0 .65rem; }
  @media (max-width: 900px) { .lead-grid { grid-template-columns: 1fr; }.lead-grid article { min-height: 10rem; }.case-header { grid-template-columns: 58px 1fr; }.case-progress { grid-column: 1 / -1; display: flex; justify-content: center; }.case-progress div { margin-left: .4rem; } }
  @media (max-width: 520px) { .briefing { grid-template-columns: 1fr; }.contact { width: 54px; height: 54px; }.optional { grid-template-columns: 38px 1fr; }.optional button { grid-column: 1 / -1; }.sample { transform: scale(.82); } }
  @media (prefers-reduced-motion: reduce) { .sample .liquid::before, .sample i, .sample b, .lead-grid article, .briefing { animation: none; } }
</style>
