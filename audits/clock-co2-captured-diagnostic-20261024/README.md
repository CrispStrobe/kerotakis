# Actual positive-time CO2 conservation failure

The protocol and expected1e−10mol/eq gas-adjusted physical ledger were committed in `d0337c96` before execution. One resource-gated run used the already captured `a0177cff` debug CLI and actual lock41bb, one CPU/1.5GiB/30-second cap, without rebuilding. Actual runtime0.932seconds, process exit0 and no SolverFailed. This does not mean the experiment passed.

The wait reports one inward CO2 event of1.0555342178004468e−6mol. Carbon, hydrogen and sodium close within1e−10; oxygen is short by2.111068435060975880351e−6mol and formal charge increases4.22213687095886859e−6eq. The diagnostic therefore FAILS the frozen conservation expectation. Native pH and gas output alone hid this failure. Full actual input/output and source/tree/lock/executable/script hashes are retained in `RESULT.json`, `diagnostic.stdout`, `diagnostic.stderr` and `LEDGER-ARITHMETIC.json`. Independent saved-byte Decimal review reproduced the failure without rerunning it.

The [clock-boundary lane](../../docs/clock-co2-boundary-lane-20261024.md) scopes the source repair and prospective controls. The observed baseline is sourcea0177, not acceptance or execution of the current candidate. Keep its106-control result and this distinct protocol separate; do not change rate/floor/energy/charge budgets. Original qualification remains338/468.
