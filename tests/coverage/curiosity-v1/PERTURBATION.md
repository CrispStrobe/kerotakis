# Perturbation cases generated from the corpus scripts

`crates/kerotakis-cli/tests/corpus_perturbation.rs` is the generator. This
file is the argument for it, the measurements the thresholds were set from,
and the honest count of how far it reaches.

## Why the corpus needed this

The 500 prompts here carry a `question`, a `script` and — for 307 of them —
an `expected`. The `expected` vocabulary is
`computed | curated | qualitative | boundary`: those are four **routes**,
not four outcomes. What a green row asserts is that the engine answered by
the route somebody intended. It asserts nothing about the chemistry.

`aq-003` is the standing illustration. The question asks whether a beaker
*cools*; the script measures a thermometer; nothing anywhere records that
the temperature must fall. An engine reporting forty degrees of warming
keeps that row green forever.

What survives is the **scripts** — reagents, quantities and an instrument,
which is the expensive half of a test case and is already paid for. The
generator spends it on the one question that needs no reference value, no
licence and no external data: *does this number move, in the right
direction, when its cause moves?*

## How the generator chooses a perturbation

Each script is parsed into steps — `add <vessel> <species> <amount><unit>
[@ <T>C]` structurally, everything else verbatim — and offered to five
rules. A rule is admissible when the script's *shape* supports it and when
applying it actually produces a different script. A prompt may admit
several; the census counts each.

| rule | perturbation | claim | shape |
|---|---|---|---|
| `Order` | reverse a contiguous run of adds to one vessel | every final quantity is unchanged | invariance |
| `Scale` | double every quantity the script states | intensive unchanged, extensive doubled | invariance + direction |
| `Solvent` | double the solvent only | solute amounts unchanged, ionic strength falls | invariance + direction |
| `Ablation` | delete the last non-solvent reagent | the instrument the script picked up must move | causality |
| `Dose` | double the last non-solvent reagent | the instrument must move | direction |
| `Sibling` | run an authored dose-sibling | two loadings, two answers, ordered | **differential** |

The ordering is the argument. An **invariance** survives an engine
improvement and cannot be satisfied by a constant. A **causal** claim is the
direct cure for what is wrong with this corpus. A **directional** claim is
the weakest, because a quantity the engine multiplies through always moves.
A **differential** claim is the strongest, because a global error moves both
systems equally and cannot fake a ratio between their responses.

### Two decisions that stop a generated case being vacuous

**The baseline run is the corpus script, byte for byte.** The generator
re-renders the baseline from its own parse. If that render were not
identical to the authored line, the file would be measuring something nobody
wrote. All 943 `add` lines round-trip exactly, and
`the_generator_renders_the_corpus_back_exactly_as_written` keeps it so.

**The causal rules read the readout, not the inventory.** The first draft
compared the whole bench, which deleting a reagent satisfies trivially by
the reagent no longer being in it — the corpus's own disease in a new
costume. Keys now carry a prefix: `r.` for a readout (instrument events from
the step stream, the derived properties of the solution, the scene's words
and verdicts), `n.` for an amount, `m.` for a molality. `Ablation` and
`Dose` read only `r.`, and where the script picked up an instrument, *that
instrument's own reading* is what must move. `aq-003` measures a
thermometer; a bench reporting the room's temperature whatever is in the
beaker fails, which is exactly what `expected = "computed"` could not say.

### What the generator refuses to read

`contents["OH-"]` is the solution's residual cation charge wearing
hydroxide's name — recorded in `perturbation.rs` 7/7, measured 1.1e6 high in
an acetate buffer, unfixed. Two rules would otherwise have rested on it.
`trustworthy()` excludes it by name from any claim of the form "something
moved", so no generated case can be satisfied by that defect moving.
`Order`, whose claim is that a number came back the same, keeps it: an
invariance over a wrong number is still a true statement about path
independence.

