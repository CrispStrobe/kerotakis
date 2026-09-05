# Gas-phase mechanism packs (BRD-041)

Project-original reduced reaction networks for the gases a school bench
actually lights, written reaction by reaction from a published
evaluation because no existing mechanism file may legally ship here.

## Why these files exist at all

BRD-040 audited the mechanisms a combustion engineer would reach for —
GRI-Mech 3.0, Ó Conaire, Boivin, the syngas sets, FFCM-1, San Diego — and
found that **not one of them carries a redistribution grant**. Cantera,
which ships several of them, states in its own documentation that it "is
not claiming to grant a license to" the mechanisms it distributes. The
full reasoning is in `provenance/brd-040-cantera-audit.md`.

So these networks are written here, reaction by reaction, from published
rate constants, and each reaction records where its numbers came from.
Rate constants are measured facts, and a table of facts is not a
copyrightable work; a mechanism *file* — somebody's selection, ordering
and tuning of those facts — is a different thing, and none was copied.

## Licence

**CC BY 4.0**, as Kerotakis-authored curated data (`CONTRIBUTING.md` §2),
separately from the AGPL-3.0 engine code. Attribute as:

> Kerotakis gas-phase mechanism packs, CC BY 4.0.

The literature each reaction cites keeps its own terms; a citation is not
a redistribution of the cited work.

## Format

Each pack is the portable Cantera-YAML subset that
`kerotakis_core::kinetics::mechanism::parse_yaml` accepts — the same
front end BRD-040 hardened. Read them with:

```sh
cargo run -p kerotakis-cli -- mechanism inspect data/mechanisms/<pack>.yaml
cargo run -p kerotakis-cli -- mechanism simulate data/mechanisms/<pack>.yaml \
    --seconds 1e-3 --volume-l 1 --temperature-k 1200 \
    --feed H2=0.02 --feed O2=0.01
```

Every reaction carries a `note` in one machine-checked shape:

```text
id=<stable-id>; source=<citation>; validity=<lo>-<hi> K; retrieved=<YYYY-MM-DD>
```

`crates/kerotakis-core/tests/gas_mechanism_packs.rs` fails if any of the
four fields is missing, if an id repeats, if a validity range is not an
increasing pair of kelvin temperatures, or if a retrieval date is not an
ISO date. A rate constant without a source cannot be merged.

## What these packs claim

- The **major species** of gas-phase oxidation under the stated
  conditions, and the direction and rough timescale in which they form.
- An endpoint that agrees with NASA-CEA Gibbs minimisation wherever CEA
  carries the species and dissociation is negligible. That agreement is a
  test, not a hope: `crates/kerotakis-cea/tests/gas_mechanism_endpoint.rs`.
- Element conservation, exactly, reaction by reaction — the parser
  refuses an unbalanced equation and the tests re-check the compiled
  network.

## What they do NOT claim

- **No soot, no luminosity, no yellow flame.** Particle formation is not
  modelled anywhere in these packs. A flame's colour is not derivable
  from them.
- **No NOx.** Nitrogen appears only as an inert diluent and third-body
  collider. Thermal-NO chemistry is absent, not zero: a pack that says
  nothing about NO is not saying there is none.
- **No ignition-delay fidelity.** A skeletal set reproduces the products
  and the sequence of the chain; it is not validated against shock-tube
  delay times.
- **No pressure dependence beyond what is written.** Where a reaction is
  written as a plain third-body step, its falloff behaviour is not
  modelled.
- **No transport, no flame structure, no flame speed.** These are
  homogeneous reactor networks. There is no diffusion in them.
- **Nothing silently outside the stated temperature range.** Each
  reaction records the range its evaluation covers, and each pack's
  header lists, by name, every step it uses outside that range and by how
  much. A skeletal mechanism whose steps were all evaluated over exactly
  the same window does not exist; naming the mismatch is the honest
  alternative to pretending it away.
- **No carbon monoxide from the global steps.** A one-step fuel reaction
  makes CO₂ and H₂O and nothing else, so it over-predicts heat release
  wherever real equilibrium would leave CO. The authors measured how
  much; the test suite checks it against CEA rather than letting it pass.
- **No dry carbon monoxide chemistry.** The 2005 evaluation recommends
  nothing for `CO + O₂`, `CO + HO₂` or `CO + O (+M)`, so the CO pack
  carries no route from CO to CO₂ that does not pass through a
  hydrogen-bearing radical. A dry vessel does nothing there. That
  understates a real, slow reaction — it is not a claim that dry carbon
  monoxide cannot burn.

## What the tests can and cannot demonstrate

The packs are validated as **data** — parsed, compiled, unit-checked,
balance-checked, provenance-checked — and then exercised dynamically over
a **bounded early-chain window** of half a microsecond, not through an
ignition transient.

