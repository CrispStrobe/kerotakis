# Saved quantitative checks: QA-001–003

This assessment reuses the authenticated c446/670 native output. No protocol or
forecast changed and no experiment ran again. The [reader](review_saved.py)
rehashes the raw artifact, exact original scripts and first stdout before
checking the original criteria. [Results](assessment.json) remain separate from
the immutable initial screen classifications.

- QA-001 recovers 0.999999733 mmol CaCO3 from the blank/sample endpoint difference.
  Retained calcium and exported CO2 each account for approximately1 mmol; the
  original silica amount remains separately accounted across its phases. Both
  original bounded checks are met.
- QA-002 recovers a0.500046387 mmol acid spike. Each aliquot retains1 mmol total
  acetate. Both original bounded checks are met.
- QA-003 gives a3.000000010:1 base-demand ratio, meeting its first check. Its two
  `measure … volume` commands select the gas/headspace instrument and refuse open
  vessels. Its second check requires measured liquid volumes and remains
  unsupported; a dose ratio or inferred volume cannot substitute for that
  missing measurement. Sealing would measure trapped gas, not fix this protocol.

Dose checks use two original burette increments; inventory comparisons use the
existing1e-8 mol audit resolution. These checks establish neither empirical
calibration nor general atom/energy closure. The working catalog is unchanged.
The next QA-003 lane should identify an actual liquid-volume observable and
freeze a separate observation-only protocol correction, preserving its original
missing-measurement result and dose forecast. Do not relax gas-meter semantics.

A separate [per-step material supplement](material-assessment.json), reproduced
by [this saved reader](review_material.py) and independently reviewed, closes
represented H/O/C/Na/Cl/Ca/Si/charge for QA001/002 including titrant carrier
water and explicit CO2 exports. [Selection](material-catalog-selection.json)
adds their two unchanged original scripts to the working catalog (150→152),
preserving the previous150 entries and all first assessments. QA003 remains
separate; no chemistry is rerun or calibration claimed.
