# Unknown-heat aqueous continuation

`aqueous_continuation::continue_without_reaction_heat` is an atomic primitive
for a separately admitted post-speciation material transformation. It installs
no reaction rows, rate laws or chemical admission rules. Native and browser
hosts can call the same core function immediately after a fresh aqueous answer;
their shared existing continuation point is `displacement::over`.
This prerequisite does not install a chemical endpoint call. The object-safe
`Equilibrator::trusted_aqueous_readback` bridge defaults to `None`; concrete
PHREEQC opts in, and both `DisplacementEquilibrator` and `BrowserAqueous` forward
that explicit capability. `characterize_via` and `continue_via` can therefore
reach the trusted adapter through the erased interface used by the shared
wrapper. Untrusted generic equilibrators refuse before any transformation.
The bridge characterizes aqueous state; it does not run other wrapper chemistry
or choose when a future endpoint should be admitted by `displacement::over`.

The caller provides a current-answer receipt, a trusted aqueous recharacterizer
and a transformation closure. A complete finite solution and active `StepStart`
are prerequisites. `Some(solution)` alone does not establish freshness: a deposit
can leave that field populated with the previous answer.

The concrete PHREEQC implementation of `TrustedAqueousReadback` performs an
actual native/hook/cache-backed solve on a clone after clearing old readings.
It returns `CurrentAqueousAnswer` binding the requested input and complete output
state. A changed inventory, configuration or transient state invalidates the
initial receipt, even if an old `SolutionInfo` is still present. Subsequent
readback must bind the exact question issued by the transaction and its final
answer. Generic `Equilibrator` output is no longer sufficient.

`CurrentAqueousAnswer::attest_computed` is an explicit trusted-adapter assertion,
not a chemistry proof or a security boundary against arbitrary Rust code. Other
implementers, including synthetic test doubles, must explicitly assume that
contract; an adapter that falsely attests an answer violates it. The maintained
product implementation is the concrete PHREEQC adapter. Receipts are in-process
state bindings, not public hashes or serialized solver-cache credentials.

The closure runs on a clone, emits its material/gas events, and preserves the
operator snapshot, vessel identity, temperature, excess enthalpy, thermal mode,
operator heat input and boundary configuration. The same configuration guards
apply to readback. Initial support is restricted to open and fixed sealed
boundaries; pressure-controlled and swept continuations remain separate work.
It owns
its chemistry and domain admission; the primitive checks finite state and
represented element/charge conservation, not scientific validity.

The clone's old solution and free-ion readings are cleared before readback.
Only a complete finite new solution, conserved inventory and unchanged thermal
ownership permit commit. A missing cache answer, returned solver error, invalid
state or changed thermal state commits neither the clone nor its success
events. The original valid pre-transformation state remains available. This
does not roll back solver-private cache/statistical state, and does not catch
Rust panics. Caller-side success events are published only after this function
returns `Ok`.

## Thermal policy

A committed continuation sets the vessel's typed
`AqueousHeatOwnership::UnmodeledContinuation`. The policy conservatively
withholds **all remaining aqueous reaction heat in this operator**, including
the transformation's associated recharacterization. This is broader than
subtracting one known reaction enthalpy: the new route has no reviewed heat
with which to perform that subtraction. It does not claim that the omitted heat
is physically zero or that the resulting temperature predicts the real mixture.
A named `not-modeled.aqueous-continuation-heat` event states this limit.

PHREEQC continues computing composition at the supplied temperature. Its
whole-step enthalpy calculation and dissolution fallback are disabled under
this policy; its thermal fixed point also holds the supplied temperature so
water/Cp relabelling cannot reconstruct the omitted transformation heat. The
native MIX optimization declines while this policy is active and leaves the
ordinary guarded readback in control. Other physical models retain their own
contracts; this primitive does not validate the resulting full thermal or phase
trajectory after an unknown-heat transformation.

The original `StepStart.contents`, acidity fields and `gas_out` ledger are
preserved. Returned gas events still pass through normal caller accounting;
contained gas remains material inventory. Aggregate `GasContained` reports must
equal actual newly retained gas of each species within relative and inventory
roundoff tolerance; omitted, underreported, invented or duplicated production
events are rejected. Loss of owned gas, for example by dissolution, requires no
negative contained event. Existing gas cannot justify new production narration.
Fixed sealed continuations reject external `GasEvolved`/`GasAbsorbed` events.
For open boundaries those events extend the conserved element ledger; the
transformation owns the physical justification for the gas exchange. The
primitive does not infer atmosphere-transfer kinetics from conservation alone.
Suppressing thermal interpretation
does not suppress atom, charge, mass or gas conservation.

