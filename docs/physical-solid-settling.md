# Settling follows current solid inventory

The rank-14 complete-stock follow-up exposed a provenance/physics mismatch.
At source `efd79bf88eb1514bb87506171c39e808c3b544e1`, the CLI dissolved
0.001 mol CuSO4 in 100 mL water, then displaced copper with 0.001 mol zinc.
Before a ten-second wait, current contents contained no solid CuSO4. The wait
nevertheless reported complete gravity settling of CuSO4 because its historical
stock lot still described a suspended solid dose. The original protocol,
forecast and failed event remain preserved; this repair does not rewrite them.
The execution's binary SHA-256 was
`75cb9b3ca1a38c05dcde7faad1efdd4cb69ec89f7df29f4d91468baa3d0b00fa`;
its producer-reported lock SHA-256 was
`82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229`.
The raw lock bytes were not included in that hosted artifact.
The [frozen protocol, assessment and raw streams](https://github.com/CrispStrobe/kerotakis/tree/7d034a1d84ae9694a8dc8a03a2edeccacae3c822/audits/reaction-dataset-200-20261015/case014-repair-20261016)
retain the first failure separately from subsequent repair validation.

`crates/kerotakis-core/src/clock.rs` now requires a finite positive current
`Phase::Solid` portion of the same species before historical lot metadata can
support gravity settling. The lot itself must also carry a finite positive
original amount. No observation threshold is used to admit material. Historical
lots, their original dose amounts and provenance remain intact; only genuinely
admitted particles have their suspension metadata updated. Settling does not
change the material inventory.

Centrifugation already enumerates current contents rather than lots and thus
does not have the demonstrated stale-only admission bug. Its solid admission
now shares the finite-positive physical-portion predicate. Existing Stokes
parameters, particle-size assumptions and separation calculations are unchanged.

Core controls distinguish dissolved, reacted and transferred historical lots
from residual solids; check multiple positive lots, tiny positive inventories,
empty/invalid lots and portions, inventory preservation and centrifugation.
Native controls use actual PHREEQC dissolution and zinc displacement before
waiting, alongside actual residual AgCl as a positive particulate control.
These require hosted execution: source formatting and diff checks alone do not
establish runtime acceptance or chemical calibration.

This repair does not reconstruct how remaining same-species material should be
allocated among several historical grain-size lots. Nor does it create missing
grain metadata for a new precipitate produced without a tracked particulate
lot. Those remain separate representation/modeling lanes; absence of a settling
event for an untracked precipitate is not evidence that it cannot settle.
