# Acidified permanganate/peroxide: implementation lanes

Status: source design prepared against main
`b22f3da5655614ae37e8874fefc023becccd1dcf` on 2026-10-15. No reaction route,
runtime change, native execution or scientific acceptance is delivered by this
document. Implement the prerequisites below in order on narrow branches from
actual integrated main.

## Observed gap and preserved evidence

The separately frozen physical-stock follow-up
`F12-permanganate-peroxide-acid` establishes acidic conditions but does not
produce the expected coupled redox endpoint. Its final pH is about 1.4592;
0.002 mol permanganate and about 0.0049996 mol peroxide remain. The small oxygen
change belongs to background peroxide decomposition and does not establish this
target reaction. Preserve the original
[design](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/stock-followups-20261015/design.json)
and
[assessment](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/stock-followups-20261015/assessment.json).

| Binding | Frozen value |
| --- | --- |
| Engine source | `d791d3062886e4a06dbc7c9d1635383da3e05359` |
| Native binary SHA256 | `e837e75f6e1c2c8d3f86e56144a1cebd8e2a6edacc3a264e94ed1117a006c21c` |
| Cargo.lock SHA256 | `12d9a91b9348a577cb83f1f16db0d89c4994ea79d1f6967a611b20e8a71a78dc` |

The proposed endpoint is:

```text
2 MnO4- + 5 H2O2 + 6 H+ -> 2 Mn2+ + 5 O2 + 8 H2O
```

