# Neutral-water pH note: focused acceptance

Source `b703b695`, tree `13a8af79`, actual pinned lock `66886813` passes all four
measurement controls, composed-locale tests, engine-locale lint and all-target
core Clippy. The [independent acceptance receipt](acceptance.json) authenticates
actual provider version2, source/worker/lock and all53 collected output files.
The complete small producer evidence is published here; provider collection log
metadata remains retained separately.

On tests-only baseline `7595b3e1`, the actual German and French measurement tests
fail and the unchanged-English/domain controls pass: two real failures and two
positives. [Baseline stdout](baseline.stdout) and [stderr](baseline.stderr) preserve
those assertions. On the repair, [all four pass](fixed.stdout): plain water and
glucose at the reference temperature carry localized LV2/LV3 caveats; English
legacy note/pH7 remain exact; warm water, uncharacterized acid and a dry vessel
still produce no fallback pH reading. Measurements preserve owned inventory.

The [first setup-only stop](../blank-note-first-setup-stop-20261019/README.md)
remains separate. Initializing pinned submodules corrected dependency resolution;
no runtime or test assertion was weakened to obtain these results.

This is focused software acceptance. The separate family-gate successor,
combined-source/full-product/platform acceptance, integration and actual changed
API/browser delivery remain pending. It adds no scientific selection or expanded
chemical-domain/calibration claim.
