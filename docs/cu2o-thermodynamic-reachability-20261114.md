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
