# Native carbonate contract correction — 2026-11-12

This capsule preserves the complete original contracts, first failures and three
once-only readbacks of the actual `e862b601fcdca68e4504656f8f6fbcc2a60bd424`
producer. Source, tree, release-binary and genuine producer-lock SHA-256 bindings
are in [binding.json](binding.json) and [FROZEN.json](FROZEN.json). The original
contracts were captured from `b2598f4411c47969cd7250f2bd4eab9ada403006` before
these successor edits. No original qualification is added. Changed-source Rust
acceptance remains pending.

## Preserve the failures

[Complete original test files](original-contracts/) retain all original assertions;
[original hashes](original-contracts.json) authenticate their exact bytes.
[First failure excerpts](first-failure-excerpts.txt) retain the actual producer's
failed curated-event assertions, gas-only contract, shell protocol and lesson
snapshot. The gas and shell assertions are also examined beyond the first panic:

- The soda protocol emitted `0.048060922758632024 mol` CO2. It fails the original
  gas-only `|gas - 0.05| < 0.001 mol` claim. Another
  `0.0019390772413679733 mol` inorganic carbon remains in the vessel. Stoichiometry
  constrains their sum; it does not require all carbon to cross the boundary.
- The concentrated calcite protocol retains `0.00119221555638 mol` solid. It does
  not satisfy the original `< 0.001 mol` residue claim. Its current typed
  `routing.crystallisation-outside-activity-domain` warning makes that mineral
  prediction **held**, rather than a scientific pass. The successor keeps the
  complete original dose and requires its actual native answer, all-element and
  charge closure, actual gas crossing, and this explicit typed hold. A separate
  dilute positive control requires no activity-domain hold and retains the
  `< 0.001 mol` residue bound alongside the existing stricter `< 1e-6 mol` bound.
- The acid-first volcano's recorded final temperature, `292.0565445176562 K`,
  already satisfies the original cooling-sign check. That check stays unchanged;
  the successor replaces the obsolete curated-event requirement with fresh
  native characterization, material closure and actual CO2 crossing.

All three saved protocols carry the activity-domain warning and the explicit
`not-modeled.carbonate-fast-endpoint` limit. Their concentrated empirical yields
remain held. Native owner selection, complete numerical characterization and
conserved material are software/model properties, not validation of an unsupported
activity domain or calibrated instantaneous gas escape.

## Independent saved arithmetic

[Raw scripts and JSONL readbacks](FROZEN.json) are retained without rewriting the
original failures. The soda/calcite CLI scripts pose the same reagent quantities
as the failing integration tests through the production CLI stack. The lesson
script is unchanged. These are three additional diagnostic processes against the
saved producer, not executions of the edited candidate tests.

[review_saved.py](review_saved.py) independently counts explicit chemical formula
atoms and formal charge on each actual addition. Unknown species fail closed;
analytical `base_equivalents` contributes one H, one O and charge −1 and is not
misread as free hydroxide. Actual gas crossings enter the boundary ledger once.
The lesson's sealed-air inlet is outside this addition-only reader; its subsequent
soda addition is checked against the actual sealed stock. The original numerical
material tolerance remains `1e-10 mol`.

[The saved arithmetic](saved-review.json) closes every audited element and formal
charge with maximum residuals of `1.78e-15 mol` (soda), `5.40e-13 mol` (calcite)
and `8.89e-16 mol` (lesson). Calcite's printed free-species activities give
`log10(aCa × aCO3) = -8.4872281`; its exact MINTEQ phase definition
`logK(298.15 K) = -8.48`, `ΔH = -8 kJ/mol` gives `-8.4873088` at the reported
`299.71295 K`. This recorded consistency observation uses rounded activities;
it does not validate the warned concentrated model or the original residue bound.
The database gitlink is `47b459c4a5e3e20b9990fa304e207eca0dcc6ff1`.

The lesson reports `53.45688617319545 g` and announces
`0.041880370345830614 mol` CO2. With the registry's exact declared CO2 molar mass
`44.009 g/mol`, the input-minus-actual-gas residual is `−6.08255e-7 g`, inside the
unchanged `1e-4 g` limit. The open `53.47 g` snapshot is therefore removed without
replacing it with another output fixture. The existing calculated balance check
and the sealed snapshot remain. The new open print is not blessed as empirical
calibration.

## Successor mechanism checks and validation

Both molecular carbonate rows now have typed native overlap ownership. Their
native successors must not replay a curated equation. The synthetic untagged-row
control uses the genuinely untagged bleach/HCl row and retains its curated-event
requirement; a separate molecular-carbonate mechanism control requires preserved
stock under the selected synthetic complete owner. Neither synthetic answer is
chemical evidence. Calcite reachability now enters the same strict native branch
as the already delegated bicarbonate rows.

The saved reader and its corruption controls run without a chemistry engine:

```console
python3 audits/carbonate-native-contract-correction-20261112/review_saved.py
python3 audits/carbonate-native-contract-correction-20261112/test_saved_review.py
```

Three test methods include 21 gas/material/scope/typed-limit corruptions and a
wrong-balance snapshot; every corruption is rejected. Rustfmt and whitespace
checks pass. The edited Rust tests still require their own hosted execution;
no compilation, workflow, protected check or physical tolerance is waived.
