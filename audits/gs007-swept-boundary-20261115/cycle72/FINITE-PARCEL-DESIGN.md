# GS007: the missing finite-sweep owner

The blocker is an absent finite boundary API and missing apparatus input, **not an identified missing carbonate equilibrium constant**. Source52404e01c exposes the existing equilibrium scope but does not repair C2. This design adds no physical parameters, executes no original and earns no qualification. The exact-source Rust projection controls remain unexecuted after the one admission below.

## Actual admission and retained result

At08:58:56UTC the nonblocking shared heavy lock was acquired. Available memory4668.63MiB passed4608MiB, and fast free15,673,675,776bytes passed strictly12GiB. Load7.817871 failed the required strictly<3.5. No child started; process exit released the lock. No admission retry, polling loop, new validation job or original replay followed.

The saved native original remains C1 PASS/C2 FAIL: rich calcite1.038837619669e-4mol becomes0. The rich snapshot is298.2014743281659K/112418.98012693944Pa; immediately after the existing sweep it is298.0383822314386K/100000Pa, and after60s298.04112821167905K/100000Pa. These are retained source6a2 observations, not new524 measurements. The native CO2 reservoir exports0.001265954140717mol plus1.063249319833e-10mol during wait after the separately booked initial finite headspace vent. Certified H/O/carbon/charge bookkeeping closes; pending carbon is zero. Neither that ledger closure nor a new warning establishes C2.

## Exact source boundaries

The accompanying reader actually inspects exact signed52404e01c and records the complete hashes of seven source files. Twelve source-contract assertions pass. Its initial relative-path lookup failure is retained; correction changes only the audits-directory lookup.

| Source interface | Current authority | Missing authority |
|---|---|---|
| `Operator::Sweep` and canonical parser, core `ops.rs`/`script.rs` | Vessel ID and total pressure only | No inlet moles, flow schedule, composition witness or contact law |
| `Headspace::Swept`, core `vessel.rs` | Pressure; `headspace_volume()` returns None | No finite carrier/contactor/outlet state or cumulative used budget |
| Sweep arm, core `bench.rs` | Vents existing headspace, then selects Swept | No owned replacement parcel; old sealed500mL is no longer a finite contact volume |
| `GasExchangeClock`, core `clock.rs` | Film transfer for Open only | Swept explicitly returns, preventing a second CO2 owner |
| `partition`, PHREEQC `aqueous.rs` | `ExternalGasKind::Reservoir`, zero initial gas, existing logpartialpressure−12atm | No finite carrier mixture/contact capacity; later calls can export carbon again |
| `ExternalGasKind::Dose` | Explicit finite added reactive gas; excess vents | No sweep carrier/flow/time state; pure finite dose is not a finite mixed-gas contactor |
| `complete_basis` and gas-aware certification, PHREEQC `inventory.rs` | Actual signed crossings, certified H/O/base reconstruction and reservoir ownership | No finite external parcel before/after witness to debit and certify |

## Inputs the unchanged original does not supply

The original specifies `sweep v1 1bar` then `wait 60s`. It supplies neither an inlet molar-flow history in mol/s nor a total carrier-gas dose in mol. The earlier `seal v1 500mL` owns the gas that the sweep explicitly vents; it cannot be reused as the volume of fresh gas flowing during60s. Pressure and duration do not determine carrier moles.

For a finite-parcel equilibrium-contact route, authority must specify a finite carrier inventory and composition, inlet thermal state, and whether the contactor has fixed pressure or fixed volume. The contact/equilibration assumption and its domain must be explicit. For a transient physical prediction, the supplied flow schedule also needs a supported apparatus-specific transfer closure, for example measured `k_L a` in s^-1 or `k_L` in m/s plus interfacial area and contact geometry. Volumetric flow needs its reference temperature, pressure and gas state; an unlabeled flow value is insufficient.

No such value/provenance is present in this original or its source-bound observation. Existing Henry/USGS carbonate data do not supply an apparatus flow or transfer coefficient. The open-room still-surface coefficient is not evidence for a forced nitrogen purge. No value is selected or tuned here. A new flow-annotated companion could validate a future route but would not replace this original or earn its credit.

## Proposed transaction API: design only

1. `FiniteSweepSupply` owns a provenance-bound finite inlet composition in mol, temperature in K, pressure in Pa and the selected contact-volume/pressure constraint. A flow schedule additionally owns elapsed interval and remaining supply. Persist it as an authoritative bench/vessel field included in `step_with` checkpoints and serialization, rather than a side map outside rollback.
2. `SweepParcelWitness` binds vessel revision, supply revision, exact inlet composition/state, liquid/solid start inventory and contact constraint. Form a private candidate; do not debit supply, emit outlet events or advance an interval while native thermal iterations or canonical reentry are provisional.
3. A finite contactor solve jointly owns liquid+solid and the **whole gas mixture**, including inert carrier and exchanged CO2/water. Outlet CO2 partial pressure follows the finite mixture and its selected volume/pressure constraint; it must not be imposed by the legacy−12 reservoir. Do not cap CO2 moles by carrier moles: that inequality is not a physical capacity law. Remove the competing reservoir only when this finite contactor is the admitted sole owner; otherwise retain the explicit legacy limitation.
4. `FiniteSweepCommit` must prove element balance including H/O, formal charge and priced heat across vessel+remaining supply+retained contactor+outlet. Carrier gas stays a gas owner; do not manufacture aqueous N totals to represent it. Preserve certified analytical acid/base reconstruction rather than substituting measured freeOH or fabricated species.
5. Commit the accepted vessel state, one supply debit, one interval advancement and the resulting outlet together. Bind a parcel identity to the accepted transaction so repeated native guesses/cache reads cannot spend it twice. Cache input/witness includes supply/parcel revision, composition, contact law and thermal state. Unsupported mapping, stale witness, insufficient supply, unpriced required heat or missing transfer authority refuses atomically and preserves the entire checkpoint.

The legacy global-reservoir allowlist is insufficient certification for this new route. A finite parcel requires an actual external before/after ledger. Existing `ClockCo2Boundary` is a signed single-CO2 parcel, not an inlet-mixture/contactor owner; repurposing it without the carrier and outlet witness would omit material authority. Extending the Open clock to Swept while leaving the native reservoir enabled would double-own carbon.

## Bounded implementation and acceptance after authority exists

Implement the finite inlet/outlet witness and atomic debit independently of a flow law first. Source-bound controls must reject stale/replayed witnesses, overdraft, nonfinite/negative amounts, missing composition/units/provenance and double ownership. Positive controls must close carrier plus CO2 and H/O/charge, retain gas enthalpy scope explicitly, and survive persistence/rollback. A finite-equilibrium contact model must be labelled as such; it is not validated transient kinetics.

Then integrate a single native finite-mixture contactor with immutable before/after witnesses and matched finite-dose/open/sealed negatives. Only actual unchanged-original execution plus independent criterion, complete material/heat and model-boundary review can decide C2. There is **no causal proof yet that finite ownership alone restores the predicted scale**, so this design is not a promise of a qualification gain. The current source-only honesty patch remains separate from that scientific repair.

Coverage stays431/468 originals,435 byte-identical catalog entries,37 remaining. PR884 remains independently DRAFT with its earlier focused proof; no duplicate acceptance run, merge, Apple action, water proposal or `ci:full` work is part of this task.
