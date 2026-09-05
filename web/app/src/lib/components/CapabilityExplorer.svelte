<script lang="ts">
  import { untrack } from "svelte";
  import { capabilityMatches, type CapabilityPrompt, type CapabilitySupport } from "../capabilities";
  import type { Session } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let { prompts, session, onclose, initial = null }: { prompts: CapabilityPrompt[]; session: Session; onclose: () => void; initial?: string | null } = $props();
  let query = $state(untrack(() => initial ?? ""));
  let support = $state<CapabilitySupport | "all">("all");
  let topic = $state("all");
  let age = $state("all");
  let open = $state<string | null>(untrack(() => initial));
  let running = $state<string | null>(null);

  const topics = $derived([...new Set(prompts.map((prompt) => prompt.topic))].sort());
  const ages = $derived([...new Set(prompts.map((prompt) => prompt.age_band))].sort());
  const shown = $derived(prompts.filter((prompt) =>
    (support === "all" || prompt.support === support) &&
    (topic === "all" || prompt.topic === topic) &&
    (age === "all" || prompt.age_band === age) &&
    capabilityMatches(prompt, query),
  ));
  /**
   * The corpus bands ages as identifiers. `age9_to12` de-underscored is
   * "age9 to12", which is not a phrase any translator would be given; these
   * are the words the dictionary is actually keyed by.
   */
  const AGE_LABELS: Record<string, string> = {
    all: "all ages",
    age9_to12: "ages 9–12",
    age13_to15: "ages 13–15",
    age16_to18: "ages 16–18",
  };
  const ageLabel = (band: string) => t(AGE_LABELS[band] ?? band.replaceAll("_", " "));

  const counts = $derived(Object.fromEntries(
    ["computed", "curated", "qualitative", "boundary", "missing"].map((key) =>
      [key, prompts.filter((prompt) => prompt.support === key).length],
    ),
  ));

  async function run(prompt: CapabilityPrompt) {
    if (running || prompt.support === "missing") return;
    running = prompt.id;
    try {
      await session.runExperiment(prompt.script.join("\n"));
      onclose();
    } finally {
      running = null;
    }
  }
</script>

