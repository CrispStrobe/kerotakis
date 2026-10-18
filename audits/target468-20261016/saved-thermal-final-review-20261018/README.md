# Ten final saved thermal selections

Original TH001–006 and TH011–014 meet all20 original criteria in their stated
bounded phase/property/dissolution domains. [Actual case reviews](assessment.json)
retain the original forecasts, scripts, source `c4469205`, raw outputs, warnings
and producer-qualified binary/lock metadata. No new simulation or API request
was made; these are historical-source selections, not current-production reruns.

The independent thermal-preparation reader reproduces all16 timelines, eight
heat calculations and the copper/water mixing balance from pinned NASA9/Shomate
Cp curves, explicit endpoint clamping and latent properties. Selected pure-water,
ethanol, ice, naphthalene and pure-acid balances close to numerical resolution;
the copper/water balance residual is−1.72e-7J. These calculations establish
source-property arithmetic, not experimental property uncertainty or empirical
calibration. No general solution/reaction/open-mixture energy closure is claimed.

The comparisons use actual solid and liquid ownership, actual temperatures,
dissolved ion inventories and each salt arm's own initial water temperature.
Boiling comparisons require actual vapour export matched to retained-material
loss; salt stays owned while the brine concentrates. The boiling-is-not-settled
warning explicitly prevents treating a changing mixture as a settled pH reading.

TH012 selects owned pure-acid coexistence at its declared289.75K and the
owned-fraction energy balance. Its repeated historical messages include four
freezes and one compensating melt: independent signed accounting closes their
net to the owned solid gain within6e-17mol. Repetition alone does not establish
a current runtime defect. The initial unreconciled assessment, catalog draft
and reader are preserved alongside the corrected signed review; no new repair
is inferred from that initial concern. The generic dissolution warning does
not imply missing pure-acid transition data.

The first review reader incorrectly required exact decimal equality for a native
floating-point reserialization of0.005mol salt as0.005000000000000001mol. It
stopped before assessment/catalog creation. Its source and failure are preserved;
the correction uses relative1e-12 numerical dose comparison, without changing
raw states, physical criteria or existing material-audit bounds.

[The complete233-entry catalog](../../working-experiments-20261018-reviewed-thermal.json)
preserves all previous223 entries exactly and qualifies229 unique fixed468 IDs;
172 more are needed for401. Four out-of-target entries remain separate. Fourteen
first-screen candidates still need final review;20 original holds remain
historical. TH007–010/015 and the original TH016 failure remain excluded by this
reader; TH016 already has its separate repaired-source selection.

Reproduce `review_saved.py`, `verify_catalog.py` and the independent thermal and
material preparation readers using the original immutable archive. Old binary
and Cargo.lock bytes were absent from that archive and are not independently
rehashed by these saved-output reviews.
