<!--
  The (i) beside a compact row.

  Small, fixed-width, and never the primary action: tapping the row still
  does what the row does, and this only ever opens the explanation. It is a
  button rather than a `title=` because a title is unreachable by a finger,
  and it carries an `aria-label` naming WHAT it will explain — "i" alone
  tells a screen reader nothing about which of ninety rows it belongs to.
-->
<script lang="ts">
  let {
    expanded,
    controls,
    label,
    onclick,
  }: {
    expanded: boolean;
    /** Id of the panel this opens. Must exist while `expanded`. */
    controls: string;
    /** "about sodium chloride" — never bare "info". */
    label: string;
    onclick: () => void;
  } = $props();
</script>

<button
  type="button"
  class="info-toggle"
  class:on={expanded}
  aria-expanded={expanded}
  aria-controls={controls}
  aria-label={label}
  {onclick}
>i</button>

<style>
  .info-toggle {
    flex: none;
    width: 2.15rem;
    min-height: 40px;
    border: 1px solid var(--edge);
    border-radius: 11px;
    color: var(--dim);
    background: var(--surface-raised);
    font: inherit;
    font-size: 0.85rem;
    font-weight: 800;
    font-style: italic;
    line-height: 1;
    cursor: pointer;
  }
  .info-toggle:hover,
  .info-toggle.on {
    color: var(--primary);
    border-color: color-mix(in srgb, var(--primary) 55%, var(--edge));
    background: color-mix(in srgb, var(--primary) 12%, var(--surface-raised));
  }
</style>
