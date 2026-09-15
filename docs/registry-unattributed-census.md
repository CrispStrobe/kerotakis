# The 105 unjudged registry citations, separated

Status: 2026-09-15. A measurement, not a repair. Nothing in this pass changes
the engine, the data, `provenance/upstreams.toml` or any gate. Defects found
while measuring are written down here rather than fixed.

`crates/kerotakis-cli/src/upstreams.rs` prints, on every run, the size of its
own blind spot: **105 of the 185 registry source citations name no audited
upstream at all.** Its module docs say what that number is not — "they are not
clean; they are unjudged" — and item 2 of "what must be true before this
becomes a gate" makes shrinking it a precondition. Nobody had looked at what
the 105 *are*.

They are three different things, and the right answer differs completely
between them. This page separates them.

## The counts

| | citations | what it is | what closing it would cost |
|---|---|---|---|
| **1 — nothing to chase** | **3** | The engine computed the number, or the record is the project's own declared editorial surrogate. A citation naming no upstream is *correct* here. | Nothing to buy. A `role: "derived"` marking would make it legible to a lint. |
| **2 — merely unrecognised** | **32** | A real source *is* named, and the `names` lists in `upstreams.toml` do not match the spelling. A defect in the audit table, not in the data. | 5 citations: one line of TOML. 27 citations: one terms reading each, across 8 new sources. |
| **3 — real gap** | **70** | At least one shipped number whose citation names nothing that could be checked. | 120 numbers, 65 species. Re-sourcing, not renaming. |

105 total. A generous reading of the ambiguous rows (below) moves six citations
from 3 to 1 and three from 3 to 2, giving **6 / 35 / 64**; the direction of the
uncertainty is one-way and small.

Under those 105 citations hang **487 numeric records**. Their own split is the
more useful number, because it is the unit a fix would act on:

| | records | |
|---|---|---|
| declared non-claims | **228** | a zero heat capacity meaning "not modelled", the documented 1.0 g/mL solvent/gas density placeholder, a boolean model switch, a zero tint strength meaning "colourless" |
| computed or declared editorial | **39** | Kopp's-rule heat capacities, molar masses from formula stoichiometry, 5R/2 for a monatomic gas, declared teaching approximations |
| named, but by an unrecognised spelling | **100** | overwhelmingly `molar_mass` from IUPAC/CIAAW 2021 atomic weights |
| **named by nothing** | **120** | 54 densities, 50 heat capacities, 3 molar masses, 3 solubilities |

**Fewer than a quarter of the numbers under the blind spot are actually
unattributed.** Just under half are declarations that no claim is being made at
all.

## Method

Four steps, all of them file-reading; no `cargo` was run.

1. **Reproduce the 105 exactly.** `named_by`, `names_source` (case-insensitive,
   whole-word-anchored at both ends) and `flatten` were reimplemented from
   `upstreams.rs` in Python and run against `provenance/upstreams.toml` and
   `data/registry/registry-source-v1.json`. The result is 185 scanned, 59
   findings, 105 unjudged — the module docs' figures to the digit. Everything
   after this rests on that agreement.
2. **Attach every numeric record to its citation.** Each record in
   `phase_thermodynamics` and `model_parameters` carries
   `quantity.source_id` and `quantity.method.{kind, detail}`. `method.kind` is
   one of `measured | calculated | derived | imported | editorial | curated`
   (`crates/kerotakis-data/src/schema.rs:1043`). 487 such records point at the
   105.
