# The fermentation magnitude, and the buffer it must not be confused with

**Status:** the rates are **calibrated** as of 2026-09-16 (PR #621, following
#618). Two of the four are fitted to published fermentations; two are carried
across from the lactic fit and remain editorial. Milk's buffer is **not**
fixed here, deliberately, and this file says how big it is and what fixing it
would cost.

## Why these are two problems and not one

#618 fixed the fermentation rate's *scaling*: it read the grams of culture
where it should have read a concentration, so a doubled batch gave four times
the product. It carried the numeric constants across unchanged rather than
re-fitting them, which left `bio-069` — counter-top yoghurt, 8 h at 25 °C —
reading **pH 2.835** with 53.2% of the milk's lactose converted, against a
real yoghurt's 4.4–4.6.

There are two defects behind that number and they point in opposite
directions:

1. **The rate was too fast.** Nothing had ever been fitted; the constants
   were editorial classroom timescales implicitly sized for about 100 mL and,
   after #618, quoted per litre and used at a tenth of one.
2. **Most of milk's buffer is missing.** `whole_milk` resolves the serum
   minerals — K, Na, the soluble share of the Ca, Cl, inorganic phosphate,
   citrate — and conserves casein, milk fat, lactose and the colloidal
   calcium phosphate together as unresolved solids. A computed pH therefore
   falls *further* than a real one at the same acid dose.

Defect 1 pushes the pH down; defect 2 pushes the computed pH down further.
**A rate chosen to make pH 4.5 come out would have cancelled one against the
other and left both in place**, and whoever fixed the buffer afterwards would
have watched the bench climb back out of the window with nothing to tell them
why. So the rate is fitted
against a quantity that is itself a rate — lactose conversion against time —
and the pH is reported as an output.

## How big is the missing buffer

One cited number carries this, and it is cited at one remove, which is said
out loud below.

> "The relative contribution of milk constituents for the BC is ranked as
> follows: soluble minerals (40%), caseins (35%), CCP (20%), and whey
> proteins (5%)"
>
> — M. Kim, S. Oh and J.-Y. Imm, *Buffering Capacity of Dairy Powders and
> Their Effect on Yoghurt Quality*, Korean Journal for Food Science of Animal
> Resources **38**(2) (2018) 273–281, doi:10.5851/kosfa.2018.38.2.273,
> PMC5960825, CC BY-NC. Read in full 2026-09-16 via the Europe PMC REST
> `fullTextXML` service.

Kim et al. attribute that split to Salaün, Mietton and Gaucheron,
*Buffering capacity of dairy products*, International Dairy Journal **15**(2)
(2005) 95–109, doi:10.1016/j.idairyj.2004.06.007, which is the original
measurement. **That paper was not read.** Elsevier answers an
automated request for it with HTTP 403, as does `journalofdairyscience.org`
for its own archive, and `pmc.ncbi.nlm.nih.gov` answers with a reCAPTCHA
page. What was read is Kim's rendering of it, and the registry citation says
"as reported by Kim et al. 2018" in those words rather than citing Salaün
directly.

So: **the engine models about 40% of milk's buffer capacity and is missing
about 60%** — caseins 35%, colloidal calcium phosphate 20%, whey proteins 5%.

Kim et al. also report that "casein micelles display a maximum BC at pH 5.2
due to CCP and phosphoserine residues". Milk starts at 6.7 and a fermentation
walks it down through 5.2, so **between 6.6 and 5.0 the missing constituents
carry more than their global share** and the soluble minerals carry less.
`crates/kerotakis-phreeqc/tests/milk_buffer.rs` has said that in prose since
the minerals were resolved; the 60% is the first number attached to it.

**Do not turn that 60% into a correction factor.** The section below measures
what it is actually worth on this bench and gets an answer *smaller* than a
flat reading of it predicts, and the two are not in conflict: 60% is a share
of milk's buffer capacity, and what a reader wants is a shift in pH. Those
differ because capacity is not flat in pH, because the split is a
whole-titration ranking, and because the acid the fermentation makes buffers
too. **The percentage says which constituents are missing. The measurement
says what their absence costs.**

## How far it moves the pH — measured, not estimated

This is answerable without any new modelling, and the answer is an
**out-of-sample check** on the whole exercise: nothing in the fermentation was
fitted to a pH, so asking what this bench reads at the acid dose the *cited*
yoghurt carries is a fair test of the milk recipe alone.

Jankowska *et al.* fermented milk at 43 °C **to pH 4.6**, converting 6.106% of
its lactose. On the balanced homolactic route that is **3.5275e-3 mol of lactic
acid in 100 mL of this recipe's milk**. Added as acid, so the rate model plays
no part:

| | pH |
|---|---|
| fresh milk, this recipe | **6.564** (`milk_buffer.rs` pins 6.4–7.0; fresh cow's milk is 6.6–6.8) |
| the same milk carrying the cited yoghurt's acid | **3.944** |
| what Jankowska *et al.* measured with that acid in it | **4.6** |

**The missing buffer was worth about 0.66 of a pH unit at a real yoghurt's
acidity.** That is the number, measured. **Half of the buffer landed on
2026-09-18 and the table above is now history** — see *What changed on
2026-09-18* at the end of this file for what the same three rows read now,
and for why the new agreement is not a validated buffer.
`crates/kerotakis-phreeqc/tests/lactate_speciation.rs::the_cited_yoghurts_own_acid_and_the_casein_that_is_still_missing`
holds both.

**It is smaller than the 60% figure would suggest, and that is worth saying
rather than smoothing over.** Read as a flat coefficient, "the model carries
40% of the buffer" predicts it needing 2.5× less acid for the same drop; over
this interval it behaves more like 80%. Three things are tangled in that
difference and none of them is resolved here:

- buffer capacity is not flat in pH;
- the 40/35/20/5 split is a ranking over a whole titration, not the local
  share between 6.6 and 4.6;
- **the lactic acid itself buffers near its own pKa of 3.86**, which is exactly
  where this beaker lands — so part of what looks like milk's buffer in the
  model is the product's.

So the **direction and the order of magnitude are established** — the computed
pH of an acidified milk is low by a few tenths of a unit at yoghurt-like doses
— and the coefficient is not. It is not a tolerance; it is a missing model.

### And the number a learner actually meets

`bio-069` pours milk on a counter at 25 °C, far below the culture's 43 °C
optimum, so eight hours converts **1.48%** of the lactose and the beaker reads
**pH 5.44** — *above* real yoghurt, not below it, because it holds about a
fifth of the acid the fermentation it was fitted to makes. Both statements are
true at once and they must be read together:

- **the bench under-ferments** relative to a finished yoghurt, because this is
  a partial fermentation at the wrong temperature, and
- **the bench under-reads** the pH of whatever acid it does make, because most
  of the buffer is absent.

The two errors point in opposite directions. That is precisely why the rate was
fitted against a conversion extent and not against pH: a constant chosen to put
`bio-069` at 4.5 would have bought one right-looking number by setting these
two defects against each other.

## Could the engine model it, and what would it cost

Yes, and the mechanism already exists in the tree — but not cheaply enough to
belong in this PR.

**The phosphoserine half is small.** `kerotakis-phreeqc`'s `minteq_v4()`
already carries `LACTATE_EXTENSION`: one `SOLUTION_MASTER_SPECIES` line and
one protonation reaction with a `log_k`, inserted before the database's
trailing `END`. Casein's buffering between 6.6 and 5.0 is dominated by its
phosphoserine residues, and representing them the same way — one master
species at a stated concentration per litre of milk, one protonation `log_k`
— is the same shape of change, perhaps thirty lines including the test.

**The colloidal-calcium-phosphate half is not.** CCP is a solid dissolving as
the pH falls, so it is a phase equilibrium coupled to the solution, not a
protonation. The solver already precipitates octacalcium phosphate out of
this recipe (see `whole_milk`'s lot assumptions and
`crates/kerotakis-phreeqc/tests/calcium_phosphate.rs`), so the machinery is
there, but getting the *amount* right means resolving how much of milk's
calcium and phosphate is colloidal — which is a change to the recipe's
composition, not to the database.

**What actually blocks it is numbers, not code.** Both halves need cited
values this session does not have: the phosphoserine phosphorus per litre of
whole milk and its pK, and the colloidal share of calcium and phosphate with
its own source. The closest thing already in the tree is `whole_milk`'s own
statement that the casein micelle carries "of the order of 10 mmol/kg of
negative charge at pH 6.7" — which fixes roughly how many sites there are but
says nothing about where they titrate, and a buffer is the second of those
rather than the first. Salaün 2005 is very likely where they are, and Salaün 2005 is
behind a 403. Adding the extension with *assumed* constants would put an
editorial number where an editorial number was just removed, and would move
every computed milk and yoghurt pH in the repository behind it.

**Recommendation: a separate PR, gated on reading Salaün 2005 or an
equivalent.** It gets its own commit and its own test, as the milk-buffer
work already has one.

**The owner ruled otherwise on 2026-09-18**, and what happened is below.

## What is claimed after this PR, and what is not

- **Claimed:** the lactic and alcoholic rate constants reproduce one
  published fermentation each, at the declared optimum, for a declared dose.
  See `kerotakis/fermentation-rate-calibration-v1` in
  `data/registry/registry-source-v1.json` for the sources and the arithmetic.
- **Not claimed:** any rate *per gram* of these cultures. None of the four
  recipes names a strain, a product form or a cell count for a specific
  activity to belong to. The dose is declared, exactly as
  `REFERENCE_VOLUME_LITRES` is declared.
- **Not claimed:** the shape of the curve. The model is first order in the
  culture and first order in the substrate; real cultures grow and their
  sugar uptake saturates. One point is matched.
- **Not claimed:** a stopping point. There is no product inhibition, so the
  modelled extent goes to one and a long enough wait converts all of the
  lactose at a pH no yoghurt has ever had.
- **Not claimed, and the reason this file exists:** a yoghurt pH. It was a
  lower bound, by about 0.66 of a unit at yoghurt-like acidity, measured.
  Since 2026-09-18 half of that buffer is modelled and half is not, so the
  claim is narrower and stranger: see *What changed on 2026-09-18* below.
  The computed pH is no longer a plain lower bound above pH 5 and is a worse
  one below it.
- **Not claimed:** that `bio-069` is a yoghurt at all. It is eight hours of
  souring at 25 °C with a 43 °C culture.
- **Not claimed at all, for two of the four:** the acetic and heterolactic
  constants are the lactic fit carried across. No measurement of an
  acetification rate per gram of vinegar mother, or of a sourdough rate per
  gram of starter, was found. Their `rate_evidence.method` reads `editorial`
  where the other two read `derived`, which is the whole reason that field was
  added.

## What changed on 2026-09-18

The owner ruled: **do the phosphate half now, leave the casein residual
recorded, and name the residual on the wire.** Waiting for both halves was
offered and not chosen, and so was declaring milk out of scope. The
non-negotiable came with it — *a half-corrected buffer that reads as a
fully-corrected one is worse than the uncorrected one, because nobody
re-checks a number that looks right.*

### What was done

`whole_milk` books **183.45 mg of octacalcium phosphate per 100 g** —
`Ca4H(PO4)3:3H2O`, 0.3667 mmol of the database's half formula unit — as a
**solid**. That is milk's colloidal calcium phosphate, and it dissolves as
acid arrives, taking five protons per formula unit with it.

The reading above said this half was blocked on "resolving how much of milk's
calcium and phosphate is colloidal". It was, and the resolution is a choice
between two budgets the recipe already contains, which is written into the
recipe's own lot assumptions rather than hidden:

- **the phosphorus budget, which is what is booked.** FDC 746782 gives 101 mg
  of total P per 100 g; 34.1 mg of it is already booked as the diffusible
  inorganic share, which the recipe calls "roughly one third" of the total;
  the colloidal inorganic third is the same size, so 34.1 mg of it goes into
  the mineral and the last third is left where it is, esterified onto casein
  and onto the phospholipids.
- **the calcium budget, which is not.** FDC's 123 mg of total Ca less the
  40 mg diffusible share leaves 83 mg, which as octacalcium phosphate is
  about 280 mg of mineral. It assumes all of milk's colloidal calcium is in
  the mineral, and it is not — part of it is bound directly to casein's
  phosphoserine and carboxyl groups and is not a phosphate phase at all.

**The two land 0.47 of a pH unit apart at a yoghurt's acidity**, 4.60 against
5.07, which is the same size as the error the addition repairs. Neither is a
citation for the quantity that actually decides it — how much phosphate the
colloid releases between pH 6.6 and 4.6 — and that is a titration curve. It
is the same titration curve the casein half is waiting on, and Salaün 2005 is
still behind a 403. **So "the data is already in the repo" is true of the
composition and false of the buffer**, and this file would rather say that
than let the new number look derived.

### What it reads now

| | before | after | measured |
|---|---|---|---|
| fresh milk | 6.5636 | **6.5636** | 6.6–6.8 |
| fresh milk, ionic strength | 0.07412 | **0.07412** | 0.073 (Holt's diffusate) |
| the cited yoghurt's acid | 3.9441 | **4.6000** | 4.6 |
| 0.5 mmol HCl in 100 mL | 6.0668 | **6.0668** | — |
| 10 mL of 5% vinegar | 4.1156 | **4.4138** | — |
| `bio-069`, 8 h on a counter at 25 °C | 5.44 | **5.80** | — |
| the colloid in the glass | 37 mg | **227 mg** | ~280 mg in real milk's micelle |

**Fresh milk did not move, and that is the property that made this possible.**
The serum this recipe books is already at octacalcium phosphate's saturation
— it is why the solver was already laying about 37 mg of the stuff down — so
adding more of the same solid cannot move a dissolved amount. A
charge-neutral mineral at its own saturation disturbs nothing until an acid
comes for it, which is exactly the behaviour milk's colloid has. The serum
the solver hands back is unchanged at Ca 8.1 and inorganic phosphate
10.1 mmol per kg of water, so every check in the recipe's
*booked serum against the measured one* assumption still reads as it did.

The 0.5 mmol HCl row did not move either, for the same reason: the solid is
present before and after, so the pH sits on the saturation surface both
times. The buffer is only visible once the acid is enough to consume it.

### Why 4.60 against a measured 4.6 is NOT a validated buffer

Three things are true at once and the third is the important one:

1. **Casein is still modelled by nothing**, and Kim *et al.* rank the caseins
   at about 35% of milk's buffer capacity, second only to the soluble
   minerals.
2. **The modelled colloid is spent at very nearly this acidity.** The dose
   curve reads 5.21 at three quarters of the acid with 4% of the solid left,
   then 4.60 with none, then 4.29 and 4.10 and 3.85 as the acid goes on. The
   agreement sits on the shoulder of an exhaustion rather than in the middle
   of a buffered region.
3. **The amount of colloid is uncertain by 40%**, as above, which is 0.47 of a
   unit at this dose.

So the honest reading is: **above pH 5 this recipe now has the buffer that
matters, with a stated uncertainty; below pH 5 it has nothing, where a real
beaker still has casein's carboxyl groups.** The computed pH is no longer a
plain lower bound in the buffered region and is a worse lower bound outside
it, because there is nothing left to resist.

`lactate_speciation.rs` asserts both halves of that — a window around 4.6,
and a fall past the colloid — and the second assertion is the residual made
visible rather than argued from a percentage.

### The residual, named on the wire

The owner's condition. Every vessel of this milk that goes below pH 6.0 now
carries a `NotYetModeled` event whose `Phrase`
(`not-modeled.milk-casein-buffer-residual`) says which half is modelled,
which is not, roughly what the missing half is worth, and where the two swap
over. In German, from `crates/kerotakis-core/i18n/de.toml` and with no code
of its own:

> diese Milch wurde auf pH 4,60 angesäuert, und nur die Hälfte ihres Puffers
> ist modelliert. Ihr kolloidales Calciumphosphat ist vorhanden und geht in
> Lösung, sobald die Säure eintrifft; die Pufferwirkung des Caseins fehlt
> dagegen vollständig, und sie macht etwa ein Drittel der Pufferkapazität von
> Milch aus. Oberhalb von pH 5 kostet das Fehlende wenig; darunter ist das
> Kolloid aufgebraucht und diesem Becherglas bleibt nichts mehr
> entgegenzusetzen, während ein wirkliches noch das Casein hat — der Wert ist
> also wieder eine untere Schranke, und zwar eine schlechtere als zuvor.

`milk_buffer.rs::the_casein_residual_is_named_on_the_wire_in_both_languages`
asserts it in both languages, because *named on the wire* means named to the
reader and not to the English-speaking reader.

### What the rate calibration is owed

Nothing. The rates were fitted against a conversion extent and not against a
pH, precisely so that moving the buffer would move the pH and leave the rates
alone — and it did. `bio-069` went 5.44 → 5.80 because the beaker now
resists, not because anything about the culture changed, and the fermentation
tests' windows moved with it rather than the constants.
