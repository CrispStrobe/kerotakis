# Explicit safety-veto baseline dispatch

This diagnostic branch starts at test-only freeze `b1823e06`, before the seven-path production repair. Its production remains main checkpoint `b31455cc`. The registered chemistry audit workflow is replaced only on this branch with a manually dispatched, serial core test job. This workflow must never merge into main.

The exact command is `cargo test -p kerotakis-core --test safety_veto_atomicity -j1 -- --test-threads=1`. It records revision, toolchain, submodules, all tracked Cargo manifests, selected production source and frozen contract/forecast hashes, raw log, exit status and every observed test outcome. Cargo generates the untracked dependency lock; the artifact captures and hashes it after Cargo execution, including failing test execution.

Twenty outcomes are expected if compilation succeeds. A compile or harness failure is not a behavioral baseline; absent outcomes must remain explicit. Hosted execution is pending.
