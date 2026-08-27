<script lang="ts">
  /**
   * GUI-066: the running quest's face. The goal speaks at the dial's
   * register; claims tick off as the ENGINE satisfies them (the client
   * never grades); sealed unknowns get a naming box whose answer is
   * spoken guidance, never a block. Nudges arrive in the feed on their
   * own — this bar is the scoreboard, not the voice.
   */
  import type { Session } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let { session }: { session: Session } = $props();

  const quest = $derived(session.quest);
  const lv = $derived(session.register as "lv1" | "lv2" | "lv3");
  let alias = $state("");
  let guess = $state("");

  async function submitGuess(e: SubmitEvent) {
    e.preventDefault();
    const a = alias.trim() || (quest?.unknowns[0] ?? "");
    if (!a || !guess.trim()) return;
    await session.answerUnknown(a, guess.trim());
    guess = "";
  }
</script>

{#if quest}
  <div class="quest" class:complete={quest.complete} role="region" aria-label={t("quest {id}", { id: quest.id })}>
    <div class="head">
      <strong>{quest.title[lv] ?? quest.id}</strong>
      <span class="goal">{quest.goal[lv] ?? ""}</span>
      <button class="leave" onclick={() => void session.stopQuest()}>
        {quest.complete ? t("done — close") : t("abandon")}
      </button>
    </div>
    <ul class="claims" aria-label={t("what the bench must show")}>
      {#each quest.claims as c (c.id)}
        <li class:ok={c.satisfied}>
          {c.satisfied ? "✓" : "○"}
          {c.title[lv] ?? c.id}
        </li>
      {/each}
    </ul>
    {#if quest.unknowns.length > 0 && !quest.complete}
      <form class="identify" onsubmit={submitGuess}>
        {#if quest.unknowns.length > 1}
          <select bind:value={alias} aria-label={t("which sealed reagent")}>
            {#each quest.unknowns as u (u)}
              <option value={u}>{u}</option>
            {/each}
          </select>
        {:else}
          <span class="alias">{quest.unknowns[0]}</span>
        {/if}
        <input
          bind:value={guess}
          placeholder={t("I think it is…")}
          aria-label={t("name the sealed reagent")}
          autocomplete="off"
        />
        <button type="submit" disabled={!guess.trim()}>{t("name it")}</button>
      </form>
    {/if}
  </div>
{/if}

<style>
  .quest {
    padding: 0.45rem 1rem;
    border-bottom: 1px solid var(--cool);
    background: var(--panel);
    font-size: 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .quest.complete {
    border-bottom-color: var(--good);
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 0.7rem;
    flex-wrap: wrap;
  }
  .goal {
    color: var(--dim);
  }
  .leave {
    margin-left: auto;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.2rem 0.6rem;
    cursor: pointer;
  }
  .claims {
    list-style: none;
    display: flex;
    gap: 1rem;
    margin: 0;
    padding: 0;
    flex-wrap: wrap;
  }
  .claims li {
    color: var(--dim);
  }
  .claims li.ok {
    color: var(--good);
  }
  .identify {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  .identify .alias {
    color: var(--warn);
    font-weight: 600;
  }
  .identify input,
  .identify select {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.25rem 0.55rem;
  }
  .identify button {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.25rem 0.7rem;
    cursor: pointer;
  }
  .identify button:disabled {
    opacity: 0.5;
  }
</style>
