# Closed-carbon carbonate law correction — 2026-11-12

This is a scientific-contract correction, not a new engine execution or original-experiment
qualification. The unchanged four protocols add 300 mL water and 0.0005/0.001/0.002/0.004 mol
NaHCO3, measure pH and inspect with no elapsed time.

The original [v2 manifest](../../tools/chemistry-audit/source-fleets-2/polyprotic-carbonate.json)
and its generators remain unchanged. Its first native report is preserved here: the old
`pc-bicarbonate-dose` ordering fails. [Original hosted run](https://github.com/CrispStrobe/kerotakis/actions/runs/43207105040)
tested merge `45aa1e0523d2811be4ec1d3c6d2416989aae827c`, PR head
`bb168622bdc97abc036e5ab6f2146f5df5737367`, native SHA256
`60d2a1e433cc509a8184335546cb1e9c35a843bd7816305a4a0ea034004eecd2`.
Artifact 13948293601 archive SHA256 is
`05096de0596f94a06d8a7baf12439bf354546473d3d46ec6b001e7f17db0a318`.
The producer lock was not downloaded with this family archive; no lock-binding claim is made.

## Why the original ordering is invalid

NaHCO3 is amphiprotic. Closed-carbon electroneutrality and carbon conservation imply, in a
four-species approximation, [CO2*]+[H+]=[CO3--]+[OH-]. With activities and sufficient
concentration, pH approaches (pK1+pK2+log10(gamma_CO3))/2. The divalent carbonate activity
coefficient changes with ionic strength; pH need not increase with dose. Even an independent
ideal four-species calculation gives adjacent increments smaller than the original 0.02 bound.
Neither reversing the direction nor fitting saved pH values is a valid repair.

Reference equations come from the [USGS PHREEQC manual](https://pubs.usgs.gov/wri/1999/4259/report.pdf)
and the [actually pinned WATEQ4F database](https://github.com/CrispStrobe/iphreeqc/blob/47b459c4a5e3e20b9990fa304e207eca0dcc6ff1/database/wateq4f.dat).
The saved four pH values are 8.271879821841, 8.264722766843, 8.243389718233 and 8.208142348793;
these are observations, not new numerical acceptance targets.

## Versioned successor and validation

[The reviewed successor manifests](../../tools/chemistry-audit/source-fleets-2-closed-carbon-v1/)
retain all 288 scripts and all 80 relation slots. Eleven families are byte-identical; one relation
becomes `pc-bicarbonate-closed-carbon-v1`. All other relations in that family remain unchanged.
The [checker](../../tools/chemistry-audit/carbonate_closed_equilibrium.py) requires the actual
WATEQ4F engine/dataset/model/routing, complete species output, exact doses, zero elapsed time and
no external gas crossing or unsupported diagnostic. It independently checks canonical Na/C/H/O
and charge, species Na/C/charge/ionic-strength sums, pH activity, carbonic and water mass action,
and both sodium ion pairs. Four-digit native report rounding supplies explicit intervals;
reference-temperature uncertainty follows the existing documented thermal fixed-point boundary.
No residual tolerance is fitted to the observed pH curve.

Source-only saved readback accepts four cases and rejects 40 corrupted variants. These results
certify the checker against this preserved producer, not acceptance of newly edited runtime code.
The checker initially used the wrong water molar mass (18.01528 instead of the registry's18.015),
failed its input check, and was corrected against the registry mass/density rows. An earlier
reviewer used phreeqc.dat water constants against WATEQ4F output, found a discrepancy, then
corrected the reference after reading actual provenance; that discrepancy established no engine bug.

Repeat the saved predicate controls without an engine:

```console
python3 tools/chemistry-audit/review_closed_carbon_contract.py --check
python3 tools/chemistry-audit/test_carbonate_closed_equilibrium.py audits/carbonate-law-correction-20261112
```

The gas-production and selectivity first failures remain separate issues. This correction neither
waives them nor establishes instantaneous acid-front gas yield, finite-time rate calibration,
new production acceptance or a uniform replay of the original corpus.
