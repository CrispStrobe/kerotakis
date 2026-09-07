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

| | before GUI-099 | after PR 1 | after PR 2 | after PR 3 | after PR 4 | after corrosion extent | after computed gas/foam motion | after ANIM-5 | after ANIM-6 | after ANIM-7 | after ANIM-8 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| done | 32 | 36 | 41 | 45 | 45 | 46 | 48 | 52 | 58 | 64 | 70 |
| partial | 18 | 17 | 12 | 11 | 11 | 11 | 9 | 9 | 9 | 9 | 3 |
| missing | 23 | 20 | 20 | 17 | 17 | 16 | 16 | 12 | 6 | 0 | 0 |
| **total rows** | **73** | **73** | **73** | **73** | **73** | **73** | **73** | **73** | **73** | **73** | **73** |

**Nothing is *missing* any more, and only three rows are still *partial*.**
Every other row the audit walked has a visual that is a function of an
engine-computed quantity. The three that remain are `gel_formed` (the
standing gel body is scene-driven; the sol→gel **transition** is not
animated), `mixed` (the swirl follows the summed pour fractions; the three
temperatures are unused) and `hydrated`/`dehydrated` (the colour change
arrives through the scene; the water leaving at `at` is not drawn).

**PR 4 deliberately moves no row, and that is the point.** It is the
engine-side lane: the list under *What the engine should add* below, put on
the wire. Every row it touches was already *done* — against a client-side
reconstruction that this audit's own standard could not distinguish from the
engine's number, because both moved when the engine moved. What separates
them is where they are wrong: a reconstruction is right at the instant of
the event and drifts afterwards, and a constant that only shows between
events is invisible to a walk that looks at events. So a score that counts
rows is the wrong instrument here, and inflating it would be the *kind* of
claim this file exists to refuse. What PR 4 changes is listed in
**Delivered** below, and it is stated as a change of source rather than as a
change of score.

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
| `state_changed` | `species`, `from`/`to` phase, `at` (K), `shifted_by`, `kind`, `moles` | `to: solid` → frost; **everything else → a `phase-change` kind nothing rendered** | boil / melt / condense / freeze / **sublimate** / deposit, each with the engine's transition temperature and the amount that moved | missing | **done** (PR 1, PR 4) |
| `boiling_point_routed` | `pressure_kpa`, `boiling` (K), `shifted_by`, `route`, `model` | nothing | the same rolling boil, held at the routed plateau (vacuum, pressure, salted solvent) | missing | **done** (PR 1) |
| `evaporated` | `moles` | three fixed steam columns, opacity from moles | plume count, reach and opacity from moles; rolling boil in the liquid | partial | **done** (PR 1) |
| `distilled` | `water`, `ethanol`, `at`, `ended`, `stages`, `energy_kj`, `azeotropic` | still rig, boiling range and column plate count drawn | — | done | done |
| `thermal_equilibrium` | the settled temperature, `reaction_energy_j`, `holds_nothing` | nothing (feed only) | the settled temperature, and whose it is when the burn left the vessel empty | missing | **done** (ANIM-7) |

The boil is the headline fix. `steaming` was `temperature_k >= 368` — a
constant that is wrong under a partial vacuum, wrong in a pressurised vessel,
wrong for a salted solvent and wrong for every solvent that is not water. The
engine already computes the plateau it held the vessel at and reports it in
`state_changed.at`; the stage now gates on that number.

PR 4 finished it. A transition event only exists at the moment of transition,
so *between* events the stage still fell back to pure water at one atmosphere
and a flask **sitting** at 350 K under a partial vacuum drew as still water.
`SceneVessel.boiling_point_k` is now a standing value from the same call
`StateEquilibrator` makes, and `melting_point_k` beside it retired the frost
gate's `< 272 K` — a constant that made brine frost early and every
non-aqueous liquid frost at water's threshold.

