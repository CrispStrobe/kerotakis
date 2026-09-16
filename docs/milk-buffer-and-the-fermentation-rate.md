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
other and left both in place**, and the next person to fix the buffer would
have found the bench suddenly reading pH 6 for yoghurt. So the rate is fitted
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

Kim et al. attribute that split to **Salaün, Mietton and Gaucheron,
*Buffering capacity of dairy products*, International Dairy Journal
**15**(2) (2005) 95–109, doi:10.1016/j.idairyj.2004.06.007**, which is the
original measurement. **That paper was not read.** Elsevier answers an
automated request for it with HTTP 403, as does `journalofdairyscience.org`
for its own archive, and `pmc.ncbi.nlm.nih.gov` answers with a reCAPTCHA
page. What was read is Kim's rendering of it, and the registry citation says
"as reported by Kim et al. 2018" in those words rather than citing Salaün
directly.

So: **the engine models about 40% of milk's buffer capacity and is missing
about 60%** — caseins 35%, colloidal calcium phosphate 20%, whey proteins 5%.

That 60% is a **lower bound on the error in the region that matters**. Kim
et al. also report that "casein micelles display a maximum BC at pH 5.2 due
to CCP and phosphoserine residues". Milk starts at 6.7 and a fermentation
walks it down through 5.2, so between 6.6 and 5.0 the missing constituents
carry *more* than their global share and the soluble minerals carry less.
`crates/kerotakis-phreeqc/tests/milk_buffer.rs` has said this in prose since
the minerals were resolved; the 60% is the first number attached to it.

## How far it would move the pH

If buffer capacity were flat in pH, a model carrying 0.4 of the real β would
overshoot every pH excursion by 1/0.4 = 2.5×; equivalently, the real pH drop
at a given acid dose would be about **40% of the modelled drop**. It is not
flat, and it is *least* flat exactly where this matters, so treat the
following as an order of magnitude and a direction rather than a correction
to apply:

| | modelled | with the missing 60%, flat-β estimate |
|---|---|---|
| fresh milk | pH 6.6–6.8 | unchanged (it is the mineral buffer that sets it — `milk_buffer.rs` pins this) |
| `bio-069`, 8 h at 25 °C, at the **calibrated** rate | see the numbers table in PR #621 | roughly 0.6 × (6.7 − modelled pH) higher |

The honest summary is: **the computed pH of any acidified milk on this bench
is a lower bound, and the gap is of order one pH unit at yoghurt-like acid
doses, not of order a tenth.** It is not a tolerance; it is a missing model.

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
its own source. Salaün 2005 is very likely where they are, and Salaün 2005 is
behind a 403. Adding the extension with *assumed* constants would put an
editorial number where an editorial number was just removed, and would move
every computed milk and yoghurt pH in the repository behind it.

**Recommendation: a separate PR, gated on reading Salaün 2005 or an
equivalent.** It gets its own commit and its own test, as the milk-buffer
work already has one.

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
- **Not claimed, and the reason this file exists:** a yoghurt pH. It is a
  lower bound, by the 60% above.