[UCSB Science Line](https://scienceline.ucsb.edu/getkey.php?key=164) supplies the
net ionic and potassium/sulfate forms, explains the need for acid, and identifies
chloride oxidation as a competing reaction when hydrochloric acid is used.
[Alonso de la Peña et al., Dental Materials Journal 32 (2013), methods, page 655](https://www.jstage.jst.go.jp/article/dmj/32/4/32_2012-317/_pdf)
uses sulfuric acid and permanganate for peroxide determination and gives the
same net equation. These sources support the acid-medium stoichiometry. They
do not supply the implementation's proposed pH ceiling, a general rate law or
an adopted reaction enthalpy.

## Why adding two rows is insufficient

The relevant ownership paths are:

| Source | Current responsibility and consequence |
| --- | --- |
| `crates/kerotakis-core/src/bench.rs`, `step_with` | Clears the touched vessel's `solution` before each solver pass and captures `StepStart` after the operator. A previously computed pH is deliberately unavailable. |
| `crates/kerotakis-stack/src/lib.rs`, `standard_solvers` | Runs `CuratedEquilibrator` before the aqueous tail. A fresh-pH-gated curated row placed only here cannot run in ordinary CLI execution. |
| `crates/kerotakis-core/src/displacement.rs`, `over` | Runs the aqueous solver, applies post-speciation metallic chemistry, and solves the changed state again. This is an existing shared continuation point. |
| `crates/kerotakis-cli/src/main.rs`, `build_stack` | Wraps the native aqueous solver with `DisplacementEquilibrator`. |
| `crates/kerotakis-wasm/src/lib.rs`, `BrowserAqueous::equilibrate` | Calls the same `displacement::over` for the browser's hook/cache-backed aqueous solver. |
| `crates/kerotakis-core/src/solve.rs`, `equilibrate_phase_coupled` | Can re-enter the aqueous tail during the same operator while settling solvent phases. The first continuation is not necessarily the last solve. |
| `crates/kerotakis-phreeqc/src/aqueous.rs`, `solve_once` | Prefers the whole operator's `step_start` inventory for its heat balance. A second solve can include a reaction already performed by another owner. |

`refresh_solute_charge` prevents one stale charge proxy, but does not remove a
redox transformation from whole-step enthalpy accounting. Temporarily replacing
`StepStart` for one re-solve and restoring it afterward is also insufficient:
later phase-coupled re-entry can price the endpoint again from the original
snapshot. An endpoint transaction and persistent step-local ownership must be
settled before adding a firing route. A recursive or native-only PHREEQC hook
would leave the common host contract unresolved.

## Lane 1: shared continuation, thermal ownership and rollback

Start by reading the source paths above and the existing displacement and
phase-coupling tests. Deliver a small common transaction/continuation primitive
in core, usable by both native and browser aqueous wrappers. Keep this a
separate prerequisite PR from the new reaction data.

Define how the operator's original `StepStart` coexists with accepted
post-speciation reactions. The state must retain the original material/gas
accounting and identify which transformations have already been owned. A later
aqueous solve must neither charge the endpoint as acid/base neutralisation nor
price its unadopted reaction enthalpy from the original operator snapshot.
That ownership must survive all phase-coupled re-entry within the operator and
be cleared at the same lifecycle boundary as other step-local data.

Specify the commit point for an endpoint candidate and its fresh aqueous
readback. On failed or unavailable recharacterisation, roll back candidate
inventory, acidity, gas transfer, temperature and success events together.
Restore the last valid state with an explicit model-limit/failure result;
stale solution readings must not describe changed contents. Browser cache misses
must follow the same transaction contract as native solver failures.

Acceptance must exercise:

- A deterministic fake aqueous solver that records snapshots and forces a
  subsequent phase-tail pass, proving the same reaction cannot be priced twice.
- Failure on recharacterisation after an endpoint candidate, proving complete
  inventory/charge/gas/temperature rollback and absence of a committed success
  event. Test both retained and externally released gas candidates.
- Successful continuation with final readings belonging to final contents,
  followed by another operator proving no stale ownership state persists.
- Native and browser wrapper call paths, cold and warm cache behavior, cache
  unavailability, and the existing displacement/phase-coupling controls.
- Explicit unknown-enthalpy handling: absence of a modeled endpoint heat is a
  limitation, not a physical prediction that the real reaction has zero heat.

Review these semantics before extending reaction data. A green fake-solver test
alone is not full native/browser acceptance.

## Lane 2: finite acidic endpoint data and a shared admission gate

After lane 1 integrates, introduce typed metadata for a pH-gated aqueous
stoichiometric endpoint. One maintainable option is an optional field on
`CuratedReaction`, mechanically absent on every existing row. Both eligibility
queries such as `fires` and the actual application must use the same gate.
Post-speciation continuation should select only rows it owns, rather than replay
all pre-aqueous curated reactions or match a printed equation string.

Start with the sulfate-acidified potassium/peroxide domain demonstrated by the
frozen follow-up. Require positive liquid water, a fresh finite computed pH
at most 3, finite positive reactant availability, and finite available acidity.
The pH ceiling is a declared implementation domain, not a universal reaction
threshold. Neutral/basic products, kinetics and competing redox mixtures are
outside this endpoint contract. In particular, the acid gate alone does not
justify silently treating chloride as an inert spectator.

Test ionic and potassium-stock input equivalence. A potassium-component sibling
must retain its two potassium ions and have a genuinely eligible dissolved
molecular pool. If fresh speciation has already converted the stock to ions,
use the ionic endpoint; do not force a nominal molecular row to consume an
inaccessible solid. Use existing `acid_protons`/`consume_acidity` ownership for the
six proton equivalents, subject to conservation tests; do not extend global
`FAST_REDOX` to oxygen or borrow the ungated permanganate/oxalate hydroxide
shortcut. Bound extent by each reactant and acid availability. Consume only
eligible aqueous/liquid portions; a dry crystal, gas portion or inaccessible
solid must not contribute simply because `moles_of` includes it.

Every committed endpoint must disclose its exact scope in a named structured
model-limit event: stoichiometric endpoint, no adopted timescale or reaction
enthalpy, and limited acidic product domain. Subsequent solution readings must
belong to the committed inventory through lane 1's shared continuation.

Acceptance includes atom, charge, H/O-water and mass closure for the complete
reagent inventory; separate peroxide, permanganate and acid limiting cases;
trace and exhausted doses; both reagent representations; potassium and sulfate
conservation; and unchanged dry, neutral, basic, uncharacterised and wrong-phase
controls. Open products require `GasEvolved` accounting; sealed products require
`GasContained` and retained oxygen. Background peroxide decomposition must be
distinguished from the target's five-oxygen/two-manganese stoichiometry.

## Lane 3: integrated stock replay and delivery

Freeze new complete-stock protocols and expectations before execution. Include
sulfuric acid first and last, each supported reactant addition order, finite
acid excess/deficiency, sealed/open boundaries, and repeated waits. Validate
fresh-pH admission, final Mn(II), depletion ratios, oxygen accounting and
spectators through the complete standard stack, not only a directly invoked
curated helper. Check that final pH, material appearance and diagnostics do
not refer to an intermediate state.

Bind source, binary and lock separately from the original follow-up; preserve
its failure unchanged. Run focused native controls and actual browser hook/cache
controls, then the applicable full product fleet. Publish a narrow reviewed PR,
enable normal exact-head auto-merge when non-protected acceptance is complete,
verify integration, and only then update deployment and current-state summaries.
API/CLI agreement and successful execution are separate from physical endpoint
assessment; neither establishes empirical calibration.


## Current implementation checkpoint — 2026-10-16

Lane 1 integrated through PR799 as `79f9e5f8`, after all 58 applicable checks
succeeded (one publication skip), including native, actual browser, Wasm, bridge
and complete own fleets. [Integration receipt](../audits/reaction-dataset-200-20261015/pr799-integration.json)
verifies parents/tree. Lane 2 is now in source implementation from that actual
main. No new endpoint execution or acceptance is claimed. Require genuinely
owned aqueous H+ in addition to fresh complete acidic readback; a charge proxy
alone cannot supply reaction matter. Retain the sulfate/potassium bounded domain,
explicit unsupported competitors, open/fixed-sealed gas ownership and unknown
heat disclosure. Original observations and forecasts above remain unchanged.