## Gas and headspace

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `gas_evolved` | `species`, `moles` | bubble curtain + vent wisps; count, radius and tempo on a log ramp (1 mmol → 0, 100 mmol → 1) | — | done | done |
| `gas_produced` | `species`, `moles`, `rate_moles_per_second` | bubble count/size from `moles`; cadence from the production rate | — | partial | **done** (computed-motion tranche) |
| `gas_contained` | `species`, `moles` | **nothing** — a sealed flask boiling was invisible | bubbles, and a headspace that fills | missing | **done** (PR 1 bubbles, PR 2 headspace) |
| `gas_absorbed` | `species`, `moles` | nothing | bubbles shrinking as the liquid takes the gas back | missing | **done** (ANIM-5) |
| `headspace_partitioned` | `to_gas`, `moles`, `gas_fraction`, `partial_pressure_pa`, `henry_mol_per_l_atm` | nothing | headspace tint by `gas_fraction`; direction from `to_gas` | missing | **done** (ANIM-5) |
| `headspace_equilibrated` | `pressure`, `total_moles` | nothing | the gauge and the piston agreeing | missing | **done** (ANIM-5) |
| `vessel_sealed` | `headspace_volume` (L), `trapped_air` (mol) — plus scene `headspace_volume_l`, `headspace_moles` | a static lid rectangle | lid at the height the headspace volume implies | partial | **done** (PR 2 event, PR 4 standing) |
| `vessel_pressure_controlled` | `pressure` (Pa), `initial_volume` (L), `trapped_gas` (mol) — plus the same scene pair | a piston drawn at a **fixed y** (`y=16`), whatever the pressure | piston height from the engine's own headspace volume, not a client-side `V = nRT/P` that goes stale | partial | **done** (PR 2 event, PR 4 standing) |
| `vessel_swept` | `pressure` (Pa) | two static arrows | arrow tempo from the sweep pressure | partial | **done** (ANIM-8) |
| `burst` | `at_pa`, `rating_pa` | star + shock ring, radius from `at_pa / rating_pa` | — | done | done |
| `bubble_ride` | `object_density`, `liquid_density`, `lift_gas_fraction` | nothing | the object rising once `lift_gas_fraction` of bubbles clings to it | missing | **done** (ANIM-6) |

## Solids

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `precipitated` | `species`, `moles` (+ scene `solids[].volume_l`, `srgb`, `settled_fraction`; + shelf `molar_volume_l_per_mol`) | 2–8 grey specks, radius on a linear moles ramp, engine colour ignored; the settled pile *is* scene-driven | particle count from moles, particle size from **molar volume**, colour from the species' `srgb`, pile from `volume_l` | partial | **done** (PR 2, PR 4) |
| `dissolved` | `species`, `moles` (+ the same shelf molar volume) | one circle, `r=4`, **magnitude hard-coded to 1** | particles shrinking in proportion to the moles that left the solid | partial | **done** (PR 2, PR 4) |
| `supersaturated` | `dissolved`, `capacity` | nothing | how far past saturation, as haze or seed crystals | missing | **done** (ANIM-5) |
| `plated` | `species`, `onto`, `moles` | a shimmer rectangle over the deposit; **magnitude a hard-coded `1`** | plating thickness from moles | partial | **done** (ANIM-8) |
| `adsorbed` | `held`, `loading_mg_per_g`, `still_dissolved` | nothing | the carbon darkening by the share that left the water, with the remainder beside it | missing | **done** (ANIM-6) |
| `consumed` | `species`, `moles`, `remaining` | nothing transient; the scene shrinks the solid | a visible ribbon eaten away, at the remainder where the engine states one | partial | **done** (ANIM-8) |
| `corroded` | `species`, `corroding`, `why`, **`corroded_moles`, `corroded_fraction`** (PR 4), plus standing scene corrosion extent | schematic oxide relation marker and “metal in oxide” readout, strength from the core fraction | plus a rust bloom on the metal itself, spot count and strength from `corroded_fraction` | missing | **done** (scene extent; event extent drawn in ANIM-5) |
| `gravity_settled` | per-population `terminal_speed_m_s`, `distance_m`, `separated_fraction`, `particle_diameter_um` | grains falling; travel from `distance_m`, count and radius from the population | — | done | done |
| `centrifuged` | `rcf`, `rpm`, `imbalance_g`, populations | rotor blur on a log `rcf` ramp, pellet per population | — | done | done |
| `ground` | `diameter_um`, `solid_moles`, `surface_area_m2`, `rate_coupled` | a magnitude on a log-area ramp, no vessel visual | finer powder in the vessel, grain radius from `diameter_um` | partial | **done** (ANIM-8) |
| `magnet_separated` | `attracted`, `remained` | bench transfer, magnitude from the attracted moles read off the scene | — | done | done |
| `filtered` | `from`, `to` | bench transfer + residue on the paper, magnitude from retained moles | — | done | done |

## Liquids, layers, mixtures

