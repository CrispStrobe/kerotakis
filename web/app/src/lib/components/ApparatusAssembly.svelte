<script lang="ts">
  /**
   * The assembly, annotated onto the vessel it is built around.
   *
   * This is an SVG layer, not a panel: it is rendered inside the vessel's
   * own `0 0 100 140` viewBox, on top of the pieces `DeployedApparatus`
   * has already drawn there. A hairline runs down the assembly in the
   * order the parts connect, and a marker sits on each piece carrying the
   * part's name as its `<title>` — so "which bit is the sealed
   * connection" is answered by pointing at the drawing rather than by
   * reading a row of chips in a 12 rem column beside it.
   *
   * A part needing attention (an unbalanced centrifuge, a mortar with no
   * solid chosen) is drawn as a warning ring with a halo, because that is
   * the one thing the picture underneath cannot say for itself. The strip
   * repeats it in words; a colour alone is never the only carrier.
   *
   * `aria-hidden`: the vessel already publishes "{tool} installed: ready"
   * as its own label and the strip names every part behind the (i). A
   * screen reader walking eleven unlabelled markers would be reading the
   * decoration twice.
   */
  import { t } from "../i18n.svelte";
  import { assemblyFor } from "../apparatusAssembly";

  let { tool, values = {} }: { tool: string; values?: Record<string, number | string> } = $props();

  const assembly = $derived(assemblyFor(tool, values));
  const at = $derived(new Map(assembly.parts.map((item) => [item.id, item.at])));
</script>

<g class="assembly">
  {#each assembly.edges as [from, to] (`${from}-${to}`)}
    {@const a = at.get(from)}
    {@const b = at.get(to)}
    {#if a && b}
      <path class="link" d={`M ${a[0]} ${a[1]} L ${b[0]} ${b[1]}`} aria-hidden="true" />
    {/if}
  {/each}
  {#each assembly.parts as item (item.id)}
    <g class="marker" class:attention={item.state === "attention"}>
      {#if item.state === "attention"}
        <circle class="halo" cx={item.at[0]} cy={item.at[1]} r="6" />
      {/if}
      <circle class="ring" cx={item.at[0]} cy={item.at[1]} r="3.4" />
      <circle class="pip" cx={item.at[0]} cy={item.at[1]} r="1.3" />
      <title>{t(item.label)}</title>
    </g>
  {/each}
</g>

<style>
  /* Pointer events stay on: the marker is small, but a `<title>` on it is
     the only way a mouse can ask the drawing what a piece is called. */
  .assembly { color: var(--instrument); }
  .link { fill: none; stroke: var(--instrument); stroke-width: .7; stroke-dasharray: 2.5 2; opacity: .5; }
  .ring { fill: var(--surface); fill-opacity: .82; stroke: var(--instrument); stroke-width: 1.1; }
  .pip { fill: var(--instrument); }
  .marker.attention .ring { stroke: var(--warning); }
  .marker.attention .pip { fill: var(--warning); }
  /* `transform-box` and `transform-origin` belong on the element, not in
     the keyframe: a scale on an SVG circle otherwise pivots on the SVG's
     own origin and the halo slides off the marker instead of pulsing on
     it. */
  .halo {
    fill: var(--warning);
    opacity: .18;
    transform-box: fill-box;
    transform-origin: center;
    animation: assembly-attention 1.4s ease-in-out infinite alternate;
  }
  @keyframes assembly-attention { to { opacity: .38; transform: scale(1.18); } }
  @media (prefers-reduced-motion: reduce) {
    .halo { animation: none; opacity: .34; }
  }
</style>
