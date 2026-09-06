# Animation audit — what the engine computes vs what the bench draws

Status: 2026-09-06. Companion to `ROADMAP-GUI.md` (**GUI-099**), which owns
the task; this file is the evidence behind it.

The brief, from the German live deploy: *"we need way better and more complete
animations for what happens. they must render what actually goes on. rendering
must follow actual physical computed parameters where possible."*

So the question this audit asks of every event is not "does something move?"
but **"does the thing that moves change when the engine's number changes?"** A
visual driven by a constant is counted as *partial* even when it looks right,
because it is a picture of the verb rather than of the result. A visual that is
absent is *missing*. Only a visual whose size, count, colour, tempo or position
is a function of an engine-computed quantity is *done*.

Three sources feed the stage, and the audit distinguishes them:

- **Event** — a typed event from `kerotakis-core::ops::Event`, mapped by
  `web/app/src/lib/magnitudes.ts` (`effectFromEvent`) into a transient
  `Effect` with a magnitude, and drawn by `Vessel.svelte` / `BenchEffect.svelte`
  for a few seconds.
- **Scene** — a standing field of `SceneVessel` (`host/EngineHost.ts`), drawn
  for as long as it is true. Layers, foam, gel, curds, swelling and the solid
  stack all live here, which is why they survive a reload and an event-driven
  effect does not.
- **Neither** — the engine computes it, the wire carries it, and nothing on
  the bench is a function of it.

## Score

| | before GUI-099 | after PR 1 | after PR 2 | after PR 3 |
|---|---|---|---|---|
| done | 32 | 36 | 41 | 45 |
| partial | 18 | 17 | 12 | 11 |
| missing | 23 | 20 | 20 | 17 |
| **total rows** | **73** | **73** | **73** | **73** |

Per section, before → after PR 3: thermal and phase 3/2/3 → 7/0/1; gas and
headspace 2/4/5 → 5/1/5; solids 4/5/3 → 6/3/3; liquids and mixtures 10/3/4 →
11/3/3; reaction, combustion and light 5/4/8 → 8/3/6; bench-level 8/0/0
unchanged. Everything still *missing* is listed with its driving number, so
the next lane can pick it up without repeating this walk.

## Thermal and phase

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `temperature_changed` | `from`, `to` (K) | hotplate glow ellipse + heat waves, opacity from a `(K−310)/300` ramp; thermometer inset; °C badge | plus **incandescence above ~800 K, coloured by K** off the blackbody locus | partial | **done** (PR 1) |
| `energy_transferred` | `heating`, `requested_j`, `delivered_j`, `time_coupled` | heat/cool magnitude from `delivered_j`; apparatus readout says delivered vs requested | — | done | done |
| `heat_of_mixing` | `joules` (signed) | heat or cool, magnitude from \|J\| | — | done | done |
| `state_changed` | `species`, `from`/`to` phase, `at` (K), `shifted_by` | `to: solid` → frost; **everything else → a `phase-change` kind nothing rendered** | boil / melt / condense / freeze, each with the engine's transition temperature | missing | **done** (PR 1) |
| `boiling_point_routed` | `pressure_kpa`, `boiling` (K), `shifted_by`, `route`, `model` | nothing | the same rolling boil, held at the routed plateau (vacuum, pressure, salted solvent) | missing | **done** (PR 1) |
| `evaporated` | `moles` | three fixed steam columns, opacity from moles | plume count, reach and opacity from moles; rolling boil in the liquid | partial | **done** (PR 1) |
| `distilled` | `water`, `ethanol`, `at`, `ended`, `stages`, `energy_kj`, `azeotropic` | still rig, boiling range and column plate count drawn | — | done | done |
| `thermal_equilibrium` | the settled temperature | nothing (feed only) | — | missing | missing |

The boil is the headline fix. `steaming` was `temperature_k >= 368` — a
constant that is wrong under a partial vacuum, wrong in a pressurised vessel,
wrong for a salted solvent and wrong for every solvent that is not water. The
engine already computes the plateau it held the vessel at and reports it in
`state_changed.at`; the stage now gates on that number.

