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

## What the corpus admits, and what it does not

Computed by `the_corpus_census_is_what_is_recorded` over all five hundred
prompts and checked in as `perturbation-census.json`, so a corpus edit that
puts a script beyond the generator's reach shows up as a diff with a number
on it rather than as silence.

| | prompts |
|---|---|
| admit at least one perturbation | **430** |
| admit none | **70** |

The seventy that admit none, with the reason:

| reason | prompts |
|---|---|
| no script: an explicit product boundary, nothing to run | 60 |
| no reagent: the script only observes or manipulates | 4 |
| a single reagent, no water, and a size-bound verb: nothing can be held fixed against the perturbation | 5 |
| *(the sixtieth boundary row counts once here)* | 1 |

The sixty are the `boundary` rows. They are not a failure of the generator:
a boundary row records a deliberate product refusal and has no script by
design, so there is nothing to perturb and nothing that should be.

**So the honest answer to "does this scale" is 430 of 500, and the 70 that
do not are 60 refusals plus 10 scripts too thin to hold anything fixed.**

By rule:

| rule | scripts admitting |
|---|---|
| dose | 398 |
| scale | 365 |
| ablation | 332 |
| solvent | 233 |
| order | 174 |

`order` reaches fewest because it needs two reagents to permute; `solvent`
needs water specifically. `dose` reaches most and is worth least.

## What the generator found

### `solution.pe` is path-dependent by 12.8 units where nothing constrains it

`th-100` — "will a smaller headspace reach more pressure from the same
generated gas?" — is bicarbonate, hydrochloric acid, a sealed 200 mL
headspace and a pressure gauge. Swap the two reagents and:

```text
                          pH        pe       ionic strength     pressure
  NaHCO3, then HCl     5.555778   12.780243     0.497017        101324.90
  HCl, then NaHCO3     5.555338   -0.055944     0.497017        101324.99
```

pH agrees to four decimal places, ionic strength to six, the pressure gauge
to a part in a million — and `pe` differs by 12.84, which is about 760 mV of
redox potential. Neither vessel contains a redox couple: `solution.redox` is
`[]` in both. The number is unconstrained, the solver returns whatever its
path left behind, and the `--json` contract publishes it as the vessel's pe
with nothing to say it means nothing.

This is the same shape as the recorded `contents["OH-"]` defect and it was
found the same way: a published quantity that does not mean what its name
says, invisible to any single run because −0.06 and 12.78 are both perfectly
plausible numbers, and obvious the moment its cause is perturbed.

### `solution.solvent_kg` carries an order-dependent residue that `contents[water]` does not

`aq-023` — calcium chloride and powdered detergent in 100 mL of water. Swap
the reagents:

```text
                       contents[water|liquid]    solution.solvent_kg
  CaCl2 first            5.5339424445 mol           0.0997010580 kg
  detergent first        5.5339424570 mol           0.0996909590 kg
```

The inventory agrees to one part in 4e8. The solvent mass the solution is
characterised against disagrees by one part in 1e4 — about 10 mg of water in
100 g. Every molality is divided by that number, so the discrepancy
propagates into the whole speciation, where the wire's four significant
figures hide it. Two surfaces of the same quantity, disagreeing; the
generator compares them because it compares everything.

### Order dependence in an open beaker is chemistry, and the test now proves it

The first two order departures were `mat-086` (limewater and carbon dioxide)
and `mat-124` (bicarbonate and vinegar), both open beakers, both carbonate.
`mat-086` measured:

```text
                        pH      CaCO3 (mol)   Ca(OH)2 left (mol)
  lime, then CO2      9.898       0.009988         none
  CO2, then lime     12.452       0.003394         0.004605
```

Two and a half pH units and a factor of three in the precipitate — a
different answer to "can carbon dioxide turn limewater cloudy?". It is also
correct: an open beaker is not a closed system, and the CO₂ poured in first
has somewhere to go before the lime arrives. **Seal the same vessel and the
two orders agree** — pH 9.109228 against 9.108648, precipitate to one part in
1e9.

`closing_the_vessel_restores_order_independence` asserts exactly that, and it
is the sharpest shape in the file: the same perturbation into two systems,
with responses that must differ by a stated factor. It converts an excuse
into a claim. If the engine ever grows a genuine path memory, this separates
it from the open-vessel effect rather than letting the atmosphere take the
blame.

### `aq-018` cannot answer its own question, and the corpus already knew

"Can a spoonful of sugar disappear into water?" — `water 100mL; sucrose 10g;
stir`. Delete the sucrose and **not one readout moves**: not pH, not ionic
strength, not the temperature, not the words the scene shows. The bench
cannot tell sugar-water from water on any surface it exposes.

The row is tagged `substance-gap`, so this is a known limitation rather than
a discovery — but it is a *smoke* prompt, `expected = "computed"`, and it has
been green for as long as the corpus has existed. The route it was answered
by is the thing the corpus checks. Whether the answer depends on the sugar is
the thing it could not.

