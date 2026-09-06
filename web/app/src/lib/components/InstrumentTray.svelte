<script lang="ts">
  /**
   * Quick access, not the catalogue (GUI-101).
   *
   * This row used to be all twelve instruments in one non-wrapping strip.
   * That was already a compromise — twelve wrapped pills became five stacked
   * rows and pushed the bench off a narrow screen — but a row that scrolls
   * sideways only moves the problem: on a phone roughly half the tray sat
   * past the right edge, so the calorimeter and the Geiger counter were in
   * the DOM and absent from the product, and the row was still a second full
   * listing of everything the Geräteschrank already held.
   *
   * So it keeps four: the ones this learner reached for most recently,
   * seeded with the four commonest measurements the vessel dock does not
   * already carry. Four pills and the cupboard door fit without scrolling,
   * where five push the door off a 320 px screen — and the door is what the
   * row exists to lead to. Everything else — every instrument, every
   * apparatus, every kit, on shelves, with an (i) each — is one tap away
   * behind that button, which is why it sits INSIDE the row and costs no
   * vertical space.
   *
   * `look`, `thermometer` and `pH` are never offered here (GUI-103): the
   * dock carries those three on every vessel as fixed landmarks, and a row
   * of four that spent three slots repeating them would have left one slot
   * for the nine instruments that have nowhere else to be.
   *
   * The glyph is what makes the row scannable once you know it, but the name
   * stays beside it: a learner meeting a calorimeter for the first time
   * cannot be expected to recognise it from a symbol. Chemistry's own
   * notation does the work where it has some — pH, mL, λ are the names of
   * these measurements, not abbreviations of them.
   */
  import { t } from "../i18n.svelte";
  import { INSTRUMENTS, instrumentCommand } from "../instruments";
  import { quickAccessRow } from "../instrumentRecents";
  import { instrumentSurface } from "../instrumentSurface.svelte";
  let {
    vessel,
    busy,
    onmeasure,
  }: {
    vessel: number;
    busy: boolean;
    onmeasure: (line: string) => void;
  } = $props();

  const v = $derived(`v${vessel + 1}`);
  const tokens = INSTRUMENTS.map((item) => item.token);
  instrumentSurface.hydrate();
  const quick = $derived(
    quickAccessRow(instrumentSurface.recent, tokens)
      .flatMap((token) => INSTRUMENTS.filter((item) => item.token === token)),
  );

  function measure(token: string) {
    instrumentSurface.used(token, tokens);
    onmeasure(instrumentCommand(vessel, token));
  }
</script>

<div class="tray" role="group" aria-label={t("instruments for {vessel}", { vessel: v })}>
  {#each quick as inst (inst.token)}
    <button
      disabled={busy}
      data-token={inst.token}
      title={t(inst.label)}
      onclick={() => measure(inst.token)}
    >
      <span class="glyph" class:word={inst.glyph.length > 1} aria-hidden="true">{inst.glyph}</span>
      <span class="name">{t(inst.label)}</span>
    </button>
  {/each}
  <!-- The door to everything else. Sticky, so it stays reachable even if a
       locale with longer names ever pushes the row past its container. -->
  <button
    class="cupboard-door"
    title={t("Open the equipment cabinet")}
    aria-label={t("Open the equipment cabinet")}
    onclick={() => (instrumentSurface.open = true)}
  >
    <span class="glyph" aria-hidden="true">▦</span>
    <span class="name">{t("all equipment")}</span>
  </button>
</div>

<style>
  .tray {
    display: flex;
    flex-wrap: nowrap;
    gap: 0.35rem;
    padding: 0.5rem 1rem;
    border-top: 1px solid var(--edge);
    /* Four pills and the door fit; the overflow rule stays as the guard for
       a locale whose names are longer than German's. */
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
    -webkit-overflow-scrolling: touch;
  }
  button {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    font: inherit;
    font-size: 0.74rem;
    padding: 0.25rem 0.7rem;
    cursor: pointer;
    min-height: 34px;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    /* Never let a long German name squeeze its neighbours thin. */
    flex: 0 0 auto;
    white-space: nowrap;
  }
  button:hover:not(:disabled) {
    color: var(--ink);
    border-color: var(--cool);
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .cupboard-door {
    position: sticky;
    right: 0;
    margin-left: auto;
    border-color: color-mix(in srgb, var(--action) 45%, var(--edge));
    color: var(--ink);
    background: color-mix(in srgb, var(--action) 12%, var(--panel));
  }
  .cupboard-door:hover {
    border-color: var(--action);
  }
  .glyph {
    font-size: 0.95rem;
    line-height: 1;
  }
  /* The typographic ones (pH, mL, λ, Rf) are read, not looked at, so they
     want the smaller, steadier treatment the emoji do not. */
  .glyph.word {
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    opacity: 0.85;
  }
  /* Below a phone's width the names are what costs the room, so the glyphs
     carry the row alone and the name survives as the accessible name. */
  @media (max-width: 30rem) {
    .name {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
      white-space: nowrap;
    }
    button {
      padding: 0.25rem;
      min-width: 40px;
      justify-content: center;
    }
    .glyph {
      font-size: 1.1rem;
    }
    .glyph.word {
      font-size: 0.8rem;
    }
  }
</style>
