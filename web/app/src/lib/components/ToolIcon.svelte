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
    bunsen: "M 5 16 H 13 M 7 16 V 9 H 11 V 16 M 6 9 H 12 M 9 8 Q 5 5 9 1 Q 13 5 9 8 Z",
    filter: "M 2 3 H 16 L 10.5 9 V 15 L 7.5 16.5 V 9 Z",
    decant: "M 3 4 L 10 2.5 L 11.5 8.5 L 5.5 10.5 Z M 12 9 Q 14 11 13.5 14 M 12.5 16 H 16",
    mix: "M 2 3 H 6 L 7 8 M 16 3 H 12 L 11 8 M 7 8 Q 9 10 11 8 M 6 9 L 4 15 H 14 L 12 9",
    drain: "M 5 2 H 13 V 8 Q 13 10 11 10 H 7 Q 5 10 5 8 Z M 9 11 V 15 M 7 13.5 L 9 16 L 11 13.5",
    magnet: "M 4 3 V 10 A 5 5 0 0 0 14 10 V 3 H 10 V 10 A 1 1 0 0 1 8 10 V 3 Z M 4 6 H 8 M 10 6 H 14",
    cell: "M 4 6 V 12 M 6.5 4 V 14 M 11.5 4 V 14 M 14 6 V 12 M 6.5 2.5 H 11.5",
    distil: "M 5 2 V 7 L 2.5 13 Q 2 15 4 15 H 8 Q 10 15 9.5 13 L 7 7 V 2 M 6 5 H 12 Q 14 5 14.5 8 L 15.5 12 M 14 14 H 17",
    dilute: "M 5 2 H 11 L 13 5 V 15 H 3 V 5 Z M 11 2 V 5 H 13 M 8 7 V 13 M 5 10 H 11",
    evaporate: "M 3 11 Q 9 15 15 11 L 13 15 H 5 Z M 6 8 Q 4 5 7 3 M 11 8 Q 9 5 12 2",
    electrolyse: "M 3 5 H 15 V 15 H 3 Z M 6 2 V 12 M 12 2 V 12 M 4.5 3.5 H 7.5 M 12 3 H 12",
    grind: "M 3 12 Q 9 16 15 12 L 13.5 15 H 4.5 Z M 5 4 L 13 11 M 7 2 L 15 9",
    centrifuge: "M 4 5 H 14 L 16 15 H 2 Z M 6 5 A 3 3 0 0 1 12 5 M 6 9 H 12 M 9 6 V 12",
    irradiate: "M 5 5 L 11 3 L 14 8 L 8 10 Z M 8 10 L 5 16 M 10 12 L 13 15 M 12 10 L 16 11",
    regulate: "M 4 7 H 14 V 15 H 4 Z M 9 1 V 7 M 5 4 H 13 M 6 10 H 12",
    sweep: "M 3 5 H 15 V 14 H 3 Z M 1 9 H 8 M 6 7 L 9 9 L 6 11 M 10 4 Q 12 1 14 4",
    stir: "M 3 12 H 15 L 14 16 H 4 Z M 5 9 H 13 M 6 6 Q 9 3 12 6 M 7 8 Q 9 10 11 8",
    heat: "M 3 12 H 15 L 14 16 H 4 Z M 6 9 Q 4 6 7 3 M 11 9 Q 9 6 12 2",
    /* `cool` named an icon that was never drawn, so the cooling bath has
       been an empty box on the shelf since the portraits landed —
       `ToolIcon` renders nothing for a name it has no path for, which is
       the right runtime behaviour and an invisible defect. Found by
       `ToolIcon.test.ts`, which now holds the catalogue against these keys.
       Drawn as the deliberate opposite of `heat` above: the same dish, a
       snowflake instead of the rising heat. */
    cool: "M 3 12 H 15 L 14 16 H 4 Z M 9 1.6 V 8 M 6.2 3.2 L 11.8 6.4 M 11.8 3.2 L 6.2 6.4",
    react: "M 6 2 H 12 M 8 2 V 7 L 4 14 Q 3 16 6 16 H 12 Q 15 16 14 14 L 10 7 V 2 M 6 12 Q 9 10 13 12",
    transport: "M 2 4 H 6 V 14 H 2 Z M 12 4 H 16 V 14 H 12 Z M 6 7 H 12 M 9 5 L 12 7 L 9 9",
    balloon: "M 9 2 C 4 2 3 6 4 9 C 5 12 7 14 9 14 C 11 14 13 12 14 9 C 15 6 14 2 9 2 Z M 8 14 L 10 14 L 9 16 M 9 16 C 7 17 11 17 9 18",
    candle: "M 6 7 H 12 V 17 H 6 Z M 9 7 C 5 5 9 1 9 1 C 13 5 11 7 9 7 Z M 9 7 V 9",
    "chromatography-paper": "M 4 2 H 14 V 16 H 4 Z M 6 13 H 12 M 7 11 H 11 M 6 8 H 12 M 7 5 H 11",
    /* GUI-109 — the measuring instruments, which had single characters
       where the apparatus had portraits. Same 18x18 box, same stroke-only
       hand, no text inside any of them. Each is one `d` with several
       subpaths so it stays a single element in the same shape as the
       drawings above. */
    // A vessel with its vapour rising: what a waft IS. Deliberately not a
    // hand — a cupped hand at 26 px is a blob, and the thing being sensed
    // is the vapour.
    waft: "M 5 9 V 14.8 Q 5 16.4 6.6 16.4 H 11.4 Q 13 16.4 13 14.8 V 9 M 4 9 H 14 M 7.3 7.2 Q 8.4 5.6 7.3 4 Q 6.2 2.4 7.3 1.2 M 10.9 7.2 Q 12 5.8 10.9 4.4 Q 9.8 3 10.9 2",
    // Stem, mercury column, scale ticks, bulb.
    thermometer: "M 7.4 4 A 1.6 1.6 0 0 1 10.6 4 V 10.8 M 7.4 4 V 10.8 M 6.3 13.4 A 2.7 2.7 0 0 0 11.7 13.4 A 2.7 2.7 0 0 0 6.3 13.4 M 9 6.6 V 12.4 M 11.7 6 H 13.6 M 11.7 8.4 H 12.9 M 11.7 10.8 H 13.6",
    // Beam, post, base, and two pans hanging from the beam ends.
    balance: "M 3 5 H 15 M 9 3 V 15 M 6.4 15 H 11.6 M 1.3 5 L 3 8.7 L 4.7 5 M 13.3 5 L 15 8.7 L 16.7 5",
    // A gas syringe: graduated barrel with the plunger drawn out of it.
    "gas-syringe": "M 2.6 6.2 H 11.4 V 11.8 H 2.6 Z M 1 9 H 2.6 M 11.4 5.2 V 12.8 M 11.4 9 H 15.4 M 15.4 6.4 V 11.6 M 5.4 6.2 V 8.4 M 8.2 6.2 V 8.4",
    // A conductivity cell: two electrodes standing in the liquid, wired up
    // to a meter. The bridge over the top is what says "a circuit", which
    // is the difference between this and a lightning bolt.
    conductivity: "M 4 6.6 V 14 Q 4 15.8 5.8 15.8 H 12.2 Q 14 15.8 14 14 V 6.6 M 3.1 6.6 H 14.9 M 6.4 4.4 V 13.2 M 11.6 4.4 V 13.2 M 7.5 9.4 H 10.6 M 9.6 8.4 L 10.6 9.4 L 9.6 10.4",
    // A Bourdon gauge: dial, needle, two marks on the face, and the stem
    // it is screwed into.
    manometer: "M 4 7.5 A 5 5 0 0 0 14 7.5 A 5 5 0 0 0 4 7.5 M 9 7.5 L 11.9 5.3 M 6.3 4.3 L 7.1 5.1 M 11.7 4.3 L 10.9 5.1 M 9 12.5 V 14 M 7.3 14 H 10.7 V 16.4 H 7.3 Z",
    // A double-walled cup with a lid and a thermometer through it: the
    // insulation is the whole point of the instrument, so it is what the
    // drawing shows.
    calorimeter: "M 3.5 5 V 14 Q 3.5 16 5.5 16 H 12.5 Q 14.5 16 14.5 14 V 5 M 5.6 6.6 V 13.2 Q 5.6 14.3 6.7 14.3 H 11.3 Q 12.4 14.3 12.4 13.2 V 6.6 M 2.4 5 H 15.6 M 10.4 1.2 V 11.6 M 9.4 1.2 H 11.4",
    // Source, beam, cuvette with a sample in it, beam, detector. The beam
    // passing THROUGH is what makes it a spectrophotometer rather than a
    // bottle.
    spectrophotometer: "M 1.4 6.6 V 11.4 M 1.4 9 H 6.2 M 6.2 3.8 H 11.8 V 14.2 H 6.2 Z M 6.2 6.6 H 11.8 M 11.8 9 H 16.6 M 15 8 L 16.2 9 L 15 10",
    magnifier: "M 3 7.4 A 4.4 4.4 0 0 0 11.8 7.4 A 4.4 4.4 0 0 0 3 7.4 M 10.6 10.6 L 15.4 15.4",
    // A developing jar with the strip standing in the solvent, a baseline
    // and two components that have run different distances. The unequal
    // heights are the separation, which is the reading.
    "chromatography-jar": "M 3.5 3.2 V 14.6 Q 3.5 16 5 16 H 13 Q 14.5 16 14.5 14.6 V 3.2 M 2.6 3.2 H 15.4 M 4.4 13.6 H 13.6 M 9 3.6 V 15.4 M 6.2 11.6 H 11.8 M 6.6 8.2 H 9 M 9 5.6 H 11.4",
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
