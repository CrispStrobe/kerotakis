<script lang="ts">
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
  const INSTRUMENTS: { token: string; label: string }[] = [
    { token: "thermometer", label: "thermometer" },
    { token: "ph", label: "pH meter" },
    { token: "balance", label: "balance" },
    { token: "volume", label: "volume" },
    { token: "conductivity", label: "conductivity" },
    { token: "pressure", label: "pressure gauge" },
    { token: "calorimeter", label: "calorimeter" },
    { token: "uvvis", label: "UV-Vis" },
    { token: "eyes", label: "look closely" },
    { token: "chromatograph", label: "chromatograph" },
  ];
</script>

<div class="tray" role="group" aria-label={`instruments for ${v}`}>
  {#each INSTRUMENTS as inst (inst.token)}
    <button
      disabled={busy}
      onclick={() =>
        onmeasure(
          inst.token === "chromatograph" ? `chromatograph ${v}` : `measure ${v} ${inst.token}`,
        )}
    >
      {inst.label}
    </button>
  {/each}
</div>

<style>
  .tray {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    padding: 0.5rem 1rem;
    border-top: 1px solid var(--edge);
  }
  button {
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 999px;
    color: var(--dim);
    font: inherit;
    font-size: 0.74rem;
    padding: 0.25rem 0.6rem;
    cursor: pointer;
    min-height: 34px;
  }
  button:hover {
    color: var(--ink);
    border-color: var(--cool);
  }
</style>
