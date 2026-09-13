# Chemistry audit continuation

The coupled reactive Nernst–Planck solve is wired into the electrode current
balance for the **steady** diffusion layer. Species diffusivities and one common
physical layer thickness are read off the authored `DiffusionLayerTransport`
records, and the solved surface activities feed the shared Nernst and
Butler–Volmer equations through the existing `solve_interfacial` fixed point, so
current feedback is implicit. A charged interfacial flux selects the boundary;
an uncharged one keeps the diffusion-only adapter, which is that boundary's
zero-potential limit, so no accepted case changed answer.

What is still open against the original acceptance list: the **transient**
layer. `TransientDiffusionLayer` has no migration term, so a charged flux on it
refuses by name rather than borrowing the steady answer, and zero-current
relaxation and time-partition invariance for a migrating layer are untested
because no such layer exists yet. Incomplete mobility and incompatible
transport still refuse explicitly, now including a network whose species do not
share one physical layer.

The boundary's bounded solver resolves a root to about
`residual_tolerance / |d residual / d phi|` volts, and that slope grows with
`(D / L) * c`. The bracket is floored by the spacing of `f64` near the root
rather than by a constant, so the aqueous cases this coupling needs -- the
hydrogen and hydroxide ions at bench strength across a micrometre-scale layer
-- are inside the working range. A case outside it refuses with
`DidNotConverge`; it must not be papered over by loosening
`current_tolerance_a_per_m2`.

That envelope was measured on `zero_current_junction`, whose residual is a
current, and it does not transfer unchanged to the electrode surface the
electrochemical caller actually uses. `electroneutral_surface` tests a **charge
concentration** against `charge_tolerance_mol_per_m3`, and the slope of that
residual is about `F / (R T) * c` -- it carries no `D / L` factor, because the
layer only enters through the flux term. Its range is therefore wider in the
thin-layer direction, and is pinned by
`the_electrode_surface_range_covers_bench_electrolytes_and_thin_layers`: every
combination of 10 mM to 12 M with layers from 10 nm to 100 um converges,
including the 10 nm / 2 M corner at which the junction refuses. No tolerance
was loosened to get there. The failure mode that does bite this solver is
different in kind: a flux that drives a surface concentration to zero makes the
residual undefined, and the solver refuses rather than clamping.

Hysteresis, evolving films and dynamic bubble coverage remain evidence-gated.
Require forward/reverse or time-series data that identify their state laws and
validity ranges; current alone does not determine coverage.

Keep both public source-informed corpora immutable. Freeze every new script and
scientific relation before execution, preserve failed evidence, and promote only
distinct bilingual concepts. Exact source mappings and source research remain in
the private repository; public manifests contain original Kerotakis scripts and
generic scientific contracts only.