That boundary is the engine's, not the packs'. `kinetics_integrator.rs`
hands diffsol a matrix-free Jacobian whose finite-difference probe is one
scalar for the whole extent vector, sized from `(1 + ‖x‖∞)`. A radical
chain has extents spanning nine orders of magnitude at once, so a probe
sized for the millimole extents linearises the nanomole ones across their
entire range; the Newton iteration fails, and on the H₂/O₂ pack it
exhausts its failure budget at about 2.7 µs — right where the chain
starts to run away. The `.max(0.0)` clamp on reconstructed amounts puts a
second corner in the same right-hand side.

So the endpoint claims here are established against **CEA
thermodynamics** rather than by integrating to one, and the kinetic tests
assert what a bounded window can honestly show: the right direction, atom
conservation, the metamorphic responses, and that the kinetic route never
passes the equilibrium the thermodynamics allows — which is true at every
instant, not only at the end.

Two exits exist for whoever takes this further: a component-scaled
Jacobian probe in the extent integrator, or the CVODE path that already
exists in `kerotakis-sundials` and is API-compatible with
`advance_network_with_options`.

## The packs

| File | Steps | Kind | What it is for |
|---|---|---|---|
| `h2-o2-skeletal-v1.yaml` | 16 | skeletal, elementary | hydrogen burning: chain branching, the HO₂ shunt, peroxide decomposition, radical recombination |
| `co-h2-wet-v1.yaml` | 20 | skeletal, elementary | wet CO oxidation — why a trace of water is a *catalyst* for burning carbon monoxide |
| `hydrocarbon-global-v1.yaml` | 3 | **global**, not elementary | methane, propane and n-butane as one-step overall reactions |

The two skeletal packs draw every rate coefficient from one evaluation —
Baulch et al., *J. Phys. Chem. Ref. Data* **34**(3), 757–1397 (2005) —
because a mechanism assembled from a single self-consistent evaluation is
a defensible object, and one assembled from whichever paper gave the
prettiest curve for each step is not. Where that evaluation gives no
recommendation, the step is **absent** and the pack's header says which
steps are missing and why. That is the whole discipline: a reaction with
no citation does not go in.

The evaluation writes rate coefficients in cm³ molecule⁻¹ s⁻¹ with the
exponential as `exp(−B/T)`, *B* in kelvin. The packs are in cm³ mol⁻¹
s⁻¹ with `Ea` in cal/mol, so every pre-exponential is multiplied by the
Avogadro constant once per molecularity beyond the first, and every *B*
by the gas constant. `crates/kerotakis-core/tests/gas_mechanism_packs.rs`
re-derives both conversions from the file's own `units:` block for every
reaction, so a mis-scaled number cannot merge.

### Global steps are a different kind of object

`hydrocarbon-global-v1.yaml` comes from somewhere else — Westbrook &
Dryer, *Combust. Sci. Technol.* **27**(1–2), 31–43 (1981), Table I — and
it is not a mechanism in the same sense at all. A **global step is a
curve fit to a flame, not a chemical event.** Nothing in it happens in a
single collision, and its reaction orders are *measured separately from
its stoichiometry* rather than read off it:

| Fuel | Equation says | Fitted orders | Overall |
|---|---|---|---|
| CH₄ | 3rd order | fuel **−0.3**, O₂ 1.3 | 1st |
| C₃H₈ | 6th order | fuel 0.1, O₂ 1.65 | 1.75 |
| n-C₄H₁₀ | 7.5th order | fuel 0.15, O₂ 1.6 | 1.75 |

Methane's fuel order is *negative*: the fuel inhibits its own
consumption. That is not a typo, it is the whole point — the authors show
that a first-order-in-everything global step cannot reproduce the rich
flammability limit, and the negative order is what fixes it. It is also
why the mechanism parser had to learn Cantera's `orders:` block (BRD-040
had listed explicit orders as *not* needed; that was wrong, and
`provenance/brd-040-cantera-audit.md` records the correction).

A global step will give you a fuel consumption rate and a heat release.
It will not give you an intermediate, a radical, an ignition delay, or
carbon monoxide — it forces complete conversion to CO₂ and H₂O and so
over-predicts heat release, which the authors quantify in their own Table
III and which `crates/kerotakis-cea/tests/gas_mechanism_endpoint.rs`
tests against CEA as an explicit limitation rather than hiding.

One thing the paper says that a reader of this directory should not skip:
the authors calibrated each pre-exponential against a measured flame
speed and warn that "the pre-exponential terms tabulated here should be
regarded as approximate values if they are used in other numerical
models". Kerotakis has **not** recalibrated them, because there is no
flame-speed model on this bench to calibrate against. Ratios and
dependences are the chemistry here; absolute times are an estimate.
