<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import { amountUnits, suggestedAmount, type AmountUnit } from "../amounts";
  import { reagentMatches } from "../catalogSearch";
  import SpeciesChip from "./SpeciesChip.svelte";
  import { i18n, t } from "../i18n.svelte";
  import { stepAmount } from "../stepAmount";
  import { access, available, type CatalogMap } from "../catalogProgress";
  import { stockRemaining } from "../storyStock";
  import { isExhausted, stockBadge, type StockLevels } from "../shelfStock";
  import type { LabMode } from "../worldState";
  import type { CatalogScope } from "../catalogScope";
  import { deriveShelfRoles, ROLE_LABELS, REAGENT_ROLES, type ReagentRole } from "../reagentRoles";
  import InfoPanel from "./InfoPanel.svelte";
  import InfoToggle from "./InfoToggle.svelte";
  import type { InfoRow } from "../infoPanel";

  let {
    items,
    register,
    target,
    targetCapacityMl = 400,
    onadd,
    kit = null,
    catalog,
    scope = "all",
    mode = "sandbox",
    completed = 0,
    stockUsed = {},
    bottles = {},
    focusRequest = null,
  }: {
    items: ShelfItem[];
    register: string;
    target: number;
    targetCapacityMl?: number;
    onadd: (line: string) => void;
    /** During a lesson: the reagents its own commands use. */
    kit?: string[] | null;
    /** WORLD-003: the engine's answer about what is reachable, by id. */
    catalog: CatalogMap;
    scope?: CatalogScope;
    mode?: LabMode;
    completed?: number;
    stockUsed?: Readonly<Record<string, number>>;
    /** BRD-002: the engine's own finite bottles, from the scene. A key
     * that is absent here is an unlimited supply, in every mode. */
    bottles?: StockLevels;
    focusRequest?: { key: string; nonce: number } | null;
  } = $props();

  // Named apart from the markup's own `access` const, which shadows it.
  const access_ = (key: string) =>
    access(catalog, key) ?? { available: false, loaned: false, granted: false, minimumCompleted: 0 };

  let query = $state("");
  const visible = $derived(items.filter((item) => {
    if (scope === "all") return true;
    if (scope === "mission") return kit?.includes(item.key) ?? false;
    if (mode === "sandbox") return true;
    return available(catalog, item.key);
  }));
  let open = $state<string | null>(null);
  /**
   * GUI: which item has its explanation showing. Separate from `open`,
   * which is the amount form, because the two answer different questions —
   * "how much of this do I add" and "what IS this" — and a learner reading
   * the second should not have to arm the first to do it. One at a time,
   * like `open`, so the shelf never grows several essays at once.
   */
  let info = $state<string | null>(null);
  const infoPanelId = (key: string) => `shelf-info-${key}`;
  let amountValue = $state(1);
  let amountUnit = $state<AmountUnit>("g");

  /** One tap narrows to a phase; the chips only exist when useful. */
  let phase = $state<string | null>(null);
  // The chips are sorted as they are shown. `phase` itself stays the
  // English wire value — it is a key, and `filtered` compares it below.
  const phases = $derived(
    [...new Set(visible.map((s) => s.phase))].sort((a, b) =>
      t(a).localeCompare(t(b), i18n.locale),
    ),
  );
  $effect(() => {
    if (phase && !phases.includes(phase)) phase = null;
  });

  /**
   * GUI-093: the chemistry axis beside the phase one. Roles are derived
   * from engine data (see reagentRoles.ts) over the whole catalogue, so a
   * material can be classified by the components it names, and the chips
   * then offer only the roles the visible shelf actually contains.
   */
  let role = $state<ReagentRole | null>(null);
  const roleIndex = $derived(deriveShelfRoles(items));
  const rolesOf = (key: string): readonly ReagentRole[] => roleIndex.get(key) ?? [];
  const roles = $derived(
    REAGENT_ROLES.filter((candidate) =>
      // i18n-ok: `candidate` is a role key, not a rendered string, and the
      // chips keep REAGENT_ROLES' pedagogical order in every language
      // rather than re-sorting acid/base/salt alphabetically per locale.
      visible.some((item) => rolesOf(item.key).includes(candidate)),
    ),
  );
  $effect(() => {
    if (role && !roles.includes(role)) role = null;
  });

  const filtered = $derived(
    visible.filter((s) => {
      if (phase && s.phase !== phase) return false;
      if (role && !rolesOf(s.key).includes(role)) return false;
      const q = query.trim().toLowerCase();
      if (!q) return true;
      return reagentMatches(s, q, t(s.name));
    }),
  );

  const stockLabel = (count: number) => count === 1 ? t("one use left") : t("{count} uses left", { count });
  const capabilityLabel = (capability: ShelfItem["capability"]) => capability === "modeled_reaction"
    ? t("modeled reaction")
    : capability === "modeled_activity"
      ? t("modeled activity")
    : capability === "modeled_observation"
      ? t("modeled observation")
      : capability === "identity_only"
        ? t("identity and dose only")
        : null;
  /**
   * What the species chip has always said in a `title`, in text a finger
   * can reach. Silence is not a clearance: an unassessed species says so.
   * Returns null only where the safety matrix has a row and it is empty,
   * which is the one case with nothing to report.
   */
  const hazardLine = (item: ShelfItem): string | null => {
    if (item.hazard_assessed === false) return t("hazards unassessed");
    const hazards = item.hazards ?? [];
    return hazards.length > 0 ? hazards.map((hazard) => t(hazard)).join(" · ") : null;
  };

  /**
   * Everything the row no longer says, as `<dl>` rows.
   *
   * A fact with no answer is left out rather than shown empty: an
   * appearance nobody recorded and a hazard row that is genuinely blank are
   * both silence, and a panel of "—" teaches a learner to stop reading it.
   */
  function speciesRows(item: ShelfItem): InfoRow[] {
    const rows: InfoRow[] = [{ term: t("physical state"), detail: t(item.phase) }];
    if (item.appearance) rows.push({ term: t("visual appearance"), detail: t(item.appearance) });
    const families = rolesOf(item.key);
    if (families.length > 0) {
      rows.push({
        term: t("chemical family"),
        detail: families.map((family) => t(ROLE_LABELS[family])).join(" · "),
      });
    }
    const computes = capabilityLabel(item.capability);
    if (computes) {
      rows.push({
        term: t("what the model computes"),
        detail: computes,
        // The two capabilities that are a LIMIT keep their colour; a fully
        // modeled species says so in plain ink.
        tone: item.capability === "modeled_activity"
          ? "info"
          : item.capability === "identity_only" ? "warn" : undefined,
      });
    }
    const hazards = hazardLine(item);
    if (hazards) rows.push({ term: t("safety labels"), detail: hazards, tone: "danger" });
    // `stockBadge` answers null for a level it has nothing to say about,
    // and a row whose value is empty is worse than no row: it teaches the
    // reader that the panel sometimes has nothing in it.
    const badge = stockBadge(bottles[item.key], t);
    if (badge) rows.push({ term: t("in the bottle"), detail: badge });
    const itemAccess = access_(item.key);
    if (mode === "story" && itemAccess.available && !itemAccess.loaned) {
      rows.push({ term: t("shelf stock"), detail: stockLabel(stockRemaining(item, stockUsed)) });
    }
    return rows;
  }

  function toggle(item: ShelfItem) {
    if (open === item.key) {
      open = null;
      return;
    }
    open = item.key;
    const suggested = suggestedAmount(item.phase, targetCapacityMl);
    amountValue = suggested.value;
    amountUnit = suggested.unit;
  }

  let handledFocus = -1;
  $effect(() => {
    if (!focusRequest || focusRequest.nonce === handledFocus) return;
    const item = items.find((candidate) => candidate.key === focusRequest.key);
    if (!item) return;
    handledFocus = focusRequest.nonce;
    query = t(item.name);
    phase = null;
    role = null;
    open = item.key;
    const suggested = suggestedAmount(item.phase, targetCapacityMl);
    amountValue = suggested.value;
    amountUnit = suggested.unit;
  });

  function add(item: ShelfItem, amount: string) {
    const a = amount.trim();
    if (!a) return;
    onadd(`add v${target + 1} ${item.key} ${a}`);
    open = null;
  }