Ownership lives on `Vessel`, separately from `StepStart`, because a single
operator may clear or recapture snapshots during delivered heat chunks or
titration substeps. Clones and in-process solver delta snapshots carry it.
`Bench::step_with` resets it at operator entry and on every ordinary success or
error return. Existing serialized vessels omit the transient field; persistence
is for completed operators, not resumable in-progress transactions. Embedders
calling solvers directly must preserve the same vessel during a continuation
and explicitly establish their equivalent operator lifecycle. A JSON roundtrip
of an in-progress candidate is not a supported continuation mechanism.

## Acceptance and limits

The [run at `bc5f46b5`](https://github.com/CrispStrobe/kerotakis/actions/runs/38850237630/job/116809626791)
passed all three focused native PHREEQC continuation tests, but the core
phase-tail control still failed: its separate fixture had not received the
inert headspace gas added to the shared native fixtures. The independently
computed phase cooling reached 294.15876266736905 K instead of the asserted
298.15 K. That core no-boiling fixture now also explicitly contains 0.04 mol N2;
the strict temperature assertion and separate conservative vacuum-boiling
control remain unchanged.

The same run's composed-locale gate identified `aqueous_continuation.rs` as
an undeclared phrase composer. It is now registered in both the Rust gate and
Python locale lint, with actual German and French translations of the heat
coverage limit. These source repairs require a fresh hosted run; the earlier
failures and the successful native subset do not establish full acceptance.

The subsequent [native run at `a45c6f45`](https://github.com/CrispStrobe/kerotakis/actions/runs/38817273479/job/116705372463)
passed cache replay but failed the sealed no-temperature-change controls.
Their one-litre headspace contained only about 0.001 mol oxygen after the
callback: roughly 2.5 kPa. The independent phase solver legitimately boiled
water and cooled it to about 294.21 K. This was not duplicated aqueous reaction
heat. The no-boiling controls now explicitly contain 0.04 mol inert N2;
a separate core control requires conservative vacuum boiling to remain active
under unknown aqueous heat ownership. The thermal policy and numerical
assertions are retained; original failed logs remain preserved. CLI prewarm
stopped on the same sealed fixture, before the new Wasm controls executed.

The first hosted build at `75acd4072f94c3814aa17a73667a1bbfab30e521`
[failed before execution](https://github.com/CrispStrobe/kerotakis/actions/runs/38807342410/job/116671491406):
the metadata guard referenced `ignition_trial` and `ignition_feed_temperature`,
which are not fields of the integrated `Vessel` type. Those invalid references
are removed; the actual thermal mode, operator heat input, headspace and pending
CO2 transfer guards remain. Source field-name and formatting checks pass, but
the corrected build and all runtime controls still require hosted execution.

The core controls use an artificial conservative callback, not an installed
reaction model. They cover released/contained gas, original snapshot retention,
actual phase-tail entry points, repeated passes, snapshot recapture, failed and
missing readback, invalid results, lost ownership, invented heat, conservation
failure, delta commit and operator cleanup. Focused native controls exercise
the actual PHREEQC recharacterizer and the shared displacement/phase wrapper.
These tests must run on hosted infrastructure before acceptance; source
formatting alone is not runtime evidence.
Native warm/export/import replay checks that the concrete adapter consumes
exact cached answers without a native solve. A separate no-default-features
test exercises concrete cache misses and failing external hooks with stale
readings present; CI runs it explicitly because native builds do not dispatch
through `SolveHook`. These are distinct controls. A successful real browser
hook and shipped-cache continuation, including changed-input cache refusal,
must pass the hosted Wasm/Emscripten acceptance before an endpoint is enabled.

`Lab.continuationAcceptance(expectMissing)` runs isolated local mechanical
fixtures through the actual browser wrapper without modifying the learner
bench. CLI prewarm includes their complete-extent native answers in its exported
cache. The existing Wasm lab harness checks successful shipped-cache replay and
a different, un-prewarmed extent refusing with complete vessel rollback. The
Wasm bridge harness checks the real Emscripten hook, then replaces it with a
throwing hook and verifies changed-input refusal and rollback. Both fixtures
also verify that a deposit invalidates the genuine initial receipt before the
callback runs. Native acceptance uses the boxed displacement wrapper.
Negative controls require the concrete cache-miss source or the deliberately
tagged external-hook failure and retain that error in the report. An unrelated
receipt, conservation or thermal guard failure cannot count as a cache/hook pass.

These existing harnesses execute actual Wasm and Emscripten in Node. They are
distinct from a Chromium/Playwright runtime measurement; no Chromium execution
or empirical chemical validation is implied. The artificial peroxide callback
is only a mechanical conservation fixture with deliberately unmodeled heat,
not an installed peroxide reaction or a forecast for its laboratory behavior.

No acidic permanganate/peroxide chemistry is added here. That next lane needs
its own reviewed gate, finite acidity/reactants, explicit product-domain and
timescale limits, integrated stock protocols and native/browser cache controls.
The primitive's existence does not establish that a particular reaction is
supported, instantaneous, calibrated or thermally complete.