| Event | What the engine computes | What the stage showed | What it should show | Before | After |
|---|---|---|---|---|---|
| `layers_formed` / `material_layers_formed` | `upper`, `lower` (+ scene `layers[].volume_l`, `srgb`) | the stack, each band's height its share of the volume | — | done (scene) | done |
| `emulsion_changed` | `dispersed_volume_l`, `dispersed_fraction`, `half_life_seconds` | **nothing, anywhere** — `SceneVessel.emulsion` was read by no component | dispersed droplets at `dispersed_fraction`, clearing over `half_life_seconds` | missing | **done** (PR 3) |
| `curdling_changed` | `to_formed_fraction`, `separation_progress`, `curd_solids_mass_g` | curd ellipses, count from `separation_progress`, colour from the scene | — | done | done |
| `foam_changed` | `trapped_gas_liters`, `volume_liters`, `height_cm`, `overflow_liters`, `half_life_seconds` | foam band from scene volume, overflow drawn, head reaches half height on the computed half-life | — | partial | **done** (computed-motion tranche) |
| `gel_formed` | `from`/`to_gelled_fraction`, `polymer_grams`, `crosslinker_moles` | gel body from the scene's `gelled_fraction`; **the transition is not animated** | the sol→gel step itself | partial | partial |
| `thickened` | `strength`, `solid_mass_fraction`, `tip_speed_m_s`, `sheared_hard` | nothing | shear-thickening resisting the stirrer | missing | **done** (ANIM-6) |
| `polymer_swelled` | `swelling_ratio_g_per_g`, `capacity_g_per_g` | snow height from the ratio against capacity | — | done | done |
| `surface_spread` | `to_cleared_fraction` | particles fleeing the surfactant | — | done | done |
| `surface_colour_spread` | `to_spread_fraction`, `spot_count` | colour spots spreading | — | done | done |
| `partitioned` | `fraction_lower` | nothing in the vessel | the solute's split across the two layers | missing | **done** (ANIM-7) |
| `osmosis_changed` | `water_moles`, `mass_change_g` | nothing | the egg/potato swelling or shrinking | missing | **done** (ANIM-7) |
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
| `did_not_ignite` / `flame_starved` | `fuel`, `burned`, `oxygen_fraction` — **`did_not_ignite` carries none of them** | nothing | a flame that catches and gutters out at `oxygen_fraction` | missing | **done** (ANIM-7, `flame_starved` only) |
| `below_autoignition` | `autoignition`, `temperature` — the gap between them | nothing | the gap itself, since the answer is that nothing burns | missing | **done** (ANIM-7) |
| `reacted` | `moles`, `seconds`, `catalyst`, `activation_energy` | nothing in the vessel | reaction extent over the elapsed bench seconds | missing | **done** (ANIM-6) |
| `reaction_heat_released` | `energy_j` | nothing | the exotherm, on the same ramp the heat of mixing uses | missing | **done** (ANIM-6) |
| `fermented` | `sucrose_moles`, `ethanol_moles`, `carbon_dioxide_moles`, `active_yeast_grams`, `seconds` | **nothing** | slow bubbling paced over `seconds`, sized by the CO₂ moles | missing | **done** (PR 3) |
| `enzyme_hydrolysed` | `converted_fraction`, `seconds` | a caption percentage | the substrate visibly clearing | partial | **done** (ANIM-8) |
| `electrolysed` | `amps`, `seconds`, `coulombs`, `electrons`, `moles`, `grams`, `per_ion`, **`anode_species`/`anode_moles`, `cathode_species`/`cathode_moles`** (PR 4) | bubbles at two electrodes, count from moles, duration from seconds; **both electrodes got the same count off one product's moles** | each electrode sized by what actually leaves *it* — twice as many bubbles at the cathode as at the anode when water splits | partial | **done** (PR 3 charge, PR 4 ratio) |
| `cell_voltage` | `volts` | connection arc, magnitude from \|V\| | — | done | done |
| `decayed` | `parent`, `daughter`, `mode`, `moles`, `half_life_s` | the Geiger comes from a `measured` reading, not from `decayed` | decay drawn from the event, not only when an instrument is held | partial | **done** (ANIM-8) |
| `nuclide_spiked` | `activity_bq`, `moles` | nothing | initial activity | missing | **done** (ANIM-7) |
| `irradiated` | `wavelength_nm`, `irradiance_w_m2` | lamp, magnitude from the irradiance | — | done | done |
| `uv_attenuated` | `wavelength_nm`, `band`, `transmitted_fraction`, `mechanism` | **nothing** | the beam dimming to `transmitted_fraction` through the sunscreen | missing | **done** (PR 3) |
| `chemiluminescence_observed` | `relative_intensity`, `half_life_s` | the scene's glow, strength from the intensity | — | done | done |
| `hydrated` / `dehydrated` | `formula_units`, `water`, `at` (K) | nothing transient; the colour change arrives through the scene | the water leaving as steam at `at` | partial | partial |
| `neutralised` | `moles` of acidity cancelled | nothing | cancellation marks, count from the moles | missing | **done** (ANIM-6) |

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