<div class="scrim" role="presentation" onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()}>
  <dialog open class="panel" aria-modal="true" aria-labelledby="capability-title" onclick={(event) => event.stopPropagation()}>
    <header>
      <div>
        <span class="eyebrow">{t("Capability explorer")}</span>
        <h2 id="capability-title">{t("What can this bench answer?")}</h2>
        <p>{t("Five hundred reviewed questions, including honest boundaries and missing science.")}</p>
      </div>
      <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    </header>

    <div class="summary" aria-label={t("support levels")}>
      {#each ["computed", "curated", "qualitative", "boundary", "missing"] as level (level)}
        <button class:active={support === level} data-support={level} onclick={() => (support = support === level ? "all" : level as CapabilitySupport)}>
          <strong>{counts[level]}</strong><span>{t(level)}</span>
        </button>
      {/each}
    </div>

    <div class="filters">
      <input type="search" bind:value={query} placeholder={t("search questions, materials, or concepts…")} aria-label={t("search capabilities")} />
      <select bind:value={topic} aria-label={t("filter by topic")}>
        <option value="all">{t("all topics")}</option>
        {#each topics as value (value)}<option value={value}>{t(value.replaceAll("_", " "))}</option>{/each}
      </select>
      <select bind:value={age} aria-label={t("filter by age")}>
        <option value="all">{t("all ages")}</option>
        {#each ages as value (value)}<option value={value}>{ageLabel(value)}</option>{/each}
      </select>
    </div>

    <p class="result-count">{t("{count} questions shown", { count: shown.length })}</p>
    <ul>
      {#each shown as prompt (prompt.id)}
        <li>
          <button class="question" onclick={() => (open = open === prompt.id ? null : prompt.id)} aria-expanded={open === prompt.id}>
            <span class="id">{prompt.id}</span>
            <strong>{t(prompt.question)}</strong>
            <span class="badge" data-support={prompt.support}>{t(prompt.support)}</span>
          </button>
          {#if open === prompt.id}
            <div class="details">
              <p>{t(prompt.topic.replaceAll("_", " "))} · {ageLabel(prompt.age_band)} · {t(prompt.material_class.replaceAll("_", " ").replaceAll("-", " "))}</p>
              <div class="tags">{#each prompt.tags as tag (tag)}<span>{t(tag.replaceAll("_", " "))}</span>{/each}</div>
              <p class="script-label">{t("bench script")}</p>
              <pre>{prompt.script.join("\n")}</pre>
              {#if prompt.support === "missing"}
                <p class="boundary">{t("Not runnable yet: {reason}", { reason: prompt.boundary ?? prompt.reason_code })} · {t("owner: {task}", { task: prompt.owning_task })}</p>
              {:else}
                <button class="run" disabled={running !== null || session.busy} onclick={() => void run(prompt)}>
                  {running === prompt.id ? t("running…") : t("try this question on the bench")}
                </button>
              {/if}
            </div>
          {/if}
        </li>
      {:else}
        <li class="empty">{t("No question matches these filters.")}</li>
      {/each}
    </ul>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 55; display: grid; place-items: center; padding: 1rem; background: var(--scrim); }
  .panel { width: min(58rem, 96vw); max-height: 92dvh; overflow: auto; padding: 1rem; border: 1px solid var(--edge); border-radius: 18px; color: var(--ink); background: var(--bg); }
  header { display: flex; justify-content: space-between; gap: 1rem; }
  header h2 { margin: .2rem 0; font-size: 1.45rem; } header p, .result-count, .details p { margin: .2rem 0; color: var(--dim); font-size: .78rem; }
  .eyebrow { color: var(--discovery); font-size: .65rem; font-weight: 850; letter-spacing: .12em; text-transform: uppercase; }
  select, input, .summary button { border: 1px solid var(--edge); border-radius: 8px; color: var(--ink); background: var(--panel-raised); font: inherit; }
  .icon-close { align-self: start; }
  .summary { display: grid; grid-template-columns: repeat(5, 1fr); gap: .35rem; margin: 1rem 0; }
  .summary button { display: grid; padding: .45rem; cursor: pointer; text-align: left; } .summary button.active { outline: 2px solid var(--action); }
  .summary strong { font-size: 1.05rem; } .summary span { color: var(--dim); font-size: .65rem; }
  [data-support="computed"], [data-support="curated"] { color: var(--success); } [data-support="boundary"], [data-support="missing"] { color: var(--warning); }
  .filters { display: grid; grid-template-columns: minmax(14rem, 1fr) auto auto; gap: .4rem; } input, select { padding: .55rem; }
  ul { list-style: none; margin: .4rem 0 0; padding: 0; } li { border-bottom: 1px solid var(--edge); }
  .question { width: 100%; display: grid; grid-template-columns: 4.2rem 1fr auto; gap: .6rem; align-items: center; padding: .7rem .2rem; border: 0; color: var(--ink); background: none; text-align: left; cursor: pointer; }
  .id { color: var(--dim); font-family: ui-monospace, monospace; font-size: .7rem; } .question strong { font-size: .84rem; }
  .badge { padding: .18rem .38rem; border: 1px solid currentColor; border-radius: 999px; font-size: .6rem; }
  .details { padding: 0 .4rem .8rem 4.8rem; } .tags { display: flex; flex-wrap: wrap; gap: .25rem; margin: .45rem 0; }
  .tags span { padding: .15rem .35rem; border-radius: 5px; color: var(--dim); background: var(--panel-raised); font-size: .62rem; }
  pre { overflow-x: auto; padding: .6rem; border-radius: 8px; background: var(--panel-raised); font-size: .7rem; }
  .run { padding: .45rem .7rem; border: 0; border-radius: 8px; color: var(--on-accent); background: var(--primary); cursor: pointer; font-weight: 750; }
  .boundary { color: var(--warning) !important; }
  .script-label { margin: .45rem 0 .2rem !important; color: var(--dim); font-size: .6rem !important; font-weight: 850; letter-spacing: .1em; text-transform: uppercase; }
  .empty { padding: .9rem .2rem; color: var(--dim); font-size: .8rem; }
  @media (max-width: 680px) { .summary { grid-template-columns: repeat(3, 1fr); } .filters { grid-template-columns: 1fr; } .question { grid-template-columns: 3.5rem 1fr; } .badge { grid-column: 2; justify-self: start; } .details { padding-left: 4.1rem; } }
</style>
