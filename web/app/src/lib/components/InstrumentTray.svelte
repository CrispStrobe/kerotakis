<script lang="ts">
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
  const GROUPS = [
    {
      label: "essential readings",
      instruments: [
        { token: "eyes", label: "look closely", icon: "◉", description: "visual appearance and visible change" },
        { token: "smell", label: "safe waft", icon: "≋", description: "waft headspace vapour — never smell directly" },
        { token: "thermometer", label: "thermometer", icon: "°", description: "sample temperature" },
        { token: "ph", label: "pH meter", icon: "pH", description: "acidity or alkalinity of an aqueous sample" },
        { token: "balance", label: "balance", icon: "⚖", description: "total material mass" },
      ],
    },
    {
      label: "physical properties",
      instruments: [
        { token: "volume", label: "gas volume meter", icon: "▱", description: "sealed headspace volume" },
        { token: "conductivity", label: "conductivity meter", icon: "ϟ", description: "modeled conductivity from ionic strength" },
        { token: "pressure", label: "pressure gauge", icon: "◔", description: "headspace pressure" },
        { token: "calorimeter", label: "calorimeter", icon: "Δ", description: "enthalpy relative to 25 °C" },
      ],
    },
    {
      label: "analytical instruments",
      instruments: [
        { token: "uvvis", label: "UV-Vis", icon: "λ", description: "absorbance spectrum" },
        { token: "chromatograph", label: "chromatograph", icon: "⋮", description: "separate and compare components" },
        { token: "geiger", label: "Geiger counter", icon: "◌", description: "radioactive decay activity" },
      ],
    },
  ];
</script>

<div class="tray" role="group" aria-label={t("instruments for {vessel}", { vessel: v })}>
  {#each GROUPS as group (group.label)}
    <section aria-labelledby={`instrument-group-${group.label.replaceAll(" ", "-")}`}>
      <h4 id={`instrument-group-${group.label.replaceAll(" ", "-")}`}>{t(group.label)}</h4>
      <div class="instrument-grid">
        {#each group.instruments as inst (inst.token)}
          <button
            disabled={busy}
            aria-label={t("Measure {quantity} in {vessel}", { quantity: t(inst.description), vessel: v })}
            onclick={() =>
              onmeasure(
                inst.token === "chromatograph"
                  ? `chromatograph ${v}`
                  : inst.token === "smell"
                    ? `smell ${v}`
                    : `measure ${v} ${inst.token}`,
              )}
          >
            <span class="instrument-icon" aria-hidden="true">{inst.icon}</span>
            <span class="instrument-copy">
              <strong>{t(inst.label)}</strong>
              <small>{t(inst.description)}</small>
            </span>
            <span class="run" aria-hidden="true">→</span>
          </button>
        {/each}
      </div>
    </section>
  {/each}
</div>

<style>
  .tray { display: grid; gap: .7rem; padding: .65rem .8rem .85rem; border-top: 1px solid var(--edge); }
  section { display: grid; gap: .3rem; }
  h4 { margin: 0; color: var(--dim); font-size: .55rem; font-weight: 850; letter-spacing: .09em; text-transform: uppercase; }
  .instrument-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .35rem; }
  button { min-width: 0; min-height: 54px; display: grid; grid-template-columns: 34px minmax(0, 1fr) auto; align-items: center; gap: .45rem; padding: .42rem .5rem; border: 1px solid var(--edge); border-radius: 12px; color: var(--ink); background: var(--surface-raised); font: inherit; text-align: left; cursor: pointer; transition: transform 140ms ease, border-color 140ms ease, box-shadow 140ms ease; }
  button:hover:not(:disabled), button:focus-visible { transform: translateY(-1px); border-color: var(--instrument); box-shadow: 0 5px 13px var(--shadow); }
  button:disabled { opacity: .42; cursor: default; }
  .instrument-icon { width: 34px; height: 34px; display: grid; place-items: center; border-radius: 10px; color: white; background: linear-gradient(145deg, var(--instrument), color-mix(in srgb, var(--instrument) 55%, var(--primary))); font-size: .88rem; font-weight: 850; }
  .instrument-copy { min-width: 0; display: grid; gap: .08rem; }
  .instrument-copy strong { overflow: hidden; font-size: .68rem; text-overflow: ellipsis; white-space: nowrap; }
  .instrument-copy small { display: -webkit-box; overflow: hidden; color: var(--dim); font-size: .56rem; line-height: 1.25; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
  .run { color: var(--primary); font-weight: 900; }
  @media (max-width: 370px) { .instrument-grid { grid-template-columns: 1fr; } }
</style>
