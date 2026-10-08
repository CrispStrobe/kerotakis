# Explicit safety-veto atomicity forecasts

These source-informed forecasts are declared before implementation and hosted execution. They test the meaning of an explicit safety refusal: the attempted operation may add its diagnostic log entry, but may not apply material/temperature changes, erase a prior characterization, create a receiver, or invoke a chemistry solver. A deterministic screen and deliberately mutating counting solver expose hidden settlement.

| Case | Fixed operation input | Expected refusal behavior | Accepted control |
| --- | --- | --- | --- |
| Add | Add 0.5 mol water to existing vessel | SafetyVeto only; entire bench except log unchanged; zero solver calls | Added event and solver settlement |
| AddMaterial | Add 1 recipe-basis unit whole milk with seed 41 | Same explicit refusal contract | MaterialAdded event and settlement |
| Decant | Transfer half of 4 mol liquid water to existing receiver | Same explicit refusal contract | Transferred event and settlement |
| Mix | Mix fractions 0.25 and 0.5 of two 4 mol liquid-water sources into existing receiver | Same explicit refusal contract | Mixed event and settlement |
| Filter | Filter 4 mol liquid water into existing receiver | Same explicit refusal contract | Filtered event and settlement |
| Discard | Discard the existing water-filled vessel into waste | Same explicit refusal contract; no waste transfer | Discarded event and settlement |
| RecoverSpill | Recover half of a 2 mol water spill into an existing receiver | Same explicit refusal contract; spill unchanged | SpillRecovered event and settlement |

Every fixture includes prior solution characterization and a nonempty spill ledger. Seven refusal and seven acceptance controls are separate tests. Two additional veto controls require fresh Decant/Filter receivers to remain absent. Two fresh-receiver warning controls require successful creation, correct transferred amounts and the existing VesselCreated → HazardWarning → success order. A missing Mix receiver retains its current NoSuchVessel error with no screening, mutation or solver invocation; this lane does not add automatic Mix receiver creation.

The twentieth control preserves the deliberate exception: an accidental Spill of half the source is physical evidence and proceeds despite a veto verdict, reporting SpillHazard and SpillCreated, moving exactly two moles, and settling the source. It must not become an atomic SafetyVeto.

This bounded lane covers seven explicit veto paths and two receiver-creation branches. General invalid-input rollback, unrelated NothingToActOn/unsupported-model diagnostics, and whole solver-stack transaction behavior remain separate work. Runtime baseline and repair validation are pending; no executed outcome is inferred from these forecasts.
