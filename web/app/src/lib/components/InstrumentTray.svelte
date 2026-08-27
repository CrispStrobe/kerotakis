<script lang="ts">
  /**
   * The instruments, as one scrolling row rather than a wall of pills.
   *
   * Ten instruments with `flex-wrap: wrap` became five stacked rows of
   * text on a narrow screen, which reads as a menu to work through rather
   * than a tray to reach into, and pushed the bench itself off-screen.
   *
   * So: one row that scrolls sideways, each instrument carrying a glyph.
   * The glyph is not decoration — it is what makes the row scannable at a
   * glance once you know it — but the name stays beside it, because a
   * learner meeting a calorimeter for the first time cannot be expected to
   * recognise it from a symbol. Chemistry's own notation does the work
   * where it has some: pH, mL, λ are already the names of these
   * measurements, not abbreviations of them.
   */
  import { t } from "../i18n.svelte";
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
  // Every instrument the grammar's measure arm accepts, by its token.
  const INSTRUMENTS: { token: string; label: string; glyph: string }[] = [
    { token: "smell", label: "safe waft", glyph: "≋" },
    { token: "thermometer", label: "thermometer", glyph: "🌡" },
    { token: "ph", label: "pH meter", glyph: "pH" },
    { token: "balance", label: "balance", glyph: "⚖" },
    { token: "volume", label: "volume", glyph: "mL" },
    { token: "conductivity", label: "conductivity", glyph: "⚡" },
    { token: "pressure", label: "pressure gauge", glyph: "bar" },
    { token: "calorimeter", label: "calorimeter", glyph: "kJ" },
    { token: "uvvis", label: "UV-Vis", glyph: "λ" },
    { token: "eyes", label: "look closely", glyph: "🔍" },
    { token: "chromatograph", label: "chromatograph", glyph: "Rf" },
    { token: "geiger", label: "Geiger counter", glyph: "Bq" },
  ];
</script>

<div class="tray" role="group" aria-label={t("instruments for {vessel}", { vessel: v })}>
  {#each INSTRUMENTS as inst (inst.token)}
    <button
      disabled={busy}
      title={t(inst.label)}
      onclick={() =>
        onmeasure(
          inst.token === "chromatograph"
            ? `chromatograph ${v}`
            : inst.token === "smell"
              ? `smell ${v}`
              : `measure ${v} ${inst.token}`,
        )}
    >
      <span class="glyph" class:word={inst.glyph.length > 1} aria-hidden="true">{inst.glyph}</span>
      <span class="name">{t(inst.label)}</span>
    </button>
  {/each}
</div>

<style>
  .tray {
    display: flex;
    flex-wrap: nowrap;
    gap: 0.35rem;
    padding: 0.5rem 1rem;
    border-top: 1px solid var(--edge);
    /* Sideways instead of downwards: the bench must stay visible. */
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
    /* Momentum scrolling that stops on an instrument rather than between two. */
    scroll-snap-type: x proximity;
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
    /* Never let a long German name (Leitfähigkeit, Chromatograph) squeeze
       its neighbours thin — the row scrolls instead. */
    flex: 0 0 auto;
    white-space: nowrap;
    scroll-snap-align: start;
  }
  button:hover:not(:disabled) {
    color: var(--ink);
    border-color: var(--cool);
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
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