- **PR 4 (scene numbers)** — the engine side. `SceneVessel` now carries
  `boiling_point_k` and `melting_point_k` as STANDING values, from the same
  call `StateEquilibrator` makes, so a flask sitting at 350 K under a partial
  vacuum reads as boiling and brine frosts at the temperature its own solutes
  bought rather than at a hard `272 K`; `headspace_volume_l` and
  `headspace_moles` where the vessel owns its gas, retiring the client-side
  `V = nRT/P` that was right at the event and stale after it.
  `Event::StateChanged` gained `kind` — one of the six transitions, named by
  the engine — and `moles`, so dry-ice fog is no longer indistinguishable
  from a boil at the wire level and a sublimation is sized by what actually
  left. `Event::Electrolysed` gained `anode_species`/`anode_moles` and
  `cathode_species`/`cathode_moles`, and the solvent cell now speaks at all
  when its cathode makes gas — it used to split water in silence, which is
  the one lesson this bench ships whose entire title is a ratio; each
  electrode is drawn at the moles that leave *it*, so hydrogen bubbles twice
  as heavily as oxygen. `Event::Corroded` gained `corroded_moles` and
  `corroded_fraction`, read off the vessel through the corrosion reaction's
  own stoichiometry — an extent, never a rate, because the rate belongs to
  `kinetics::REGISTRY` and travels as `Reacted`. And the species shelf now
  carries `molar_volume_l_per_mol`, so a solid that precipitates and
  redissolves in one step — leaving no scene row behind — is still drawn at
  its own grain size.

- **Corrosion extent** — `SceneVessel.corrosion` projects the current fraction
  of tracked metal atoms locked in the modeled oxide from
  `corrosion::corroded_extent`. The vessel keeps a labelled percentage and a
  restrained schematic marker after the event has passed. It deliberately
  claims no rate, history, thickness or surface coverage; directly added oxide
  is indistinguishable from oxide formed in the vessel.

- **ANIM-5 (the unread numbers)** — five quantities the wire has been
  carrying and the bench has not been reading. `gas_absorbed` is the mirror
  of `gas_evolved` and drew nothing: the same moles on the same log ramp,
  as bubbles that sink and shrink into the liquid rather than rise out of
  it. `headspace_partitioned` tints the band at the share of the volatile's
  whole inventory that is now gas — `gas_fraction`, capped below the band's
  own pressure tint so a full partition does not hide the piston — with
  arrows that say only which way this step went. `headspace_equilibrated`
  puts the settled `pressure` and `total_moles` on a gauge beside the lid
  that is drawn from the same headspace, so the two can be checked against
  each other. `supersaturated` hazes at `dissolved ÷ capacity` and at
  nothing below 1: a solution exactly at its limit looks like any other
  solution, and the distance past it is the whole quantity — which is why
  it is the number rock candy is about. And `corroded`'s own
  `corroded_fraction` — on the wire since PR 4 and read by nobody — now
  sizes a rust bloom on the metal layer itself, spot count and strength
  both functions of it, so a verdict with no extent yet draws no rust.
  Beside them, the sublimation fog is finally sized: dry ice in an open
  beaker often reports the transition and nothing else, and the plume was
  falling back to its two-column minimum, so `state_changed.moles` now
  reaches the plume through the same vapour magnitude a boil uses.

