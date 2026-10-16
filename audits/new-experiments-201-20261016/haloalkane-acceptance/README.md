# Four haloalkane procedure follow-ups

The first request was **HTTP 422 before engine execution** because `register lv3`
is outside the protected API command subset. The immutable first
[design](design.json), [raw refusal](D3-SO-001.response.json),
[execution receipt](execution.json) and [assessment](assessment.json) preserve it.
The other three cases in that first batch were not issued. No chemistry verdict
follows from this refusal.

A separately authorized [API-envelope follow-up](api-envelope-v2/design.json)
removes only exact `register lv3`; every physics and measurement command remains
unchanged. CLI source shows this directive controls rendering detail, not a bench
operator. All other command verbs were checked against the pinned API allowlist.
The allowlist was not changed. The original scripts and four native candidates
remain in the parent intake. In the v2 design, inherited `original_script_path`
values are relative to the first design's directory; `native_cli_candidate_script_path`
values are relative to the v2 directory. Original hashes unambiguously identify both; the repository-relative
[script associations](api-envelope-v2/script-associations.json) resolve every path.

## Actual results

All four adapted requests completed **once**, serially, HTTP 200/exit 0, on the
existing source-74 preview. Fresh health matched the source, binary and
producer-reported lock binding before requests. The lock qualification is retained:
raw producer lock bytes were not independently rehashed. This is not newer
production acceptance. Parser and selectivity source bytes match the intake's
integrated c446 source; the complete runtimes are not claimed identical.

| Case | Frozen principal forecast | Actual result |
| --- | --- | --- |
| SO-001 | Cool primary SN2, hot primary E2 | Both named primary+weak-nucleophile refusals; substrate remains. |
| SO-002 | Neutral tertiary SN1 | SN1, 0.0002 mol tert-butanol and bromide, matched substrate/water debits. |
| SO-003 | Primary SN2, tertiary strong-base E2 | Primary refuses; tertiary selects SN1 and forms tert-butanol instead of isobutylene. |
| SO-004 | Cool neutral SN1, hot neutral E1 | Both principal products/mechanisms and 0.0002 mol extents match. |

Two principal forecasts match within a bounded completion-on-command table.
Two reveal a concrete ownership/admission gap. These are not measured yields,
rates, reaction heats or general selectivity validation; **zero catalogue cases
are closed**. All seven React-step material/charge comparisons pass the frozen
1e-8 mol ledger tolerance, counting analytical basis and external gas terms.
Temperature and actual pre-existing headspace inventories are preserved per vessel.
No missing mass is demonstrated.

The strong-base witnesses contain sodium plus `base_equivalents` before React,
but no molecular `NaOH`. `find_nucleophile` looks for literal NaOH then water,
so it misclassifies these solved inputs as weak nucleophile. The standalone
[MODEL-006 intake](../../reaction-dataset-200-20261015/model-issues-20261016/MODEL-006.json)
records positive controls, salt/sour-water negatives, finite ownership/debit,
rollback and client-parity requirements. Analytical alkalinity must not simply
be relabeled as free hydroxide. No model source was changed.

Read the [v2 saved assessment](api-envelope-v2/assessment.json),
[receipts](api-envelope-v2/execution.json) and exact responses alongside the
frozen original forecasts. The [saved reader](api-envelope-v2/review_saved.py)
never issues requests; to rereview, pass a new `--report` path because it refuses
to overwrite its assessment. No further requests or retries were made.
