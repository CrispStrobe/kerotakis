# Liquid-transfer ownership follow-up — preparation only

Source inspection binds `d67205091a22bcdb2fc490866224c4bc7fef8a4e`;
[preparation.json](preparation.json) authenticates the inspected files and
[proposed regression fixture](partition_ownership.rs). No command in this fixture
has been executed. The two anticipated failures and positive result are forecasts,
not observations. No runtime change or scientific selection is claimed.

Finish PR841's existing exact-source validation/integration/changed delivery first.
Do not modify its active candidate for this follow-up or reissue its hosted job.
After integration, refresh actual main and start a narrow ownership branch.

## Demonstrated source seams

In `crates/kerotakis-core/src/bench.rs`, `Operator::Drain` treats
`try_partition_k` returning `Ok(None)` as permission to move the complete dissolved
portion with water. The helper returns that value for absent molecular
parameters, including neutral ethyl acetate. Missing neutral parameters do not
establish complete retention in water. Explicit dissolved ions are a different
case and need their existing supported water-transfer behavior preserved.

`Operator::Extract` inventories only `Phase::Aqueous` and `Phase::Solid` portions;
`Phase::Liquid` ethanol is skipped despite having a reviewed decomposition. Drain
already considers that liquid representation. The same owned supported solute
should not disappear from extraction eligibility because of its storage tag.

Do not classify every charge-neutral formula as a molecular neutral: complete
salt reagents and analytical acid/base coordinates also have neutral formulas.
Use actual species/phase ownership, dissociation capability and admitted data.
Unknown identities must not silently gain a distribution coefficient. Preserve
free versus complex-bound ligand and molecular versus analytical-carrier limits.

## Bounded implementation and acceptance

1. Copy the frozen fixture unchanged to
   `crates/kerotakis-core/tests/partition_ownership.rs` on refreshed main. Verify its
   SHA against the preparation. Issue one resource-appropriate baseline run with
   `cargo test -p kerotakis-core --test partition_ownership`. Record actual compiler,
   assertion and positive outcomes separately; a compiler failure proves neither
   forecast. Preserve raw stdout/stderr, source, actual lock and binary bindings.
2. Replace the ambiguous missing-coefficient transfer decision with explicit
   admitted transfer versus missing-neutral/unknown ownership. For a drain whose
   neutral placement cannot be computed, refuse atomically before creating the
   receiver, consuming stock or changing either inventory. Do not fabricate
   solvent retention, ideal coefficients or empirical parameters. Keep explicit
   dissolved-ion transfer and numerical refusals distinct.
3. Include supported non-solvent liquid portions in extraction's molecular pool.
   Review mixed liquid/aqueous/solid lots before mutation: `Vessel::withdraw`
   consumes species inventory without choosing its phase. Withdraw only the
   admitted owned portions; do not turn an unrelated crystalline reservoir into
   dissolved material or count one species twice. Preserve supported partial
   extraction and explicit unsupported-material notes.
4. Add mixed-phase/solid-reservoir controls, unknown identity and complete-salt
   controls after their ownership contract is specified. Include destination and
   stock rollback, material/charge closure and supported-solute partial outcomes.
   Existing numerical boundary, empirical temperature/solubility, layer and
   chromatography assertions remain unchanged.
5. Emit a typed missing-data/ownership reason, with English/German/French owned
   text and localized slots. Do not use English matching to select behavior.
   Keep numerical-range errors separate from absent chemical coverage.
6. Require actual baseline evidence, focused repaired controls, applicable core,
   thermo/native/external checks, unchanged500 coverage and full own fleet, both
   native-platform logs and protected CI. Use normal exact-head auto-merge, then
   authenticate changed API/browser behavior against actual served artifacts.
   Freeze any additional original scientific replay separately; these software
   controls alone earn no corpus selection.

This closes an ownership decision, not the missing ester parameter lane. MO011
still needs compatible attributed data and unchanged original-criterion review.
