# Checked partition source: accepted regression results

Exact source `d67205091a22bcdb2fc490866224c4bc7fef8a4e`, tree `76820b9c`,
passes engine-locale lint, all-target Clippy,2153 core tests,184 thermo tests,
620 native PHREEQC/CEA tests and two external-hook controls, with zero ignored
or failed tests. All500 original coverage checks have zero unmet requirements,
solver failures or descriptive drift. All33 original product jobs/142 steps,
29 named-law reports and three original aggregates pass.

The [independent acceptance receipt](acceptance.json) binds the actual release
binary `41ca6122`, actual producer lock `66886813`,854 evidence files and2193
archive members. Approved parameter tables and independent oracle values remain
unchanged. Two new genuine interaction-factor baseline failures plus their
positive control are preserved in the [raw baseline](factor-baseline.stdout).
The earlier liquid-transfer and actual macOS failures remain separately bound.

The [first saved reader](first-reader.stderr) incorrectly expected two unit-test
names without Cargo's `unifac::tests::` prefix. The [correction receipt](saved-reader-correction.json)
binds both original and corrected reader hashes. The corrected reader requires
the exact qualified names; counts, hashes, fixtures, scientific assertions,
source/lock/law checks and runtime remain unchanged. No engine request was
repeated. [Portable corrected reader](review_saved_qualified.py) preserves its
separate `--run-dir` adaptation. Published stderr removes host-directory prefixes;
its original bytes and hashes remain retained in the execution evidence.

This is a compact public checkpoint. Full collected output, actual archive and
authenticated binaries remain retained; copying the remaining full raw archive
into the public audit is separate resource-gated publication work. This checkpoint
does not claim to contain every raw file needed by the saved reviewer.

PR841 still requires current protected CI and both actual native-platform log
reviews, then normal exact-head auto-merge and actual merge-tree verification.
Changed artifact/API/browser acceptance follows integration. Previous stopped
observers issued no merge, deployment or browser request. New observers retain
the same source and acceptance requirements with the corrected saved receipt.
No production delivery, original experiment replay or scientific-catalog
selection follows from this engineering acceptance.
