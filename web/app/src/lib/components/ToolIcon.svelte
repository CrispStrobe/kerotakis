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
    drain: "M 5 2 H 13 V 8 Q 13 10 11 10 H 7 Q 5 10 5 8 Z M 9 11 V 15 M 7 13.5 L 9 16 L 11 13.5",
    cell: "M 4 6 V 12 M 6.5 4 V 14 M 11.5 4 V 14 M 14 6 V 12 M 6.5 2.5 H 11.5",
    distil: "M 5 2 V 7 L 2.5 13 Q 2 15 4 15 H 8 Q 10 15 9.5 13 L 7 7 V 2 M 6 5 H 12 Q 14 5 14.5 8 L 15.5 12 M 14 14 H 17",
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
