<script lang="ts">
  import { untrack } from "svelte";
  import type { ApparatusSpec } from "../apparatus";
  import type { ShelfItem } from "../session.svelte";
  import { i18n, t } from "../i18n.svelte";

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
  const solids = $derived(shelf.filter((s) => s.phase.toLowerCase().includes("solid")));
  const line = $derived(spec.build(vessel, values));
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
  <div class="head">
    <span class="live-mark" aria-hidden="true"></span>
    <span class="title"><small>{t("workstation · vessel v{vessel}", { vessel: vessel + 1 })}</small><strong>{t(spec.title)}</strong></span>
    {#if selectedVessel !== vessel && onretarget}
      <button class="retarget" disabled={busy} onclick={onretarget}>{t("move to selected v{vessel}", { vessel: selectedVessel + 1 })}</button>
    {/if}
    <button class="icon-close" aria-label={t("put away")} title={t("put away")} onclick={onclose}>×</button>
  </div>
  <p class="blurb">{t(spec.blurb)}</p>
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
  </div>
</section>

<style>
  .apparatus {
    position: relative;
    z-index: 12;
    width: auto;
    max-height: min(17rem, 42%);
    overflow: auto;
    flex: none;
    display: grid;
    grid-template-columns: minmax(10.5rem, 0.7fr) minmax(0, 2.3fr);
    grid-template-rows: auto 1fr;
    column-gap: 0.9rem;
    margin: 0.55rem 0.65rem 0.65rem;
    padding: 0.7rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 35%, var(--edge));
    border-radius: 16px;
    background: color-mix(in srgb, var(--surface) 92%, var(--instrument) 8%);
    box-shadow: 0 8px 24px var(--shadow);
  }
  .head {
    display: flex;
    gap: 0.6rem;
    align-items: center;
  }
  .title { min-width: 0; display: flex; flex: 1; flex-direction: column; }
  .head small { color: var(--instrument); font-size: .56rem; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
  .live-mark { width: 10px; height: 10px; flex: none; border: 2px solid var(--surface); border-radius: 50%; background: var(--instrument); box-shadow: 0 0 0 2px var(--instrument); }
  .retarget { flex: none; min-height: 30px; padding: .25rem .48rem; border: 1px solid var(--instrument); border-radius: 8px; background: color-mix(in srgb, var(--surface) 82%, var(--instrument)); color: var(--ink); font-size: .62rem; font-weight: 750; }
  .blurb {
    margin: 0.28rem 0 0 1.6rem;
    color: var(--dim);
    font-size: 0.69rem;
    line-height: 1.3;
  }
  .fields {
    grid-column: 2;
    grid-row: 1 / span 2;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(8.5rem, 1fr));
    gap: 0.55rem;
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
  .warning { margin: 0; max-width: 15rem; color: var(--danger); font-size: .75rem; font-weight: 750; }
  .icon-close {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    flex: none;
    padding: 0;
    border: 1px solid var(--edge);
    border-radius: 9px;
    color: var(--dim);
    background: var(--surface);
    font: inherit;
    font-size: 1rem;
    cursor: pointer;
  }
  .icon-close:hover { color: var(--danger); border-color: var(--danger); }
  @media (max-width: 700px) {
    .apparatus {
      position: fixed;
      display: block;
      /* The phone shell keeps both the command bar and its three-pane tabs
         below the bench. Clear both, not only the final tab row. */
      inset: auto 0.55rem calc(7.25rem + env(safe-area-inset-bottom)) 0.55rem;
      width: auto;
      max-height: min(62vh, 30rem);
      border-radius: 18px;
    }
    .blurb { margin-bottom: 0.55rem; }
    .fields { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    label:has(select), .readouts, .run { grid-column: 1 / -1; }
  }
</style>
