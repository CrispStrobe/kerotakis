# Main-based native budget baseline dispatch

This branch starts at the contract-only freeze `00ea2cdd`, before the production port. Production remains the main-based implementation; the only aqueous change is private test module inclusion. This branch replaces the registered chemistry audit workflow with a manual diagnostic job and must not merge into main.

The hosted command runs native library tests and the existing `solid_solution` integration target with engine support, serial execution and `--no-fail-fast`. The artifact records revision, toolchain, submodules, every phase-contract and selected source/manifest hash, exact command, raw log and exit status. Cargo generates the untracked dependency lock; the job captures and hashes it after Cargo finishes, including after a failing test run.

Dispatch and actual results are pending. Test failure evidence must distinguish assertion failures from compilation or workflow failures. A passing donor branch is not this main-based baseline.
