<!--
  The (i) beside a compact row, or in the corner of a tile.

  Small, fixed-width, and never the primary action: tapping the row still
  does what the row does, and this only ever opens the explanation. It is a
  button rather than a `title=` because a title is unreachable by a finger,
  and it carries an `aria-label` naming WHAT it will explain — "i" alone
  tells a screen reader nothing about which of ninety rows it belongs to.

  GUI-110 — the owner, of the equipment cabinet: "the (i) buttons there
  should be more tiny and e.g. in upper right corner of the drawings of the
  devices, not occupy that much room". In a shelf row a 2.15 rem control is
  proportionate. On a 6 rem tool tile it was a second row of its own under
  every tool, which is a whole row of cabinet spent on a control nobody
  presses most visits.

  So `placement="corner"` separates the two things a button usually
  conflates: what you SEE and what you can HIT. The button element is a
  44 px square anchored to the tile's top-right corner — the touch floor
  this repo audits for, and it does not move — while the thing drawn inside
  it is a 19 px mark in that corner. The tile underneath gives up its
  top-right corner to make this work; `InstrumentCupboard.svelte` lays the
  tile out left-aligned so the drawing and the name both stand clear of it,
  and the comment there says why.
-->
<script lang="ts">
  let {
    expanded,
    controls,
    label,
    placement = "row",
    onclick,
  }: {
    expanded: boolean;
    /** Id of the panel this opens. Must exist while `expanded`. */
    controls: string;
    /** "about sodium chloride" — never bare "info". */
    label: string;
    /**
     * `row` is the full-width control beside a list row. `corner` is a
     * small mark inside a 44 px hit square, absolutely placed — the parent
     * must be `position: relative` and must keep that corner clear.
     */
    placement?: "row" | "corner";
    onclick: () => void;
  } = $props();
</script>

<button
  type="button"
  class="info-toggle"
  class:on={expanded}
  class:corner={placement === "corner"}
  aria-expanded={expanded}
  aria-controls={controls}
  aria-label={label}
  {onclick}
>{#if placement === "corner"}<span class="mark" aria-hidden="true">i</span>{:else}i{/if}</button>

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

  /* The corner variant. The BUTTON is the hit target and stays 44 px; the
     `.mark` is the paint. Nothing here shrinks below the floor — what
     shrinks is what you can see, which is the whole of what was asked
     for. */
  .info-toggle.corner {
    position: absolute;
    top: 0;
    right: 0;
    z-index: 2;
    width: 44px;
    height: 44px;
    min-height: 0;
    display: grid;
    place-items: start end;
    padding: 4px;
    border: 0;
    border-radius: 0;
    background: transparent;
  }
  .info-toggle.corner .mark {
    width: 19px;
    height: 19px;
    display: grid;
    place-items: center;
    border: 1px solid var(--edge);
    border-radius: 50%;
    color: var(--dim);
    background: var(--surface);
    font-size: 0.62rem;
    line-height: 1;
  }
  .info-toggle.corner:hover,
  .info-toggle.corner.on {
    background: transparent;
  }
  .info-toggle.corner:hover .mark,
  .info-toggle.corner.on .mark {
    color: var(--primary);
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 14%, var(--surface));
  }
  /* Focus has to be visible on the MARK, not on a 44 px transparent box
     whose outline would sit over the tile and read as the tile's own. */
  .info-toggle.corner:focus-visible {
    outline: none;
  }
  .info-toggle.corner:focus-visible .mark {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }
</style>