## Gas and headspace

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `gas_evolved` | `species`, `moles` | bubble curtain + vent wisps; count, radius and tempo on a log ramp (1 mmol → 0, 100 mmol → 1) | — | done | done |
| `gas_produced` | `species`, `moles`, `rate_moles_per_second` | same curtain from `moles`; **`rate` unused** | tempo from the rate, amount from the moles | partial | partial |
| `gas_contained` | `species`, `moles` | **nothing** — a sealed flask boiling was invisible | bubbles, and a headspace that fills | missing | **done** (PR 1 bubbles, PR 2 headspace) |
| `gas_absorbed` | `species`, `moles` | nothing | bubbles shrinking as the liquid takes the gas back | missing | missing |
| `headspace_partitioned` | `to_gas`, `moles`, `gas_fraction`, `partial_pressure_pa`, `henry_mol_per_l_atm` | nothing | headspace tint by `gas_fraction`; direction from `to_gas` | missing | missing |
| `headspace_equilibrated` | `pressure`, `total_moles` | nothing | the gauge and the piston agreeing | missing | missing |
| `vessel_sealed` | `headspace_volume` (L), `trapped_air` (mol) | a static lid rectangle | lid at the height the headspace volume implies | partial | **done** (PR 2) |
| `vessel_pressure_controlled` | `pressure` (Pa), `initial_volume` (L), `trapped_gas` (mol) | a piston drawn at a **fixed y** (`y=16`), whatever the pressure | piston height from `V = nRT/P` — squeeze the gas and watch it descend | partial | **done** (PR 2) |
| `vessel_swept` | `pressure` (Pa) | two static arrows | arrow tempo from the sweep pressure | partial | partial |
| `burst` | `at_pa`, `rating_pa` | star + shock ring, radius from `at_pa / rating_pa` | — | done | done |
| `bubble_ride` | `object_density`, `liquid_density`, `lift_gas_fraction` | nothing | the object rising once `lift_gas_fraction` of bubbles clings to it | missing | missing |

## Solids

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `precipitated` | `species`, `moles` (+ scene `solids[].volume_l`, `srgb`, `settled_fraction`) | 2–8 grey specks, radius on a linear moles ramp, engine colour ignored; the settled pile *is* scene-driven | particle count from moles, particle size from **molar volume**, colour from the species' `srgb`, pile from `volume_l` | partial | **done** (PR 2) |
| `dissolved` | `species`, `moles` | one circle, `r=4`, **magnitude hard-coded to 1** | particles shrinking in proportion to the moles that left the solid | partial | **done** (PR 2) |
| `supersaturated` | `dissolved`, `capacity` | nothing | how far past saturation, as haze or seed crystals | missing | missing |
| `plated` | `species`, `onto`, `moles` | a shimmer rectangle over the deposit; **moles unused** | plating thickness from moles | partial | partial |
| `adsorbed` | `held`, `loading_mg_per_g`, `still_dissolved` | nothing | the carbon darkening toward its isotherm ceiling | missing | missing |
| `consumed` | `species`, `moles`, `remaining` | nothing transient; the scene shrinks the solid | a visible ribbon eaten away | partial | partial |
| `corroded` | `species`, `corroding`, `why` | nothing | pitting/oxide on the metal object | missing | missing |
| `gravity_settled` | per-population `terminal_speed_m_s`, `distance_m`, `separated_fraction`, `particle_diameter_um` | grains falling; travel from `distance_m`, count and radius from the population | — | done | done |
| `centrifuged` | `rcf`, `rpm`, `imbalance_g`, populations | rotor blur on a log `rcf` ramp, pellet per population | — | done | done |
| `ground` | `surface_area_m2` | a magnitude on a log-area ramp, no vessel visual | finer powder in the vessel | partial | partial |
| `magnet_separated` | `attracted`, `remained` | bench transfer, magnitude from the attracted moles read off the scene | — | done | done |
| `filtered` | `from`, `to` | bench transfer + residue on the paper, magnitude from retained moles | — | done | done |

