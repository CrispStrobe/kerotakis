<script lang="ts">
  /**
   * The shelf pane's equipment tab, now rendering the ONE catalogue.
   *
   * This component used to hold its own idea of what equipment exists: an
   * inline `TRANSFER_TOOLS` array, four hand-written special cards, and a
   * second full listing of the twelve instruments the MESSEN strip already
   * showed. Four `{#each}` blocks over four sources, each with its own copy
   * of the locked/loaned/deployed markup.
   *
   * It now reads `equipmentCatalogue.ts`, which is the same list
   * `InstrumentCupboard.svelte` shows — so the two surfaces cannot drift
   * while both exist, and GUI-102 can delete this one without deleting
   * knowledge. The props are unchanged.
   */
  import { t } from "../i18n.svelte";
  import ToolIcon from "./ToolIcon.svelte";
  import { equipmentAccess, requirement, type CatalogMap } from "../catalogProgress";
  import type { LabMode } from "../worldState";
  import type { CatalogScope } from "../catalogScope";
  import { equipmentMatches } from "../catalogSearch";
  import type { TwoVesselAction } from "../directActions";
  import InfoPanel from "./InfoPanel.svelte";
  import InfoToggle from "./InfoToggle.svelte";
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
    buretteOut,
    apparatusOut,
    transferVerb,
    mixActive,
    reactAvailable,
    mode,
    completed,
    scope = "all",
    missionVerbs = [],
    catalog,
    onburette,
    onapparatus,
    ontransfer,
    onmix,
    onmeasure,
    busy = false,
  }: {
    target: number;
    targetLabel: string;
    buretteOut: boolean;
    apparatusOut: string | null;
    transferVerb: TwoVesselAction | null;
    mixActive: boolean;
    reactAvailable: boolean;
    mode: LabMode;
    /** Kept for the caller's shape; the tally is the engine's answer now. */
    completed: number;
    scope?: CatalogScope;
    missionVerbs?: string[];
    /** WORLD-003: what the ENGINE says is reachable, indexed by id. The
     * cabinet no longer computes this — it asks, and shows the answer. */
    catalog: CatalogMap;
    onburette: () => void;
    /** `preset` seeds the panel's form — a kit that IS a candle says so. */
    onapparatus: (verb: string, preset?: Record<string, string | number>) => void;
    ontransfer: (verb: TwoVesselAction) => void;
    onmix: () => void;
    onmeasure: (line: string) => void;
    busy?: boolean;
  } = $props();

  const ids = $derived(gatedIds(reactAvailable));
  // `equipmentAccess`, not `access`: a verb the engine does not tier at all
  // (the cooling bath's `cool` is a bench control, not earned equipment) has
  // no catalog row, and reading that silence as a refusal disabled the card
  // in Sandbox and made this tally read 33/34 under a sentence saying
  // everything was available. See catalogProgress.ts for the whole story.
  const availableCount = $derived(ids.filter((id) => equipmentAccess(catalog, id).available).length);
  const accessOf = (entry: EquipmentEntry) => equipmentAccess(catalog, accessId(entry));
  let filter = $state("");
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
  /** The mixer and the transfer tools choose their own vessels on the
   * bench; while one of them is armed, the selected vessel is not where the
   * next thing goes, so the card would be a wrong answer rather than a
   * quiet one. */
  const namesNextTarget = $derived(transferVerb === null && !mixActive);
  /** One explanation open at a time, like the shelf: several at once is the
   * wall of text the (i) exists to prevent. */
  let openInfo = $state<string | null>(null);
  const panelId = (id: string) => `equipment-info-${id}`;
  const choose = (entry: EquipmentEntry) =>
    runEquipment(entry, target, { onmeasure, onapparatus, ontransfer, onmix, onburette });
</script>

