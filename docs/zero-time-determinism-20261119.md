# aq-023 zero-time "non-determinism": root-cause analysis (2026-11-19)

Open item (handoff 6(d)): `order_invariance.rs`
(`the_same_beaker_reads_the_same_solvent_mass_in_either_order`) runs aq-023
(water 100 mL, CaCl2 0.01 mol, laundry_detergent 5 g), then `wait 0s`, and
compares the native Pitzer auxiliary diagnostics. The tolerance was widened
to 1e-4 relative because the value before the repeat was believed to differ
run to run.

## Finding: the addition is reproducible; the values moved with the source

All evidence is the hosted Kaggle CPU logs
`kero-kaggle-engines-20261027/out-{ct,cu,cv,cw,cx}/kerotakis-evidence/final-workspace.stdout`
(source SHAs from each run's `final-source.json`). Every run executed the
workspace tests twice, so each run gives two independent processes.

| run | source | failing coordinate | before -> after (both processes) |
|-----|--------|--------------------|----------------------------------|
| ct | 6950d5c8e | particle molality | 0.5282154183896 -> 0.5282154084043 (x2, identical) |
| cu | ba052e425 | particle molality | 0.5282154183896 -> 0.5282154084043 (x2, identical) |
| cv | ee60b04de | water activity    | 0.9915672895706189 -> 0.9915675537941044 (x2, identical) |
| cw | 680b1ca0f | water activity    | 0.9915681515703431 -> 0.9915675537744669 (x2, identical) |
| cx | 307ee0853 | particle molality | 0.5281771070355 -> 0.5282154092781 (x2, identical) |

(cv/cw fail on water activity first only because ee60b04de tightened the
tolerance to 1e-7 and water activity is checked first.)

1. Within every run the two processes agree to the last printed digit. A
   per-process `RandomState` hash order, thread scheduling or a cross-test
   cache would have to give identical answers in two processes and change
   only between commits; that is not what such defects look like.
2. Between ct/cu/cv and cw/cx the only non-test source change is
   `680b1ca0f` ("Zero time covers every addition, not only `add`"),
   `crates/kerotakis-core/src/bench.rs`: `AddMaterial`, `AddSolution` and
   `Dilute` became zero-time doses. In salt-first aq-023 the LAST step is
   the laundry detergent, an `AddMaterial`. Before 680b1ca0f it relaxed to
   room air (pre-repeat 0.52821542); after, it is a zero-time dose
   (pre-repeat 0.52817711). The commit message of 680b1ca0f itself already
   calls the cv difference "run to run", which is where the misreading
   started.
3. `audits/carbonate-focused-source-20261112/AQ023-EIGHT-PROCESS.json`: eight
   serial CLI processes of the same binary give bit-identical aq-023 values,
   including `aux_particle_molality` 0.5281771070355, the same value cx saw
   before the repeat.
4. Static check of the candidates named in the brief:
   - `crates/kerotakis-phreeqc/src`: the only `HashMap`s are the
     content-addressed caches in `aqueous.rs:955,959,995` (lookup/insert/
     clear only, never iterated; per-`PhreeqcEquilibrator`, not
     process-global). `partition` sorts `totals` and `phases` before
     writing input (`aqueous.rs` ~3717).
   - `crates/kerotakis-core/src`: `orchestrator.rs:53,65` iterates a
     `HashMap` in `diff_vessels`, so `StateDelta::mole_changes` order is
     hash-random, but every `diff_vessels` delta carries a snapshot and
     `StateDelta::apply` (`delta.rs:1143`) clones the candidate vessel,
     ignoring the change order. `solve.rs:1608`, `bench.rs:6207`,
     `clock.rs:1644` use `HashSet` only for membership.
   - Process-global state (`species.rs` loaded index, `kinetics.rs:1937`
     donor cache, `material.rs:65` recipes) is deterministic given the data
     and is not written by the two tests in this binary.
   - No wall-clock budgets (`Instant`, `elapsed`) in core, stack or phreeqc
     sources.

Conclusion: there is no evidence of non-determinism. The 7.3e-5 is a
deterministic non-idempotence: the zero-time addition and the zero-time
repeat on the same contents settle at different auxiliary values. The
repeat value is stable across heads (0.52821541), and matches what the
addition gave when it still relaxed to room air.

## The real open defect: addition and repeat are different fixed points

Not fixed here: the mechanism could not be pinned without running the
engine, and the rule was not to change behaviour without confidence.
Ranked candidates:

1. **The auxiliary pose follows the path while the primary is
   canonicalised.** `recharacterise_canonically` (`aqueous.rs:7846`)
   re-poses the settled contents and rewrites `solvent_kg`,
   `particle_molality`, `ph`, `ionic_strength` and `species`, but never
   `info.solvent_activity`. That one stays from the last `solve_once`, posed
   from "previous contents + reagent" (`aqueous.rs:8272-8296`). The repeat
   poses the settled contents, so its auxiliary is the canonical one. This
   fits "before is path-dependent, after is stable". Caveat: aq-023 carries
   an opaque material on an open beaker, so it probably takes the
   `ReservoirWithOpaque` deferred route (`aqueous.rs:4408-4490`,
   `reservoir_opaque_component_owner` at 11002). There
   `physical_canonical_closed` may be true and the re-pose is skipped,
   which would put the path dependence in the deferred strict-closure pose
   (`aqueous.rs:9330-9350`) instead. One `KERO_TRACE_SOLVENT_ACTIVITY=1` /
   `KERO_DUMP_INPUT` run of both steps settles which.
2. **The zero-time gas boundary.** With `zero_time_dose`, every
   atmospheric reservoir targets the effervescence pressure
   (`aqueous.rs:3652`), not room air. The addition starts from a CaCl2
   solution that was room-air-equilibrated, plus solid Na2CO3; the repeat
   starts from a supersaturated carbonate state (see the test comment). If
   the readback keeps dissolved CO2/O2 differently from how it was posed,
   the second pose holds a slightly different neutral-gas inventory.
   (Element ledgers agree to 1e-10, so this would be a speciation split,
   for example CO2(aq) against carbonate, not lost carbon.)
3. **PHREEQC convergence from different starting estimates** (the
   1.6e-10 / 6e-7 scale named in the test). This is too small to explain
   7.3e-5 alone.

Upstream since then: `bench.rs:1028` makes every operation without a
positive duration zero-time, and `carbonate_endpoint::prepare`
(`carbonate_endpoint.rs:71`) runs only for additions with an acid or
carbonate dose against prior acid. For salt-first aq-023 the CaCl2
solution has no literal acid and pH above pK1, so the endpoint should
return `None`. The addition and the repeat therefore both stay zero-time
and endpoint-free, and the picture above is unchanged. The latest hosted
runs (cy 797b1586c, cz 8fc97c87c) pass at the 1e-4 budget; their actual
deltas are not printed.

## What changed in this branch

- `order_invariance.rs`: the stopgap comment now states the evidence
  rather than "run to run". The tolerance stays at 1e-4 relative, because
  the non-idempotence above is real and unfixed.
- New test `repeated_fresh_benches_read_bit_identically`: three fresh
  salt-first aq-023 builds in one process must give identical bits for pH,
  I, solvent kg and the three auxiliary diagnostics. It guards the
  determinism conclusion; if it ever fails, this analysis is wrong.
