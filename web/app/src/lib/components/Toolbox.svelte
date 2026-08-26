<script lang="ts">
  /**
   * The toolbox drawer (GUI-027): CAP-5's named relations as a calculator
   * whose every answer carries its provenance and explains itself at the
   * bench's current register — nernst, arrhenius, eyring, henderson-
   * hasselbalch, ionic strength, debye–hückel, van 't hoff. The engine
   * evaluates; this panel only builds `k=v` arguments and shows the
   * RelationResult verbatim.
   */
  import type { Session } from "../session.svelte";
  import { buildArgs, parseArgSpec, type RelationField } from "../relationArgs";
  import { t } from "../i18n.svelte";

  let { session, onclose }: { session: Session; onclose: () => void } = $props();

  type Relation = { name: string; equation: string; args: string };
  type CalcResult =
    | { ok: true; value: number; unit: string; provenance: string; lv1: string; lv2: string; lv3: string }
    | { ok: false; error: string };

  let relations = $state<Relation[]>([]);
  let picked = $state<Relation | null>(null);
  let fields = $state<RelationField[]>([]);
  let freeform = $state(false);
  let values = $state<Record<string, string>>({});
  let freeText = $state("");
  let result = $state<CalcResult | null>(null);
  let computing = $state(false);

  $effect(() => {
    void session.relations().then((list) => {
      relations = list;
      if (!picked && list.length > 0) pick(list[0]!);
    });
  });

  function pick(r: Relation) {
    picked = r;
    const spec = parseArgSpec(r.args);
    fields = spec.fields;
    freeform = spec.freeform;
    values = {};
    freeText = "";
    result = null;
  }

  const ready = $derived(
    freeform ? freeText.trim().length > 0 : buildArgs(fields, values) !== null,
  );

  async function compute() {
    if (!picked) return;
    const args = freeform
      ? freeText.trim().split(/\s+/)
      : (buildArgs(fields, values) ?? []);
    computing = true;
    try {
      result = (await session.calc(picked.name, args)) as CalcResult;
    } finally {
      computing = false;
    }
  }

  /** The explanation at the bench's own register — the dial reaches here too. */
  const prose = $derived(
    result?.ok ? result[session.register as "lv1" | "lv2" | "lv3"] : null,
  );

  function formatValue(v: number): string {
    if (v === 0) return "0";
    const magnitude = Math.abs(v);
    if (magnitude >= 1e5 || magnitude < 1e-3) return v.toExponential(4);
    return v.toPrecision(5);
  }
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <section
    class="toolbox"
    role="dialog"
    aria-modal="true"
    aria-label={t("relation calculator")}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{t("Toolbox")}</h2>
      <p class="sub">{t("named relations, computed by the engine — with sources")}</p>
      <button class="close" onclick={onclose} aria-label={t("close the toolbox")}>×</button>
    </header>
    <div class="body">
      <nav aria-label={t("relations")}>
        {#each relations as r (r.name)}
          <button class:on={picked?.name === r.name} onclick={() => pick(r)}>
            <span class="rname">{r.name}</span>
            <span class="req">{r.equation}</span>
          </button>
        {/each}
        {#if relations.length === 0}
          <p class="empty">{t("the engine has not answered with its relations yet")}</p>
        {/if}
      </nav>
      {#if picked}
        <form
          onsubmit={(e) => {
            e.preventDefault();
            void compute();
          }}
        >
          <p class="equation">{picked.equation}</p>
          {#if freeform}
            <label>
              <span>{t("arguments")} <small>{picked.args}</small></span>
              <input
                bind:value={freeText}
                placeholder={picked.args}
                autocomplete="off"
                spellcheck="false"
              />
            </label>
          {:else}
            {#each fields as f (f.name)}
              <label>
                <span>
                  {f.name}
                  <small>{t(f.hint)}{f.optional ? ` · ${t("optional")}` : ""}</small>
                </span>
                <input
                  bind:value={values[f.name]}
                  inputmode="decimal"
                  autocomplete="off"
                  spellcheck="false"
                />
              </label>
            {/each}
          {/if}
          <button class="go" type="submit" disabled={!ready || computing}>
            {computing ? t("computing…") : t("compute")}
          </button>
          {#if result}
            {#if result.ok}
              <output>
                <strong class="value">{formatValue(result.value)}</strong>
                <span class="unit">{result.unit}</span>
                {#if prose}<p class="prose">{prose}</p>{/if}
                <p class="provenance">{result.provenance}</p>
              </output>
            {:else}
              <p class="refusal" role="alert">{result.error}</p>
            {/if}
          {/if}
        </form>
      {/if}
    </div>
  </section>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 45%);
    display: grid;
    place-items: center;
    z-index: 10;
  }
  .toolbox {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 10px;
    width: min(46rem, calc(100vw - 2rem));
    max-height: calc(100vh - 3rem);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 0.7rem;
    padding: 0.9rem 1.1rem 0.6rem;
    border-bottom: 1px solid var(--edge);
  }
  h2 {
    margin: 0;
    font-size: 1rem;
  }
  .sub {
    margin: 0;
    color: var(--dim);
    font-size: 0.78rem;
  }
  .close {
    margin-left: auto;
    background: none;
    border: 0;
    color: var(--dim);
    font-size: 1.3rem;
    cursor: pointer;
    line-height: 1;
    min-width: 36px;
    min-height: 36px;
  }
  .body {
    display: flex;
    min-height: 0;
    overflow: hidden;
  }
  nav {
    width: 15rem;
    border-right: 1px solid var(--edge);
    overflow-y: auto;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  nav button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    background: none;
    border: 1px solid transparent;
    border-radius: 7px;
    color: var(--ink);
    font: inherit;
    text-align: left;
    padding: 0.45rem 0.6rem;
    cursor: pointer;
  }
  nav button:hover {
    background: var(--panel-raised);
  }
  nav button.on {
    border-color: var(--hot);
    background: var(--panel-raised);
  }
  .rname {
    font-size: 0.85rem;
    font-weight: 600;
  }
  .req {
    font-size: 0.68rem;
    color: var(--dim);
  }
  .empty {
    color: var(--dim);
    font-size: 0.8rem;
    padding: 0.5rem;
  }
  form {
    flex: 1;
    padding: 0.9rem 1.1rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .equation {
    margin: 0;
    font-size: 0.95rem;
    color: var(--cool);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.8rem;
  }
  label small {
    color: var(--dim);
    margin-left: 0.35rem;
  }
  input {
    background: var(--panel-raised);
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    padding: 0.35rem 0.55rem;
    min-height: 36px;
  }
  input:focus {
    outline: 2px solid var(--hot);
    outline-offset: -1px;
  }
  .go {
    align-self: flex-start;
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    font-size: 0.85rem;
    padding: 0.35rem 0.9rem;
    cursor: pointer;
    min-height: 36px;
  }
  .go:disabled {
    opacity: 0.45;
    border-color: var(--edge);
    cursor: default;
  }
  output {
    display: block;
    border: 1px solid var(--edge);
    border-radius: 8px;
    padding: 0.7rem 0.9rem;
    background: var(--panel-raised);
  }
  .value {
    font-size: 1.45rem;
  }
  .unit {
    color: var(--dim);
    margin-left: 0.35rem;
  }
  .prose {
    margin: 0.45rem 0 0;
    font-size: 0.85rem;
  }
  .provenance {
    margin: 0.45rem 0 0;
    font-size: 0.72rem;
    color: var(--dim);
  }
  .refusal {
    margin: 0;
    color: var(--warn);
    font-size: 0.85rem;
  }
  @media (max-width: 700px) {
    .body {
      flex-direction: column;
      overflow-y: auto;
    }
    nav {
      width: auto;
      border-right: 0;
      border-bottom: 1px solid var(--edge);
      flex-direction: row;
      flex-wrap: wrap;
    }
    .req {
      display: none;
    }
  }
</style>
