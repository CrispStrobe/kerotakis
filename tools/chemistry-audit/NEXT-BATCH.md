# Preserved experiments and second discovery batch

The original 50 experiments remain unchanged. Cases 51–86 in `next_batch.py`
ask new questions, including ammonium-buffer perturbations, bisulfate,
alternative strong acid/base identities, gypsum, magnesium hydroxide,
selective filtration, fractional native MIX, reverse titration, organic-acid
proton ladders, thermal mixing, product-loaded/reverse ester equilibrium,
conductivity controls, additional alcohols, and ammonium nitrate dissolution.

Run from this worktree using a verified current binary and a fresh directory:

```sh
python3 tools/chemistry-audit/next_batch.py --out tools/chemistry-audit/next-batch-1
python3 tools/chemistry-audit/analyse_next.py tools/chemistry-audit/next-batch-1 --out tools/chemistry-audit/next-batch-1-checks.json
```

The recorder preserves the exact input, every output document, stderr,
execution outcome, binary/lock hashes, and source-diff identity. It refuses
nonempty output directories. The analyser refuses to overwrite check results.
Successful execution and physically plausible numbers are separate checks.
Unmet checks, including declared missing models, remain visible and exit 1.

The element checks cover only the nonvolatile elements actually supplied in
these inputs, across **all** vessels. They do not pretend that an open vessel
is closed to H/O/C/N or that conserving an analytical element inventory proves
native oxidation-state or species correctness. Endpoint, direction, phase
selection, order invariance, enrichment, and controls are checked separately.
Input stoichiometry uses the existing registry, not another unreviewed data
source. No batch case identifier is consumed by production chemistry code.

## Catalog preservation

Two experiments address gaps not filled by the existing lessons:

- `endpoint-is-not-a-full-drop`: paired 7 mL and 1 mL automatic-titration
  trial increments for a deliberately non-multiple 23 mL equivalent volume.
  The concept is computational refinement, explicitly distinguished from
  physical burette/indicator uncertainty.
- `equilibrium-can-run-backward`: one reactant-only and one product-only
  starting mixture, with the same explicit equilibrium request. The entry
  discloses the existing K=4 ideal-mixture approximation and lack of a rate
  prediction; it is a virtual-model experiment, not a synthesis recipe.

Both are discoverable codex entries with complete German presentation fields,
prediction questions, diagnosed distractors, three explanation levels, and
original model-assisted editorial provenance. Matching `lessons/*.lab` files
are replayed by the existing lesson harness. No external numeric datum was
introduced, and existing buffer/fizz/neutrality lessons were not duplicated.

The earlier preservation table links neutralization order to `neutral-moves`;
that lesson actually teaches temperature-dependent neutral water pH. The
permanent **order** checks are in `reaction_heat.rs`, not that lesson.

Draft script evidence is in `catalog-endpoint-draft.ndjson` and
`catalog-equilibrium-draft.ndjson`. These files are developmental replay
evidence, not the final source-revision verification gate.

## Additional gap found by catalog lint

Replaying the full catalog exposed an old thiosulfate approximation: the
kinetic law read acid activity but its surrogate stoichiometry did not spend
acid. The repaired reaction network consumes two represented strong-acid
equivalents per sulfur, forms balanced sodium/sulfur dioxide/water products,
and is bounded by every consumed reactant. Its initial rate still uses the
native activity, distinct from the analytical acid inventory. Within an
uncoupled kinetic interval the activity coefficient is locally fixed while
acid depletion changes the activity. No empirical rate coefficient changed.

An independent integrated second-order solution checks the new depletion
behavior. Conservation/capacity tests vary inventory scale and acid ratio;
time-partition comparisons check that cached pH does not become a renewed
reservoir on each wait. A registered conjugate-pair capability check declines
significant weak-acid/buffer reservoirs rather than pretending that the free
protons equal the titratable inventory. It is conservative formula/charge
matching, not an equilibrium-constant inference; isomer ambiguity can cause
over-withholding. Trace donors below the disclosed numerical/relative tolerance
do not trigger a spurious buffer refusal.

Seven existing English/German rate entries retain their stable IDs and input
scripts but no longer teach the old infinite-acid behavior or exact finite-time
yield ratios. The shared-clock lesson remains; initial rate comparisons are
distinguished from accumulated yields, and reagent-capacity checks precede
cloudiness threshold claims. The historical ID `the-acid-is-never-used-up` is
retained for links but now explicitly teaches the repaired finite-acid result.
The `cold-from-baking-soda` entry also separates dissolution-only arithmetic
from the complete open-vessel speciation/gas energy balance.

