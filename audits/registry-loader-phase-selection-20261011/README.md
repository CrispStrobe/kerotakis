# Runtime registry phase selection

On [PR779 source](https://github.com/CrispStrobe/kerotakis/tree/2e03067196bd7d8da7516592b1df0202029fb2be), direct loading and generated construction select the first species/property record, independent of phase. The current registry has one mass, density and fallback heat-capacity record per species, with matching phases throughout.

[Fourteen frozen controls](freeze.json) record [4 pass / 10 fail](local-first-baseline-review.json). Preserve a probe limitation: foreign/wrong-phase records changed record phase while retaining the original condition phase. Those cases alone do not establish correct handling of internally consistent alternate-phase metadata. Same-phase conflicting records use unique IDs and expose silent ambiguity independently.

[Six separately frozen controls](supplemental-freeze.json) update both phase fields together and use unique IDs. Their [actual baseline](supplemental-baseline-review.json) records **2 pass / 4 fail**: prepending foreign-phase Cp/density changes runtime values, while sole wrong-phase Cp/density records are silently admitted. Appending the foreign records preserves canonical values, showing order dependence. Synthetic doubled values are adversarial test inputs, not new chemical measurements.

Next implement one shared selection policy for runtime fallback construction and generated species. Require a unique molar-mass anchor; select Cp/density for its standard phase, refusing missing phase coverage or ambiguous competing records. Keep legitimate multi-phase typed documents, property resolution, optional properties and phase-dependent curves unchanged. Do not add a whole-schema prohibition on multiple records.

Require unchanged original14 and consistent6 controls, prior scalar/unit controls, complete affected libraries and registry suites, actual generated alternate-phase/ambiguity controls, format/lint and fresh source-bound hosted validation. Use a separate source branch from PR779. Hosted queues and prior repair acceptance do not validate this new candidate. No repair is accepted yet.
