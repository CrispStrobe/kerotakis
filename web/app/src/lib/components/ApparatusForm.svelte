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
    onrun,
    onpreview,
    onclose,
  }: {
    spec: ApparatusSpec;
    vessel: number;
    shelf: ShelfItem[];
    busy: boolean;
    onrun: (line: string) => void;
    onpreview?: (values: Record<string, number | string>) => void;
    onclose: () => void;
  } = $props();

  let values = $state<Record<string, number | string>>(
    untrack(() => Object.fromEntries(spec.fields.map((f) => [f.name, f.default]))),
  );
  const solids = $derived(shelf.filter((s) => s.phase.toLowerCase().includes("solid")));
  const line = $derived(spec.build(vessel, values));
  $effect(() => onpreview?.({ ...values }));
</script>

<section class="apparatus" aria-label={t("{apparatus} over v{vessel}", { apparatus: t(spec.title), vessel: vessel + 1 })}>
  <div class="head">
    <span class="live-mark" aria-hidden="true"></span>
    <span><small>{t("deployed at vessel v{vessel}", { vessel: vessel + 1 })}</small><strong>{t(spec.title)}</strong></span>
    <span class="blurb">{t(spec.blurb)}</span>
  </div>
  <div class="fields">
    {#each spec.fields as f (f.name)}
      <label>
        {t(f.label)}
        {#if f.type === "species"}
          <select bind:value={values[f.name]}>
            <option value="">{t("choose…")}</option>
            {#each solids.length > 0 ? solids : shelf as s (s.key)}
              <option value={s.key}>{s.name}</option>
            {/each}
          </select>
        {:else}
          <span>
            <input
              type="number"
              bind:value={values[f.name]}
              min={f.min}
              max={f.max}
              step={f.step ?? 1}
            />
            {#if f.unit}{f.unit}{/if}
          </span>
        {/if}
      </label>
    {/each}
    <button class="run" disabled={busy || line === null} onclick={() => line && onrun(line)}>
      {busy ? t("running…") : t("go")}
    </button>
    <button class="close" onclick={onclose}>{t("put away")}</button>
  </div>
  {#if line}<code>{line}</code>{/if}
</section>

<style>
  .apparatus {
    position: relative;
    z-index: 7;
    margin: 0.55rem;
    padding: 0.75rem;
    border: 1px solid color-mix(in srgb, var(--instrument) 35%, var(--edge));
    border-radius: 16px;
    background: color-mix(in srgb, var(--surface) 92%, var(--instrument) 8%);
    box-shadow: 0 8px 24px var(--shadow);
  }
  .head {
    display: flex;
    gap: 0.6rem;
    align-items: baseline;
    margin-bottom: 0.35rem;
  }
  .head > span:nth-child(2) { display: flex; flex-direction: column; }
  .head small { color: var(--instrument); font-size: .56rem; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
  .live-mark { width: 10px; height: 10px; flex: none; border: 2px solid var(--surface); border-radius: 50%; background: var(--instrument); box-shadow: 0 0 0 2px var(--instrument); }
  .blurb {
    color: var(--dim);
    font-size: 0.78rem;
  }
  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: flex-end;
    font-size: 0.82rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    color: var(--dim);
  }
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
  .close {
    background: none;
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--dim);
    font: inherit;
    font-size: 0.82rem;
    padding: 0.3rem 0.7rem;
    cursor: pointer;
    min-height: 36px;
  }
  code {
    display: block;
    margin-top: 0.3rem;
    color: var(--dim);
    font-size: 0.72rem;
  }
</style>