## Liquids, layers, mixtures

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `layers_formed` / `material_layers_formed` | `upper`, `lower` (+ scene `layers[].volume_l`, `srgb`) | the stack, each band's height its share of the volume | — | done (scene) | done |
| `emulsion_changed` | `dispersed_volume_l`, `dispersed_fraction`, `half_life_seconds` | **nothing, anywhere** — `SceneVessel.emulsion` was read by no component | dispersed droplets at `dispersed_fraction`, clearing over `half_life_seconds` | missing | **done** (PR 3) |
| `curdling_changed` | `to_formed_fraction`, `separation_progress`, `curd_solids_mass_g` | curd ellipses, count from `separation_progress`, colour from the scene | — | done | done |
| `foam_changed` | `trapped_gas_liters`, `volume_liters`, `height_cm`, `overflow_liters`, `half_life_seconds` | foam band from the scene volume, overflow drawn; **`half_life_seconds` unused** | the head collapsing on its own half-life | partial | partial |
| `gel_formed` | `from`/`to_gelled_fraction`, `polymer_grams`, `crosslinker_moles` | gel body from the scene's `gelled_fraction`; **the transition is not animated** | the sol→gel step itself | partial | partial |
| `thickened` | `strength`, `solid_mass_fraction`, `tip_speed_m_s`, `sheared_hard` | nothing | shear-thickening resisting the stirrer | missing | missing |
| `polymer_swelled` | `swelling_ratio_g_per_g`, `capacity_g_per_g` | snow height from the ratio against capacity | — | done | done |
| `surface_spread` | `to_cleared_fraction` | particles fleeing the surfactant | — | done | done |
| `surface_colour_spread` | `to_spread_fraction`, `spot_count` | colour spots spreading | — | done | done |
| `partitioned` | `fraction_lower` | nothing in the vessel | the solute's split across the two layers | missing | missing |
| `osmosis_changed` | `water_moles`, `mass_change_g` | nothing | the egg/potato swelling or shrinking | missing | missing |
| `diluted` | `volume`, `moles` | swirl, magnitude from the added volume | — | done | done |
| `mixed` | `fraction_a`, `fraction_b`, `temperature_a`/`_b`/`_into` | swirl from the summed fractions; **the three temperatures unused** | thermal mixing visible as the streams meet | partial | partial |
| `transferred` | `fraction` | pour stream; angle and particle mass from the accepted fraction | — | done | done |
| `drained` | `solvent`, `moles` | pour with the engine's lower/upper layer colours | — | done | done |
| `stirred` | `rpm`, `tip_speed_m_s`, `resuspended_fraction`, `rate_coupled` | vortex, stirrer tempo and resuspended grains, all from the tip speed | — | done | done |
| `titrated` | `curve`, `steps`, `total_volume`, `final_ph`, `concentration` | burette playback paced by the steps + the live curve chart; indicator colour arrives through the scene liquid `srgb` | — | done | done |

