<script lang="ts">
  /**
   * The workstation, as a strip over the stage.
   *
   * This panel used to be three columns wide: a title and blurb; a
   * "PHYSISCHER AUFBAU" row of labelled chips; and the controls. On a
   * German build the middle column read "◯ Ballon oder Gasbeutel — ◉
   * Dichte Verbindung — ▽" inside 12 rem, scrolling sideways, while the
   * controls were squeezed into what was left and the stage lost the
   * panel's whole height from the top of the bench.
   *
   * All three of those are the same mistake: the panel was explaining
   * equipment that is DRAWN, a few centimetres above it, on the vessel it
   * is attached to. So the assembly moved onto the vessel's own SVG
   * (`ApparatusAssembly.svelte`, mounted by `Vessel.svelte` over the
   * pieces `DeployedApparatus` draws), the sentence moved behind the (i)
   * beside the title, and what is left here is one strip: which vessel,
   * which apparatus, anything needing attention, and the controls that
   * run it.
   *
   * The strip is absolutely positioned inside `.bench-pane`, which is the
   * stage's own positioned, clipped box — so it overlays the bench rather
   * than pushing it up, and it can never widen the page. Below 700 px it
   * becomes what it already was: a full-width bottom sheet clearing the
   * command bar, never a narrow column.
   */
  import { untrack } from "svelte";
  import type { ApparatusSpec } from "../apparatus";
  import type { InfoRow } from "../infoPanel";
  import type { ShelfItem } from "../session.svelte";
  import { i18n, t } from "../i18n.svelte";
  import { assemblyAttention, assemblyFor, drawnOnStage } from "../apparatusAssembly";
  import InfoPanel from "./InfoPanel.svelte";
  import InfoToggle from "./InfoToggle.svelte";

  let {
    spec,
    vessel,
    selectedVessel = vessel,
    shelf,
    busy,
    initialValues = {},
    onrun,
    onpreview,
    onretarget,
    onclose,
  }: {
    spec: ApparatusSpec;
    vessel: number;
    selectedVessel?: number;
    shelf: ShelfItem[];
    busy: boolean;
    initialValues?: Record<string, number | string>;
    onrun: (line: string) => void;
    onpreview?: (values: Record<string, number | string>) => void;
    onretarget?: () => void;
    onclose: () => void;
  } = $props();

  let values = $state<Record<string, number | string>>(
    untrack(() => ({
      ...Object.fromEntries(spec.fields.map((f) => [f.name, f.default])),
      ...initialValues,
    })),
  );
  // `phase` is a wire key from the engine and is always English, which is
  // why this literal is safe. Localise `phase` at the source and this
  // silently yields no solids at all — translate it for display only.
  const solids = $derived(
    // i18n-ok: `phase` is an engine wire key and always English.
    shelf.filter((s) => s.phase.toLowerCase().includes("solid")),
  );
  let infoOpen = $state(false);
  const infoId = $derived(`apparatus-info-${spec.verb}`);
  const assembly = $derived(assemblyFor(spec.verb, values));
  const attention = $derived(assemblyAttention(assembly));
  /**
   * What the (i) says, and nothing the strip already shows.
   *
   * The parts are named here as well as on the stage markers, because a
   * `<title>` on a 4 px ring is a mouse affordance and this has to work
   * for a finger and a screen reader too. Where the operator has no
   * drawing on the vessel — the centrifuge, whose tubes leave the bench —
   * the panel says so rather than implying a picture that is not there.
   */
  const infoRows = $derived<InfoRow[]>([
    { term: t("what this does"), detail: t(spec.blurb), block: true },
    { term: t("parts"), detail: assembly.parts.map((item) => t(item.label)).join(" · "), block: true },
    ...(drawnOnStage(spec.verb)
      ? []
      : [{ term: t("physical setup"), detail: t("this setup is not drawn on the vessel"), block: true } as InfoRow]),
  ]);
  const line = $derived(spec.build(vessel, values));
  const secondaryLine = $derived(spec.secondary?.build(vessel, values) ?? null);
  const warning = $derived(spec.warning?.(values) ?? null);
  const readouts = $derived(spec.readouts?.(values) ?? []);
  const formatReadout = (readout: (typeof readouts)[number]) =>
    `${new Intl.NumberFormat(i18n.locale, {
      minimumFractionDigits: readout.digits,
      maximumFractionDigits: readout.digits,
    }).format(readout.value)} ${readout.unit}`;
  $effect(() => onpreview?.({ ...values }));
