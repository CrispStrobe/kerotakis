# QA-011: actual fractional endpoint doses

The unchanged original protocol meets both of its original conditional checks on
c446/670. The engine refines the last burette increment and **physically dispenses
that fraction**. The coarse1mL arm delivers10.350003839mL and the fine0.1mL arm
10.350003910mL, versus the ideal10.35mL reference. Eleven and104 dose steps include
the respective fractional final doses; they are not eleven or104 whole drops.

The [saved reader](review_saved.py) authenticates the raw artifact, original script
and first stdout, then checks endpoint events against retained sodium, titrant
carrier water and H/O/Na/Cl/charge. Maximum ledger residual is9.60e-14mol, below the
existing1e-8 audit resolution. No solver failures, coverage limits or gas exchanges
occur. [Case arithmetic](assessment.json) preserves both original criteria.
The actual-source titration implementation clones each pre-increment inventory,
then commits the accepted fractional dose; no iterative trial dose is accumulated.

This source-qualified substantive expectation agreement adds one unchanged CLI
protocol to the [working catalog](../../../working-experiments-20261016.json),
raising its count from149 to150 while preserving every previous entry. It does
not increase the repaired-original200 count. The [selection receipt](catalog-selection.json)
records that distinction. No experiment ran again, and this is no empirical
burette, uncertainty, rate or heat calibration.

The [independent saved-output review](independent-review.json) reproduces the
physical endpoint interpretation and finds no selection blocker. Root separately
verified the selection hash and that the preceding149 catalog entries remain
unchanged. An inaccurate reviewer workflow claim has a separate correction;
neither review runs the chemistry engine or expands calibration claims.