</script>

<section class="shelf" aria-label={t("reagent shelf")}>
  <input
    type="search"
    placeholder={t("find a substance…")}
    aria-label={t("find a substance")}
    bind:value={query}
  />
  <!-- One rail, not a stack. The two axes wrapped to four rows of chips on
       a phone and pushed the shelf itself below the fold, so the filter was
       larger than the thing it filtered. They remain two radiogroups in
       this order, so the DOM order — and therefore the tab order — is
       exactly what it was; they are laid side by side in a single
       horizontal scroller instead of each wrapping onto new lines. -->
  {#if phases.length > 1 || roles.length > 1}
    <div class="filter-rail">
      {#if phases.length > 1}
        <div class="phases" role="radiogroup" aria-label={t("phase filter")}>
          {#each phases as p (p)}
            <button
              data-phase={p}
              role="radio"
              aria-checked={phase === p}
              class:on={phase === p}
              onclick={() => (phase = phase === p ? null : p)}
            >
              {t(p)}
            </button>
          {/each}
        </div>
      {/if}
      {#if roles.length > 1}
        <div class="roles" role="radiogroup" aria-label={t("role filter")}>
          {#each roles as r (r)}
            <button
              data-role={r}
              role="radio"
              aria-checked={role === r}
              class:on={role === r}
              onclick={() => (role = role === r ? null : r)}
            >
              {t(ROLE_LABELS[r])}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
  <ul>
    {#each filtered as item (item.key)}
      {@const access = access_(item.key)}
      {@const remaining = mode === "story" ? stockRemaining(item, stockUsed) : Number.POSITIVE_INFINITY}
      {@const bottle = bottles[item.key]}
      {@const emptyBottle = isExhausted(bottle)}
      <!-- An engine-empty bottle is depleted whatever the mode says: a
           mission kit cannot loan what the ledger no longer holds. -->
      {@const depleted = emptyBottle || (access.available && !access.loaned && remaining === 0)}
      {@const usable = access.available && !depleted}
      <li data-phase={item.phase}>
        <!-- The row carries identity and nothing else: chip, name, formula,
             and the two states that change what a tap will DO (a mission
             loan, an empty bottle). Everything descriptive moved behind the
             (i) — with all of it inline the row wrapped to three lines per
             substance, and a shelf of ~90 of them was mostly badge. -->
        <div class="row">
          <button
            class="species"
            class:locked={!access.available}
            class:depleted
            aria-expanded={open === item.key}
            aria-disabled={!usable}
            draggable={usable}
            ondragstart={(e) => {
              if (!usable) return;
              e.dataTransfer?.setData(
                "application/x-kero-species",
                JSON.stringify({ key: item.key, phase: item.phase }),
              );
            }}
            onclick={() => toggle(item)}
          >
            <SpeciesChip {item} />
            <span class="name">{t(item.name)}</span>
            <span class="formula">{item.formula}</span>
            {#if access.loaned}<span class="loan">{t("mission kit")}</span>{/if}
            {#if emptyBottle}<span class="bottle out">{t("empty")}</span>{/if}
            {#if !access.available}<span class="lock" aria-hidden="true">⌁</span>{/if}
          </button>
          <InfoToggle
            expanded={info === item.key}
            controls={infoPanelId(item.key)}
            label={t("about {name}", { name: t(item.name) })}
            onclick={() => (info = info === item.key ? null : item.key)}
          />
        </div>
        {#if info === item.key}
          <InfoPanel id={infoPanelId(item.key)} rows={speciesRows(item)} />
        {/if}
        {#if open === item.key}
          {#if usable}
            <form
              class="amounts"
              aria-label={t("amount of {name}", { name: t(item.name) })}
              onsubmit={(e) => {
                e.preventDefault();
                add(item, `${amountValue}${amountUnit}`);
              }}
            >
              <!-- The captions are gone, not the names: "Stoffmenge" and
                   "Einheit" each cost a whole row above a control that a
                   screen reader still hears through `aria-label`, and that
                   a sighted reader can already tell apart by shape. Tab
                   order is unchanged: −, value, +, unit, add. -->
              <div class="stepper">
                <button
                  type="button"
                  class="step"
                  aria-label={t("less")}
                  onclick={() => (amountValue = stepAmount(amountValue, -1))}
                >−</button>
                <input
                  type="number"
                  min="0.000001"
                  step="any"
                  required
                  aria-label={t("amount")}
                  bind:value={amountValue}
                />
                <button
                  type="button"
                  class="step"
                  aria-label={t("more")}
                  onclick={() => (amountValue = stepAmount(amountValue, 1))}
                >+</button>
              </div>
              <select aria-label={t("unit")} bind:value={amountUnit}>
                {#each amountUnits(register, item.phase) as unit (unit)}
                  <option value={unit}>{unit}</option>
                {/each}
              </select>
              <button class="add-amount" type="submit">{t("add")}</button>
              {#if item.phase === "liquid"}
                <!-- Short enough to share a line. The sentence it was is
                     still here, as the tooltip. -->
                <small title={t("selected vessel capacity: {capacity} mL", { capacity: targetCapacityMl })}>
                  {t("vessel: {capacity} mL", { capacity: targetCapacityMl })}
                </small>
              {/if}
            </form>
          {:else if !access.available}
            <p class="stock-lock">{access.minimumCompleted === 1
              ? t("Permanent stock unlocks after one completed mission. Mission kits loan required materials.")
              : t("Permanent stock unlocks after {count} completed missions. Mission kits loan required materials.", { count: access.minimumCompleted })}</p>
          {:else if emptyBottle}
            <p class="stock-lock depleted-note">{t("This bottle is empty — the lab would refuse the pour. Stock the shelf again to keep going.")}</p>
          {:else}
            <p class="stock-lock depleted-note">{t("This bottle is empty. Mission kits still supply required materials, and permanent stock refills after a new discovery.")}</p>
          {/if}
        {/if}
      </li>
    {/each}
    {#if filtered.length === 0}
      <li class="none">{t("nothing on the shelf matches")}</li>
    {/if}
  </ul>
  <p class="tally">
    {t("{shown} of {total} substances", { shown: filtered.length, total: items.length })}
  </p>
</section>

<style>
  .shelf {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  /* The rail scrolls; the groups inside it never wrap. `flex: none` on the
     chips matters as much as `nowrap` — without it they shrink to fit and
     "Oxidationsmittel" becomes an ellipsis instead of scrolling. */
  .filter-rail {
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    /* Without this the scroller reports its content's min-content width up
       to the pane and the 320px page gains a horizontal scrollbar — the
       overflow has to happen HERE, which is the whole point of the rail. */
    min-width: 0;
    gap: 0.5rem;
    margin: 0.4rem 0.65rem 0;
    padding-bottom: 0.28rem;
    overflow-x: auto;
    overscroll-behavior-x: contain;
    scrollbar-width: thin;
  }
  .phases {
    display: flex;
    flex-wrap: nowrap;
    gap: 0.25rem;
    margin: 0;
  }
  .filter-rail button {
    flex: none;
    white-space: nowrap;
  }
  .phases button {
    --phase-color: var(--primary);
    background: color-mix(in srgb, var(--phase-color) 7%, var(--surface));
    border: 1px solid color-mix(in srgb, var(--phase-color) 35%, var(--edge));
    border-radius: 999px;
    color: color-mix(in srgb, var(--phase-color) 76%, var(--ink));
    font: inherit;
    font-size: 0.72rem;
    padding: 0.15rem 0.6rem;
    cursor: pointer;
  }
  .phases button.on {
    color: var(--ink);
    border-color: var(--phase-color);
    background: color-mix(in srgb, var(--phase-color) 18%, var(--surface));
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--phase-color) 28%, transparent);
  }
  .phases button[data-phase="liquid"] { --phase-color: var(--instrument); }
  .phases button[data-phase="gas"] { --phase-color: var(--discovery); }
  .phases button[data-phase="solid"] { --phase-color: var(--action); }
  /* The chemistry axis reads as a second row of the same chip, one step
     quieter than the phase row above it: the phase filter is where the
     eye lands first, and two equally loud rows would compete. */
  .roles {
    display: flex;
    flex-wrap: nowrap;
    gap: 0.25rem;
    margin: 0;
  }
  /* A hairline where the axes meet, so one long row still reads as two
     questions rather than eleven unrelated chips. */
  .phases + .roles {
    border-left: 1px solid var(--edge);
    padding-left: 0.5rem;
  }
  .roles button {
    --role-color: var(--dim);
    background: transparent;
    border: 1px dashed color-mix(in srgb, var(--role-color) 40%, var(--edge));
    border-radius: 999px;
    color: color-mix(in srgb, var(--role-color) 80%, var(--ink));
    font: inherit;
    font-size: 0.68rem;
    padding: 0.12rem 0.55rem;
    cursor: pointer;
  }
  .roles button.on {
    color: var(--ink);
    border-style: solid;
    border-color: var(--role-color);
    background: color-mix(in srgb, var(--role-color) 16%, var(--surface));
  }
  .roles button[data-role="acid"] { --role-color: var(--warning); }
  .roles button[data-role="base"] { --role-color: var(--instrument); }
  .roles button[data-role="oxidiser"],
  .roles button[data-role="reducer"] { --role-color: var(--discovery); }
  .roles button[data-role="metal"] { --role-color: var(--action); }
  .roles button[data-role="indicator"] { --role-color: var(--primary); }
  /* "Unsorted" is a gap in the data, not a category — it stays grey so it
     never reads as a chemical family of its own. */
  .roles button[data-role="unsorted"] { --role-color: var(--dim); font-style: italic; }
  .none {
    hyphens: auto;
    overflow-wrap: anywhere;
    color: var(--dim);
    font-size: 0.8rem;
    padding: 0.6rem 0.8rem;
    list-style: none;
  }
  .tally {
    margin: 0;
    padding: 0.35rem 0.8rem 0.6rem;
    color: var(--dim);
    font-size: 0.7rem;
    border-top: 1px solid var(--edge);
  }
  input[type="search"] {
    margin: 0.65rem;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 11px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.55rem 0.7rem;
    min-height: 44px;
  }
  input[type="search"]:focus {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 13%, transparent);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0.65rem 0.8rem;
    overflow-y: auto;
  }
  .row {
    display: flex;
    align-items: stretch;
    gap: 0.3rem;
    margin-bottom: 0.32rem;
  }
  .species {
    --phase-color: var(--primary);
    width: 100%;
    display: flex;
    /* Still allowed to wrap, but it rarely has to now that the capability
       badge and the stock counts sit behind the (i): chip + a German name
       + a formula is one line for almost everything on the shelf. Wrapping
       stays the fallback because truncating a chemical name — which is what
       `nowrap` would do to "Natriumhydrogencarbonat" — hides the one word
       the reader actually needs. */
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.32rem;
    background: color-mix(in srgb, var(--phase-color) 5%, var(--surface-raised));
    border: 1px solid color-mix(in srgb, var(--phase-color) 13%, transparent);
    border-left: 3px solid color-mix(in srgb, var(--phase-color) 72%, var(--edge));
    border-radius: 11px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    text-align: left;
    padding: 0.55rem 0.6rem;
    cursor: pointer;
    min-height: 40px;
    /* The row is the flexible half of `.row`; the (i) beside it is fixed. */
    flex: 1;
    min-width: 0;
    margin-bottom: 0;
  }
  li[data-phase="liquid"] .species { --phase-color: var(--instrument); }
  li[data-phase="gas"] .species { --phase-color: var(--discovery); }
  li[data-phase="solid"] .species { --phase-color: var(--action); }
  .species:hover .name {
    color: var(--phase-color);
  }
  .species:hover,
  .species[aria-expanded="true"] {
    border-color: color-mix(in srgb, var(--phase-color) 55%, var(--edge));
    background: color-mix(in srgb, var(--phase-color) 11%, var(--surface-raised));
  }
  .species.locked { opacity: .62; border-color: color-mix(in srgb, var(--edge) 75%, transparent); cursor: pointer; filter: saturate(.55); }
  .species.depleted { opacity: .72; border-color: color-mix(in srgb, var(--warning) 38%, var(--edge)); cursor: pointer; }
  .species.locked:hover .name { color: var(--ink); }
  .lock { color: var(--dim); font-weight: 900; }
  .loan { padding: .12rem .28rem; border-radius: 6px; color: var(--instrument); background: color-mix(in srgb, var(--instrument) 11%, var(--surface)); font-size: .48rem; font-weight: 850; text-transform: uppercase; }
  /* Only the empty bottle stays on the row. A LEVEL is a quantity and
     belongs behind the (i) with the other numbers; an empty one changes
     what the next tap does, so it has to be visible without one. */
  .bottle.out { margin-left: auto; flex: none; padding: .12rem .3rem; border-radius: 6px; color: var(--warning); background: color-mix(in srgb, var(--warning) 13%, transparent); font-size: .54rem; font-weight: 800; line-height: 1.15; white-space: nowrap; }
  .stock-lock { hyphens: auto; overflow-wrap: anywhere; margin: 0 .2rem .5rem; padding: .5rem; border-left: 3px solid var(--instrument); border-radius: 7px; color: var(--dim); background: color-mix(in srgb, var(--instrument) 7%, transparent); font-size: .68rem; line-height: 1.35; }
  .depleted-note { border-left-color: var(--warning); background: color-mix(in srgb, var(--warning) 7%, transparent); }
  .name {
    flex: 1;
    /* "Wasserstoffperoxid" is wider than the 240px pane it lives in. The
       document carries `lang` (i18n.svelte.ts sets it on every locale
       change), so `hyphens: auto` can break the compound where German
       allows; `overflow-wrap: anywhere` is the fallback for the names no
       hyphenation dictionary knows. `min-width: 0` is what lets a flex
       item narrower than its content actually happen. */
    min-width: 0;
    hyphens: auto;
    overflow-wrap: anywhere;
  }
  .formula {
    color: var(--dim);
  }
  .amounts {
    /* At most two rows at every width this pane is ever given, and the
       wrapping does the deciding rather than a media query.
       - Where there is room (a phone's full-width cabinet, ~296px) the
         stepper, the unit and "add" share row one and only the capacity
         hint wraps under them.
       - At the 207px the desktop cabinet gives this form, "add" wraps too
         and the hint sits beside it. Still two rows.
       What it replaced was six: one control per row, each under its own
       caption, plus a sentence about the vessel's capacity. */
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    padding: 0.5rem 0.2rem;
  }
  .stepper {
    /* 88px of touch targets plus a number wide enough to read. Below this
       the number field was measured at 33px — too narrow to edit in. */
    flex: 1 1 8.5rem;
    min-width: 8.2rem;
    display: grid;
    grid-template-columns: 2.75rem minmax(0, 1fr) 2.75rem;
  }
  .amounts select {
    flex: 0 1 auto;
    min-width: 3.6rem;
  }
  .add-amount {
    flex: 1 1 4.6rem;
    min-width: 4.6rem;
  }
  .stepper input {
    border-radius: 0;
    text-align: center;
  }
  .step {
    /* 44px is the smallest target a finger hits reliably; the measured
       height here was 38. */
    min-height: 2.75rem;
    min-width: 2.75rem;
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    color: var(--ink);
    font-size: 1.05rem;
    line-height: 1;
    cursor: pointer;
  }
  .step:first-child {
    border-radius: 9px 0 0 9px;
  }
  .step:last-child {
    border-radius: 0 9px 9px 0;
  }
  .step:hover {
    background: var(--panel);
  }
  .amounts input,
  .amounts select,
  .add-amount {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--ink);
    font: inherit;
    font-size: 0.8rem;
    padding: 0.38rem 0.45rem;
    min-width: 0;
    /* 44px, not the 38px this used to say: measured on the bench at iPad
       size, 38 is under the touch minimum and this whole row is meant to
       be tapped. */
    min-height: 2.75rem;
  }
  .add-amount { color: var(--on-accent); background: var(--action); border-color: var(--action); cursor: pointer; font-weight: 750; }
  .amounts small { flex: 1 1 6.5rem; min-width: 6rem; color: var(--dim); font-size: 0.6rem; line-height: 1.2; }
</style>
