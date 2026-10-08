# Thermodynamic kernel integration

This main-based slice ports six source files and nine preserved contract files
from accepted source
[`8cdab1f6ecd0b1132c4008014a5a0c5476429d17`](https://github.com/CrispStrobe/kerotakis/commit/8cdab1f6ecd0b1132c4008014a5a0c5476429d17)
onto main base `b31455cc9b5946cfdf8c3e43f29707f76b7ac688`.
[The port receipt](thermo-port.json) binds original and destination bytes.
Original nine contract files are retained as adjacent `.rs.txt` snapshots.
Rustfmt changed no source or test bytes; imports and assertions are unchanged.

The kernels preserve positive trace phase components, check published phase
states against their activity coefficients, join the ethanol fitted overlap
continuously, and make still completion/refusal boundaries explicit. Exactly
pure still stocks use one phase solve with a checked closed-form latent account;
mixtures retain staged integration. Existing Option-returning functions remain;
checked still diagnostics and contributor-list accessors are additive APIs.

## Compatibility and remaining boundaries

`VapourPressure::Blended` adds a public enum variant. External exhaustive matches
must add a case; no existing in-repository main exhaustive match was found.
`segment_at` and `provenance_at` remain leading-contributor views. Callers that
claim to report the full overlap must use `segments_at` and `provenances_at`.
The original fitted coefficients and source records remain unchanged; the join
is a numerical interpolation, not a new measured fit.

Core still callers still consume the Option wrapper and label every refusal as
an unavailable bubble point. Porting the checked diagnostic categories into
those operators is separate work. Scalar vessel withdrawal, residual cutoffs,
donor/receiver certificates and whole-operation rollback remain open. Kernel
acceptance alone cannot establish those owner-level properties.

The existing runtime instrument obtains its model label from the pack, so the
new overlap model string propagates without a caller change. No registry-export,
registry data, codex golden or licence/provenance approval changes belong in
this slice. Existing diagnostic spike code using `segment_at` should adopt
multi-contributor reporting before claiming complete overlap attribution.

## Validation

Preparation verified all 15 original/destination hashes, the nine original-byte
snapshots, rustfmt and whitespace checks. No local Cargo or application runs
were performed. Main-based hosted PR gates are pending. Prior accepted-branch
results do not establish acceptance of this port or release performance.
