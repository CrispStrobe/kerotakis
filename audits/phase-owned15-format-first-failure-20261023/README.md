# First CORE15 harness stops before tests

Exact source1833e6b1 was issued with15 planned core controls, strict core Clippy and a format check. The check incorrectly added `--all` to the canonical `cargo fmt --check` command. That extra flag traversed pinned third-party SUNDIALS path dependencies; all107 reported diff locations are in that submodule. The worker stopped before Clippy, test compilation or any of the15 controls. No source failure or pass is inferred.

The complete original logs/manifest are preserved privately; this explicitly labeled receipt projection retains their hashes and all repository-relative diff locations. The once-only successor keeps the exact source/tree/lock, test list, physical assertions, strict Clippy command and bounds; only the format invocation changes to the existing canonical repository gate. No third-party source or formatting baseline is modified. Actual successor outcomes remain pending.
