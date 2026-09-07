# Clean native third-fleet validation

The CLI rebuilt with the atomic-refusal and fail-closed startup changes
completed all 36 cases 87–122. All 174 predeclared checks pass, including
exact physical-state preservation on refused ammonia distillation. The
native inventory checker also passes all 69 comparisons.

No tolerances or experiment expectations were changed after the earlier
failure. The original 173/174 discovery and the later four-timeout replay
remain in their separate directories. This fresh run supersedes neither
as historical evidence; it demonstrates the repaired executable's result.

The recorder summary contains source, lock and executable digests. This is
native CLI validation, not a claim that all cross-branch CI gates pass.
