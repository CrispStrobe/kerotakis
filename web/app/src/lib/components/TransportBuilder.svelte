<script lang="ts">
  import { buildTransportLine } from "../titration";
  import type { SceneVessel } from "../host/EngineHost";

  let {
    vessels,
    busy,
    onrun,
    onclose,
  }: {
    vessels: SceneVessel[];
    busy: boolean;
    onrun: (line: string) => void;
    onclose: () => void;
  } = $props();

  let cells = $state<number[]>([]);
  let inlet = $state<number>(-1);
  let receiver = $state<number>(-1);
  let steps = $state(3);

  const line = $derived(
    inlet >= 0 && receiver >= 0
      ? buildTransportLine({ cells, inlet, receiver, steps })
      : null,
  );
  function toggleCell(id: number) {
    cells = cells.includes(id) ? cells.filter((c) => c !== id) : [...cells, id];
  }
</script>

<section class="train" aria-label="column train">
  <strong>column train</strong>
  <span class="hint">cells in flow order, then where solution enters and collects</span>
  <div class="roles">
    <fieldset>
      <legend>cells</legend>
      {#each vessels as v (v.id)}
        <label>
          <input type="checkbox" checked={cells.includes(v.id)} onchange={() => toggleCell(v.id)} />
          v{v.id + 1}
        </label>
      {/each}
    </fieldset>
    <label>
      inlet
      <select bind:value={inlet}>
        <option value={-1}>choose…</option>
        {#each vessels as v (v.id)}<option value={v.id}>v{v.id + 1}</option>{/each}
      </select>
    </label>
    <label>
      receiver
      <select bind:value={receiver}>
        <option value={-1}>choose…</option>
        {#each vessels as v (v.id)}<option value={v.id}>v{v.id + 1}</option>{/each}
      </select>
    </label>
    <label>
      steps
      <input type="number" min="1" max="50" bind:value={steps} />
    </label>
    <button class="run" disabled={busy || line === null} onclick={() => line && onrun(line)}>
      run the column
    </button>
    <button class="close" onclick={onclose}>put away</button>
  </div>
  {#if line}<code>{line}</code>{/if}
</section>

<style>
  .train {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--edge);
    background: var(--panel);
    font-size: 0.82rem;
  }
  .hint {
    color: var(--dim);
    margin-left: 0.6rem;
    font-size: 0.76rem;
  }
  .roles {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: flex-end;
    margin-top: 0.35rem;
  }
  fieldset {
    border: 1px solid var(--edge);
    border-radius: 6px;
    display: flex;
    gap: 0.6rem;
    padding: 0.2rem 0.6rem 0.4rem;
  }
  legend {
    color: var(--dim);
    font-size: 0.74rem;
    padding: 0 0.3rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    color: var(--dim);
  }
  fieldset label {
    flex-direction: row;
    align-items: center;
    gap: 0.25rem;
    color: var(--ink);
  }
  select,
  input[type="number"] {
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
  .run {
    background: var(--panel-raised);
    border: 1px solid var(--hot);
    border-radius: 6px;
    color: var(--ink);
    font: inherit;
    padding: 0.3rem 0.8rem;
    cursor: pointer;
    min-height: 36px;
  }
  .close {
    background: none;
    border: 1px solid var(--edge);
    border-radius: 6px;
    color: var(--dim);
    font: inherit;
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