## Reaction, combustion, light

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `ignited` | `flame` (colour word), `energy_j` | flame scale from `energy_j` (100 J → 50 kJ), colour from `flame`, WebGPU flame where enabled | — plus the driving number readable from the DOM | done | done (`data-flame-energy-j`, PR 2) |
| `flame_test` | `species`, `colour` | burner rig, flame colour from the event; a restrained fixed size, because the event carries no energy | — | done | done |
| `did_not_ignite` / `flame_starved` | `fuel`, `burned`, `oxygen_fraction` | nothing | a flame that catches and gutters out at `oxygen_fraction` | missing | missing |
| `below_autoignition` | the gap to the autoignition temperature | nothing | — | missing | missing |
| `reacted` | `moles`, `seconds`, `catalyst`, `activation_energy` | nothing in the vessel | reaction extent over the elapsed bench seconds | missing | missing |
| `reaction_heat_released` | `energy_j` | nothing | the exotherm, as a temperature the thermometer then reads | missing | missing |
| `fermented` | `sucrose_moles`, `ethanol_moles`, `carbon_dioxide_moles`, `active_yeast_grams`, `seconds` | **nothing** | slow bubbling paced over `seconds`, sized by the CO₂ moles | missing | **done** (PR 3) |
| `enzyme_hydrolysed` | `converted_fraction`, `seconds` | a caption percentage | the substrate visibly clearing | partial | partial |
| `electrolysed` | `amps`, `seconds`, `coulombs`, `electrons`, `moles`, `grams`, `per_ion` | bubbles at two electrodes, count from moles, duration from seconds; **both electrodes got the same count off one product's moles** | both electrodes sized by the **charge** they shared — the honest driver until the counter-electrode's half-reaction is on the wire | partial | **done** (PR 3) |
| `cell_voltage` | `volts` | connection arc, magnitude from \|V\| | — | done | done |
| `decayed` | `parent`, `daughter`, `mode`, `moles`, `half_life_s` | the Geiger comes from a `measured` reading, not from `decayed` | decay drawn from the event, not only when an instrument is held | partial | partial |
| `nuclide_spiked` | `activity_bq` | nothing | initial activity | missing | missing |
| `irradiated` | `wavelength_nm`, `irradiance_w_m2` | lamp, magnitude from the irradiance | — | done | done |
| `uv_attenuated` | `wavelength_nm`, `band`, `transmitted_fraction`, `mechanism` | **nothing** | the beam dimming to `transmitted_fraction` through the sunscreen | missing | **done** (PR 3) |
| `chemiluminescence_observed` | `relative_intensity`, `half_life_s` | the scene's glow, strength from the intensity | — | done | done |
| `hydrated` / `dehydrated` | `formula_units`, `water`, `at` (K) | nothing transient; the colour change arrives through the scene | the water leaving as steam at `at` | partial | partial |
| `neutralised` | `moles` of acidity cancelled | nothing | — | missing | missing |

## Bench-level

| Event | What the engine computes | What the stage showed | Before | After |
|---|---|---|---|---|
| `spill_created` | `fraction`, `destination.surface`, `destination.zone` | bench pool, size from the fraction, on the named surface | done | done |
| `container_broken` | `impulse_ns`, `destination` | shards + pool, spread from the impulse | done | done |
| `gas_tested` | `test`, `positive`, `notes` | the four rigs, each with its result | done | done |
| `smelled` | per-species notes | waft rig | done | done |
| `observed` | `liquid` rgb, `cloudiness`, `deposit`, `bubbling` | inspection lens | done | done |
| `chromatographed` | per-peak `retention_time_s`, `width_s`, `relative_area`, `partition_k` | column with bands at the computed retention times | done | done |
| `measured` | `value`, `unit`, per instrument | the instrument insets | done | done |
| `transported` | destination | bench move | done | done |

Events with no vessel visual **by design** — `added`, `material_added`,
`vessel_created`, `vessel_removed`, `vessel_opened`, `hazard_warning`,
`safety_veto`, `solver_failed`, `not_yet_modelled`, `inert`,
`inert_in_solvent`, `dissolved_in_solvent`, `solution`, `reaction`,
`reaction_occurred`, `shelf_stocked`, `stock_exhausted`, `spill_recovered`,
`spill_hazard`, `collision_withstood`, `sealed_cell`, `no_cell`,
`transition_point`, `particles_counted`, `org_reacted` — are not counted
above. They change the feed, the shelf, the safety board or a panel, not the
picture of a vessel.

## Delivered

- **PR 1 (ANIM-1)** — the boil held at `state_changed.at` /
  `boiling_point_routed.boiling` instead of 368 K, with the rolling boil and
  the steam plume both sized by the moles of vapour the step made and matched
  by species so a fizz is not mistaken for steam; incandescence above ~800 K
  coloured off the blackbody locus by `temperature_k` alone; condensation
  beading under the room's Magnus dew point, with frost still owning the water
  below freezing; `gas_contained`, `boiling_point_routed` and the non-solid
  half of `state_changed` mapped at last.
