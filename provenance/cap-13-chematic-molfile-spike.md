# CAP-13 spike — the chematic molfile bridge, and where the fix belongs

**Date:** 2026-08-30 · **Scope:** the eleven registry species CAP-13 deferred at
the 23→65 tranche, and the stereo/isotope loss BRD-010's two-route check
surfaced · **Deliverable:** a dependency-routing decision with its evidence.

## What the bridge was

`kerotakis-org`'s `native_inchikey_from_smiles` went

```
SMILES ──chematic::smiles::parse──▶ Molecule
       ──chematic::mol::write_mol──▶ V2000 molfile
       ──inchi::from_molfile───────▶ Standard InChI ──▶ InChIKey
```

The middle step is a lossy narrows. A `Molecule` parsed from SMILES carries a
formal charge, an explicit bracket-atom hydrogen count, an isotope, a
tetrahedral `Chirality`, and `/` `\` E/Z bond directions. It carries **no
coordinates**. What chematic 0.18's V2000 writer emits from that is the atom
symbol, the charge, the bond orders — and zeros in every other field.

## The four defects, read off the writer

Source: `chematic-mol-0.18.0/src/mol2000.rs`, `write_mol_with_coords`. The
same code is byte-identical at upstream `v0.23.0` and `HEAD`, so none of this
is fixed by a version bump.

1. **No valence field.** The atom line is written with a hardcoded `0` in
   `vvv`, which means "use the periodic-table default valence". `[Mg]` says
   *zero* hydrogens; the molfile says "default", and a reader that honours the
   default — the IUPAC reference implementation among them — reconstructs
   MgH₂. Same for `[Pb]`→PbH₂, `[C]`→CH₄, `[S]`→H₂S. **The V2000 valence field
   is the only channel the format has for this** (`15` is its spelling for
   zero valence).
2. **No isotope.** The mass-difference field is hardcoded `" 0"` and no
   `M  ISO` property line is ever written. Every isotopologue writes out as
   its natural-abundance parent.
3. **No stereo.** `Atom.chirality` has no molfile field at all in this writer
   (the atom-block parity field is hardcoded `0`), and with all coordinates at
   the origin there is no geometry for the library to read either. Every
   stereocentre disappears.
4. **E/Z direction mis-encoded as a wedge.** chematic represents SMILES `/`
   and `\` as `BondOrder::Up`/`Down`, and the writer maps those to the V2000
   *bond stereo* codes 1 (wedge) and 6 (hash) — which mean tetrahedral
   out-of-plane direction, not double-bond geometry. So the E/Z information is
   not merely dropped, it is re-spelled as a claim about a different kind of
   stereocentre.

Defects 3 and 4 are the `UHFFFAOYSA` signature BRD-010 found: the
second InChIKey block that says "no stereo, no isotope", on 23 of 100
records whose own PubChem InChI re-keys 100/100 clean.

A fifth, not on our critical path but worth an upstream note: the reader
skips every `M  ` property line, and `encode_charge` silently writes `0` for
any charge outside ±3 — so a molfile round-trip through chematic cannot carry
`M  CHG`/`M  ISO` either way.

## What the fix turned out to be

The official library does not require a molfile. `inchi` 0.1.4 exposes the
reference implementation's own **0D input structure** — `inchi::Molecule` with
`Atom::charge`/`isotope`/`implicit_hydrogens(ImplicitH::Exactly(n))`,
`add_bond`, and `add_stereo(Stereo::Tetrahedral{..} | Stereo::DoubleBond{..})`
— which is exactly the API for a structure that has no coordinates. Every one
of the four losses is a field that API has and the molfile does not.

So the bridge now goes SMILES → chematic `Molecule` → `inchi::Molecule` →
Standard InChI, with **no molfile in the path and no dependency change of any
kind**. See `crates/kerotakis-org/src/native_inchi.rs`.

The two parity conventions (`@`/`@@` → odd/even; cis/trans → odd/even) are
read off `inchi_api.h`'s own diagrams and then *confirmed against the corpus*:
the spike ran all four flip combinations over the 100-record PubChem fixture
and the documented reading is the one that agrees.

## The eleven deferred species, case by case

Every key below is computed, not remembered: the spike ran both routes over
the same curated SMILES against the vendored IUPAC library (`inchi` 0.1.4,
InChI v1.07.5). "molfile" is the bridge CAP-13 shipped; "direct" is the 0D
structure route.

| species | SMILES | registry key | molfile route | direct route | verdict |
|---|---|---|---|---|---|
| Mg | `[Mg]` | FYYHWMGAXLPEAU-…-N | RSHAOIXHUHAZPM (`1S/Mg.2H`) | **FYYHWMGAXLPEAU** (`1S/Mg`) | **fixed** |
| Pb | `[Pb]` | WABPQHHGFIMREM-…-N | FOSOXHMVHOGFCF (`1S/Pb.2H`) | **WABPQHHGFIMREM** (`1S/Pb`) | **fixed** |
| C | `[C]` | OKTJSMMVPCPJKN-…-N | VNWKTOKETHGBQD (`1S/CH4`) | **OKTJSMMVPCPJKN** (`1S/C`) | **fixed** |
| S | `[S]` | NINIDFKCEFEMDL-…-N | RWSOTUBLDIXVET (`1S/H2S`) | **NINIDFKCEFEMDL** (`1S/S`) | **fixed** |
| phenolphthalein | closed lactone | KJFMBFZCATUALV-…-N | **KJFMBFZCATUALV** | **KJFMBFZCATUALV** | **fixed — by curation** |
| Cu(OH)₂ | `O[Cu]O` | PTTPXKJBFFKCEK-…-N | JJLJMEJHUUYSSY-…-L | JJLJMEJHUUYSSY-…-L | registry-key candidate |
| MnO₄⁻ | `[O-][Mn](=O)(=O)=O` | VLTRZXGMWDSKGL-…-M | NPDODHDPVPPRDJ-…-N | NPDODHDPVPPRDJ-…-N | registry-key candidate |
| Pb²⁺ | `[Pb+2]` | XMOCLSLCDHWDHP-…-N | RVPVRDXYQKGNMQ (`1S/Pb/q+2`) | RVPVRDXYQKGNMQ | registry-key candidate |
| Pb(NO₃)₂ | `[Pb+2].2×NO₃⁻` | RLJMLMKIBZAXJO-…-**L** | RLJMLMKIBZAXJO-…-**N** | RLJMLMKIBZAXJO-…-**N** | registry-key candidate |
| methyl orange | Na⁺ salt | BSKHPKMHTQYZBB-…-**N** | STZCRXQWRGQSJD-…-**M** | STZCRXQWRGQSJD-…-M | structure-curation candidate |
| bromothymol blue | closed sultone | FBSFWRHWHYMIOG-…-N | WSPVEJBDABYTCR-…-N | WSPVEJBDABYTCR-…-N | structure-curation candidate |

**Five of the eleven are fixed** and joined `CURATED_STRUCTURES`. The other
six are *not* `write_mol` bugs at all — that was the 2026-08-24 diagnosis and
it is now retired. Both routes compute the same key for all six, which is the
proof: a writer defect would show as a difference between the columns. What
these six are is a disagreement between the curated structure and the stored
identity, and CAP-13's own contract already names that a curation bug. Four
look like the stored key (Cu(OH)₂, MnO₄⁻, Pb²⁺, Pb(NO₃)₂ — note the last one
agrees on connectivity and differs only in the final block, i.e. the
protonation flag); two look like the curated SMILES picking the wrong
tautomer/form (methyl orange, bromothymol blue). None of them can be settled
from inside this tree — they need an external identity source, which is
exactly what BRD-010's PubChem adapter is for. They stay deferred, with the
diagnosis corrected.

The three "aromatics awaiting kekulisation" were also misdiagnosed:
phenolphthalein's aromatic rings kekulise fine under both routes and its key
matches as soon as the curated SMILES is the closed lactone the registry
means. Kekulisation was never the blocker.

## What the spike found that nobody was looking for

`Al` was **already in** `CURATED_STRUCTURES` and the gate was green on it —
because the registry key `AZDRQVAHHNSJOQ-UHFFFAOYSA-N` is **alumane (AlH₃)**,
and the molfile bridge recomputed AlH₃ from `[Al]`. Two wrong answers agreed
and the gate certified the pair. The metal is
`XAGFODPZIPBFFR-UHFFFAOYSA-N` (PubChem CID 5359268, present in BRD-010's
pinned snapshot). The registry is corrected in this branch, and
`tests/native_identity.rs` pins both keys so the retired one is on record as
the hydride's.

This is the strongest argument in the whole spike for the change: the old
bridge did not merely fail loudly on eleven species, it *passed quietly* on
at least one wrong one.

## The stereo and isotope class

Corpus: the 100-record PubChem fixture BRD-010 pinned
(`crates/kerotakis-data/tests/fixtures/quarantine/pubchem-v1`), keys being
PubChem's own.

| route | records agreeing |
|---|---|
| molfile (shipped) | **73 / 100** |
| direct 0D structure | **100 / 100** |

The 27 the molfile route lost are exactly the stereo and isotope rows plus
the bare metals: both glucose anomers and L-glucose, sucrose, both alanines,
both lactic acids, all three tartaric acids (meso included — the case that
proves it is real stereo perception and not a coin flip), both limonenes,
both 2-butenes, L-ascorbic acid, D₂, T₂, D₂O, methanol-d₄, chloroform-d,
benzene-d₆, and Al/Mg/S/brass.

Nineteen of those are checked in as a standing gate,
`crates/kerotakis-org/tests/stereo_isotope_identity.rs`, together with the
mirror assertion that the molfile route loses every one of them.

The two parity conventions are read off `inchi_api.h` and then *confirmed*:
the spike ran all four flip combinations over the corpus and only the
documented reading scores 100/100 (`flip_tet` scores 86, `flip_ez` 98).

## Routing decision

See CAPABILITIES.md § CAP-13 for the recommendation and its costs.
