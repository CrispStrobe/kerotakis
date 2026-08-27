<script lang="ts">
  import { untrack } from "svelte";
  import type { ApparatusSpec } from "../apparatus";
  import type { ShelfItem } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let {
    spec,
    vessel,
    shelf,
    busy,
    initialValues = {},
    onrun,
    onpreview,
    onclose,
  }: {
    spec: ApparatusSpec;
    vessel: number;
    shelf: ShelfItem[];
    busy: boolean;
    initialValues?: Record<string, number | string>;
    onrun: (line: string) => void;
    onpreview?: (values: Record<string, number | string>) => void;
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
  $effect(() => onpreview?.({ ...values }));
</script>

<section class="apparatus" aria-label={t("{apparatus} over v{vessel}", { apparatus: t(spec.title), vessel: vessel + 1 })}>
  <div class="head">
    <span class="live-mark" aria-hidden="true"></span>
    <span class="title"><small>{t("workstation · vessel v{vessel}", { vessel: vessel + 1 })}</small><strong>{t(spec.title)}</strong></span>
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
    {#if warning}<p class="warning" role="alert">⚠ {t(warning)}</p>{/if}
    <button class="run" disabled={busy || line === null || warning !== null} onclick={() => line && !warning && onrun(line)}>
      {busy ? t("running…") : t("run {apparatus}", { apparatus: t(spec.title) })}
    </button>
  </div>
</section>

<style>
  .apparatus {
    position: absolute;
    z-index: 12;
    top: 0.7rem;
    right: 0.7rem;
    width: min(23rem, calc(100% - 1.4rem));
    max-height: calc(100% - 5.8rem);
    overflow: auto;
    margin: 0;
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
  .blurb {
    margin: 0.28rem 0 0.55rem 1.6rem;
    color: var(--dim);
    font-size: 0.69rem;
    line-height: 1.3;
  }
  .fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
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
  label:has(select) { grid-column: 1 / -1; }
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
    width: 5rem;
  }
  .parameter-control { display: flex; align-items: center; gap: .45rem; }
  .exact-value { display: flex; align-items: center; gap: .25rem; color: var(--ink); }
  .exact-value small { min-width: 1.5rem; color: var(--dim); font-size: .66rem; }
  .dial {
    width: clamp(5rem, 9vw, 8rem);
    min-height: 34px;
    padding: 0;
    border: 0;
    background: transparent;
    accent-color: var(--instrument);
    cursor: ew-resize;
  }
  .run {
    grid-column: 1 / -1;
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
      inset: auto 0.55rem calc(4rem + env(safe-area-inset-bottom)) 0.55rem;
      width: auto;
      max-height: min(62vh, 30rem);
      border-radius: 18px;
    }
  }
</style>