The first fresh catalog replay is `catalog-depletion-1`: **10/10 executions
complete, no solver-failure events, 30/30 independent checks pass**. Its source
snapshot records the initial revised prose before the five stale first-vessel
pH ranges were reviewed against the new acid-consuming results. The full
subsequent `kero codex lint` passes all **107 entries** with those revised
ranges (`catalog-depletion-1-lint.txt`). It still lists 234 prose-number warnings;
passing the enforceable catalog checks is not a claim that every prose number
is independently verified. The German locale lint passes, with 18/18 fields
translated in the new preserved-catalog file.

The reviewed export snapshot changes exactly two new entries and eight revised
entries (seven thiosulfate entries plus the cold bicarbonate entry); model and
concept exports are unchanged. No serialized solver output was promoted to a
runtime chemistry table.

## First next-batch result and additional generic repairs

Final fresh-binary replay: `next-batch-2` completes **36/36 executions** with
**121/121 independent checks passing**, zero unmet checks and no solver-failure
events. The native inventory check passes 62/62 comparisons across 198 rows,
without internal species aliases. `catalog-depletion-2` likewise completes all
10 preserved/corrected entries with **30/30 checks passing**. Both record the
workspace-rebuilt CLI SHA256
`99c22ca16ab90c637dc8e5348a48fbb6215196e7936282712cfe4be932566658`.
The original failed evidence below remains intact.

`next-batch-1` executed all 36 new probes without process or solver failures.
Its immutable `next-batch-1-checks.json` records **118 passing checks and three
unmet checks**, not an all-green result. Those three revealed HBr gas uptake
and silently omitted methanol/isopropanol in distillation. Their original
inputs, failed checks, raw output and start-of-run source snapshot remain.

The old still counted only water and ethanol; when another solvent accompanied
water, it transferred water alone without disclosing the missing solvent.
The repair routes all condensed volatile components through a single generic
plan before withdrawing anything. Missing competing-liquid properties now
refuse atomically. Supported additional solvents use independently sourced,
narrowly reviewed USCG CHRIS normal-boiling/latent data with a bounded
constant-latent Clausius–Clapeyron/Raoult Rayleigh calculation. The existing
water/ethanol UNIFAC route remains intact. No experiment identifier, target
yield or quarantined Antoine coefficient enters the production solver.

The new result reports every condensed component and its model limitation.
Ideal-liquid additional-solvent cuts do **not** predict real activity
coefficients or azeotropes; ±40 K is a stated local approximation boundary,
not an accuracy certification. See
[the field review](../../provenance/uscg-chris-still-review.md).
Generic tests cover pure limits, constant-relative-volatility Rayleigh law,
scale/permutation invariance, energy and species conservation, non-finite
inputs, and missing-property refusal. The centralized focused rerun
(`next-batch-fixes-tests-2.txt`) passes all six core distillation tests and all
65 thermodynamics tests. The full workspace gate and fresh CLI replay remain
separate checks.

A separate exact-zero proton-capacity edge in kinetics now clears stale
provisional free-proton/pH state instead of retaining pre-depletion acidity
or inventing pH 7. The equilibrium owner recomputes afterward. Its regression
also awaits the fresh-revision test run.

`distillation-provenance-lint.txt` passes all static provenance, checksum,
runtime-license and source-coverage checks. The cargo-based promotion fixture
was explicitly skipped because the parent owns the central build/test slot;
it remains part of the integrated verification gate. A follow-up read-only
review caught aqueous ammonia's volatility classification: registered Henry
volatiles now participate in the same competing-component guard even when
their standard phase is aqueous. Both acetone and ammonia are negative
controls for atomic missing-property refusal. The first integrated test run
identified `acetone` as a noncanonical fixture identifier (`UnknownSpecies`);
the fixture now uses its registered key `propanone`, retaining the same
chemical negative control and assertions. This was a test setup correction,
not a runtime model change; the original failure is preserved in the parent's
`next-batch-fixes-tests-1.txt`.

While the integrated core build was contended, `standalone_batch.rs` imported
the production numerical source directly (only the two-variant request enum
is mirrored). Its first compile caught an ambiguous floating-literal type in
a test, preserved in `standalone-batch-1-compile.txt`. After the explicit
`f64` suffix, **both embedded production-kernel tests pass** in
`standalone-batch-2.txt`: 20 scale/fraction combinations and their permutations,
pure boiling and energy limits, and three independent Rayleigh identities.
This confirms the numerical kernel, not yet the core/CLI integration.

Independent review found no clear numerical/unit/conservation defect, and
refined the physical disclosure: stage compositions use a total-reflux
enrichment approximation, not a literal operating column (which would have
no net product at total reflux). The energy number counts withdrawn
condensate latent heat only, **not** reflux/reboiler circulation duty or
sensible heating. The structured result and its rendering carry this limit.
