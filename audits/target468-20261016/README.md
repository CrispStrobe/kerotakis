# Fixed 468-experiment target

Target: the original147 catalog plus first120, batch2’s101 and batch3’s100.
Success means at least401 of these fixed cases have reviewed working evidence;
new designs, variants and engineering checks do not increase this count.

| Batch | Declared cases | Selected working evidence on recorded versions |
| --- | ---: | ---: |
| Original catalog | 147 | 147 retained unchanged from159fa6e0 |
| First additional batch | 120 | Authenticated120; first execution pending |
| Batch2 | 101 | 6: QA001, QA002, QA009, QA010, QA011, QA012 |
| Batch3 | 100 | 0 selected so far |
| Total | 468 | 153 accounted for; no unified latest-release count yet |

The [scoreboard](scoreboard.json) now has all468 actual case records. The first120
file is present and authenticated against the exact SHA declared by batch2. The
[previous checkpoint](scoreboard-before-first120-arrival.json) preserves the earlier
missing-file state; it is superseded by this inventory.
The working catalog has155 entries, but its zinc/copper and conducting-stock
water-electrolysis repairs come from the separate200-reaction corpus and are
outside these four batches. Original147 entries match their frozen export.

All available201 were screened once on c446/670. After the six selected cases,
105 clean-exit cases still need physical assessment,85 carry named model limits,
two carry solver failures and three refuse admission. These are triage categories,
not pass/fail judgments against every original physical forecast. No merged fix
is assigned new experiment passes before source-bound follow-up acceptance.

## Ordered closure lanes

1. Execute the [frozen remaining267 protocols](common-engine-completion/README.md):
   the original147 plus the now-authenticated120, unchanged, using the same
   c446/670 engine as the completed201. Preserve first outcomes and then assess
   all468 on that common source. This does not certify latest main or deployment.
2. Assess the105 remaining clean-exit candidates against their original criteria,
   using saved raw evidence and represented material/charge/gas ledgers. Record
   explicit pass, failure, unsupported or inconclusive outcomes per original ID.
3. Rank unselected cases by shared causes and repair impact. The old screen
   flags missing aqueous component mapping in23 cases, incomplete absorbance in10
   and several overlapping mineral/saturation limits. These overlapping counts
   are not additive, and a warning alone does not establish criterion failure.
   Keep transport, transfer and reaction readback repairs bound to their own
   accepted source; replay only affected original IDs after changes.
4. Run source-bound acceptance of the fixed target against the chosen integrated
   release and qualify actual deployment separately. Update the scoreboard from
   per-case physical outcomes, preserving previous versions and first failures.

There are248 further selections needed to reach401 from153. Even if all105
clean candidates and the first120 eventually work, the total would be378;
at least23 additional limited/failing/refused cases must then be repaired.
This is an optimistic planning bound, not a prediction that all228 will pass.
No empirical calibration or latest-release certification is claimed here.

## Separate curriculum corpus:668 records in all

The200 curriculum reaction descriptions are an additional corpus, bringing the
combined record total to668. The newly supplied copy is byte-identical to the
already audited dataset; it is not another200 new cases. Existing CLI/API
contact-probe evidence covers all200:92 completed and108 refused. Complete
protocol repairs and physical assessment remain separate. See the
[200-record assessment](../reaction-dataset-200-20261015/CHEMISTRY-RESULTS.md).
Semantic overlap across the corpora is not yet exhaustively assessed.

All101 batch2 protocols have executed:101 zero exits, including61 requiring
physical review,39 with named limits and one with a solver failure. Only six
have completed the catalog-selection review; this is not a claim that95 failed.
Batch3 has97 zero exits and three process refusals.

[Three further saved assessments](saved-assessment-next3/README.md) select
QA009/010/012 without rerunning their protocols. The remaining267 and separate
curriculum200 source follow-up are dispatched once, with their exact source
and input bindings; consult each dispatch receipt for the live run.

## API duration and command coverage

The previous200 API batch took758.01545seconds including pacing, with3.27second
median request time. Linear extrapolation is approximately30minutes for468
or42minutes for668; longer protocols, cold starts and quotas can add time. This
is a measured historical extrapolation, not a current production benchmark.
The [actual pinned validator check](api-admission-all668.json) admits603 of668
after only presentation/eyes alias adaptation. The other65 require commands
outside the current subset or exceed duration limits. This admission result
performs no HTTP or chemistry execution. Full CLI execution preserves all
commands, observations and simulated durations without weakening API policy.
