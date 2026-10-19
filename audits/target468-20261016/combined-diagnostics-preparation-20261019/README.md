# Combined diagnostic-domain and locale validation

PR839 now combines both narrow repairs by normal merge as `20ea572c`, with a constant-aware locale scanner and tests-only corrected-gate baseline `342bfa75`. [Frozen bindings](preparation.json) retain identical corrected-gate regression files and both original native baseline bindings. The completed [first candidate outcome](../peroxide-first-outcome-20261019/README.md) remains unchanged. PR840’s separate existing validation continues; that draft is superseded for integration, and must not merge independently.

One new bounded CPU job validates the actual combined runtime: Clippy, full native/core, external-hook profile and all33 unchanged product logical jobs. These are new combined-source checks, not a repeated unchanged experiment corpus. Original peroxide native baseline failures are retained without another execution. Actual results and protected/platform/integration acceptance remain pending.
