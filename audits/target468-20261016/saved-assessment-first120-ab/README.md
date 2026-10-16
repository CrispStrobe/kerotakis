# Saved assessment: first120 acid-base cases AB-001–024

This is a separate, retrospective assessment of the original 24 acid-base
forecasts, using the first authenticated native output from
[run38909751180](https://github.com/CrispStrobe/kerotakis/actions/runs/38909751180).
No experiment, API request or engine execution was repeated. The original
[dataset](../common-engine-completion/originals/new-experiments-20261016.json),
scripts, criteria and first outputs remain unchanged. A zero process exit is
only an execution result.

The [reader](review_saved.py) authenticates original JSON SHA
`8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd`, raw artifact SHA
`9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f`, all24 exact
original scripts, stdout/stderr hashes and per-case receipts. It binds the saved
workflow/input revisions to engine `c4469205`, tree `f1721665`, binary `670bd3d8`.
The producer lock hash is reported; this artifact supplies no original Cargo.lock
bytes. These are saved-binding checks, not a fresh engine or GitHub execution.

Run from any repository checkout containing the saved artifact:

```console
python audits/target468-20261016/saved-assessment-first120-ab/review_saved.py
```

[Assessment](assessment.json) preserves each original criterion and forecast,
reports concrete measured pH/conductivity, relevant species amounts and boundaries,
and separates criterion status from proposed catalogue selection. Numerical sign
and ordering claims are asserted or directly evaluated. Original words such as
“similar”, “near” and activity-qualified “approximately” have no frozen numeric
pass band: their recorded conclusions are explicit manual semantic judgments,
with the actual values supplied for independent review. No new physical pass
band was chosen to make them succeed.

## Proposed selections and holds

Twenty-two cases meet both original bounded criteria on this saved evidence.
Twenty-one are proposed for bounded selection after excluding the ferric case's
explicit phase-coverage limits. These are proposals for independent review,
**not catalogue additions**, empirical calibration or unrestricted model claims.
The exact proposed IDs are **AB-001, AB-003–016, AB-019–024**.

| Case | Saved bounded result | Disposition |
| --- | --- | --- |
| AB-001 | Conductivity4058.46→1204.28→2359.46µS/cm; pH2.038→6.993→11.635. | Propose |
| AB-002 | Equivalence conductivity475.06 exceeds initial145.20µS/cm. Final0.5mmol response1162.26µS/cm exceeds preceding0.5mmol response226.86. | Hold C2: preceding equal dose ends at equivalence; early doses are0.25mmol, not a single equal-size dose well before equivalence. |
| AB-003 | pH2.266→2.618→4.705→6.982→9.313; phosphorus stays1mmol. | Propose |
| AB-004 | One/two equivalents pH2.197/7.074; sulfate1mmol retained. | Propose |
| AB-005 | Bicarbonate/carbonate pH9.109/10.973; bicarbonate exports0.077277mmol CO2. Retained+exported carbon closes separately in both vessels. | Propose; open-boundary comparison, sodium differs1/2mmol. |
| AB-006 | Acid pulse shifts ammonium buffer pH9.350→9.163; both NH3/NH4+ remain, reagent N2mmol includes NH3 gas. | Propose |
| AB-007 | NH3-only/mixed pH10.539/9.279; reagent N1/2mmol includes gas. | Propose |
| AB-008 | Buffer/water own-baseline pH shifts+0.09194/+3.98754; NH4+ remains. | Propose |
| AB-009 | pH4.712→4.883; acid falls, acetate rises, family2mmol retained. | Propose |
| AB-010 | Reversed buffer ratio changes pH by1.83332 versus ideal log10(81)=1.90849. | Propose as the original approximate, activity-qualified comparison. |
| AB-011 | Initial pH difference0.04934; dilute/concentrated absolute shifts0.45481/0.04337. | Propose |
| AB-012 | Postdilution pH difference0.03052; concentrated/dilute decreases0.08635/0.43743 under the stated approximate normalised challenge. | Propose |
| AB-013 | Adding1nmol HCl lowers pH6.99738→6.97802 at the actual open-water temperature. | Propose; no atmospheric or empirical calibration. |
| AB-014 | Initial pH4.00366/4.00375; after equal base challenge9.99362/4.48652. | Propose |
| AB-015 | Ionised fraction rises on dilution; pH rises0.51975 and family remains1mmol. | Propose |
| AB-016 | Acetate/salt-control pH8.331/6.997; acid and measured OH appear without a NaOH bottle species. | Propose |
| AB-017 | Mixed pH6.875; NH3 gas and aqueous pools accounted. Same-vessel prior NH4Cl-only state is available. | Hold C1: no NaOAc-only control with the same sealed boundary. AB-016 is open. |
| AB-018 | Ferric/calcium pH3.012/6.996; Fe(OH)3 forms without external hydroxide. | Both bounded criteria met; hold unrestricted selection because Goethite/other iron phases and saturation indices have explicit coverage limits. |
| AB-019 | Both phosphate buffer species remain; buffer/water decreases0.17390/4.27954. | Propose |
| AB-020 | HCO3-/CO3 ratio increases; carbon2mmol plus trapped-air carbon retained across aqueous/gas phases. | Propose |
| AB-021 | Pure water25/60°C gives pH6.99738/6.50764 with equal measured H/OH inventories in each state. | Propose as model temperature dependence, not empirical pKw calibration. |
| AB-022 | Mixed pH2.32539 vs initial2.03811/4.00366; H+/Cl-0.505mmol each, donors empty. | Propose |
| AB-023 | NaOH/Ca(OH)2 comparison pH2.04971/6.96936; calcium dose dissolved, Ca1mmol. | Propose |
| AB-024 | Mg/Ca residual H+0.570645/2.001739mmol; explicit CO2 only from carbonate0.997819mmol. | Propose; carbonate retained carbon also counted. |

The three holds are procedure/coverage follow-ups, not evidence of missing mass
or an established new numerical defect. A later AB-002 matched-dose control or
AB-017 same-boundary comparator needs its own frozen design before execution.
AB-018 needs explicit phase-domain review; its matching acidity comparison does
not close excluded-phase formation or kinetics.

## Material accounting and limits

All24 whole-script and final per-vessel represented H/O/C/N/P/S/Na/Cl/Ca/Mg/Fe and
charge ledgers close within the existing1e-8mol inventory audit resolution; the
largest residual is2.204e-12mol in AB-003. This is a **final material ledger**, not
a proof of per-step closure or energy/enthalpy conservation. It counts literal
inputs, diluted carrier water, actual gas exports and initial trapped atmosphere
(N2/O2/CO2). Transfers in AB-012 and AB-022 use declared fractions of prior owned
liquid/aqueous portions, not desired final values. Diagnostic free H/OH,
solute_charge, solver-species tables, historical lots and scene descriptions are
not counted as additional matter. Ammonia-family nitrogen is distinguished from
trapped N2; all-phase carbon is distinguished from exported CO2.

Targeted inspect output can omit other vessels. The reader permits that only
when the original script command names that exact single vessel, the operator is
nonmutating inspect, no events occur, and its published state equals the previous
state. Other operators must publish every previously known vessel. Thus the last
AB-024 inspection of v2 cannot erase the magnesium control from final accounting.
No solver failure is accepted as a chemistry result; AB-018's three named domain
notices are retained verbatim. Full thermal, unrestricted phase, rate and empirical
claims remain outside this assessment. Catalogue additions: **zero**.
