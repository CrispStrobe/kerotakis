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
- **No dry carbon monoxide chemistry.** The 2005 evaluation recommends
  nothing for `CO + O₂`, `CO + HO₂` or `CO + O (+M)`, so the CO pack
  carries no route from CO to CO₂ that does not pass through a
  hydrogen-bearing radical. A dry vessel does nothing there. That
  understates a real, slow reaction — it is not a claim that dry carbon
  monoxide cannot burn.

## The packs

| File | Steps | Kind | What it is for |
|---|---|---|---|
| `h2-o2-skeletal-v1.yaml` | 16 | skeletal, elementary | hydrogen burning: chain branching, the HO₂ shunt, peroxide decomposition, radical recombination |
| `co-h2-wet-v1.yaml` | 20 | skeletal, elementary | wet CO oxidation — why a trace of water is a *catalyst* for burning carbon monoxide |

Both packs draw every rate coefficient from one evaluation — Baulch et
al., *J. Phys. Chem. Ref. Data* **34**(3), 757–1397 (2005) — because a
mechanism assembled from a single self-consistent evaluation is a
defensible object, and one assembled from whichever paper gave the
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

### A note on what is *not* here yet

There is no hydrocarbon pack in this directory. Methane, propane and
butane need **global** steps — Westbrook & Dryer's one-step fits, whose
reaction orders are non-integer and, for methane, negative — and the
mechanism parser derives orders from the equation and does not yet read
an explicit `orders:` block. That is BRD-041's next piece of work, and
until the parser can express the orders honestly there is no point
writing the file: a global step with the wrong orders is not the same
rate law, it just looks like one.
