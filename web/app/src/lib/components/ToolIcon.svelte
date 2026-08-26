<script lang="ts">
  /**
   * Tool portraits (GUI-033 art): each bench tool drawn as the apparatus
   * it stands for, in the current text colour. Pure decoration over the
   * existing labels — the name stays beside the picture, because an icon
   * language nobody has learned yet is not a UI.
   */
  let { name }: { name: string } = $props();

  // 18×18 viewBox, stroke-only line art.
  const PATHS: Record<string, string> = {
    burette: "M 9 1 V 12 M 7 1 H 11 M 9 12 L 7.5 14 H 10.5 L 9 12 M 9 14 V 17 M 6 8 H 12",
    filter: "M 2 3 H 16 L 10.5 9 V 15 L 7.5 16.5 V 9 Z",
    decant: "M 3 4 L 10 2.5 L 11.5 8.5 L 5.5 10.5 Z M 12 9 Q 14 11 13.5 14 M 12.5 16 H 16",
    mix: "M 2 3 H 6 L 7 8 M 16 3 H 12 L 11 8 M 7 8 Q 9 10 11 8 M 6 9 L 4 15 H 14 L 12 9",
    drain: "M 5 2 H 13 V 8 Q 13 10 11 10 H 7 Q 5 10 5 8 Z M 9 11 V 15 M 7 13.5 L 9 16 L 11 13.5",
    cell: "M 4 6 V 12 M 6.5 4 V 14 M 11.5 4 V 14 M 14 6 V 12 M 6.5 2.5 H 11.5",
    distil: "M 5 2 V 7 L 2.5 13 Q 2 15 4 15 H 8 Q 10 15 9.5 13 L 7 7 V 2 M 6 5 H 12 Q 14 5 14.5 8 L 15.5 12 M 14 14 H 17",
    dilute: "M 5 2 H 11 L 13 5 V 15 H 3 V 5 Z M 11 2 V 5 H 13 M 8 7 V 13 M 5 10 H 11",
    evaporate: "M 3 11 Q 9 15 15 11 L 13 15 H 5 Z M 6 8 Q 4 5 7 3 M 11 8 Q 9 5 12 2",
    electrolyse: "M 3 5 H 15 V 15 H 3 Z M 6 2 V 12 M 12 2 V 12 M 4.5 3.5 H 7.5 M 12 3 H 12",
    grind: "M 3 12 Q 9 16 15 12 L 13.5 15 H 4.5 Z M 5 4 L 13 11 M 7 2 L 15 9",
    irradiate: "M 5 5 L 11 3 L 14 8 L 8 10 Z M 8 10 L 5 16 M 10 12 L 13 15 M 12 10 L 16 11",
    regulate: "M 4 7 H 14 V 15 H 4 Z M 9 1 V 7 M 5 4 H 13 M 6 10 H 12",
    sweep: "M 3 5 H 15 V 14 H 3 Z M 1 9 H 8 M 6 7 L 9 9 L 6 11 M 10 4 Q 12 1 14 4",
    react: "M 6 2 H 12 M 8 2 V 7 L 4 14 Q 3 16 6 16 H 12 Q 15 16 14 14 L 10 7 V 2 M 6 12 Q 9 10 13 12",
    transport: "M 2 4 H 6 V 14 H 2 Z M 12 4 H 16 V 14 H 12 Z M 6 7 H 12 M 9 5 L 12 7 L 9 9",
  };
  const d = $derived(PATHS[name]);
</script>

{#if d}
  <svg viewBox="0 0 18 18" aria-hidden="true" class="toolicon">
    <path {d} />
  </svg>
{/if}

<style>
  .toolicon {
    width: 16px;
    height: 16px;
    vertical-align: -3px;
    margin-right: 0.25rem;
  }
  path {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.3;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