3. **Classify each record, not each citation.** In order:
   - the record is a **declared non-claim** if the value is 0.0 (not modelled /
     colourless), or is a `mass_density` of exactly 1.0 (the placeholder the
     citations themselves describe: *"density is the bench's shared gas
     placeholder of 1.0 g/mL … no run reads this number"*), or is a boolean
     model switch such as `dissolves-without-speciation`;
   - else **derived** if `method.kind` is `derived`/`calculated`;
   - else the citation is split into clauses on `;` and sentence boundaries and
     the clause covering *this property* is located by keyword (`M from` /
     `molar mass` / `atomic weight` for molar mass, `Cp` / `heat capacit` for
     heat capacity, `densit` for density, `ε` / `absorptivity` / `colour` for
     tint strength, `solubilit` for solubility). That clause is then read for a
     **named source**, a **computed route**, a **declared editorial
     approximation**, or a **hedge**;
   - a property with no clause of its own is a **gap**. A citation-wide source
     name is deliberately *not* allowed to cover it — that fallback is exactly
     the laundering the lint warns about, and letting it through inflated
     category 2 by fifteen citations in an earlier pass of this census.
4. **Roll up to citations.** A citation is category 1 if every record under it
   is a non-claim, computed or declared; category 2 if the rest are all named;
   category 3 otherwise. So **a citation lands in category 3 if even one number
   under it is unattributed** — the conservative direction.

The classifier, its two readings, and the per-record verdicts are reproducible
from the description above; the full membership lists are in the appendix so a
reader can disagree row by row rather than in general.

## Category 1 — nothing to chase (3)

`kerotakis/aqueous-basis-v1`, `legacy/OH-`, `kerotakis/material-recipes-v1`.

`kerotakis/aqueous-basis-v1` says it plainly: *"Kerotakis analytical aqueous
basis: formula stoichiometry and existing registry atomic mass basis. Aqueous
partial heat capacities are not modeled (zero); density is an unused
placeholder because solvent carries volume. No equilibrium or kinetic constants
introduced."* All 68 of its records are `method.kind = derived` or zero. There
is no upstream, because nothing came from one.

`legacy/OH-` is the same shape at species scale: Cp 0.0, density 1.0, one
boolean. Its molar mass is not a record at all.

`kerotakis/material-recipes-v1` is here by hand rather than by the classifier,
which could not place it: it owns 493 evidence links across 132 recipes and
*none* of them is a `phase_thermodynamics` or `model_parameters` row. Every one
is `curated`/`editorial` with a detail that declares itself — "fixed
teaching-surrogate composition", "unbranded household teaching surrogate with
an explicit concentration", "room-temperature teaching-surrogate density". A
70% isopropanol recipe is 0.7/0.3 because the project chose 0.7/0.3. That is
category 1 — with the caveat below.

## Category 2 — merely unrecognised (32)

Two quite different sub-cases, with costs an order of magnitude apart.

**2a — a spelling miss against a source already in the table (5 citations).**
`phreeqc-databases` is a *cleared* row whose `names` list reads
`["PHREEQC", "phreeqc.dat", "wateq4f.dat", "minteq.v4.dat", "llnl.dat", "USGS"]`.
Five citations name those files without the `.dat` — `legacy/atacamite`,
`brochantite`, `antlerite`, `langite` and `C6H5O7-3` all say *"wateq4f and
minteq.v4"*. `names_source` anchors on whole words at both ends, so the needle
`wateq4f.dat` does not occur in the haystack `wateq4f and`. Five citations are
invisible to the audit table because of a four-character suffix. This is the
cheapest defect in the set and the purest example of the category.

**2b — a real, identifiable source with no row at all (27 citations).** Eight
distinct sources, none of them in `upstreams.toml`:

| source | citations | example |
|---|---|---|
| IUPAC/CIAAW 2021 atomic weights | 93 (32 decisive) | *"M from IUPAC/CIAAW 2021 atomic weights"* — nearly every `legacy/*` row |
| primary journal papers | 5 | `legacy/betanin`: *"ε: Stintzing & Carle, Trends in Food Science & Technology 15 (2004) 64-72"*; `literature/hartley-campbell-iodine-water`: a 1908 J. Chem. Soc. paper cited with a DOI |
| NBS/Wagman 1982 | 2 | `legacy/ZnSO4`: *"dissolution enthalpy -80.4 kJ/mol derived from NBS/Wagman 1982 formation enthalpies (ZnSO4(s) -982.8, Zn2+(aq) -153.9, SO4 2-(aq) -909.3 kJ/mol)"* |
| CODATA | 1 | `legacy/water`: *"Cp(l), density: CODATA/standard reference values"* |
| USDA FoodData Central | 1 | `kerotakis/material-recipes-v1`, quoting FDC 746782 with a retrieval date |
| ACS / FAO / JCE / extension services | 1 | same citation |
| a textbook (Voet & Voet) | 1 | `legacy/amylase`: *"Alpha-amylase MW ~55 kDa (typical); Voet & Voet Biochemistry 4th ed."* |

The 93 IUPAC/CIAAW citations are only *decisive* for 32, because in the other
61 the molar mass is properly cited and the Cp or density in the same string is
not — those land in category 3 on their weakest number, which is the rule
working as intended.

`literature/hartley-campbell-iodine-water` is the sharpest case in the whole
census. It is the best-sourced row in the registry — author, title, journal,
volume, year, pages, DOI, the measured value, the conversion, and a CC0
release of the transcription — and the audit table cannot see it.

## Category 3 — real gap (70)

120 numbers over 65 species. The shape is monotonous, which is the useful part:

- **54 densities and 50 heat capacities**, almost all on solids, almost all
  attributed to the phrase *"Cp(s), density: standard reference values"*. That
  string says a source exists and declines to say which. `legacy/NaCl` — Cp
  50.5 J/(mol.K), density 2.17 g/mL — is the template, and 40-odd rows repeat it.
- **Self-declared gaps**, already flagged in their own citations and needing
  only to be counted, not discovered: `legacy/dry_ice` (*"PENDING REVIEW: no
  positively identified page was opened for either"*), `legacy/activated_charcoal`
  (*"recorded as commonly tabulated, no positively identified source is
  claimed"*), `legacy/hair_pigment` (*"IS PENDING REVIEW"*).
- **Three molar masses**: `legacy/catalase` (240 kDa, *"bovine liver catalase,
  standard reference"*), `legacy/amylase` (55 kDa, textbook), and
  `legacy/anthocyanin` (448.38, from a formula the citation calls "accepted"
  without saying accepted by whom).
- **Three solubilities**: `legacy/CaCO3` 0.0013, `legacy/S` 1e-5,
  `legacy/naphthalene` 0.003 g/100 mL — the last described as *"the reviewed
  room-temperature figure"*, reviewed by nobody named.

None of these is a *refused-source* problem. Every one of them is the older
problem the refused-source lint cannot reach: a number with no origin recorded.

## Would the count move the wrong way?

The brief named the trap: closing category 2 makes more strings legible, so the
lint's finding count could rise as its blind spot falls. It was measured rather
than argued, by re-running the reproduced lint over both surfaces with the
audit table amended.

| amendment | registry findings | unjudged | Rust findings |
|---|---|---|---|
| baseline, as shipped | 59 | 105 | 19 |
| 2a only — `phreeqc-databases` names widened to `wateq4f`, `minteq.v4` | **59** | **100** | 19 |
| 2a + 2b, every new row cleared | **59** | **6** | 19 |
| 2a + 2b, the textbook row judged `avoid` like the CRC Handbook | **60** | **6** | 19 |
| 2a + 2b, IUPAC/CIAAW judged `decision-required` | 59 | 6 | 19 |

**The headline count does not move the wrong way, with one exception worth one
finding.** Closing category 2 in full takes the blind spot from 105 to 6 and
leaves the finding count at 59, because every source category 2 names is a
public-domain body, a government work, or a properly cited primary paper —
none of them belongs on a refused row.

The exception is `legacy/amylase`, whose citation names a copyrighted textbook.
If a `voet-biochemistry` row were added and judged like `crc-handbook`, the
count goes 59 → 60. That is the count getting *worse because the data got more
legible*, exactly as predicted, and it is one row. It is also arguably not a
finding at all under the owner's 2026-09-14 ruling recorded in
`upstreams.toml`: *"we should be able to cite any book, only not harvest the
books per systematic scraping"*. One molar mass with author, title and edition
is citing a book. Whoever adds that row decides that, and should decide it
before adding it rather than after seeing the count move.

The third row is the one to watch for a different reason. If IUPAC/CIAAW is
judged `decision-required` rather than cleared, the headline stays at 59 —
the lint deliberately reports open questions apart from offences — but the
open-question tally goes from 0 to **152** on the registry and 1 on Rust,
because 93 citations name it. That is not a regression; it is a large number
appearing in a column that has always read zero, and it would be read as one
unless the person who adds the row says in advance that it will happen.

The Rust surface was reproduced with an independent scanner and reads 192
value-bound strings and 19 findings on `main` at `da4adcc1`, against the module
docs' 210 and 98 at `8c2d03f6`. The difference is the Rust sweep that was in
flight when the lint landed, not a disagreement about method; the registry
figures, which this census actually rests on, reproduce exactly.

## Recommendation

**Buy 2a. Buy 2b. Do not buy a category-3 sweep, and do not reverse the
2026-09-14 decision.**

| | scope | estimated cost | what it buys |
|---|---|---|---|
| **2a** | add `wateq4f`, `minteq.v4` (and, while there, the bare `llnl` and `phreeqc.dat` stems) to one existing `names` list | **minutes**, one line, no licence work | blind spot 105 → 100; removes a false negative in a *cleared* row, which is the kind that hides rather than the kind that shouts |
| **2b** | eight new `[[upstream]]` rows: CIAAW, CODATA, NBS/Wagman, USDA FoodData Central, the primary-literature route, ACS/JCE/FAO educational material, and a decision on the textbook | **one terms reading each, half a day**, plus the judgement on Voet & Voet and on CIAAW | blind spot 105 → 6. This is what makes item 2 of the gate preconditions satisfiable at all |
| **3** | 120 numbers over 65 species, each needing a traceable origin found, checked and written | **weeks**, and it is the accuracy corpus's job, not a lint's | correctness, eventually — but bought one number at a time, not as a sweep |

The reasoning for the split is that 2a and 2b fix the *instrument* and category
3 is the *reading*. The audit table is currently unable to see a correctly
cited 1908 paper with a DOI, or the atomic-weight table behind nearly every
molar mass in the registry. Until that is fixed, the 105 cannot be used as
evidence of anything, and item 2 of the gate preconditions cannot be met by any
amount of work on the data.

Category 3 is a genuine 70-citation, 120-number debt and it should stay
*counted and reported*, which is what the owner decided. Three reasons not to
sweep it:

- **It is not the lint's debt.** Nothing in category 3 names a refused source.
  A sweep would be sourcing work booked against a licensing gate, and would
  finish with the same numbers in the registry and a different sentence beside
  them.
- **The unit is wrong.** A citation is not a number. `legacy/NaCl`'s molar mass
  is properly sourced and its density is not, in one string; a sweep that
  operates on citations either over-claims or rewrites 65 strings to say what
  the proposed `upstreams: [{id, role, covers}]` field would say structurally.
  Build the field first, or the sweep is work that has to be redone.
- **Half of it may not be numbers anyone reads.** 228 of the 487 records under
  the 105 are declared non-claims, and several citations state outright that
  nothing reads the value (*"no run reads this number"*). A sweep should be
  ordered by whether a number reaches an answer, and that ordering does not
  exist yet.

What would change this recommendation: if the accuracy work needs `Cp(s)` and
density on the common solids to be defensible — and those are 104 of the 120 —
then the sweep is worth buying *as accuracy work*, in the order the corpus
needs them, and the provenance count improving is a side effect rather than the
reason.

## Defects found and deliberately not fixed

1. **`wateq4f.dat` / `minteq.v4.dat` in the `phreeqc-databases` `names` list**
   do not match the `wateq4f` / `minteq.v4` spelling five registry citations
   actually use. Add the bare stems. (`provenance/upstreams.toml`.)
2. **`kerotakis/material-recipes-v1` is the coarsest citation in the
   registry**: one free-text string bearing 493 evidence links across 132
   recipes, and it spans all three categories at once — declared surrogate
   compositions (1), USDA/Gaucheron/Le Graët figures named but unrecognised
   (2), and unattributed numbers such as the 0.90 and 0.80 g/mL bulk densities
   for candle wax and paper and "roughly three quarters silica" for flat glass
   (3). It is classified 1 here on the strength of every record's own
   `method.kind`, which is the best available evidence and is not the same as a
   claim that all 493 are fine. This single row is the strongest argument for
   the structured `upstreams[]` field the lint's module docs propose.
3. **The brief's pointer is off by a crate.** The rule that every numeric
   record names a resolvable source id is at
   `crates/kerotakis-data/src/validate.rs:1314`, in `fn evidence`, not in
   `kerotakis-phreeqc`. `kerotakis-phreeqc` has no `validate.rs`.
4. **`legacy/water` is the one borderline in category 2.** Its citation reads
   *"Cp(l), density: CODATA/standard reference values"* — CODATA is named, and
   the slash makes it unresolvable which of the two numbers came from CODATA
   and which from the unnamed tables. Counted as 2; a reviewer could defensibly
   call it 3, which would make the counts 3 / 31 / 71.
5. **`legacy/KMnO4` and `legacy/NaHCO3` carry known-wrong dissolution
   enthalpies**, already named as unresolved in
   `crates/kerotakis-registry-export/src/lib.rs` (`DISSOLUTION_CITATION`):
   +16.2 against a commonly tabulated +43.6, and +16.7 against both +18.7 and
   +17.5. Out of scope here and noted only because a category-3 sweep would
   walk straight into them.

## Appendix — membership

**Category 1 (3):** `kerotakis/aqueous-basis-v1`, `kerotakis/material-recipes-v1`, `legacy/OH-`

**Category 2 (32):** `legacy/Ag+`, `legacy/Ba+2`, `legacy/Br-`, `legacy/C6H5O7-3`, `legacy/CH3COO-`, `legacy/Ca+2`, `legacy/Cl-`, `legacy/Cu+1`, `legacy/Cu+2`, `legacy/Fe+3`, `legacy/H2PO4-`, `legacy/HCO3-`, `legacy/K+`, `legacy/Mg+2`, `legacy/Mn+2`, `legacy/Mn+3`, `legacy/MnO4-`, `legacy/MnO4-2`, `legacy/NH4+`, `legacy/NO3-`, `legacy/Na+`, `legacy/PVA`, `legacy/Pb+2`, `legacy/SO4-2`, `legacy/Zn+2`, `legacy/chlorophyll`, `legacy/hair_pigment_ox`, `legacy/helium`, `legacy/nylon`, `legacy/paraffin`, `legacy/water`, `literature/hartley-campbell-iodine-water`

**Category 3 (70):** `legacy/Ag`, `legacy/AgCl`, `legacy/AgNO3`, `legacy/C`, `legacy/CH3COOH`, `legacy/CO2`, `legacy/CaCO3`, `legacy/CaCl2`, `legacy/CaO`, `legacy/Cl2`, `legacy/Cu`, `legacy/Cu(OH)2`, `legacy/CuSO4`, `legacy/Fe`, `legacy/FeSO4`, `legacy/H2`, `legacy/H2SO4`, `legacy/H3PO4`, `legacy/HCl`, `legacy/KCl`, `legacy/KMnO4`, `legacy/Mg`, `legacy/MgO`, `legacy/MgSO4`, `legacy/MnO2`, `legacy/N2`, `legacy/NH2Cl`, `legacy/NH3`, `legacy/NH4NO3`, `legacy/Na2CO3`, `legacy/NaCl`, `legacy/NaHCO3`, `legacy/NaOAc`, `legacy/NaOCl`, `legacy/NaOH`, `legacy/O2`, `legacy/Pb`, `legacy/Pb(NO3)2`, `legacy/S`, `legacy/SO2`, `legacy/Zn`, `legacy/ZnSO4`, `legacy/activated_charcoal`, `legacy/amylase`, `legacy/anthocyanin`, `legacy/antlerite`, `legacy/atacamite`, `legacy/betanin`, `legacy/betanin_ox`, `legacy/brochantite`, `legacy/bromothymol_blue`, `legacy/butane`, `legacy/catalase`, `legacy/curcumin`, `legacy/curcumin_ox`, `legacy/diamond`, `legacy/dry_ice`, `legacy/ethanol`, `legacy/graphite`, `legacy/gypsum`, `legacy/hair_pigment`, `legacy/hydrogen_sulfide`, `legacy/indigo_carmine`, `legacy/indigo_carmine_ox`, `legacy/langite`, `legacy/methyl_orange`, `legacy/naphthalene`, `legacy/phenolphthalein`, `legacy/propane`, `legacy/uranium`

The six that stay unjudged even after category 2 is closed —
`kerotakis/aqueous-basis-v1`, `legacy/anthocyanin`, `legacy/betanin_ox`,
`legacy/catalase`, `legacy/curcumin_ox`, `legacy/indigo_carmine_ox` — are the
irreducible floor: one is category 1 by construction, three are bleaching
products defined by stoichiometry from a parent, and two are category 3
awaiting a source that does not exist yet.

## Addendum, 2026-09-15 — the field this page asked for, half built

The measurements above are left exactly as they were taken. What follows is
what happened next, so a reader arriving at the recommendation does not go and
build something that now exists.

This page argued twice that the unit was wrong — "A citation is not a number"
— and recommended building `upstreams: [{id, role, covers}]` before any
category-3 sweep, "or the sweep is work that has to be redone". The `role`
half landed the same day.

**Where it landed, and why not where this page put it.** Not on the registry
source record: `data/registry/registry-source-v1.json` and its byte-exact
golden mirror belong to another change in flight. It landed in
`provenance/upstreams.toml` instead, as `[[citation]]` rows replacing
`[[excuse]]`, keyed by (surface, subject, upstream) with an optional
`matching` to narrow a Rust judgement from a file to a string.

**What that costs is `covers`, and it is the half this page cared most about.**
The role is declared per citation, not per quantity, so `legacy/HBr` — one
string over fifteen numeric records, which this census used as its own worked
example — can still only be judged as a whole. The finding still sits on the
citation. Defect 2 above, `kerotakis/material-recipes-v1` with its 493
evidence links under one string, is untouched for the same reason. **The
argument against a category-3 sweep therefore still stands in full**: build
`covers` first.

**The prediction held and the numbers behaved.** This page predicted the
textbook row would take the registry 59 → 60 and that the blind spot would go
105 → 6, and both did. The role vocabulary then moved the *Rust* surface 18 →
7 and left the registry at **60, unchanged**. That is the shape to expect: the
eleven that moved were all strings naming a refused source in order to reject
it, and the registry's 46 CRC citations read "CRC Handbook, 97th ed." beside a
quantity with no page, which is what bulk dependence looks like and does not
clear.

**On the question this page left open.** It said of `legacy/amylase` that "one
molar mass with author, title and edition is citing a book" and that "whoever
adds that row decides that". The row was added, judged `avoid`, and the
citation **stays a finding** — because it carries no page and calls its own
value "typical", so it fails the carve-out the rule itself states. The
vocabulary now distinguishes that case from a citation carrying author, title,
edition and page, which was the defect; it does not clear this one.

**One row this page's method would have caught and did not.** `legacy/Fe+2`
is classified category 2 above on the strength of its molar mass. Its CRC
mention is doing something the census had no column for: supporting a
*qualitative* fact — that iron(II) sulfate solutions are pale green — beside
Greenwood and Earnshaw, while the citation states that "no edition of any
handbook was opened for a per-wavelength epsilon and none is claimed". No
shipped number rests on the Handbook there. It is left undeclared rather than
judged `mentioned`, because a reviewer could defensibly say the colour claim
does rest on it, and that is a judgement about a citation rather than about an
instrument.