- **ANIM-6 (six more unread numbers)** — `reacted` is the commonest event
  the bench emits and it drew nothing in the vessel at all: the extent
  ring's strength is the moles on a log ramp and its tempo is
  `moles ÷ seconds`, because the same tenth of a mole in one second and
  over one hour are different observations and only the pair separates
  them; the catalyst and the activation energy actually used travel beside
  it. `reaction_heat_released` glows on exactly the ramp `heat_of_mixing`
  uses — dissolving lye and a hand warmer are the same claim about the same
  quantity, and two ramps would say they were not. `neutralised` — the
  commonest reaction a school lab runs, and the only one that happened with
  nothing at all against it — draws cancellation marks counted from the
  moles of acidity that went. `bubble_ride` draws the raisin with the gas
  that has to cling to it, and draws **no** bubbles on an object that
  floats unaided, because they are not why it is up there and saying so is
  the misconception KID-13 exists against. `adsorbed` darkens the sorbent
  by `held ÷ (held + still_dissolved)` — the share that actually left the
  water, which is the answer to "can charcoal take this dye out" — and
  keeps the remainder and the loading on the readout, because the event
  carries both halves for a reason and neither can be read without the
  other; no isotherm ceiling is claimed, since the wire carries no
  capacity. `thickened` shows the stirrer's arc blunted by `strength`, and
  only where the engine says `sheared_hard`: oobleck stirred slowly is a
  liquid, and drawing resistance there would be a picture of the recipe.

- **ANIM-7 (the last six)** — `flame_starved` draws the flame guttering at
  the oxygen fraction the engine says the air had fallen to, and draws
  **no flame at all** when `burned` is zero, because that is a carbon-
  dioxide extinguisher and the air was already too thin to light in; the
  readout is the fraction itself, which is the number that contradicts "it
  used up all the oxygen". `below_autoignition` draws the gap, because the
  answer BRD-041 gives is that nothing burns — the bar fills toward the
  autoignition point and never reaches it, since reaching it is a
  different event. `nuclide_spiked` ticks at `activity_bq` on a log ramp
  from one disintegration a second to a teaching source's megabecquerels.
  `partitioned` splits ten dots across the two layers at `fraction_lower`,
  drawn inside the bands the engine's own volumes made, and the two counts
  always sum to ten because the solute did not go anywhere else. 
  `osmosis_changed` swells or shrinks the object and points the arrows by
  the **sign** of `mass_change_g`, which is the whole observation: an egg
  in syrup and an egg in water arrive as the same event. And
  `thermal_equilibrium` shows the settled temperature — with the badge
  stepping off the glass and saying "exhaust, not the glass" wherever
  `holds_nothing` is set, because "thermal equilibrium at 2496 °C" once
  reached a reader over an empty beaker: a true number attached to a
  picture that invites the wrong reading.

- **ANIM-8 (the constants that were left)** — six rows whose visual existed
  but did not move when the engine's number did. `plated`'s magnitude was a
  literal `1`, so a copper blush on a nail and a nail gone orange drew the
  same shimmer; the coating's thickness is now the moles on a log ramp.
  `consumed` drew nothing transient at all; a ribbon is now eaten at the
  share the engine says is **left**, and where the event carries no
  `remaining` — which is optional precisely because "is used up" once
  reported half a magnesium ribbon gone — the ribbon is drawn dissolving
  at its edge with no length claimed. `ground` had a magnitude on a
  log-area ramp and no powder anywhere; the grain radius is now the
  `diameter_um` the engine actually ground to, so grinding twice draws
  visibly finer powder rather than the same specks with a new caption, and
  the count follows the area a heterogeneous rate would later see.
  `vessel_swept` drew two static arrows whatever the sweep; their cycle is
  now the carrier pressure. `enzyme_hydrolysed` had a caption percentage
  beside a liquid that never changed — a number nothing on the stage
  agreed with — and the substrate haze is now `1 − converted_fraction`.
  And `decayed` drew nothing unless a Geiger happened to be in hand; the
  ticks now come from `ln2 ÷ half_life_s × moles`, which is the same
  activity that instrument reads, so a long-lived tracer ticks slowly and
  a large parcel ticks often.

- **Computed gas/foam motion** — `gas_produced.rate_moles_per_second` now sets
  the visible-bubble cadence while total moles continue to set count and size;
  `foam_changed.half_life_seconds` sets the foam head's collapse to half height.
  Both raw values and the derived bubble period are readable and exposed as
  `data-*` evidence; reduced-motion mode keeps the evidence and stops motion.

Every one of those carries a `data-*` attribute naming the number that drives
it, so the browser UX gate and any later test can assert on the *quantity*
rather than on the presence of a shape.

## What the engine still lacks