</script>

<section class="apparatus" aria-label={t("{apparatus} over v{vessel}", { apparatus: t(spec.title), vessel: vessel + 1 })}>
  <div class="strip">
    <span class="live-mark" aria-hidden="true"></span>
    <span class="title"><small>v{vessel + 1}</small><strong>{t(spec.title)}</strong></span>
    {#if attention.length > 0}
      <!-- The one thing the drawing on the vessel cannot say for itself.
           The marker up there is a warning ring; this is the same fact in
           words, so the colour is never the only carrier. -->
      <span class="attention" role="status">⚠ {attention.map((item) => t(item.label)).join(" · ")}</span>
    {/if}
    <InfoToggle
      expanded={infoOpen}
      controls={infoId}
      label={t("about {name}", { name: t(spec.title) })}
      onclick={() => (infoOpen = !infoOpen)}
    />
    {#if selectedVessel !== vessel && onretarget}
      <button class="retarget" disabled={busy} onclick={onretarget}>{t("move to selected v{vessel}", { vessel: selectedVessel + 1 })}</button>
    {/if}
    <button class="icon-close" aria-label={t("close")} title={t("close")} onclick={onclose}>×</button>
  </div>
  {#if infoOpen}<InfoPanel id={infoId} rows={infoRows} />{/if}
  <div class="fields">
    {#each spec.fields as f (f.name)}
      <label>
        {t(f.label)}
        {#if f.type === "species"}
          <select bind:value={values[f.name]}>
            <option value="">{t("choose…")}</option>
            {#each solids.length > 0 ? solids : shelf as s (s.key)}
              <option value={s.key}>{t(s.name)}</option>
            {/each}
          </select>
        {:else if f.type === "choice"}
          <!-- A fixed list the spec owns, not a shelf lookup: the values
               are grammar tokens the engine parses, so only the label is
               translated. -->
          <select bind:value={values[f.name]}>
            {#each f.options ?? [] as option (option.value)}
              <option value={option.value}>{t(option.label)}</option>
            {/each}
          </select>
        {:else}
          <span class="parameter-control">
            {#if f.min !== undefined && f.max !== undefined}
              <input
                class="dial"
                type="range"
                aria-label={t("{parameter} slider", { parameter: t(f.label) })}
                bind:value={values[f.name]}
                min={f.min}
                max={f.max}
                step={f.step ?? 1}
              />
            {/if}
            <span class="exact-value">
              <input
                type="number"
                bind:value={values[f.name]}
                min={f.min}
                max={f.max}
                step={f.step ?? 1}
              />
              {#if f.unit}<small>{f.unit}</small>{/if}
            </span>
          </span>
        {/if}
      </label>
    {/each}
    {#if readouts.length > 0}
      <div class="readouts" class:multiple={readouts.length > 1} aria-label={t("computed operating values")}>
        {#each readouts as readout (readout.label)}
          <output>
            <small>{t(readout.label)}</small>
            <strong>{formatReadout(readout)}</strong>
          </output>
        {/each}
      </div>
    {/if}
    {#if warning}<p class="warning" role="alert">⚠ {t(warning)}</p>{/if}
    <button class="run" disabled={busy || line === null || warning !== null} onclick={() => line && !warning && onrun(line)}>
      {busy ? t("running…") : t("run {apparatus}", { apparatus: t(spec.title) })}
    </button>
    {#if spec.secondary}
      <button class="secondary" disabled={busy || secondaryLine === null} onclick={() => secondaryLine && onrun(secondaryLine)}>
        {t(spec.secondary.label)}
      </button>
    {/if}
  </div>
</section>

<style>
  /* An overlay on the stage, not a band under it. `.bench-pane` is
     positioned and clipped, so this is anchored to the stage's own foot,
     takes no height from the bench, and can never widen the page. */
  .apparatus {
    position: absolute;
    z-index: 12;
    inset: auto 0.65rem 0.65rem;
    max-height: min(13.5rem, 52%);
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.55rem 0.7rem 0.7rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 35%, var(--edge));
    border-radius: 16px;
    background: color-mix(in srgb, var(--surface) 92%, var(--instrument) 8%);
    box-shadow: 0 8px 24px var(--shadow);
  }
  /* One row wherever it fits: the title shrinks and ellipses first, and
     only the retarget button — which appears rarely and is a sentence —
     is allowed to take a second line rather than push the close off the
     end of a scrolling row. */
  .strip {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: center;
  }
  .title { min-width: 0; display: flex; flex: 1; align-items: baseline; gap: 0.4rem; }
  .title strong { min-width: 0; overflow: hidden; font-size: .84rem; text-overflow: ellipsis; white-space: nowrap; }
  .strip small { flex: none; color: var(--instrument); font-size: .6rem; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
  .attention { min-width: 0; overflow: hidden; color: var(--warning); font-size: .66rem; font-weight: 750; text-overflow: ellipsis; white-space: nowrap; }
  .live-mark { width: 10px; height: 10px; flex: none; border: 2px solid var(--surface); border-radius: 50%; background: var(--instrument); box-shadow: 0 0 0 2px var(--instrument); }
  .retarget { min-width: 0; min-height: 30px; padding: .25rem .48rem; border: 1px solid var(--instrument); border-radius: 8px; background: color-mix(in srgb, var(--surface) 82%, var(--instrument)); color: var(--ink); font-size: .62rem; font-weight: 750; }
  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(8.5rem, 1fr));
    gap: 0.5rem;
    align-items: flex-end;
    font-size: 0.82rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    color: var(--dim);
  }
  label:has(select) { grid-column: span 2; }
  select,
  input {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.25rem 0.4rem;
    min-height: 34px;
  }
  input[type="number"] {
    width: 4rem;
  }
  .parameter-control {
    width: 100%;
    display: grid;
    grid-template-columns: minmax(3.5rem, 1fr) auto;
    align-items: center;
    gap: .35rem;
  }
  .exact-value { display: flex; align-items: center; gap: .25rem; color: var(--ink); }
  .exact-value small { min-width: 1.5rem; color: var(--dim); font-size: .66rem; }
  .dial {
    width: 100%;
    min-width: 0;
    min-height: 34px;
    padding: 0;
    border: 0;
    background: transparent;
    accent-color: var(--instrument);
    cursor: ew-resize;
  }
  .run {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.3rem 0.9rem;
    cursor: pointer;
    min-height: 36px;
  }
  .secondary {
    min-height: 36px;
    padding: 0.3rem 0.9rem;
    border: 1px solid var(--danger);
    border-radius: 6px;
    color: var(--ink);
    background: color-mix(in srgb, var(--danger) 10%, var(--panel-raised));
    font: inherit;
    font-size: 0.82rem;
    cursor: pointer;
  }
  .readouts {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .readouts.multiple { grid-column: span 2; }
  .readouts.multiple output {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.08rem;
  }
  .readouts output {
    min-width: 0;
    display: flex;
    flex: 1;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.42rem 0.55rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 28%, var(--edge));
    border-radius: 9px;
    color: var(--instrument);
    background: color-mix(in srgb, var(--instrument) 7%, var(--surface));
  }
  .readouts small { color: var(--dim); font-size: 0.63rem; }
  .readouts strong { color: var(--ink); font-size: 0.76rem; white-space: nowrap; }
  .warning { grid-column: 1 / -1; margin: 0; color: var(--danger); font-size: .75rem; font-weight: 750; }
  /* Where the stage cannot host a strip, a full-width bottom sheet — never
     a narrow column. `fixed` rather than the stage's own foot, because the
     phone shell puts the command bar and the three-pane tabs below the
     bench and the sheet has to clear both. */
  @media (max-width: 700px) {
    .apparatus {
      position: fixed;
      inset: auto 0.55rem calc(7.25rem + env(safe-area-inset-bottom)) 0.55rem;
      max-height: min(62vh, 30rem);
      border-radius: 18px;
    }
    .fields { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    label:has(select), .readouts, .run { grid-column: 1 / -1; }
  }
  /* Full-bleed on a narrow phone: 0.55rem of gutter either side of a
     390 px viewport is 2.5 rem of a two-column control grid gone. */
  @media (max-width: 24rem) {
    .apparatus {
      inset: auto 0 calc(7.25rem + env(safe-area-inset-bottom)) 0;
      border-radius: 16px 16px 0 0;
    }
  }
</style>
