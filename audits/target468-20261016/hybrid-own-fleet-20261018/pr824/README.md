# PR824: complete original own-fleet acceptance across bound providers

All33 original logical jobs are satisfied:28 jobs completed on GitHub; four
remaining24-case families and their288-case aggregate ran locally using the
same accepted producer binary and the exact GitHub-tested source tree. All96
newly executed cases and their original named assertions passed. No completed
physics case was repeated and no test, manifest, golden or forecast changed.

The original GitHub fleet run39104378835 was cancelled only after a snapshot
showed no active job, to transfer its unstarted queue. Its cancelled status is
preserved and **is not claimed to be a successful33-job GitHub run**. Required CI
39104378850 passed all25 checks and REST39104378816 passed. Branch protection and
normal exact-head auto-merge remain required.

Read the [full acceptance](acceptance.json), [source/test binding](source-and-test-binding.json),
[original aggregate result](v2-aggregate.json) and [byte manifest](evidence-sha256.json).
The [original local recordings](local-recordings/) retain all96 raw streams,
scripts and summaries. The aggregate has12 families,288 cases,80 relations,
368 checks and zero unmet families, including the unchanged earlier CI artifacts.

Reviewed head `654cf041` and tested merge `b9a5441d` have the same tree
`d4365ac21120bef0b70a98871476008e7a515def`. The actual binary SHA256 is
`e094e4909922ef4febd21f9e6acae5a16dbadcfef6157bc0e7b2a94cbf14dc65`.
[Producer lock bytes](producer-Cargo.lock) are independently bound to that
artifact; they are generated producer data, not a tracked source lockfile.
Source, raw producer artifacts, runtime-library identities and original queue
snapshots are retained; private host details stay in the private operating record.

This is regression acceptance for a repair, not96 additions to the working
catalog, a current-deployment claim, empirical calibration or a rerun of668.
