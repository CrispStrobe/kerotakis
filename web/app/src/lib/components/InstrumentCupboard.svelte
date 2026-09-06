<script lang="ts">
  /**
   * The one cupboard (GUI-101).
   *
   * Instruments were offered in three places — the MESSEN strip, the
   * Geräteschrank and the vessel dock — with three different ideas of what
   * a tool is. This is the single surface all three collapse into: shelves
   * of rendered items, grouped by what the thing DOES, each with its name
   * under the picture and an (i) that says what the model computes and what
   * it does not.
   *
   * It owns no equipment knowledge. `equipmentCatalogue.ts` is the list, the
   * engine's catalog is the authority on what is reachable, and every
   * selection is handed straight to the handler that already existed for it.
   */
  import { t } from "../i18n.svelte";
  import ToolIcon from "./ToolIcon.svelte";
  import InfoPanel from "./InfoPanel.svelte";
  import InfoToggle from "./InfoToggle.svelte";
  import { equipmentAccess, requirement, type CatalogMap } from "../catalogProgress";
  import { equipmentMatches } from "../catalogSearch";
  import type { CatalogScope } from "../catalogScope";
  import type { LabMode } from "../worldState";
  import type { TwoVesselAction } from "../directActions";
  import {
    EQUIPMENT_CATALOGUE,
    EQUIPMENT_GROUPS,
    GROUP_LABELS,
    accessId,
    deployedLabel,
    equipmentInfoRows,
    gatedIds,
    runEquipment,
    type EquipmentEntry,
    type EquipmentGroup,
  } from "../equipmentCatalogue";

  let {
    target,
    targetLabel,
    catalog,
    mode,
    scope = "all",
    missionVerbs = [],
    reactAvailable,
    apparatusOut,
    buretteOut,
    transferVerb,
    mixActive,
    busy = false,
    onmeasure,
    onapparatus,
    ontransfer,
    onmix,
    onburette,
    onclose,
  }: {
    target: number;
    targetLabel: string;
    /** WORLD-003: what the ENGINE says is reachable, indexed by id. */
    catalog: CatalogMap;
    mode: LabMode;
    scope?: CatalogScope;
    missionVerbs?: string[];
    reactAvailable: boolean;
    apparatusOut: string | null;
    buretteOut: boolean;
    transferVerb: TwoVesselAction | null;
    mixActive: boolean;
    busy?: boolean;
    onmeasure: (line: string) => void;
    onapparatus: (verb: string, preset?: Record<string, string | number>) => void;
    ontransfer: (verb: TwoVesselAction) => void;
    onmix: () => void;
    onburette: () => void;
    onclose: () => void;
  } = $props();

  let filter = $state("");
  /** One explanation at a time: several open panels is the wall of text the
   * (i) exists to prevent. */
  let openInfo = $state<string | null>(null);
  const panelId = (id: string) => `cupboard-info-${id}`;

  const ids = $derived(gatedIds(reactAvailable));
  const availableCount = $derived(ids.filter((id) => equipmentAccess(catalog, id).available).length);
  const accessOf = (entry: EquipmentEntry) => equipmentAccess(catalog, accessId(entry));
  const inScope = (entry: EquipmentEntry) => {
    const id = accessId(entry);
    if (scope === "all") return true;
    if (scope === "mission") return missionVerbs.includes(id);
    return mode === "sandbox" || equipmentAccess(catalog, id).available;
  };
  const shown = $derived(
    EQUIPMENT_CATALOGUE.filter((entry) =>
      (entry.id !== "react" || reactAvailable)
      && inScope(entry)
      && equipmentMatches(
        { verb: entry.id, title: entry.name, blurb: entry.blurb },
        filter,
        t(entry.name),
        `${t(entry.blurb)} ${t(entry.boundary)}`,
      )),
  );
  const inGroup = (group: EquipmentGroup) => shown.filter((entry) => entry.group === group);
  const deployment = $derived({ apparatusOut, buretteOut, transferVerb, mixActive });
  const requirementLabel = (id: string) => {
    const count = requirement(catalog, id);
    // Silent rather than guessing while the engine has not answered.
    if (count === null) return "";
    return count === 1 ? t("after one mission") : t("after {count} missions", { count });
  };
  /** The transfer verbs and the mixer pick their own vessels on the bench,
   * so while one is armed the selected vessel is not where the next thing
   * goes and the target card would be a wrong answer rather than a quiet one. */
  const namesNextTarget = $derived(transferVerb === null && !mixActive);

  function choose(entry: EquipmentEntry) {
    runEquipment(entry, target, { onmeasure, onapparatus, ontransfer, onmix, onburette });
    onclose();
  }
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(event) => event.key === "Escape" && onclose()}
>
  <dialog
    open
    class="cupboard"
    aria-modal="true"
    aria-labelledby="cupboard-title"
    onclick={(event) => event.stopPropagation()}
  >
    <header>
      <span class="mark" aria-hidden="true">▦</span>
      <span class="titles">
        <small>{t("lab wall utility")}</small>
        <h2 id="cupboard-title">{t("equipment cabinet")}</h2>
      </span>
      <b title={t("{available} of {total} instruments unlocked", { available: availableCount, total: ids.length })}>{availableCount}/{ids.length}</b>
      <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
    </header>

    <div class="controls">
      {#if namesNextTarget}
        <p class="target-card">
          <span class="target-orbit" aria-hidden="true"><span></span></span>
          <span><small>{t("next instrument installs on")}</small><strong>v{target + 1} · {t(targetLabel)}</strong></span>
        </p>
      {/if}
      <label class="equipment-search">
        <span aria-hidden="true">⌕</span>
        <input bind:value={filter} placeholder={t("filter…")} aria-label={`${t("filter…")} ${t("equipment")}`} />
        {#if filter}<button type="button" onclick={() => (filter = "")} aria-label={t("clear")}>×</button>{/if}
      </label>
      {#if mode !== "sandbox"}
        <p class="lead">{t("Complete investigations to earn permanent access to more instruments.")}</p>
      {/if}
    </div>

    <div class="shelves">
      {#each EQUIPMENT_GROUPS as group (group)}
        {@const entries = inGroup(group)}
        {#if entries.length > 0}
          <section class="shelf" class:sets={group === "sets"} aria-label={t(GROUP_LABELS[group])}>
            <h3><span>{t(GROUP_LABELS[group])}</span><small>{entries.length}</small></h3>
            <div class="shelf-items">
              {#each entries as entry (entry.id)}
                {@const entryAccess = accessOf(entry)}
                {@const badge = deployedLabel(entry, deployment)}
                <div class="slot" class:open={openInfo === entry.id}>
                  <button
                    class="item"
                    class:locked={!entryAccess.available}
                    class:deployed={badge !== null}
                    aria-pressed={badge !== null}
                    disabled={busy || !entryAccess.available}
                    onclick={() => choose(entry)}
                  >
                    <span class="item-render">
                      {#if entry.render.kind === "icon"}
                        <ToolIcon name={entry.render.name} />
                      {:else}
                        <span class="glyph" class:word={entry.render.text.length > 1} aria-hidden="true">{entry.render.text}</span>
                      {/if}
                    </span>
                    <span class="item-name">{t(entry.name)}</span>
                    {#if badge}<span class="deployed-label">{t(badge)}</span>{/if}
                    {#if entryAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
                    {#if !entryAccess.available}<span class="locked-label">⌁ {requirementLabel(accessId(entry))}</span>{/if}
                  </button>
                  <InfoToggle
                    expanded={openInfo === entry.id}
                    controls={panelId(entry.id)}
                    label={t("about {name}", { name: t(entry.name) })}
                    onclick={() => (openInfo = openInfo === entry.id ? null : entry.id)}
                  />
                </div>
                {#if openInfo === entry.id}
                  <div class="slot-info">
                    <p class="purpose">{t(entry.blurb)}</p>
                    <InfoPanel id={panelId(entry.id)} rows={equipmentInfoRows(entry, t)} />
                  </div>
                {/if}
              {/each}
            </div>
            <span class="board" aria-hidden="true"></span>
          </section>
        {/if}
      {/each}
      {#if shown.length === 0}
        <p class="empty-scope">
          {#if scope === "mission" && missionVerbs.length === 0}
            {t("This mission needs no additional cabinet equipment.")}
          {:else}
            {t("nothing matches that filter")}
          {/if}
        </p>
      {/if}
    </div>
  </dialog>
</div>

<style>
  .scrim { position: fixed; inset: 0; z-index: 84; display: grid; place-items: center; padding: 1rem; background: var(--scrim); backdrop-filter: blur(10px) saturate(1.12); }
  .cupboard { position: static; width: min(56rem, 96vw); max-height: 92vh; margin: 0; padding: 0; display: flex; flex-direction: column; overflow: hidden; border: 1px solid color-mix(in srgb, var(--action) 40%, var(--edge)); border-radius: 23px; color: var(--ink); background: var(--surface); box-shadow: 0 28px 80px var(--overlay-shadow); }
  header { display: flex; align-items: center; gap: .75rem; padding: .9rem 1rem; background: linear-gradient(110deg, color-mix(in srgb, var(--action) 15%, var(--surface)), color-mix(in srgb, var(--instrument) 10%, var(--surface))); }
  .mark { width: 40px; height: 40px; display: grid; place-items: center; flex: none; border-radius: 12px; color: var(--on-accent); background: linear-gradient(145deg, var(--action), var(--instrument)); font-size: 1.2rem; }
  .titles { min-width: 0; display: grid; gap: .05rem; }
  .titles small { color: var(--instrument); font-size: .55rem; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.05rem; }
  header b { margin-left: auto; color: var(--instrument); font-size: .72rem; }
  .controls { display: grid; gap: .5rem; padding: .7rem 1rem .2rem; }
  .lead { margin: 0; color: var(--dim); font-size: .67rem; line-height: 1.35; }
  .target-card { display: flex; align-items: center; gap: .6rem; margin: 0; padding: .5rem .6rem; border: 1px solid color-mix(in srgb, var(--instrument) 30%, var(--edge)); border-radius: 13px; background: color-mix(in srgb, var(--instrument) 8%, var(--surface-raised)); }
  .target-card > span:last-child { min-width: 0; display: flex; flex-direction: column; }
  .target-card small { color: var(--instrument); font-size: .55rem; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
  .target-card strong { overflow: hidden; font-size: .72rem; text-overflow: ellipsis; white-space: nowrap; }
  .target-orbit { width: 28px; height: 28px; display: grid; place-items: center; flex: none; border: 1px solid color-mix(in srgb, var(--instrument) 45%, transparent); border-radius: 50%; }
  .target-orbit span { width: 8px; height: 8px; border: 2px solid var(--surface); border-radius: 50%; background: var(--instrument); box-shadow: 0 0 0 2px var(--instrument); }
  .equipment-search { min-height: 38px; display: grid; grid-template-columns: auto 1fr auto; align-items: center; gap: .4rem; padding: 0 .55rem; border: 1px solid var(--edge); border-radius: 11px; color: var(--dim); background: var(--surface-raised); }
  .equipment-search:focus-within { border-color: var(--primary); box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 18%, transparent); }
  .equipment-search input { min-width: 0; border: 0; outline: 0; color: var(--ink); background: transparent; font: inherit; font-size: .72rem; }
  .equipment-search input::placeholder { color: var(--dim); }
  .equipment-search button { width: 25px; height: 25px; border: 0; border-radius: 50%; color: var(--dim); background: transparent; cursor: pointer; font-size: 1rem; }
  .shelves { min-height: 0; padding: .5rem 1rem 1.1rem; overflow-y: auto; }
  /* A shelf, drawn as one: the items stand ON a board rather than floating
     in a grid, because "which shelf is it on" is the question the grouping
     is answering and a bare grid does not ask it. */
  .shelf { margin-bottom: 1.2rem; }
  .shelf h3 { display: flex; align-items: center; justify-content: space-between; margin: 0 0 .4rem; color: var(--dim); font-size: .62rem; font-weight: 800; letter-spacing: .07em; text-transform: uppercase; }
  .shelf h3 small { min-width: 1.35rem; padding: .12rem .3rem; border-radius: 999px; background: var(--surface-raised); text-align: center; }
  .shelf-items { display: grid; grid-template-columns: repeat(auto-fill, minmax(6.4rem, 1fr)); gap: .4rem; align-items: end; }
  .board { display: block; height: 6px; margin-top: .2rem; border-radius: 3px; background: linear-gradient(180deg, color-mix(in srgb, var(--action) 26%, var(--surface-raised)), var(--surface-raised)); box-shadow: 0 3px 7px var(--shadow); }
  .sets { padding: .5rem; border: 1px solid color-mix(in srgb, var(--discovery) 25%, var(--edge)); border-radius: 14px; background: color-mix(in srgb, var(--discovery) 5%, transparent); }
  .slot { min-width: 0; display: flex; flex-direction: column; align-items: center; gap: .1rem; }
  .item { position: relative; width: 100%; min-height: 84px; display: flex; flex-direction: column; align-items: center; justify-content: flex-start; gap: .3rem; padding: .5rem .35rem; overflow: hidden; border: 1px solid var(--edge); border-radius: 12px; color: var(--ink); background: linear-gradient(160deg, var(--surface), color-mix(in srgb, var(--surface-raised) 78%, var(--surface))); cursor: pointer; font: inherit; text-align: center; }
  .item:hover:not(:disabled) { border-color: var(--action); transform: translateY(-2px); box-shadow: 0 7px 16px var(--shadow); }
  .item.deployed { border-color: var(--action); background: color-mix(in srgb, var(--action) 9%, var(--surface)); }
  .item.locked, .item:disabled { opacity: .78; filter: saturate(.55); cursor: not-allowed; }
  .item.locked:hover { transform: none; border-color: var(--edge); box-shadow: none; }
  .item-render { width: 38px; height: 38px; display: grid; place-items: center; flex: none; border-radius: 11px; color: var(--action); background: color-mix(in srgb, var(--action) 10%, var(--surface)); }
  .item-render :global(svg) { width: 26px; height: 26px; margin: 0; }
  .glyph { font-size: 1.1rem; line-height: 1; }
  /* pH, mL, λ, Rf are read rather than looked at; they want the steadier
     typographic treatment the emoji do not. */
  .glyph.word { font-size: .74rem; font-weight: 800; letter-spacing: .02em; }
  .item-name { font-size: .64rem; line-height: 1.2; overflow-wrap: anywhere; }
  .deployed-label { position: absolute; top: .25rem; right: .25rem; padding: .12rem .3rem; border-radius: 999px; color: var(--on-accent); background: var(--action); font-size: .45rem; font-weight: 800; letter-spacing: .04em; text-transform: uppercase; }
  .locked-label { margin-top: auto; padding: .16rem .3rem; border-radius: 7px; color: var(--dim); background: color-mix(in srgb, var(--surface-raised) 90%, transparent); font-size: .5rem; font-weight: 800; line-height: 1.2; }
  .loaned-label { margin-top: auto; padding: .16rem .3rem; border-radius: 999px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 10%, var(--surface)); font-size: .46rem; font-weight: 800; letter-spacing: .04em; text-transform: uppercase; }
  /* The explanation takes the whole row: a sentence in a 6 rem column is a
     column of two-word lines. */
  .slot-info { grid-column: 1 / -1; }
  .purpose { margin: .3rem .2rem .1rem; color: var(--dim); font-size: .68rem; line-height: 1.35; }
  .empty-scope { margin: 1rem .2rem; color: var(--dim); font-size: .72rem; line-height: 1.4; }
  @media (max-width: 30rem) {
    .cupboard { width: 100vw; max-height: 100vh; border-radius: 0; }
    .shelf-items { grid-template-columns: repeat(auto-fill, minmax(5.2rem, 1fr)); }
  }
</style>