- **PR 2 (ANIM-2)** — precipitate grain count from moles and grain size from
  the registry's molar volume (`SceneSolid.volume_l ÷ moles`), in the species'
  own `srgb`; dissolving grains shrinking rather than puffing outward, and
  there being more than one of them; the piston's height from the volume the
  trapped gas occupies at the held pressure (`V = nRT/P` while the event is
  live, Boyle off the scene's `pressure_pa` afterwards), so squeezing a gas
  finally moves something; a headspace band whose density follows the pressure
  over atmospheric; the flame's `energy_j` readable from the DOM.

- **PR 3 (ANIM-3)** — the three events that changed a vessel and drew nothing
  at all. An emulsion's droplet count from `dispersed_fraction`, droplet size
  from `dispersed_volume_l` split between them, and their drift back together
  on the engine's coalescence half-life. Fermentation bubbling at the rate the
  engine computed — `carbon_dioxide_moles ÷ seconds`, turned into a tempo by
  the ~41 µmol in one visible millilitre bubble — so an overnight brew ticks
  every few seconds and a lively dough every one. A UV beam whose exit band's
  opacity *is* `transmitted_fraction`. And electrolysis bubbling at both
  electrodes on the **charge** they shared rather than on one product's moles.

Every one of those carries a `data-*` attribute naming the number that drives
it, so the browser UX gate and any later test can assert on the *quantity*
rather than on the presence of a shape.

## What the engine should add

None of these are blocking; each removes a client-side fallback that is
honest but weaker than the number the engine already has.

1. **`SceneVessel.boiling_point_k`** (and, ideally, `melting_point_k`). The
   plateau is computed every time a vessel boils and reported in
   `state_changed.at`, but scene v1 carries no standing value, so between
   events the stage must fall back to pure water at one atmosphere
   (`NORMAL_BOILING_K`). A vessel that is *sitting* at 350 K under a partial
   vacuum is boiling and the bench cannot know it.
2. **`SceneVessel.headspace_volume_l`** and **`headspace_moles`**. PR 2
   reconstructs the piston height from `V = nRT/P` using
   `vessel_pressure_controlled.trapped_gas` plus the scene's `pressure_pa`
   and `temperature_k`. That is correct for an ideal gas and correct at the
   moment of the event, but it is client-side physics and it goes stale as
   soon as anything else changes the headspace. `vessel_sealed` already
   carries `headspace_volume`; the scene should carry it too.
3. **Molar volume on `precipitated` / `dissolved`**, or simply
   `volume_l` alongside `moles`. PR 2 reads it off `SceneVessel.solids[]`
   after the fact, which works only while the solid is still in the vessel —
   a species that precipitates and immediately redissolves has no scene row
   to read.
4. **Per-electrode product moles on `electrolysed`.** The event carries
   `per_ion` and one product; drawing 2:1 hydrogen and oxygen correctly needs
   both half-reactions, or the moles evolved at each electrode. PR 3 sizes
   both electrodes by the **charge** instead, which is shared by definition
   and therefore never a guess — but it also cannot show that hydrogen comes
   off twice as fast as oxygen, which is the observation the experiment is
   for.
5. **`gas_produced.rate_moles_per_second` is already there** — nothing to
   add; the client simply has not used it yet. Same for
   `foam_changed.half_life_seconds` and `emulsion_changed.half_life_seconds`,
   which would let a head or an emulsion decay on the bench without a further
   event.
6. **A `sublimated` distinction.** Dry-ice fog is a `state_changed` from
   `solid` to `gas`, which is indistinguishable from a boil at the wire
   level. Either the event or the species phase diagram has to say which,
   or the fog has to be inferred from the species — the client should not
   guess.
7. **`corroded` carries no extent.** `corroding: bool` and a `why` string
   cannot size a visual; moles of metal lost, or a corroded fraction, would.
