# Aqueous admission for non-water liquids (2026-10-27)

Generalises PR #839 (peroxide only). `nonaqueous::REGISTRY_LIQUID_MEDIA`
declares a medium class for every registry species whose standard phase is
liquid; `tests/nonaqueous_routing.rs::every_registry_liquid_has_a_declared_medium_class`
enumerates the registry and fails on any new unclassified liquid.

| Class | Registry liquids | Admission effect |
|---|---|---|
| Water | water | the solvent |
| Organic | ethanol, hexane, ethyl_acetate, propanone, CH3COOH (KNOWN_SOLVENTS), **methanol, isopropanol, tert_butanol, lactic_acid, bromoethane, tert_butyl_bromide, dodecane, hexadecane** (new) | counted in the water mole fraction; majority -> existing `not-modeled.mostly-organic-solvent` honesty route and PHREEQC stands aside. Immiscible oils are counted too: no validated two-liquid activity/partition model exists, so refusing is the conservative choice |
| Peroxide | H2O2 | unchanged: `not-modeled.peroxide-rich-medium`, readings withdrawn |
| MineralAcid | **H2SO4, H3PO4** (new) | counted; majority -> **new** typed `not-modeled.concentrated-acid-medium` (EN/DE/FR), pH/I/speciation withdrawn, inventory retained |
| AqueousReagentSolute | HCl, HI, NH3, NaOCl | registry "liquids" that are the dissolved solute of a reagent bottle; not a matrix (unchanged) |
| NotAMatrix | liquid_nitrogen | cryogen; not a solvent matrix (unchanged) |

Floor unchanged: `AQUEOUS_WATER_FRACTION_FLOOR = 0.5` (editorial). Exactly 0.5 is admitted.

Not in the registry, so not classified (no data invented): glycerol, HNO3,
HClO4. A pack that adds one must classify it before the test passes.

Deferred: an actual mixed-solvent or concentrated-acid activity model;
separate-layer (immiscible) aqueous solves.
