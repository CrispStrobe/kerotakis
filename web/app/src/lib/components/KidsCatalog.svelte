<script lang="ts">
  import { i18n, t } from "../i18n.svelte";
  import { codexLearningLabel, guidedLearningLabel, kidsConnections, kidsExperimentMatches, kidsText, type KidsExperiment, type KidsStatus } from "../kidsCatalog";

  let { entries, capabilityIds, codexIds, completedMissions, completedExperiments, onlesson, onquest, oncapability, oncodex, onsandbox, onclose }: {
    entries: KidsExperiment[];
    capabilityIds: ReadonlySet<string>;
    codexIds: ReadonlySet<string>;
    completedMissions: ReadonlySet<string>;
    completedExperiments: ReadonlySet<string>;
    onlesson: (file: string) => void;
    onquest: (id: string) => void;
    oncapability: (id: string) => void;
    oncodex: (id: string) => void;
    onsandbox: (entry: KidsExperiment) => void;
    onclose: () => void;
  } = $props();

  const statuses: KidsStatus[] = ["computed", "partial", "boundary", "declined", "unreachable"];
  let query = $state("");
  let status = $state<KidsStatus | null>(null);
  let topic = $state("");
  const topics = $derived([...new Set(entries.flatMap((entry) => entry.topics))].sort());
  const shown = $derived(entries.filter((entry) =>
    (!status || entry.status === status) && (!topic || entry.topics.includes(topic))
      && kidsExperimentMatches(entry, query, i18n.locale),
  ));
  const linksById = $derived(new Map(entries.map((entry) => [entry.id,
    kidsConnections(entry, capabilityIds, codexIds, completedMissions, completedExperiments),
  ])));
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open aria-modal="true" aria-labelledby="kids-title" onclick={(event) => event.stopPropagation()}>
    <header>
      <div><span>{t("sixty experiments for curious kids")}</span><h1 id="kids-title">{t("Kids Lab")}</h1><p>{t("See what the bench can compute, what has a guided lesson, and where its honest boundaries are.")}</p></div>
      <strong>{shown.length}/{entries.length}</strong>
      <button aria-label={t("close")} onclick={onclose}>×</button>
    </header>
    <section class="filters">
      <input type="search" bind:value={query} placeholder={t("find an experiment, ingredient, or tool…")} aria-label={t("find a kids experiment")} />
      <div class="chips">
        <button class:on={status === null} onclick={() => (status = null)}>{t("all")}</button>
        {#each statuses as value}<button class:on={status === value} data-status={value} onclick={() => (status = status === value ? null : value)}>{t(value)}</button>{/each}
      </div>
      <select bind:value={topic} aria-label={t("topic")}><option value="">{t("all topics")}</option>{#each topics as value}<option value={value}>{t(value)}</option>{/each}</select>
    </section>
    <main>
      {#each shown as entry (entry.id)}
        {@const links = linksById.get(entry.id)!}
        <article data-status={entry.status}>
          <div class="card-head"><span class="kid-id">{entry.id}</span><span class="status">{t(entry.status)}</span><span class="safety">{entry.safety === "home" ? t("home-friendly") : t("school supervision")}</span></div>
          {#if links.linkedLearning > 0}
            <div class="learning-progress" data-progress={links.progress}>
              <span>{t(`${links.progress} linked learning`)}</span>
              <strong>{links.completedLearning}/{links.linkedLearning}</strong>
            </div>
          {/if}
          <h2>{kidsText(entry, "title", i18n.locale)}</h2><p>{kidsText(entry, "phenomenon", i18n.locale)}</p>
          <dl><div><dt>{t("ingredients")}</dt><dd>{entry.ingredients.map((value) => t(value.replaceAll("_", " "))).join(" · ")}</dd></div><div><dt>{t("apparatus")}</dt><dd>{entry.apparatus.map((value) => t(value.replaceAll("_", " "))).join(" · ")}</dd></div></dl>
          {#if entry.boundary}<p class="boundary">{kidsText(entry, "boundary", i18n.locale)}</p>{/if}
          {#if links.capabilities.length || links.codex.length || links.lessonCompleted}
            <div class="connections" aria-label={t("related learning and saved progress")}>
              {#each links.capabilities as id (id)}
                <button class="related" onclick={() => oncapability(id)}>{t("related question")} <span>{id}</span> →</button>
              {/each}
              {#each links.codex as id (id)}
                <button class="related" onclick={() => oncodex(id)}>{t(codexLearningLabel(links.codexCompleted.includes(id)))} <span>{id.replaceAll("-", " ")}</span> →</button>
              {/each}
              {#if links.lessonCompleted}<span class="saved">✓ {t("guided completion saved")}</span>{/if}
            </div>
          {/if}
          <footer>
            <span>{entry.topics.map((value) => t(value)).join(" · ")}</span>
            {#if entry.lesson}<button onclick={() => onlesson(entry.lesson!)}>{t(guidedLearningLabel(links.lessonCompleted))} →</button>{/if}
            {#if entry.quest}<button onclick={() => onquest(entry.quest!)}>{t("start quest")} →</button>{/if}
            {#if !entry.lesson && !entry.quest && (entry.status === "computed" || entry.status === "partial")}<button class="sandbox" onclick={() => onsandbox(entry)}>{t("explore in Sandbox")} →</button>{/if}
            {#if !entry.lesson && !entry.quest && entry.status !== "computed" && entry.status !== "partial"}<span class="no-launch">{t("documented boundary")}</span>{/if}
          </footer>
        </article>
      {:else}<p class="empty">{t("nothing matches that filter")}</p>{/each}
    </main>
  </dialog>
</div>

<style>
  .scrim{position:fixed;inset:0;z-index:110;display:grid;place-items:center;padding:1rem;background:var(--scrim);backdrop-filter:blur(12px)}dialog{width:min(76rem,100%);height:min(52rem,calc(100dvh - 2rem));display:grid;grid-template-rows:auto auto 1fr;overflow:hidden;padding:0;border:1px solid var(--edge);border-radius:26px;color:var(--ink);background:var(--surface);box-shadow:0 30px 90px var(--overlay-shadow)}header{display:flex;align-items:center;gap:1rem;padding:1.1rem 1.35rem;border-bottom:1px solid var(--edge);background:linear-gradient(110deg,color-mix(in srgb,var(--discovery) 12%,var(--surface)),var(--surface))}header div{flex:1}header span{color:var(--discovery);font-size:.62rem;font-weight:850;letter-spacing:.1em;text-transform:uppercase}h1{margin:.12rem 0;font-size:1.7rem}header p{margin:0;color:var(--dim);font-size:.78rem}header>strong{color:var(--discovery)}header>button{width:40px;height:40px;border:1px solid var(--edge);border-radius:50%;color:var(--ink);background:var(--surface);font-size:1.25rem;cursor:pointer}.filters{display:grid;grid-template-columns:minmax(14rem,1fr) auto 12rem;gap:.7rem;padding:.75rem 1rem;border-bottom:1px solid var(--edge);background:var(--surface-raised)}input,select{min-height:40px;padding:0 .65rem;border:1px solid var(--edge);border-radius:10px;color:var(--ink);background:var(--surface)}.chips{display:flex;gap:.3rem;overflow-x:auto}.chips button{border:1px solid var(--edge);border-radius:999px;color:var(--dim);background:var(--surface);cursor:pointer}.chips button.on{color:var(--on-accent);background:var(--primary)}main{display:grid;grid-template-columns:repeat(auto-fill,minmax(19rem,1fr));align-content:start;gap:.7rem;overflow:auto;padding:1rem}article{display:flex;flex-direction:column;padding:.85rem;border:1px solid var(--edge);border-radius:16px;background:var(--surface-raised)}article[data-status="boundary"],article[data-status="declined"],article[data-status="unreachable"]{border-style:dashed}.card-head{display:flex;align-items:center;gap:.35rem}.kid-id,.status,.safety,.no-launch{padding:.18rem .4rem;border-radius:999px;font-size:.55rem;font-weight:850;text-transform:uppercase}.kid-id{color:var(--on-accent);background:var(--primary)}.status{color:var(--discovery);background:color-mix(in srgb,var(--discovery) 12%,var(--surface))}.safety{margin-left:auto;color:var(--dim)}h2{margin:.55rem 0 .25rem;font-size:1rem}article>p{margin:0;color:var(--dim);font-size:.72rem;line-height:1.4}dl{display:grid;gap:.35rem;margin:.7rem 0}dl div{display:grid;grid-template-columns:5rem 1fr;gap:.35rem}dt{color:var(--dim);font-size:.58rem;font-weight:800;text-transform:uppercase}dd{margin:0;font-size:.64rem}.boundary{padding:.5rem;border-left:3px solid var(--warning);background:color-mix(in srgb,var(--warning) 7%,transparent)}.connections{display:flex;flex-wrap:wrap;gap:.3rem;margin:.2rem 0 .35rem}.connections .related{padding:.3rem .45rem;border:1px solid var(--edge);border-radius:8px;color:var(--ink);background:var(--surface);font-size:.6rem;cursor:pointer;text-align:left}.connections .related span{color:var(--discovery)}.saved{padding:.3rem .45rem;border-radius:8px;color:var(--success);background:color-mix(in srgb,var(--success) 8%,transparent);font-size:.6rem;font-weight:800}article footer{display:flex;align-items:center;gap:.5rem;margin-top:auto;padding-top:.65rem}article footer>span:first-child{flex:1;color:var(--dim);font-size:.58rem}article footer button{min-height:34px;border:0;border-radius:9px;color:var(--on-accent);background:var(--primary);cursor:pointer;font-weight:800}.no-launch{color:var(--dim);background:var(--surface)}.empty{grid-column:1/-1;text-align:center}@media(max-width:760px){.scrim{padding:0}dialog{width:100%;height:100dvh;border:0;border-radius:0}.filters{grid-template-columns:1fr}.chips{order:3}main{grid-template-columns:1fr}}
  .learning-progress{display:flex;align-items:center;gap:.4rem;margin-top:.5rem;color:var(--dim);font-size:.6rem}.learning-progress strong{margin-left:auto}.learning-progress[data-progress="all"]{color:var(--success)}.learning-progress[data-progress="some"]{color:var(--discovery)}
</style>
