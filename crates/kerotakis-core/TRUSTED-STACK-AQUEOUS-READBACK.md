# Trusted aqueous readback through the standard stack

`characterize_via` and `continue_via` require an explicitly trusted concrete
aqueous adapter. The production CLI wraps its adapter as
`SolverStack → PhaseEquilibrator → DisplacementEquilibrator → PHREEQC`.
The stack and phase wrappers previously hid this optional capability.

The stack now forwards exactly one opted-in provider and refuses empty,
generic-only or ambiguous stacks. Phase forwards its inner aqueous adapter.
No receipt is manufactured from cached `Some(solution)`, and no provider is
selected by display name or guessed from ordinary equilibrium output.

This interface characterizes only the current aqueous input. It does not run
or certify the stack's other chemistry, phase settlement or route-policy
acceptance. Optional partial answers remain partial and carry no complete-answer
receipt. Existing continuation conservation, gas, freshness and unknown-heat
checks still apply. No haloalkane route or new chemistry is installed here.

Four core regression groups cover unique/nested adapters, cached and ambiguous
refusal, stale receipt and failed-readback rollback, retained/exported gas and
optional partial scope. One native group runs the existing mechanical acceptance
suite through the exact production CLI wrapper topology. These callbacks are
artificial transaction fixtures, not physical endpoint calibration.

Independent source review and rustfmt/diff checks pass. Hosted compile, native,
cache-only/Wasm and applicable full-fleet validation remain pending. Keep the
PR draft until nonprotected acceptance is complete, then use normal protected
exact-head auto-merge. The separate hydroxide reaction proposal remains held
for solver-aware integration, conservative admission and real regression tests.
