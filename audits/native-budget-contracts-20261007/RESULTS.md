# Native budget baseline

[Hosted run 37704022496](https://github.com/CrispStrobe/kerotakis/actions/runs/37704022496)
compiled and executed 25 contracts at `3746d277`: **11 passed, 14 failed**.
The seven original raw component guards passed. Of 18 new allocation and
reconciliation controls, four passed and fourteen failed.
[The source-bound baseline receipt](baseline.json) lists every outcome and
binds the raw artifact/log hash.

Demonstrated failures include Ca/Sr and closed-carbon overdraw, malformed
owned gas and raw aqueous values, pure-phase co-owner omission, arbitrary
rescaling and synthesized missing aqueous inventory. Missing owned gas already
refuses; it must keep doing so.

Eight additional contracts are frozen before production edits:
[receipt](supplement-freeze.json). They cover an independently chosen numerical
cap boundary, aggregate overflow, malformed initial owners and open carbon.
The cap separates native residual (1e-7 relative), selected-output rounding
(5e-12 relative for twelve significant decimal places) and positive-sum
arithmetic gamma(n). It is an adapter acceptance ceiling, not empirical
validation of PHREEQC accuracy. Comparisons have no absolute inventory floor.
The raw cap scales to the aqueous/remainder comparison; the phase cap scales
to available/final owned totals. Zero raw aqueous with a positive remainder
refuses even when the remainder is small.

Supplemental baseline and repair validation are pending. Live native
precipitation/dissolution/repeated equilibrium and native/stack rollback must
also pass before claiming this bounded repair accepted.