One row cannot be closed from the client at all. **`Event::DidNotIgnite`
carries nothing but the vessel id** — no fuel, no oxygen fraction, no gap
to the autoignition temperature — so there is no quantity for a visual to
be a function of, and drawing anything for it would be a picture of the
word. Its sibling `FlameStarved` carries `fuel`, `burned` and
`oxygen_fraction` and is drawn; the shared row above is marked done on
that half alone, and deliberately says so. Giving `DidNotIgnite` the same
three fields (or the gap `BelowAutoignition` already computes) would close
the other half.

## What the engine should add

Seven items were listed here after PR 3. **Six are now on the wire (PR 4)**;
what each closed is recorded below so the reasoning survives the fix, and the
one that is still open is still open.

1. ~~**`SceneVessel.boiling_point_k`** (and, ideally, `melting_point_k`).~~
   **Done (PR 4).** Both are standing scene fields, computed by
   `scene::liquid_transitions` — water through `solve::vessel_transitions`,
   which is the same call `StateEquilibrator` makes, so the standing value
   and the `state_changed.at` a boil reports are one number rather than two
   that agree by inspection; any other liquid through the registry's reviewed
   normal points, shifted for the vessel's pressure by the BRD-031
   correlation the boiling-point apparatus already uses. The stage reads
   `boiling_point_k` for the steaming gate and `melting_point_k` for the
   frost gate.
2. ~~**`SceneVessel.headspace_volume_l`** and **`headspace_moles`**.~~
   **Done (PR 4).** Present exactly where the vessel owns its gas (sealed,
   pressure-controlled) and absent for open and swept, whose headspace is the
   room. The `V = nRT/P` reconstruction and the Boyle fallback behind it are
   kept for older logs and now sit below the engine's own figure.
3. ~~**Molar volume on `precipitated` / `dissolved`**.~~ **Done (PR 4), by a
   different route than proposed, and the difference is worth stating.** The
   field belongs on the event, but the two events are constructed inside
   `kerotakis-phreeqc`, which another lane owns; adding a field to the
   variant would have edited six literals there. Molar volume is a property
   of the *substance* and not of any vessel, so it ships as
   `molar_volume_l_per_mol` on the species shelf instead, derived by
   `SpeciesData::molar_volume_l_per_mol` from the same registry mass and
   density the bench uses. That closes the actual defect — a solid that
   precipitates and redissolves in one step leaves no scene row to read — and
   closes it for every event that names a species rather than for two of
   them.
4. ~~**Per-electrode product moles on `electrolysed`.**~~ **Done (PR 4).**
   Both half-reactions travel: `anode_species`/`anode_moles` and
   `cathode_species`/`cathode_moles`. The solvent cell also *speaks* now when
   its cathode evolves gas, which it did not before — splitting water emitted
   the two gases and never the run that made them, so the bench had no
   `electrolysed` event at all for the one lesson whose title is the ratio.
   The stage draws each electrode at the moles that leave it, and falls back
   to the shared charge for a log that carries only one product.
5. ~~**`gas_produced.rate_moles_per_second` is already there**~~ — **done in
   the computed-motion tranche**, as the visible-bubble cadence. The same
   tranche uses `foam_changed.half_life_seconds` for collapse to half height.
   `emulsion_changed.half_life_seconds` was already driving its coalescence
   animation. No engine addition was needed.
6. ~~**A `sublimated` distinction.**~~ **Done (PR 4)**, as a `kind` field on
   `state_changed` rather than a new event, because `phase_route` already
   emits `state_changed` for all six transitions and a second event would
   have split one route in two. `Event::state_changed` is now the only
   constructor, so the six emitters cannot disagree about which transition a
   given `from`→`to` is; `moles` travels beside it. The codex names
   `sublimed`, `deposited` and `condensed` where it used to say
   `state_changed`.
7. ~~**`corroded` carries no extent.**~~ **Done (PR 4).** `corroded_moles`
   and `corroded_fraction`, from `corrosion::corroded_extent`: the metal
   atoms per formula unit of the product come from the registry's own formula
   for it, so `4 Fe + 3 O₂ → 2 Fe₂O₃` counts two irons per oxide unit without
   a table saying so. It is an *extent* and never a rate — the rate is
   `kinetics::REGISTRY`'s and travels as `Reacted`, and a second opinion
   about the same nail is exactly what the event's own doc refuses. Nothing
   draws it yet: the pitting visual is the next lane's, and the row above
   still reads *missing* for that reason.
