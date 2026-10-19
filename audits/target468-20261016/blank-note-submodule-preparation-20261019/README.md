# Blank-note focused worker: corrected setup

The [first setup stop](../blank-note-first-setup-stop-20261019/README.md) executed
no test assertion. This separate [exact worker](worker.py) initializes the
source's pinned git submodules before Cargo resolution. Actual provider version2,
source `b703b695`, unchanged tests-only baseline `7595b3e1` and worker SHA are
bound in [issued.json](issued.json). The original source/fixture/lock plan remains
in the [first preparation](../blank-note-preparation-20261019/frozen-plan.json).

No test, numerical parameter, expectation or source admission rule changed.
The worker still requires two genuine baseline locale assertion failures plus
two positive/domain passes, then all four repaired controls, engine-locale lint,
composed-locale controls and core all-target Clippy. Execution is issued, not
accepted. [Portable saved reviewer](review_saved.py) requires `--run-dir` with
the actual collected output, original plan, exact upload worker and
`ISSUE-BINDING.json`; it checks actual version/source and every raw manifest hash.
Full combined-source/native/product/platform and changed-delivery acceptance
remain required after current PR841 integration/delivery. No original experiment
or scientific-catalog selection is issued here.