<section class="equipment-cabinet" aria-label={t("equipment") }>
  {#if namesNextTarget}
    <!-- What this pane is FOR: everything below installs on, or measures,
         one vessel, and this names it. -->
    <div class="target-card">
      <span class="target-orbit" aria-hidden="true"><span></span></span>
      <span><small>{t("next instrument installs on")}</small><strong>v{target + 1} · {t(targetLabel)}</strong></span>
    </div>
  {/if}

  <div class="cabinet-intro">
    <span>{t("Instrument wall")} <b title={t("{available} of {total} instruments unlocked", { available: availableCount, total: ids.length })}>{availableCount}/{ids.length}</b></span>
    {#if mode !== "sandbox"}<p>{t("Complete investigations to earn permanent access to more instruments.")}</p>{/if}
  </div>

  <label class="equipment-search">
    <span aria-hidden="true">⌕</span>
    <input bind:value={filter} placeholder={t("filter…")} aria-label={`${t("filter…")} ${t("equipment")}`} />
    {#if filter}<button type="button" onclick={() => (filter = "")} aria-label={t("clear")}>×</button>{/if}
  </label>

  {#each EQUIPMENT_GROUPS as group (group)}
    {@const entries = inGroup(group)}
    {#if entries.length > 0}
      <div class="equipment-group" class:kids-equipment={group === "sets"}>
        <h2><span>{t(GROUP_LABELS[group])}</span><small>{entries.length}</small></h2>
        <div class="equipment-grid">
          {#each entries as entry (entry.id)}
            {@const entryAccess = accessOf(entry)}
            {@const badge = deployedLabel(entry, deployment)}
            <!-- Row, not card: icon, name, one line of purpose, and an (i) —
                 the same shape the reagent shelf settled on, for the same
                 reason. What the model computes lives behind the (i). -->
            <div class="kit-row">
              <button
                class="equipment-card"
                class:locked={!entryAccess.available}
                class:deployed={badge !== null}
                aria-pressed={badge !== null}
                disabled={busy || !entryAccess.available}
                onclick={() => choose(entry)}
              >
                <span class="equipment-icon">
                  {#if entry.render.kind === "icon"}
                    <ToolIcon name={entry.render.name} />
                  {:else}
                    <span class="instrument-glyph" aria-hidden="true">{entry.render.text}</span>
                  {/if}
                </span>
                <span class="equipment-copy"><strong>{t(entry.name)}</strong><small>{t(entry.blurb)}</small></span>
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
              <InfoPanel id={panelId(entry.id)} rows={equipmentInfoRows(entry, t)} />
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  {/each}

  {#if filter && shown.length === 0}
    <p class="empty-scope">{t("nothing matches that filter")}</p>
  {/if}
  {#if scope === "mission" && missionVerbs.length === 0}
    <p class="empty-scope">{t("This mission needs no additional cabinet equipment.")}</p>
  {/if}
</section>

<style>
  .equipment-cabinet { min-height: 0; overflow-y: auto; padding: 0.65rem; }
  .target-card { display: flex; align-items: center; gap: 0.6rem; margin-bottom: 0.75rem; padding: 0.65rem; border: 1px solid color-mix(in srgb, var(--instrument) 30%, var(--edge)); border-radius: 13px; background: color-mix(in srgb, var(--instrument) 8%, var(--surface-raised)); }
  .target-card > span:last-child { min-width: 0; display: flex; flex-direction: column; }
  .target-card small { color: var(--instrument); font-size: 0.57rem; font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; }
  .target-card strong { overflow: hidden; font-size: 0.75rem; text-overflow: ellipsis; white-space: nowrap; }
  .target-orbit { width: 31px; height: 31px; display: grid; place-items: center; flex: none; border: 1px solid color-mix(in srgb, var(--instrument) 45%, transparent); border-radius: 50%; }
  .target-orbit span { width: 9px; height: 9px; border: 2px solid var(--surface); border-radius: 50%; background: var(--instrument); box-shadow: 0 0 0 2px var(--instrument); }
  .cabinet-intro { margin: 0.2rem 0.2rem 1rem; }
  .cabinet-intro > span { display: flex; justify-content: space-between; color: var(--ink); font-size: 0.85rem; font-weight: 800; }
  .cabinet-intro b { color: var(--instrument); font-size: .72rem; }
  .cabinet-intro p { margin: 0.15rem 0 0; color: var(--dim); font-size: 0.67rem; line-height: 1.35; }
  .equipment-search { min-height: 38px; display: grid; grid-template-columns: auto 1fr auto; align-items: center; gap: .4rem; margin: 0 0 1rem; padding: 0 .55rem; border: 1px solid var(--edge); border-radius: 11px; color: var(--dim); background: var(--surface-raised); }
  .equipment-search:focus-within { border-color: var(--primary); box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 18%, transparent); }
  .equipment-search input { min-width: 0; border: 0; outline: 0; color: var(--ink); background: transparent; font: inherit; font-size: .72rem; }
  .equipment-search input::placeholder { color: var(--dim); }
  .equipment-search button { width: 25px; height: 25px; border: 0; border-radius: 50%; color: var(--dim); background: transparent; cursor: pointer; font-size: 1rem; }
  .equipment-search button:hover { color: var(--ink); background: var(--surface); }
  .equipment-group { margin-bottom: 1.15rem; }
  .equipment-group h2 { display: flex; align-items: center; justify-content: space-between; margin: 0 0 0.45rem; color: var(--dim); font-size: 0.62rem; letter-spacing: 0.07em; text-transform: uppercase; }
  .equipment-group h2 small { min-width: 1.35rem; padding: 0.12rem 0.3rem; border-radius: 999px; background: var(--surface-raised); text-align: center; }
  /* One per line: the row is short enough that the pane shows a whole group
     at once, and an opened panel needs the full width to set a sentence in. */
  .equipment-grid { display: grid; grid-template-columns: 1fr; gap: 0.42rem; }
  .equipment-card { position: relative; flex: 1; min-width: 0; min-height: 52px; display: flex; flex-direction: row; align-items: center; gap: 0.5rem; padding: 0.58rem; overflow: hidden; border: 1px solid var(--edge); border-radius: 13px; color: var(--ink); background: linear-gradient(145deg, var(--surface), color-mix(in srgb, var(--surface-raised) 76%, var(--surface))); cursor: pointer; text-align: left; }
  .equipment-card:hover:not(:disabled) { border-color: var(--action); transform: translateY(-2px); box-shadow: 0 7px 16px var(--shadow); }
  .equipment-card.deployed { border-color: var(--action); background: color-mix(in srgb, var(--action) 9%, var(--surface)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--action) 20%, transparent); }
  .equipment-card.locked, .equipment-card:disabled { opacity: .8; filter: saturate(.55); cursor: not-allowed; }
  .equipment-card.locked:hover { transform: none; border-color: var(--edge); box-shadow: none; }
  .equipment-icon { width: 36px; height: 36px; display: grid; place-items: center; flex: none; border-radius: 11px; color: var(--action); background: color-mix(in srgb, var(--action) 10%, var(--surface)); }
  .equipment-icon :global(svg) { width: 26px; height: 26px; margin: 0; }
  .instrument-glyph { font-size: .72rem; font-weight: 850; }
  .equipment-copy { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.15rem; }
  .equipment-copy strong { font-size: 0.72rem; line-height: 1.15; }
  .equipment-copy small { color: var(--dim); font-size: 0.61rem; line-height: 1.25; }
  .deployed-label { position: absolute; top: 0.35rem; right: 0.35rem; padding: 0.14rem 0.32rem; border-radius: 999px; color: var(--on-accent); background: var(--action); font-size: 0.48rem; font-weight: 800; letter-spacing: 0.04em; text-transform: uppercase; }
  .locked-label { padding: .2rem .36rem; border-radius: 7px; color: var(--dim); background: color-mix(in srgb, var(--surface-raised) 90%, transparent); font-size: .55rem; font-weight: 800; line-height: 1.2; }
  .loaned-label { padding: .2rem .36rem; border-radius: 999px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 10%, var(--surface)); font-size: .5rem; font-weight: 800; letter-spacing: .04em; text-transform: uppercase; }
  .empty-scope { margin: 1rem .2rem; color: var(--dim); font-size: .72rem; line-height: 1.4; }
  .kids-equipment { padding: .55rem; border: 1px solid color-mix(in srgb, var(--discovery) 25%, var(--edge)); border-radius: 14px; background: color-mix(in srgb, var(--discovery) 5%, transparent); }
  .kit-row { display: flex; align-items: stretch; gap: .3rem; min-width: 0; }
</style>
