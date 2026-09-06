<!--
  What an (i) opens: a definition list, not a paragraph.

  The point of moving copy behind a disclosure is lost if what comes out is
  the same wall with a delay in front of it. So the panel is `<dt>`/`<dd>`
  pairs — every line says what it is a fact ABOUT — and the caller decides
  which rows exist, so a fact that has no answer is absent rather than
  present and empty.
-->
<script lang="ts">
  import type { InfoRow } from "../infoPanel";

  let { id, rows }: { id: string; rows: InfoRow[] } = $props();
</script>

<dl class="detail" {id}>
  {#each rows as row, index (index)}
    <div class:block={row.block} data-tone={row.tone ?? null}>
      <dt>{row.term}</dt>
      <dd>{row.detail}</dd>
    </div>
  {/each}
</dl>

<style>
  .detail {
    margin: 0 0.2rem 0.5rem;
    padding: 0.5rem 0.55rem;
    border-left: 3px solid var(--primary);
    border-radius: 7px;
    background: color-mix(in srgb, var(--primary) 6%, transparent);
    font-size: 0.67rem;
    line-height: 1.35;
    /* German compounds are longer than the 240px pane; the document's
       `lang` lets the browser hyphenate them rather than widen it. */
    hyphens: auto;
    overflow-wrap: anywhere;
  }
  .detail div {
    display: flex;
    justify-content: space-between;
    gap: 0.6rem;
  }
  .detail div + div {
    margin-top: 0.22rem;
  }
  /* A sentence gets the full width; only short values sit opposite. */
  .detail div.block {
    display: block;
  }
  .detail div.block dd {
    text-align: left;
  }
  .detail dt {
    color: var(--dim);
  }
  .detail dd {
    margin: 0;
    text-align: right;
  }
  .detail div[data-tone="info"] dd {
    color: var(--instrument);
  }
  .detail div[data-tone="warn"] dd {
    color: var(--warning);
  }
  .detail div[data-tone="danger"] dd {
    color: var(--warning);
    font-weight: 700;
  }
</style>
