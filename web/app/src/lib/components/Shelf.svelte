<script lang="ts">
  import type { ShelfItem } from "../session.svelte";
  import { amountUnits, suggestedAmount, type AmountUnit } from "../amounts";
  import { reagentMatches } from "../catalogSearch";
  import SpeciesChip from "./SpeciesChip.svelte";
  import { i18n, t } from "../i18n.svelte";
  import { stepAmount } from "../stepAmount";
  import { reagentAccess, reagentRequirement } from "../catalogProgress";
  import { stockRemaining } from "../storyStock";
  import { isExhausted, stockBadge, type StockLevels } from "../shelfStock";
  import type { LabMode } from "../worldState";
  import type { CatalogScope } from "../catalogScope";
  import { deriveShelfRoles, ROLE_LABELS, REAGENT_ROLES, type ReagentRole } from "../reagentRoles";

  let {
    items,
    register,
    target,
    targetCapacityMl = 400,
    onadd,
    kit = null,
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
    scope?: CatalogScope;
    mode?: LabMode;
    completed?: number;
    stockUsed?: Readonly<Record<string, number>>;
    /** BRD-002: the engine's own finite bottles, from the scene. A key
     * that is absent here is an unlimited supply, in every mode. */
    bottles?: StockLevels;
    focusRequest?: { key: string; nonce: number } | null;
  } = $props();

  let query = $state("");
  const visible = $derived(items.filter((item) => {
    if (mode === "sandbox" || scope === "all") return true;
    if (scope === "mission") return kit?.includes(item.key) ?? false;
    return completed >= reagentRequirement(item);
  }));
  let open = $state<string | null>(null);
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
  <ul>
    {#each filtered as item (item.key)}
      {@const access = reagentAccess(mode, completed, item, kit?.includes(item.key) ?? false)}
      {@const remaining = mode === "story" ? stockRemaining(item, stockUsed) : Number.POSITIVE_INFINITY}
      {@const bottle = bottles[item.key]}
      {@const emptyBottle = isExhausted(bottle)}
      <!-- An engine-empty bottle is depleted whatever the mode says: a
           mission kit cannot loan what the ledger no longer holds. -->
      {@const depleted = emptyBottle || (access.available && !access.loaned && remaining === 0)}
      {@const usable = access.available && !depleted}
      <li data-phase={item.phase}>
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
          {#if mode === "story" && access.available && !access.loaned}<span class="stock">{stockLabel(remaining)}</span>{/if}
          {#if bottle}<span class="bottle" class:out={emptyBottle}>{stockBadge(bottle, t)}</span>{/if}
          {#if !access.available}<span class="lock" aria-hidden="true">⌁</span>{/if}
        </button>
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
              <label>
                <span>{t("amount")}</span>
                <div class="stepper">
                  <button
                    type="button"
                    class="step"
                    aria-label={t("less")}
                    onclick={() => (amountValue = stepAmount(amountValue, -1))}
                  >−</button>
                  <input type="number" min="0.000001" step="any" required bind:value={amountValue} />
                  <button
                    type="button"
                    class="step"
                    aria-label={t("more")}
                    onclick={() => (amountValue = stepAmount(amountValue, 1))}
                  >+</button>
                </div>
              </label>
              <label>
                <span>{t("unit")}</span>
                <select bind:value={amountUnit}>
                  {#each amountUnits(register, item.phase) as unit (unit)}
                    <option value={unit}>{unit}</option>
                  {/each}
                </select>
              </label>
              <button class="add-amount" type="submit">{t("add")}</button>
              {#if item.phase === "liquid"}
                <small>{t("selected vessel capacity: {capacity} mL", { capacity: targetCapacityMl })}</small>
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
    {filtered.length === items.length
      ? t("{count} substances — every one computed, none painted on", { count: items.length })
      : t("{shown} of {total} substances", { shown: filtered.length, total: items.length })}
  </p>
</section>

<style>
  .shelf {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .phases {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin: 0.4rem 0.8rem 0;
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
    flex-wrap: wrap;
    gap: 0.25rem;
    margin: 0.3rem 0.8rem 0;
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
  .species {
    --phase-color: var(--primary);
    width: 100%;
    display: flex;
    /* German names are long enough that chip + name + formula + the stock
       count do not fit one line: "Natriumchlorid" left the count showing
       as "no". Wrapping puts the count on its own line when it has to,
       and leaves it inline when it fits — rather than truncating a
       chemical name, which the reader actually needs to read. */
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
  .stock { margin-left: auto; flex: none; color: var(--dim); font-size: .54rem; font-weight: 800; line-height: 1.15; text-align: right; }
  /* The engine's own bottle level. Reads as a quantity, not a warning,
     until it is actually empty — a shelf of orange badges teaches
     nothing except to ignore orange badges. */
  .bottle { margin-left: auto; flex: none; padding: .12rem .3rem; border-radius: 6px; color: var(--dim); background: color-mix(in srgb, var(--edge) 40%, transparent); font-size: .54rem; font-weight: 800; line-height: 1.15; white-space: nowrap; }
  .bottle.out { color: var(--warning); background: color-mix(in srgb, var(--warning) 13%, transparent); }
  .stock-lock { margin: 0 .2rem .5rem; padding: .5rem; border-left: 3px solid var(--instrument); border-radius: 7px; color: var(--dim); background: color-mix(in srgb, var(--instrument) 7%, transparent); font-size: .68rem; line-height: 1.35; }
  .depleted-note { border-left-color: var(--warning); background: color-mix(in srgb, var(--warning) 7%, transparent); }
  .name {
    flex: 1;
  }
  .formula {
    color: var(--dim);
  }
  .amounts {
    display: grid;
    /* Two rows, not one. Widening the amount column for the steppers
       pushed the add button out of the panel entirely — the shelf is
       ~370px and steppers + number + unit + button do not fit across it.
       The button spanning its own row is also the easier target. */
    /* One control per row. Measured: the shelf gives this form 207px, and
       steppers (88) + number + unit (80) across it left the number 33px
       wide — too narrow to read the value in, let alone edit it. */
    grid-template-columns: 1fr;
    gap: 0.35rem;
    padding: 0.5rem 0.2rem;
    align-items: end;
  }
  .amounts label,
  .add-amount,
  .amounts small {
    grid-column: 1 / -1;
  }
  .stepper {
    display: grid;
    grid-template-columns: 2.75rem minmax(0, 1fr) 2.75rem;
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
  .amounts label { display: grid; gap: 0.15rem; color: var(--dim); font-size: 0.62rem; }
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
  .amounts small { grid-column: 1 / -1; color: var(--dim); font-size: 0.6rem; }
</style>
