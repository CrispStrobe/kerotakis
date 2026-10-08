# Native gate review

The reviewed #768 Linux and #769 Mac receipts identify the actual CI checkout, verify complete Git tree equality with the PR head and match every expected fixture function to an unfiltered passing result. They cover one native gate each; required-gate and merge acceptance remain in the separate merge receipt.

[The reusable offline reviewer](../../../tools/native-controls-review.py) automates these checks. It requires archived job logs, `gh run view --json headSha,event,jobs` metadata and the Git commit response for the actual checkout from `gh api repos/CrispStrobe/kerotakis/git/commits/COMMIT`. The checkout SHA must come from the log's `git log -1 --format=%H` output. Preserve these inputs before changing a failed head. Download raw logs through the Actions job logs endpoint once the job has completed.

[The #770 plan](pr770-plan.json) binds ten targets and 105 functions: 94 policy/owner/schema/transaction functions plus 11 persistence functions. [The #771 plan](pr771-plan.json) binds fourteen targets and 143 functions, adding the 38 historical tracer/crystal controls. These are expected coverage plans, not execution results. Their exact source heads and fixture hashes must match the archived native job.

For #770, with downloaded evidence named `native.log`, `run.json` and `checkout.json`, invoke:

```sh
python3 tools/native-controls-review.py \
  --pr 770 --head ee7132fe6835eb7535b413aa61430f94c618f0be \
  --job JOB_ID --log native.log --run-metadata run.json \
  --checkout-metadata checkout.json \
  --plan audits/integration-20261008/native-controls/pr770-plan.json \
  --report native-review.json
```

For #771, select its head, job and plan. Store a new report outside the preserved artifact; the tool exclusively creates its report. After any source-head change, create a separately named plan for the new exact head and verify every fixture hash; preserve previous plans and receipts. The reviewer refuses missing, failed, ignored, filtered or repeated fixture executions. It handles Cargo announcing the next test binary on stderr before the previous binary flushes its stdout summary.

The reviewer was checked against the original #768/#769 archives (42 and 11 functions). Six focused parser controls also verify rejection of incomplete/filtered/failed/ignored/repeated results and acceptance of interleaved output. Run them with:

```sh
python3 -m unittest discover -s tools/tests -p test_native_controls_review.py
```

A successful native review does not replace the remaining exact-head required gates, candidate/main source comparison, historical full-audit acceptance or scientific model validation.

The initial #771 preflight exposed five tracer validity failures. [The repaired #771 plan](pr771-26cd8fc7-plan.json) binds head `26cd8fc7` and all 151 functions, including eight separately frozen ordinary-commit controls. Preserve the initial 143-function plan as its historical source snapshot. Neither plan is an execution receipt.

[The integrated #771 plan](pr771-dec6b593-plan.json) binds final head `dec6b593` after actual main ancestry was merged. All fifteen fixture hashes and 151 functions are unchanged from `26cd8fc7`, but fresh exact-head native logs and required gates remain necessary. The existing `pr771-mac.json` receipt remains pinned to its original source.

[Fresh #771 Mac review](pr771-dec6-mac.json) now verifies all 151 functions on `dec6b593`. All five required gates now pass. [The separate native-host failure](pr771-dec6-native-host-first-failure.json) occurred during SUNDIALS download before compilation; [its same-head retry passed](pr771-dec6-native-host-retry.json), and #771 is merged. The original failure remains unchanged.

Next plans: [#772 native151](pr772-plan.json) at `8037f069` and [#773 native204](pr773-plan.json) at `fb50bebc`. Both include actual merged main ancestry; their new gates remain pending. Create separately named plans after any later source-head change.
