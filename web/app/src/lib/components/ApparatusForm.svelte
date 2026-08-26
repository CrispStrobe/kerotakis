<script lang="ts">
  import type { ApparatusSpec } from "../apparatus";
  import type { ShelfItem } from "../session.svelte";
  import { t } from "../i18n.svelte";

  let {
    spec,
    vessel,
    shelf,
    busy,
    onrun,
    onclose,
  }: {
    spec: ApparatusSpec;
    vessel: number;
    shelf: ShelfItem[];
    busy: boolean;
    onrun: (line: string) => void;
    onclose: () => void;
  } = $props();

  let values = $state<Record<string, number | string>>(
    Object.fromEntries(spec.fields.map((f) => [f.name, f.default])),
  );
  const solids = $derived(shelf.filter((s) => s.phase.toLowerCase().includes("solid")));
  const line = $derived(spec.build(vessel, values));
</script>

<section class="apparatus" aria-label={t("{apparatus} over v{vessel}", { apparatus: t(spec.title), vessel: vessel + 1 })}>
  <div class="head">
    <strong>{t(spec.title)}</strong>
    <span class="blurb">{t(spec.blurb)} · v{vessel + 1}</span>
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
      {t("go")}
    </button>
    <button class="close" onclick={onclose}>{t("put away")}</button>
  </div>
  {#if line}<code>{line}</code>{/if}
</section>

<style>
  .apparatus {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
  }
  .head {
    display: flex;
    gap: 0.6rem;
    align-items: baseline;
    margin-bottom: 0.35rem;
  }
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
