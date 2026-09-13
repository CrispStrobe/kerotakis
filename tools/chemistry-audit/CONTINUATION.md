# Chemistry audit continuation

Wire the coupled reactive Nernst–Planck solve into the electrode current balance.
Derive species diffusivities and one common physical layer thickness from the
reviewed transport records; feed the solved surface activities back into the
shared Nernst and Butler–Volmer equations. Acceptance requires implicit current
feedback, conserved Faradaic commits, zero-current relaxation, time-partition
invariance and explicit refusal of incomplete mobility or incompatible transient
transport data.

The boundary's bounded solver resolves a root to about
`residual_tolerance / |d residual / d phi|` volts, and that slope grows with
`(D / L) * c`. The bracket is floored by the spacing of `f64` near the root
rather than by a constant, so the aqueous cases this coupling needs -- the
hydrogen and hydroxide ions at bench strength across a micrometre-scale layer
-- are inside the working range. A case outside it refuses with
`DidNotConverge`; it must not be papered over by loosening
`current_tolerance_a_per_m2`.

Hysteresis, evolving films and dynamic bubble coverage remain evidence-gated.
Require forward/reverse or time-series data that identify their state laws and
validity ranges; current alone does not determine coverage.

Keep both public source-informed corpora immutable. Freeze every new script and
scientific relation before execution, preserve failed evidence, and promote only
distinct bilingual concepts. Exact source mappings and source research remain in
the private repository; public manifests contain original Kerotakis scripts and
generic scientific contracts only.
