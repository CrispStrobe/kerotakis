<script lang="ts">
  import { APPARATUS } from "../apparatus";
  import type { TwoVesselAction } from "../directActions";
  import { t } from "../i18n.svelte";
  import ToolIcon from "./ToolIcon.svelte";
  import { equipmentAccess, requirement, type CatalogMap } from "../catalogProgress";
  import type { LabMode } from "../worldState";
  import type { CatalogScope } from "../catalogScope";
  import { equipmentMatches } from "../catalogSearch";
  import { INSTRUMENTS, instrumentCommand, instrumentVerb } from "../instruments";
  import { KIDS_EQUIPMENT, kitInfoRows, type KidsEquipment } from "../kidsEquipment";
  import InfoPanel from "./InfoPanel.svelte";
  import InfoToggle from "./InfoToggle.svelte";

  const TRANSFER_TOOLS: { verb: TwoVesselAction; title: string; blurb: string }[] = [
    { verb: "filter", title: "filter", blurb: "separate solids from liquid" },
    { verb: "decant", title: "decant", blurb: "pour off a chosen fraction" },
    { verb: "drain", title: "drain", blurb: "move the lower liquid layer" },
    { verb: "magnet", title: "magnet", blurb: "lift out magnetic solids" },
    { verb: "cell", title: "voltmeter", blurb: "connect two half-cells" },
    { verb: "distil", title: "still", blurb: "separate by volatility" },
  ];

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

  const allVerbs = $derived([
    "burette", ...APPARATUS.map((item) => item.verb),
    ...TRANSFER_TOOLS.map((item) => item.verb), ...INSTRUMENTS.map((item) => instrumentVerb(item.token)),
    "mix", "transport", ...(reactAvailable ? ["react"] : []),
  ]);
  // `equipmentAccess`, not `access`: a verb the engine does not tier at all
  // (the cooling bath's `cool` is a bench control, not earned equipment) has
  // no catalog row, and reading that silence as a refusal disabled the card
  // in Sandbox and made this tally read 33/34 under a sentence saying
  // everything was available. See catalogProgress.ts for the whole story.
  const availableCount = $derived(allVerbs.filter((verb) => equipmentAccess(catalog, verb).available).length);
  const visible = (verb: string) => scope === "all"
    || (scope === "mission" ? missionVerbs.includes(verb) : mode === "sandbox" || equipmentAccess(catalog, verb).available);
  const accessOf = (verb: string) => equipmentAccess(catalog, verb);
  let filter = $state("");
  const matches = (verb: string, title: string, blurb: string) => equipmentMatches(
    { verb, title, blurb },
    filter,
    t(title),
    t(blurb),
  );
  const visibleApparatus = $derived(APPARATUS.filter((item) => visible(item.verb) && matches(item.verb, item.title, item.blurb)));
  const visibleTransfers = $derived(TRANSFER_TOOLS.filter((item) => visible(item.verb) && matches(item.verb, item.title, item.blurb)));
  const visibleInstruments = $derived(INSTRUMENTS.filter((item) => {
    const verb = instrumentVerb(item.token);
    return visible(verb) && matches(verb, item.label, item.purpose);
  }));
  const visibleKidsEquipment = $derived(KIDS_EQUIPMENT.filter((item) =>
    visible(item.engineVerb) && matches(item.engineVerb, item.title, `${item.blurb} ${item.parts.join(" ")}`)
  ));
  const showBurette = $derived(visible("burette") && matches("burette", "burette", "controlled addition"));
  const showMix = $derived(visible("mix") && matches("mix", "mixer", "combine two sources into a receiver"));
  const showTransport = $derived(visible("transport") && matches("transport", "column train", "move solution through connected cells"));
  const showReact = $derived(reactAvailable && visible("react") && matches("react", "curated reaction", "choose a verified reaction family"));
  const resultCount = $derived(visibleApparatus.length + visibleTransfers.length + visibleInstruments.length + visibleKidsEquipment.length + Number(showBurette) + Number(showMix) + Number(showTransport) + Number(showReact));
  const requirementLabel = (verb: string) => {
    const count = requirement(catalog, verb);
    // Silent rather than guessing while the engine has not answered.
    if (count === null) return "";
    return count === 1 ? t("after one mission") : t("after {count} missions", { count });
  };
  /** The mixer and the transfer tools choose their own vessels on the
   * bench; while one of them is armed, the selected vessel is not where the
   * next thing goes, so the card would be a wrong answer rather than a
   * quiet one. */
  const namesNextTarget = $derived(transferVerb === null && !mixActive);
  /**
   * Which kit has its explanation open. One at a time, like the shelf: the
   * cabinet is a pane a learner scans, and several open panels at once is
   * the wall of text this moved the copy out of.
   */
  let openKit = $state<string | null>(null);
  const kitPanelId = (id: string) => `kit-info-${id}`;
  const useKidsEquipment = (item: KidsEquipment) => {
    if (item.action === "apparatus") return onapparatus(item.engineVerb, item.preset);
    if (item.action === "instrument" && item.instrument) {
      return onmeasure(instrumentCommand(target, item.instrument));
    }
    if (item.action === "transfer") return ontransfer(item.engineVerb as TwoVesselAction);
  };
