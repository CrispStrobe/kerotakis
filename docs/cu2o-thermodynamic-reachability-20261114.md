# Copper(I) oxide thermodynamic reachability — 2026-11-14

The preserved th-042 failure at source `5122af69e` brackets the CuO(cr)
record's 1400 K upper endpoint: TP enthalpy jumps from about 6288 J to
8800 J around a 6912 J HP target. The former HP stopping rule returned a
pure endpoint without closing that balance. Its apparent success was not
an energy-conserving qualification; the energy acceptance bound remains
1 J absolute plus 1e-8 relative.

The registry now names Cu2O, allowing the existing NASA Cu2O(cr) and
Cu2O(L) records into the same formula-derived product pool. This adds no
out-of-range product continuation. The original case must still earn its
actual full-pool HP result, element balance and named inventory readback.
Source review or a dominant-species estimate is not execution acceptance.

## Property authority

- Identity: official PubChem PUG REST CID 10313194, checked 2026-11-14,
  [formula and InChIKey](https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/cid/10313194/property/InChIKey,MolecularFormula,IUPACName/JSON).
  The ionic Cu(I) identity is `KRFJLUBVMFXRPN-UHFFFAOYSA-N`.
- Mass, Cp curves, absolute H/G and phase boundary: the unchanged vendored
  `vendor/nasa-cea/thermo.inp` Cu2O(cr)/Cu2O(L) records, citing King (1973),
  p.60, under the existing NASA Apache-2.0 source. Solid intervals are
  300–600 and 600–1516.7 K; liquid is 1516.7–6000 K.
- The stocked crystal Cp scalar is the NASA value at 300 K, held down to
  room temperature by the existing core model. It is not a measured
  298.15 K value. The liquid curve has its own phase and domain.
- Specific gravity 6.14 and red appearance: J. Michael Howard,
  [Arkansas Geological Commission Bulletin 23](https://geology.arkansas.gov/docs/pdf/publication/bulletins/bulletin-23.pdf),
  1987/revised 2007, printed p.36, opened 2026-11-14. The density scalar
  6.14 g/mL is an approximate conventional conversion, with no
  temperature-resolved density measurement claimed. No spectrum or
  reflective RGB was invented.
- Core fusion is derived from the SAME NASA enthalpies at 1516.7 K:
  64.236585445 kJ/mol. Their Gibbs difference there is below 0.001 J/mol.
  This is a model-consistent enthalpy difference, not a new measurement.

The authored property summary's AGPL licence covers that summary and its
factual transcriptions. It does not relicense the cited publications or
replace the separate Apache-2.0 NASA coefficient source.

## Aqueous boundary

The Cuprite database identity matches the new solid formula, but the aqueous
route currently books copper as Cu(II). Its existing oxide-valence guard
therefore refuses Cu(I) oxide; it must not turn Cu2O into Cu+2 without an
electron source. No aqueous Cu(I), solubility, dissolution enthalpy or
Cuprite precipitation qualification is added. Crosswalk `posable` now
reports actual candidate admission, including pre-existing unsupported
oxide/element exclusions, instead of inferring admission from a name.

## Acceptance still required

The changes include the actual copper HP target with unchanged energy and
Cu/O/N bounds, named crystal/liquid mapping and fusion consistency,
partial-freezing conservation, aqueous Cu(I) refusal and a CuSO4 negative.
Existing registry export, golden, coefficient-fidelity, stocking and
crosswalk controls remain applicable. Native builds and tests were not run
locally during implementation.

Hosted validation should regenerate and review these outputs:

```sh
cargo run -p kerotakis-registry-export -- data/registry/registry-source-v1.json
cargo run -p kerotakis-cea --bin generate-cea-subset > data/cea/reachable-subset.json
python tools/registry_solid_coverage.py
python tools/stocking_phase_coverage.py --check
python tools/transition_point_coverage.py --check
```

The solid-coverage files have the narrow Cuprite composition-match update
from their existing source inventory; the actual database generator must
confirm it. The older reachable-subset file was left untouched until a real
current-source generator can replace its stale 75-species inventory. All
previous registry golden objects remain unchanged; the additive Cu2O object
must pass the actual Rust projection and byte-for-byte export checks.

## Subsequent energy-control findings

The complete hosted source `3f2e4492828cd8ddd17f62dd34f342750081468c`
exposed two controls requiring corrections to their physical oracles. Their
first failures remain preserved; neither warrants changing runtime heat or
loosening a conservation tolerance.

The phase-table check used latent heat divided by the room-temperature Cp
scalar, reporting a 1024 K Cu2O excursion. The actual phase ledger integrates
Cp(T), and the hot crystal's heat capacity is substantially larger. The check
now retains its 1000 K bound and compares the integrated carrier energy with
each transition's latent heat. An actual AmbientClock control also requires a
molten Cu2O sample to freeze completely and reach room temperature without a
stranded liquid tail.

The unchanged chalk control was replayed with a transparent per-call observer.
Its original test SHA-256 is
`3bc495105ca0ed8ce2e9a28bd865713b7db930ff564b51404a21a1966f1aeb99`;
the diagnostic stderr SHA-256 is
`9b7bff3c73fc10b7138781becd6c2d665f0b185038743c282523a73a01e5faf7`.
Three heat parcels delivered 28715.123861 J. Carbon dioxide evolved as
0.0615046140 mol at 1006.700134 K and 0.0384953859 mol at 1003.271484 K,
before the remaining lime was reheated to 1773.15 K. Independently integrating
the sourced Cp coefficients gives 3369.200901 J of outgoing gas sensible
heat, rather than 7783.045019 J from pricing all gas at the final crucible
temperature. Lime warming plus actual gas heat plus the NASA reference
reaction enthalpy closes to 0.364024 J. The original independent 178.8 kJ/mol
reaction oracle differs by 63.465369 J, or 0.22% of the total.

The chalk test now observes each actual parcel before coalescing and integrates
its Cp independently of the production absolute-enthalpy evaluator. Parcel
moles must match the accepted transcript. The original 1% bound is retained;
the obsolete final-temperature estimate is an explicit negative control.
These changed controls still require their own hosted execution.


The first hosted followup of these controls exposed two oracle details.
The independent CaO warming integral now explicitly holds the 300 K source
endpoint Cp over 298.15–300 K, matching the existing core model without
extrapolating coefficients or dropping that heat; interior data gaps remain
errors. The long ambient-cooling transcript aggregates 1.358362969 mol of
freezing and 0.358362969 mol of remelting over bounded numerical substeps.
These are gross transfers of the same inventory, not 1.358 mol present at
once. The actual final inventory is one mole of solid and no liquid. The
control now requires the signed net transitions to reproduce that inventory,
positive finite event amounts, and stable subsequent phase settlement.
It does not claim a separately measured whole-clock heat exchange. Neither
runtime physics nor the 1%, amount, temperature or carrier bounds changed.
