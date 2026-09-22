<script lang="ts">
  import { REGISTERS } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let {
    value,
    onchange,
    /**
     * GUI-127: whether the log carries the aqueous routing announcement.
     *
     * It rides beside the dial rather than inside it because it is a
     * different axis — the dial is HOW MUCH chemistry, this is WHICH KINDS
     * OF LINE — and folding them together is what shipped: the routing
     * paragraph belongs to lv3, so the only way to be rid of it was to
     * leave lv3 and give up every number lv3 was turned on for.
     */
    routing,
    onroutingchange,
  }: {
    value: string;
    onchange: (level: string) => void;
    routing: boolean;
    onroutingchange: (on: boolean) => void;
  } = $props();

  // One sentence, both states, because "off" has to say what is lost and
  // where it went: nothing is hidden, it moves to the drawer that exists
  // for it.
  const routingTitle = $derived(
    routing
      ? t("the log says which dataset and activity model answered each vessel")
      : t("the log leaves the dataset and the activity model to the provenance drawer"),
  );
</script>

<!-- The dial is the product: same bench, mid-session switchable detail.
     Three wide buttons said all three levels at once and spent a third of
     the header rail doing it. A dropdown says only where you are standing
     and still offers the other two in one press — and it is the control
     every phone keyboard and screen reader already knows how to drive. -->
<label class="dial">
  <span class="sr-only">{t("detail level")}</span>
  <select
    aria-label={t("detail level")}
    {value}
    onchange={(event) => onchange(event.currentTarget.value)}
  >
    {#each REGISTERS as reg (reg.level)}
      <option value={reg.level}>{reg.level} · {t(reg.label)}</option>
    {/each}
  </select>
</label>
<!-- One glyph, not a labelled row: the rail is the same rail GUI-107
     trimmed, and this is a switch a reader sets once. The state is in
     `aria-pressed` and in the fill, and the sentence is in the title —
     which is also the accessible name, so a screen reader is told what
     the switch does rather than what it is called. -->
<button
  type="button"
  class="routing"
  class:on={routing}
  aria-pressed={routing}
  aria-label={routingTitle}
  title={routingTitle}
  onclick={() => onroutingchange(!routing)}
>
  <span aria-hidden="true">⇝</span>
</button>

<style>
  .dial {
    display: inline-flex;
    align-items: center;
    min-height: 40px;
  }
  select {
    max-width: 10rem;
    min-height: 40px;
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--primary);
    background: var(--surface-raised);
    font: inherit;
    font-size: 0.78rem;
    font-weight: 650;
    cursor: pointer;
  }
  select:hover {
    border-color: var(--primary);
  }
  .routing {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
    min-height: 40px;
    margin-inline-start: 0.3rem;
    padding: 0;
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    background: var(--surface-raised);
    font: inherit;
    font-size: 0.95rem;
    line-height: 1;
    cursor: pointer;
  }
  .routing:hover {
    border-color: var(--primary);
  }
  .routing.on {
    color: var(--primary);
    border-color: var(--primary);
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