</script>

<section class="equipment-cabinet" aria-label={t("equipment") }>
  {#if namesNextTarget}
    <!-- What this card is FOR: everything below installs on, or measures,
         one vessel, and this names it. "Active work area" did not say that,
         and it kept claiming a target while the mixer and the transfer
         tools were waiting for the learner to pick their own vessels on the
         bench — the one time it was actively wrong. -->
    <div class="target-card">
      <span class="target-orbit" aria-hidden="true"><span></span></span>
      <span><small>{t("next instrument installs on")}</small><strong>v{target + 1} · {t(targetLabel)}</strong></span>
    </div>
  {/if}

  <div class="cabinet-intro">
    <span>{t("Instrument wall")} <b title={t("{available} of {total} instruments unlocked", { available: availableCount, total: allVerbs.length })}>{availableCount}/{allVerbs.length}</b></span>
    {#if mode !== "sandbox"}<p>{t("Complete investigations to earn permanent access to more instruments.")}</p>{/if}
  </div>

  <label class="equipment-search">
    <span aria-hidden="true">⌕</span>
    <input bind:value={filter} placeholder={t("filter…")} aria-label={`${t("filter…")} ${t("equipment")}`} />
    {#if filter}<button type="button" onclick={() => (filter = "")} aria-label={t("clear")}>×</button>{/if}
  </label>

  {#if visibleKidsEquipment.length > 0}<div class="equipment-group kids-equipment">
    <h2><span>{t("children's activity kits")}</span><small>{visibleKidsEquipment.length}</small></h2>
    <div class="equipment-grid">
      {#each visibleKidsEquipment as item (item.id)}
        {@const itemAccess = accessOf(item.engineVerb)}
        <!-- Row, not card: icon, name, one line of purpose, and an (i) —
             the same shape the reagent shelf settled on, for the same
             reason. The inventory and the modelling caveat are behind the
             (i), which is why this fits in one line instead of four. -->
        <div class="kit-row">
          <button
            class="equipment-card kids-card"
            class:locked={!itemAccess.available}
            class:deployed={item.action === "apparatus" && apparatusOut === item.engineVerb}
            disabled={busy || !itemAccess.available}
            onclick={() => useKidsEquipment(item)}
          >
            <span class="equipment-icon"><ToolIcon name={item.icon} /></span>
            <span class="equipment-copy"><strong>{t(item.title)}</strong><small>{t(item.blurb)}</small></span>
            {#if itemAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
            {#if !itemAccess.available}<span class="locked-label">⌁ {requirementLabel(item.engineVerb)}</span>{/if}
          </button>
          <InfoToggle
            expanded={openKit === item.id}
            controls={kitPanelId(item.id)}
            label={t("about {name}", { name: t(item.title) })}
            onclick={() => (openKit = openKit === item.id ? null : item.id)}
          />
        </div>
        {#if openKit === item.id}
          <InfoPanel id={kitPanelId(item.id)} rows={kitInfoRows(item, t)} />
        {/if}
      {/each}
    </div>
  </div>{/if}

  {#if showBurette || visibleApparatus.length > 0}<div class="equipment-group">
    <h2><span>{t("measure and transform")}</span><small>{visibleApparatus.length + Number(showBurette)}</small></h2>
    <div class="equipment-grid">
      {#if showBurette}<button class="equipment-card feature" class:deployed={buretteOut} aria-pressed={buretteOut} onclick={onburette}>
        <span class="equipment-icon"><ToolIcon name="burette" /></span>
        <span class="equipment-copy"><strong>{t("burette")}</strong><small>{t("controlled addition")}</small></span>
        {#if buretteOut}<span class="deployed-label">{t("on bench")}</span>{/if}
      </button>{/if}
      {#each visibleApparatus as item (item.verb)}
        {@const itemAccess = accessOf(item.verb)}
        <button class="equipment-card" class:locked={!itemAccess.available} class:deployed={apparatusOut === item.verb} aria-pressed={apparatusOut === item.verb} disabled={!itemAccess.available} onclick={() => onapparatus(item.verb)}>
          <span class="equipment-icon"><ToolIcon name={item.verb} /></span>
          <span class="equipment-copy"><strong>{t(item.title)}</strong><small>{t(item.blurb)}</small></span>
          {#if apparatusOut === item.verb}<span class="deployed-label">{t("on bench")}</span>{/if}
          {#if itemAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
          {#if !itemAccess.available}<span class="locked-label">⌁ {requirementLabel(item.verb)}</span>{/if}
        </button>
      {/each}
    </div>
  </div>{/if}

  {#if visibleInstruments.length > 0}<div class="equipment-group">
    <h2><span>{t("observe and measure")}</span><small>{visibleInstruments.length}</small></h2>
    <div class="equipment-grid">
      {#each visibleInstruments as item (item.token)}
        {@const verb = instrumentVerb(item.token)}
        {@const itemAccess = accessOf(verb)}
        <button class="equipment-card instrument-card" class:locked={!itemAccess.available} disabled={busy || !itemAccess.available} onclick={() => onmeasure(instrumentCommand(target, item.token))}>
          <span class="equipment-icon instrument-glyph" aria-hidden="true">{item.glyph}</span>
          <span class="equipment-copy"><strong>{t(item.label)}</strong><small>{t(item.purpose)}</small></span>
          {#if itemAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
          {#if !itemAccess.available}<span class="locked-label">⌁ {requirementLabel(verb)}</span>{/if}
        </button>
      {/each}
    </div>
  </div>{/if}

  {#if visibleTransfers.length > 0 || showMix || showTransport}<div class="equipment-group">
    <h2><span>{t("transfer and separation")}</span><small>{visibleTransfers.length + Number(showMix) + Number(showTransport)}</small></h2>
    <div class="equipment-grid">
      {#each visibleTransfers as item (item.verb)}
        {@const itemAccess = accessOf(item.verb)}
        <button class="equipment-card" class:locked={!itemAccess.available} class:deployed={transferVerb === item.verb} aria-pressed={transferVerb === item.verb} disabled={!itemAccess.available} onclick={() => ontransfer(item.verb)}>
          <span class="equipment-icon"><ToolIcon name={item.verb} /></span>
          <span class="equipment-copy"><strong>{t(item.title)}</strong><small>{t(item.blurb)}</small></span>
          {#if transferVerb === item.verb}<span class="deployed-label">{t("select source")}</span>{/if}
          {#if itemAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
          {#if !itemAccess.available}<span class="locked-label">⌁ {requirementLabel(item.verb)}</span>{/if}
        </button>
      {/each}
      {#if showMix}<button class="equipment-card" class:deployed={mixActive} aria-pressed={mixActive} onclick={onmix}>
        <span class="equipment-icon"><ToolIcon name="mix" /></span>
        <span class="equipment-copy"><strong>{t("mixer")}</strong><small>{t("combine two sources into a receiver")}</small></span>
        {#if mixActive}<span class="deployed-label">{t("select sources")}</span>{/if}
      </button>{/if}
      {#if showTransport}{@const transportAccess = accessOf("transport")}<button class="equipment-card" class:locked={!transportAccess.available} class:deployed={apparatusOut === "transport"} aria-pressed={apparatusOut === "transport"} disabled={!transportAccess.available} onclick={() => onapparatus("transport")}>
        <span class="equipment-icon"><ToolIcon name="transport" /></span>
        <span class="equipment-copy"><strong>{t("column train")}</strong><small>{t("move solution through connected cells")}</small></span>
        {#if apparatusOut === "transport"}<span class="deployed-label">{t("on bench")}</span>{/if}
        {#if transportAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
        {#if !transportAccess.available}<span class="locked-label">⌁ {requirementLabel("transport")}</span>{/if}
      </button>{/if}
    </div>
  </div>{/if}

  {#if showReact}
    {@const reactAccess = accessOf("react")}
    <div class="equipment-group">
      <h2><span>{t("reaction studio")}</span><small>1</small></h2>
      <button class="equipment-card wide" class:locked={!reactAccess.available} class:deployed={apparatusOut === "react"} aria-pressed={apparatusOut === "react"} disabled={!reactAccess.available} onclick={() => onapparatus("react")}>
        <span class="equipment-icon"><ToolIcon name="react" /></span>
        <span class="equipment-copy"><strong>{t("curated reaction")}</strong><small>{t("choose a verified reaction family")}</small></span>
        {#if apparatusOut === "react"}<span class="deployed-label">{t("on bench")}</span>{/if}
        {#if reactAccess.loaned}<span class="loaned-label">{t("mission kit")}</span>{/if}
        {#if !reactAccess.available}<span class="locked-label">⌁ {requirementLabel("react")}</span>{/if}
      </button>
    </div>
  {/if}
  {#if filter && resultCount === 0}
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
  .equipment-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0.42rem; }
  .equipment-card { position: relative; min-width: 0; min-height: 100px; display: flex; flex-direction: column; align-items: flex-start; gap: 0.48rem; padding: 0.58rem; overflow: hidden; border: 1px solid var(--edge); border-radius: 13px; color: var(--ink); background: linear-gradient(145deg, var(--surface), color-mix(in srgb, var(--surface-raised) 76%, var(--surface))); cursor: pointer; text-align: left; }
  .equipment-card:hover { border-color: var(--action); transform: translateY(-2px); box-shadow: 0 7px 16px var(--shadow); }
  .equipment-card.deployed { border-color: var(--action); background: color-mix(in srgb, var(--action) 9%, var(--surface)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--action) 20%, transparent); }
  .equipment-card.locked { min-height: 118px; opacity: .8; filter: saturate(.55); cursor: not-allowed; }
  .equipment-card.locked:hover { transform: none; border-color: var(--edge); box-shadow: none; }
  .equipment-card.feature { grid-column: 1 / -1; min-height: 82px; flex-direction: row; align-items: center; background: linear-gradient(135deg, color-mix(in srgb, var(--primary) 10%, var(--surface)), color-mix(in srgb, var(--instrument) 9%, var(--surface))); }
  .equipment-card.wide { width: 100%; min-height: 82px; flex-direction: row; align-items: center; }
  .equipment-icon { width: 36px; height: 36px; display: grid; place-items: center; flex: none; border-radius: 11px; color: var(--action); background: color-mix(in srgb, var(--action) 10%, var(--surface)); }
  .equipment-icon :global(svg) { width: 26px; height: 26px; margin: 0; }
  .instrument-glyph { font-size: .72rem; font-weight: 850; }
  .equipment-copy { min-width: 0; display: flex; flex-direction: column; gap: 0.15rem; }
  .equipment-copy strong { font-size: 0.72rem; line-height: 1.15; }
  .equipment-copy small { color: var(--dim); font-size: 0.61rem; line-height: 1.25; }
  .deployed-label { position: absolute; top: 0.35rem; right: 0.35rem; padding: 0.14rem 0.32rem; border-radius: 999px; color: var(--on-accent); background: var(--action); font-size: 0.48rem; font-weight: 800; letter-spacing: 0.04em; text-transform: uppercase; }
  .locked-label { margin-top: auto; padding: .2rem .36rem; border-radius: 7px; color: var(--dim); background: color-mix(in srgb, var(--surface-raised) 90%, transparent); font-size: .55rem; font-weight: 800; line-height: 1.2; }
  .loaned-label { margin-top: auto; padding: .2rem .36rem; border-radius: 999px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 10%, var(--surface)); font-size: .5rem; font-weight: 800; letter-spacing: .04em; text-transform: uppercase; }
  .empty-scope { margin: 1rem .2rem; color: var(--dim); font-size: .72rem; line-height: 1.4; }
  .equipment-card.wide .locked-label { margin: 0 0 0 auto; }
  .kids-equipment { padding: .55rem; border: 1px solid color-mix(in srgb, var(--discovery) 25%, var(--edge)); border-radius: 14px; background: color-mix(in srgb, var(--discovery) 5%, transparent); }
  /* One kit per line rather than two per line and four lines tall: the
     card is now short enough that the pane shows all of them at once, and
     an opened panel needs the full width to set a sentence in. */
  .kids-equipment .equipment-grid { grid-template-columns: 1fr; }
  .kit-row { display: flex; align-items: stretch; gap: .3rem; min-width: 0; }
  .kids-card.equipment-card { flex: 1; min-width: 0; min-height: 52px; flex-direction: row; align-items: center; gap: .5rem; }
  .kids-card .equipment-copy { flex: 1; }
  /* `margin-top: auto` bottom-aligns a badge in a column card; in a row it
     would drop it below the words it belongs to. */
  .kids-card .loaned-label, .kids-card .locked-label { margin-top: 0; }
</style>
