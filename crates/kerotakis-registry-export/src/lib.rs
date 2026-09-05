//! Build-only conversion of the handwritten seed registry into DATA-001's
//! reviewable source-record contract.
//!
//! This crate is intentionally not a dependency of any simulation or app
//! crate. Its output remains an oracle-side review artefact until each legacy
//! provenance statement has been cleared into the runtime source lane.

use std::collections::BTreeMap;

use kerotakis_core::{
    species::{Colour, Phase as LegacyPhase, SpeciesData, REGISTRY},
    spectrum::BAND_NM,
    stoich::parse_formula,
};
use kerotakis_data::{
    Applicability, CompositionRecord, CultureMetabolism, Dimension, ElementAmount, Evidence,
    FractionRange, IdentityRecord, Interval, MaterialBasis, MaterialComponent, MaterialConfidence,
    MaterialExpansionPolicy, MaterialGeometry, MaterialPhysicalForm, MaterialRecipe, MaterialRole,
    Method, ModelParameterRecord, ModelSubject, NumericRecord, OpticalRecord, Phase, PhaseProperty,
    PhaseThermodynamicRecord, RegistryDocument, SourceLane, SourceRecord, SpectralSample,
    Uncertainty, Unit,
};

const IMPORT_METHOD: &str = "verbatim export from kerotakis_core::species::REGISTRY";
const LEGACY_LICENCE: &str = "LicenseRef-Kerotakis-Legacy-Provenance-Review-Required";
const ISOPROPANOL_SOURCE: &str = "us-federal/isopropanol-chris";
const ISOPROPANOL_CITATION: &str = "PubChem CID 3776 identity crosswalk plus U.S. Coast Guard CHRIS isopropanol liquid density (0.785 at 68 F) and liquid heat capacity (0.605 BTU/lb-F at 70 F); molar heat capacity converted to SI; retrieved 2026-08-27";
const SUCROSE_SOURCE: &str = "kerotakis/sucrose-teaching-properties-v1";
const SUCROSE_CITATION: &str = "PubChem CID 5988 identity crosswalk; Kerotakis room-temperature teaching approximations: crystal density 1.59 g/mL, conservative aqueous solubility 200 g/100 mL water, and 484 J/(mol.K) solid heat capacity estimated by Kopp's rule. These are explicit editorial parameters, not redistributed NIST SRD data; retrieved 2026-08-27";
const IRON_III_OXIDE_SOURCE: &str = "us-federal/nasa-cea-hematite";
const IRON_III_OXIDE_CITATION: &str = "PubChem CID 14833 identity crosswalk; NASA CEA thermo.inp Fe2O3(cr) record cites Pankratz (1983) for hematite thermochemistry. Room-temperature density 5.24 g/mL and reddish-brown appearance are explicit teaching properties; retrieved 2026-08-27";
const EPSOMITE_SOURCE: &str = "us-federal/usgs-epsomite";
const EPSOMITE_CITATION: &str = "PubChem CID 24843 identity crosswalk for magnesium sulfate heptahydrate; USGS PHREEQC wateq4f.dat Epsomite phase supplies MgSO4:7H2O dissolution stoichiometry. Density 1.68 g/mL and 360 J/(mol.K) heat capacity are explicit room-temperature teaching approximations; dissolution enthalpy remains unclaimed; retrieved 2026-08-27";
/// EXP-33. Transition temperatures get their own source record rather than
/// riding each species' molar-mass citation, because they are a different
/// claim with a different origin: the melting-point apparatus prints THIS
/// line, and a reader who wants to check the number must land on the book
/// the number came from.
const PHASE_TRANSITION_SOURCE: &str = "kerotakis/phase-transitions-v1";
const PHASE_TRANSITION_CITATION: &str = "Kerotakis curated phase-transition tranche v1: normal melting, boiling, sublimation, decomposition and hydrate-dehydration temperatures at 101.325 kPa. Each value is an individually entered editorial constant, taken from the standard published values for these substances and cross-checked against general reference tables. It is NOT a transcription from a positively identified copy of any single handbook edition, and no edition-level provenance is claimed: CRC Handbook of Chemistry and Physics, 97th ed. is the intended primary reference and every value here is flagged for reviewer confirmation against a positively identified copy of it before any stronger claim is made. (Reviewer confirmation recorded 2026-09-01 in the registry source's own citation; this seed constant is historical - the bootstrap only fires when the source record is absent.) Values are recorded only to the precision a school apparatus resolves; where a substance decomposes or sublimes rather than melting, no melting point is claimed at all; and where two general references disagreed the value was dropped rather than averaged or guessed. Compiled 2026-08-29";
const CHALCANTHITE_SOURCE: &str = "us-federal/usgs-chalcanthite";
/// The dissolution enthalpies get their own source record for the same
/// reason the transition temperatures do: an enthalpy of dissolution is a
/// different claim with a different origin from a molar mass, and a heat
/// balance built on it will print it. Three of the seventeen used to ride a
/// species citation that did not mention them at all.
const DISSOLUTION_SOURCE: &str = "kerotakis/dissolution-enthalpies-v1";
const DISSOLUTION_CITATION: &str = "Kerotakis curated dissolution-enthalpy tranche v1: molar enthalpy of dissolution in water at 298.15 K and infinite dilution, in kJ/mol, positive = endothermic. These values used to ride each species' molar-mass citation, which for three of them said nothing about the enthalpy at all; they are separated here for the same reason the phase-transition tranche was, because an enthalpy of dissolution is a different claim with a different origin from a molar mass, and a heat balance built on it will print it. Fourteen of the seventeen values are consistent with the difference of the standard formation enthalpies commonly tabulated for the aqueous ions and the solid; each row's own `notes` field states that arithmetic explicitly so a reviewer can check it rather than take it on trust. Four of those rows already carried their derivation or a CRC Handbook 97th ed. citation in the species' own provenance line (KOH, NH4Cl, Na2SO4, NH4NO3, plus the NBS/Wagman derivations recorded for ZnSO4 and Pb(NO3)2) and are unchanged in substance. TWO ROWS ARE UNRESOLVED AND ARE NOT SILENTLY FIXED HERE: potassium permanganate's +16.2 kJ/mol reproduces from nothing the reviewer could find and is roughly 27 kJ/mol away from the commonly tabulated heat of solution near +43.6, which is not a sign, unit or hydrate difference; sodium bicarbonate's +16.7 matches neither the formation-enthalpy difference (about +18.7) nor the commonly tabulated figure (about +17.5). Both values are LEFT AS THEY STAND, because changing a number is a separate decision from sourcing one, and both are flagged by name for that decision. The vendored PHREEQC databases were checked as a candidate source and are deliberately NOT cited: Halite's delta_h is 3.84 kJ in wateq4f.dat and 3.7 kJ in minteq.v4.dat against this table's 3.88; Thenardite's is -2.39 kJ and -9.12 kJ in the two files, which disagree with each other; Portlandite's dissolution is written with two protons so its delta_h is the enthalpy of a different reaction entirely; and Chalcanthite is the pentahydrate while this table's copper sulfate is anhydrous. Where a database row IS the better source for a future value it should be cited as the file and reaction line, and none of these seventeen is that case. As with the phase-transition tranche, this is NOT a transcription from a positively identified copy of any single handbook edition, and no edition-level provenance is claimed: the CRC Handbook of Chemistry and Physics, 97th ed. and the NBS/Wagman 1982 tables are the intended primary references and every value here is flagged for reviewer confirmation against a positively identified copy before any stronger claim is made. Compiled 2026-09-05";
const DISSOLUTION_METHOD: &str = "curated dissolution-enthalpy tranche; the row's own note states the arithmetic or the handbook the value is claimed from, and flags it where neither holds";
/// Per-row provenance: the arithmetic a reviewer can check, or the reason
/// there is none. Keyed by registry species key; a value without a row here
/// exports no note, which is a state the tranche does not currently have.
const DISSOLUTION_NOTES: &[(&str, &str)] = &[
    ("NaCl", "+3.88 kJ/mol. Consistent with the difference of the commonly tabulated standard formation enthalpies: Na+(aq) about -240.1 and Cl-(aq) about -167.2 against NaCl(s) about -411.2 kJ/mol. Not the vendored Halite delta_h, which is 3.84 kJ in wateq4f.dat and 3.7 kJ in minteq.v4.dat."),
    ("AgNO3", "+22.6 kJ/mol. Consistent with Ag+(aq) about +105.6 and NO3-(aq) about -207.4 against AgNO3(s) about -124.4 kJ/mol. The species' own citation did not mention this value at all before this tranche."),
    ("AgCl", "+65.7 kJ/mol, and this is a lattice-to-ion enthalpy rather than anything a beaker measures: silver chloride is insoluble, so no bench observation corresponds to it. Consistent with Ag+(aq) about +105.6 and Cl-(aq) about -167.2 against AgCl(s) about -127.0 kJ/mol. The species' own citation did not mention this value at all before this tranche."),
    ("Ca(OH)2", "-16.7 kJ/mol. Consistent with Ca+2(aq) about -542.8 and two OH-(aq) at about -230.0 against Ca(OH)2(s) about -986.1 kJ/mol. Deliberately NOT the vendored Portlandite delta_h: PHREEQC writes that dissolution with two protons, so its -31 kcal (wateq4f.dat) / -128.62 kJ (minteq.v4.dat) is the enthalpy of a different reaction."),
    ("NaOH", "-44.5 kJ/mol, the exotherm a lye solution actually shows. Consistent with Na+(aq) about -240.1 and OH-(aq) about -230.0 against NaOH(s) about -425.6 kJ/mol."),
    ("NaOAc", "-17.3 kJ/mol. Consistent with Na+(aq) about -240.1 and CH3COO-(aq) about -486.0 against NaOAc(s) about -708.8 kJ/mol."),
    ("NaHCO3", "+16.7 kJ/mol, UNRESOLVED. The formation-enthalpy difference gives about +18.7 and the commonly tabulated heat of solution is about +17.5, so this value matches neither. It is left as it stands rather than replaced, because sourcing a number and changing it are separate decisions; a reviewer should settle this row."),
    ("KCl", "+17.2 kJ/mol. Consistent with K+(aq) about -252.4 and Cl-(aq) about -167.2 against KCl(s) about -436.7 kJ/mol."),
    ("CaCl2", "-82.8 kJ/mol for the ANHYDROUS salt, which is what this registry entry is. Consistent to within the spread of tabulated solid enthalpies with Ca+2(aq) about -542.8 and two Cl-(aq) at about -167.2 against CaCl2(s) about -795 to -796 kJ/mol. The dihydrate and hexahydrate sold as road salt dissolve far less exothermically, and neither is this row."),
    ("CuSO4", "-73.1 kJ/mol for the ANHYDROUS salt. Consistent with Cu+2(aq) about +64.8 and SO4-2(aq) about -909.3 against CuSO4(s) about -771.4 kJ/mol. The vendored Chalcanthite delta_h is not this number and is not meant to be: that phase is the pentahydrate, which dissolves endothermically."),
    ("KMnO4", "+16.2 kJ/mol, UNRESOLVED AND PROBABLY WRONG. The formation-enthalpy difference gives about +43 and the commonly tabulated heat of solution is about +43.6 kJ/mol; the gap is not a sign error, a unit error or a hydrate difference. The value is LEFT AS IT STANDS because changing a number is a separate decision from sourcing one, and this row is flagged for that decision as loudly as a data field can flag anything."),
    ("ZnSO4", "-80.4 kJ/mol, derived in the species' own citation from NBS/Wagman 1982 formation enthalpies: ZnSO4(s) -982.8, Zn+2(aq) -153.9, SO4-2(aq) -909.3 kJ/mol. Unchanged in substance by this tranche; recorded here so the row carries its own source."),
    ("Pb(NO3)2", "+35.4 kJ/mol, derived in the species' own citation from NBS/Wagman 1982 formation enthalpies: Pb(NO3)2(s) -451.9, Pb+2(aq) -1.7, NO3-(aq) -207.4 kJ/mol. Unchanged in substance by this tranche."),
    ("KOH", "-57.6 kJ/mol, taken from the CRC Handbook of Chemistry and Physics 97th ed. per the species' own citation, and consistent with K+(aq) about -252.4 and OH-(aq) about -230.0 against KOH(s) about -424.8 kJ/mol."),
    ("NH4Cl", "+14.78 kJ/mol at infinite dilution, from the CRC Handbook of Chemistry and Physics 97th ed. per the species' own citation. This is the one row here quoted to two decimal places, and the precision is the handbook's rather than an estimate's."),
    ("Na2SO4", "-2.43 kJ/mol for the anhydrous salt at infinite dilution, from the CRC Handbook of Chemistry and Physics 97th ed. per the species' own citation. The vendored Thenardite delta_h is -2.39 kJ in wateq4f.dat and -9.12 kJ in minteq.v4.dat; the two databases disagree with each other and neither is this row's source."),
    ("NH4NO3", "+25.7 kJ/mol, the cold-pack number, and one of the most strongly endothermic dissolutions a kitchen can produce. Stated in the species' own citation and recorded here so the row carries its own source."),
];
/// The electrical claim a wire is chosen for. It gets its own source
/// record for the same reason the transition temperatures and the
/// dissolution enthalpies got theirs: a resistivity is a different claim
/// with a different origin from a molar mass, and the meter that prints it
/// must be able to print the book behind it.
const RESISTIVITY_SOURCE: &str = "kerotakis/electrical-resistivity-v1";
const RESISTIVITY_CITATION: &str = "Kerotakis curated electrical-resistivity tranche v1: bulk DC electrical resistivity of the pure solid at 293.15 K (20 C), in ohm.m. THE PROVENANCE LANE OF THIS TRANCHE IS PENDING REVIEW AND THE VALUES ARE RECORDED AS COMMONLY TABULATED. The CRC Handbook of Chemistry and Physics table 'Electrical Resistivity of Pure Metals' is the intended primary reference and these numbers agree with it to the precision quoted, but this is NOT a transcription from a positively identified copy of any single edition and no edition-level provenance is claimed: every row is flagged for reviewer confirmation against a positively identified copy before any stronger claim is made, exactly as the phase-transition tranche is. The values themselves are physical constants of the pure elements rather than anyone's compilation, and they are quoted only to the three or four figures that a room-temperature handbook column carries; no temperature coefficient, no purity dependence and no cold-worked or alloyed value is claimed, and each row's own `notes` field states what it does not cover. Graphite is the one row that is an order of magnitude rather than a measurement, and its note says so: graphite is strongly anisotropic and its resistivity depends on the grade, so the value here describes a polycrystalline bench rod and nothing finer. Compiled 2026-09-05";
const RESISTIVITY_METHOD: &str = "curated electrical-resistivity tranche, provenance lane pending review; the row's own note states what the value does not cover";
/// The same claim for a named OBJECT rather than a pure substance.
///
/// It is a separate tranche and not an extension of the one above,
/// because it is a different kind of number. A metal's resistivity is a
/// constant of the metal; a fired porcelain body has no such constant,
/// and the row that gives it one has to say so. The citation therefore
/// travels INSIDE the recipe role rather than in a source record, for
/// the reason `corrosion::Barrier` carries its own source: the meter
/// prints the number, so it must be able to print the book, and the
/// runtime holds recipes without holding the source records.
const MATERIAL_RESISTIVITY_CITATION: &str = "Kerotakis curated material-resistivity tranche v1: room-temperature bulk DC volume resistivity of the named object, in ohm.m, with the span its class of material covers. THE PROVENANCE LANE OF THIS TRANCHE IS PENDING REVIEW AND THE VALUES ARE RECORDED AS COMMONLY TABULATED. General materials-science and electrical-engineering reference tables for insulators and semiconductors are the intended primary reference, this is NOT a transcription from a positively identified copy of any single edition, no edition-level provenance is claimed, and every row is flagged for reviewer confirmation against a positively identified copy before any stronger claim is made - exactly as the pure-solid electrical-resistivity tranche is. What separates these rows from that one is that an insulator's resistivity is not a constant of the substance the way a metal's is: it moves by orders of magnitude with composition, temperature and surface condition, and a semiconductor's is set by a dopant concentration no recipe here states. Every row therefore carries the span its class covers beside the single value the meter reads, and every row's own boundary states what the number does not cover. Compiled 2026-09-05";
/// The one property that separates the two families of plastic.
///
/// Its own tranche for the same reason the resistivity rows are:
/// these are temperatures of an OBJECT, they belong to no species
/// record, and the row that quotes one has to carry the book. A
/// polymer transition is also grade-dependent in a way a melting
/// point is not, which every row here says.
const POLYMER_HEAT_CITATION: &str = "Kerotakis curated polymer heat-response tranche v1: the softening (crystalline melt or flow) and decomposition temperatures of the two families of plastic, in K at 101.325 kPa, with a room-temperature specific heat capacity in J/(g.K). THE PROVENANCE LANE OF THIS TRANCHE IS PENDING REVIEW AND THE VALUES ARE RECORDED AS COMMONLY TABULATED. Polymer handbooks and general materials references are the intended primary source, this is NOT a transcription from a positively identified copy of any single edition, no edition-level provenance is claimed, and every row is flagged for reviewer confirmation against a positively identified copy before any stronger claim is made - exactly as the phase-transition and electrical-resistivity tranches are. Polymer transition temperatures are grade-dependent by nature: molecular weight, crystallinity, plasticiser content and degree of cure each move them by tens of kelvin, so these are class figures for a teaching object rather than a specification for any material that could be bought. Compiled 2026-09-05";
/// `(species key, resistivity in ohm.m at 293.15 K, what the row does not claim)`.
///
/// Seven rows, and the gap in them is the point: the registry carries no
/// elemental silicon, so the semiconductor case is absent rather than
/// approximated. Doped silicon needs a carrier-density model this bench
/// does not have, and the intrinsic value alone would answer no question
/// anyone asked.
const RESISTIVITY: &[(&str, f64, &str)] = &[
    (
        "Ag",
        1.587e-8,
        "Silver is the least resistive metal known at room temperature, and only about 5% below copper - which is why wire is copper and not silver: the metal is thirty to a hundred times the price for a twentieth less resistance. Price is not a property this registry records, and this row does not claim it.",
    ),
    (
        "Cu",
        1.678e-8,
        "Annealed copper of ordinary commercial purity. Hard-drawn copper reads a percent or two higher and alloyed copper much higher; neither is this row.",
    ),
    (
        "Al",
        2.65e-8,
        "About 58% more resistive than copper by volume, but under a third of its density - which is why overhead transmission line is aluminium and house wiring is not. This row records the resistivity only; the per-mass comparison is arithmetic a reader can do with the density row beside it.",
    ),
    (
        "Mg",
        4.39e-8,
        "Pure magnesium. This bench's magnesium is ribbon, which is an alloy of unstated composition, and an alloy's resistivity is not its parent metal's.",
    ),
    (
        "Zn",
        5.9e-8,
        "Pure zinc. Galvanised steel is a zinc coating on iron and conducts as neither.",
    ),
    (
        "Fe",
        9.71e-8,
        "Pure iron, about 5.8 times copper's resistivity - the whole of the answer to why wire is not made of it. Steel is higher again, and rust is not a conductor at all; neither is this row.",
    ),
    (
        "graphite",
        1e-5,
        "AN ORDER OF MAGNITUDE, NOT A MEASUREMENT. Graphite is strongly anisotropic - roughly 4e-7 ohm.m along the basal plane and some thousands of times more across it - and commercial grades differ widely. This value describes a polycrystalline bench rod or pencil lead, and is recorded so the meter can say 'a conductor, but a poor one beside a metal' rather than nothing. It supports no comparison finer than that.",
    ),
];
const SILICA_SOURCE: &str = "us-federal/pubchem-silica";
const SILICA_CITATION: &str = "PubChem CID 24261 silica identity crosswalk. Quartz-like room-temperature teaching properties use molar mass 60.084 g/mol, density 2.65 g/mL and heat capacity 44.6 J/(mol.K); polymorph, grain coatings and natural-sand impurities remain separate material assumptions; retrieved 2026-08-27";

// Bootstrap a new data-driven species exactly once. After the generated source
// document is checked in, core's build script includes it in REGISTRY and the
// normal export loop reproduces the same record.
const ISOPROPANOL_SEED: SpeciesData = SpeciesData {
    key: "isopropanol",
    name: "isopropanol",
    formula: "C3H8O",
    inchikey: "KFZMGEQAYNKOFK-UHFFFAOYSA-N",
    molar_mass: 60.096,
    heat_capacity: 152.2,
    density: 0.785,
    standard_phase: LegacyPhase::Liquid,
    appearance: Some("colourless"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    aqueous_solubility_g_per_100_ml_at_100c: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    electrical_resistivity: None,
    provenance: ISOPROPANOL_CITATION,
};

const SUCROSE_SEED: SpeciesData = SpeciesData {
    key: "sucrose",
    name: "sucrose",
    formula: "C12H22O11",
    inchikey: "CZMRCDWAGMRECN-SFOFJGFUSA-N",
    molar_mass: 342.2965,
    heat_capacity: 484.0,
    density: 1.59,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("white"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: true,
    aqueous_solubility_g_per_100_ml: Some(200.0),
    aqueous_solubility_g_per_100_ml_at_100c: Some(487.0),
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    electrical_resistivity: None,
    provenance: SUCROSE_CITATION,
};

const IRON_III_OXIDE_SEED: SpeciesData = SpeciesData {
    key: "Fe2O3",
    name: "iron(III) oxide (hematite)",
    formula: "Fe2O3",
    inchikey: "JEIPFZHSYJVQDO-UHFFFAOYSA-N",
    molar_mass: 159.687,
    heat_capacity: 103.9,
    density: 5.24,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("reddish brown"),
    flame_colour: None,
    colour: Some(Colour {
        r: 145,
        g: 66,
        b: 54,
        strength: 0.0,
    }),
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    aqueous_solubility_g_per_100_ml_at_100c: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    electrical_resistivity: None,
    provenance: IRON_III_OXIDE_CITATION,
};

const EPSOMITE_SEED: SpeciesData = SpeciesData {
    key: "epsomite",
    name: "magnesium sulfate heptahydrate (epsomite)",
    formula: "MgSO4·7H2O",
    inchikey: "WRUGWIBCXHJTDG-UHFFFAOYSA-L",
    molar_mass: 246.471,
    heat_capacity: 360.0,
    density: 1.68,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("colourless to white crystals"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    aqueous_solubility_g_per_100_ml_at_100c: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    electrical_resistivity: None,
    provenance: EPSOMITE_CITATION,
};

const SILICA_SEED: SpeciesData = SpeciesData {
    key: "SiO2",
    name: "silicon dioxide (quartz)",
    formula: "SiO2",
    inchikey: "VYPSYNLAJGMNEJ-UHFFFAOYSA-N",
    molar_mass: 60.084,
    heat_capacity: 44.6,
    density: 2.65,
    standard_phase: LegacyPhase::Solid,
    appearance: Some("colourless to white grains"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: false,
    aqueous_solubility_g_per_100_ml: None,
    aqueous_solubility_g_per_100_ml_at_100c: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    electrical_resistivity: None,
    provenance: SILICA_CITATION,
};

/// Export every current declaration without changing or replacing the runtime
/// registry. All legacy sources remain build-oracle material pending explicit
/// licence and provenance review.
pub fn export_current_registry() -> Result<RegistryDocument, String> {
    let mut document = RegistryDocument::empty();
    for species in REGISTRY {
        export_species(&mut document, species)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "isopropanol") {
        export_species(&mut document, &ISOPROPANOL_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "sucrose") {
        export_species(&mut document, &SUCROSE_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "Fe2O3") {
        export_species(&mut document, &IRON_III_OXIDE_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "epsomite") {
        export_species(&mut document, &EPSOMITE_SEED)?;
    }
    if !REGISTRY.iter().any(|species| species.key == "SiO2") {
        export_species(&mut document, &SILICA_SEED)?;
    }
    // EXP-33: one source record for the whole transition tranche, pushed
    // once and only if something actually cites it — an orphan source is a
    // claim nobody made.
    if document
        .phase_thermodynamics
        .iter()
        .any(|record| record.quantity.source_id == PHASE_TRANSITION_SOURCE)
    {
        document.sources.push(SourceRecord {
            id: PHASE_TRANSITION_SOURCE.to_string(),
            citation: PHASE_TRANSITION_CITATION.to_string(),
            licence: "AGPL-3.0-or-later".to_string(),
            lane: SourceLane::Runtime,
            origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
            revision: Some("v1".to_string()),
            retrieved: Some("2026-08-29".to_string()),
        });
    }
    // Same guard as the transition tranche above: an orphan source record is
    // a claim nobody made.
    if document
        .phase_thermodynamics
        .iter()
        .any(|record| record.quantity.source_id == DISSOLUTION_SOURCE)
    {
        document.sources.push(SourceRecord {
            id: DISSOLUTION_SOURCE.to_string(),
            citation: DISSOLUTION_CITATION.to_string(),
            licence: "AGPL-3.0-or-later".to_string(),
            lane: SourceLane::Runtime,
            origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
            revision: Some("v1".to_string()),
            retrieved: Some("2026-09-05".to_string()),
        });
    }
    // Same guard again. This tranche's lane is pending review, and the
    // citation says so in prose rather than in a lane the schema does not
    // have: `SourceLane` has no pending-review variant, so the caveat that
    // `kerotakis-thermo`'s `RightsLane::PrimaryLiteratureCoefficientsPendingReview`
    // carries as a type is carried here as the first sentence a reviewer reads.
    if document
        .phase_thermodynamics
        .iter()
        .any(|record| record.quantity.source_id == RESISTIVITY_SOURCE)
    {
        document.sources.push(SourceRecord {
            id: RESISTIVITY_SOURCE.to_string(),
            citation: RESISTIVITY_CITATION.to_string(),
            licence: "AGPL-3.0-or-later".to_string(),
            lane: SourceLane::Runtime,
            origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
            revision: Some("v1".to_string()),
            retrieved: Some("2026-09-05".to_string()),
        });
    }
    export_material_recipes(&mut document);
    document.validate().map_err(|error| error.to_string())?;
    Ok(document)
}

fn export_material_recipes(document: &mut RegistryDocument) {
    const SOURCE: &str = "kerotakis/material-recipes-v1";
    document.sources.push(SourceRecord {
        id: SOURCE.to_string(),
        citation: "Kerotakis household-material assumptions v1: explicit unbranded teaching surrogates for common household substances; ACS middle-school chemistry uses 3% peroxide for yeast catalysis, documents detergent-lowered surface tension, teaches that vegetable oil is less dense than water and does not dissolve in it, demonstrates that detergent helps oil and water mix, and its Colors on the Move activity records detergent driving food colouring rapidly across whole milk; a Journal of Chemical Education baker's-yeast gasometer study measures CO2 evolution, induction, steady production and nutrient depletion, while FAO fermentation material gives the balanced hexose-to-ethanol-and-CO2 pathway; American Society of Baking compressed-yeast technical guidance reports 70% moisture and 30% solids; USDA ERS reports cow's milk as approximately 87% water with the balance milk fat and skim solids; ACS Making Glue and Mississippi State Extension describe vinegar separating milk casein into heavy white curds and liquid whey; USDA FoodData Central's white all-purpose wheat flour entry reports starch as the large majority of its carbohydrate, with protein, moisture, fibre, lipid and ash making up the rest, and its unsweetened apple-juice entry reports roughly 88% water with the sugars dominated by fructose and glucose rather than sucrose and the acidity carried mainly by malic acid; its raw-lemon-juice entry reports roughly 91% water with citric acid as the dominant acid at about 4.7% and ascorbic acid present at about 0.05%; ordinary flat glass is a soda-lime composition of roughly three quarters silica with soda, lime, magnesia and alumina as network modifiers; solid paraffin candle wax and a sheet of office paper are dispensed against room-temperature bulk densities of 0.90 and 0.80 g/mL; USDA FoodData Central's seedless-raisin entry reports roughly 15% water and 79% carbohydrate of which about 59% is sugars, and a dried grape is denser than water at roughly 1.35 g/mL, which is why raisins sink in it".to_string(),
        licence: "AGPL-3.0-or-later".to_string(),
        lane: SourceLane::Runtime,
        origin: Some("crates/kerotakis-registry-export/src/lib.rs".to_string()),
        revision: Some("2".to_string()),
        retrieved: Some("2026-08-29".to_string()),
    });
    let component = |species_id: &str, fraction: f64| MaterialComponent {
        species_id: species_id.to_string(),
        fraction: FractionRange {
            lower: fraction,
            upper: fraction,
        },
        evidence: Evidence {
            source_id: SOURCE.to_string(),
            method: Method::Curated("fixed teaching-surrogate composition".to_string()),
        },
    };
    let evidence = || Evidence {
        source_id: SOURCE.to_string(),
        method: Method::Editorial(
            "unbranded household teaching surrogate with an explicit concentration".to_string(),
        ),
    };
    let density = |value: f64| NumericRecord {
        value,
        unit: Unit {
            symbol: "g/mL".to_string(),
            dimension: Dimension::MassDensity,
        },
        conditions: Applicability::default(),
        uncertainty: Uncertainty::NotReported,
        source_id: SOURCE.to_string(),
        method: Method::Editorial("room-temperature teaching-surrogate density".to_string()),
    };
    let resistivity = |ohm_m: f64, span_lower_ohm_m: f64, span_upper_ohm_m: f64, boundary: &str| {
        MaterialRole::BulkElectricalResistivity {
            ohm_m,
            span_lower_ohm_m,
            span_upper_ohm_m,
            boundary: boundary.to_string(),
            source: MATERIAL_RESISTIVITY_CITATION.to_string(),
        }
    };
    let heat_response = |specific_heat_j_per_g_k: f64,
                         softens_above_k: Option<f64>,
                         chars_above_k: f64,
                         boundary: &str| {
        MaterialRole::PolymerHeatResponse {
            specific_heat_j_per_g_k,
            softens_above_k,
            chars_above_k,
            boundary: boundary.to_string(),
            source: POLYMER_HEAT_CITATION.to_string(),
        }
    };
    let familiar_solid = |id: &str,
                          canonical_key: &str,
                          name: &str,
                          species: &str,
                          shape: &str,
                          de_aliases: &[&str],
                          en_aliases: &[&str],
                          assumptions: &[&str]| MaterialRecipe {
        id: id.to_string(),
        version: 1,
        canonical_key: canonical_key.to_string(),
        name: name.to_string(),
        aliases: BTreeMap::from([
            (
                "de".to_string(),
                de_aliases
                    .iter()
                    .map(|alias| (*alias).to_string())
                    .collect(),
            ),
            (
                "en".to_string(),
                en_aliases
                    .iter()
                    .map(|alias| (*alias).to_string())
                    .collect(),
            ),
        ]),
        basis: MaterialBasis::MassFraction,
        bulk_density: None,
        components: vec![component(species, 1.0)],
        unresolved_fraction: None,
        physical_form: MaterialPhysicalForm::CompositeObject {
            geometry: Some(MaterialGeometry {
                shape: Some(shape.to_string()),
                surface_area_m2: None,
                characteristic_length_m: None,
            }),
        },
        roles: Vec::new(),
        preparation: Some(format!(
            "idealised {name} represented by the installed {species} species"
        )),
        lot_assumptions: assumptions
            .iter()
            .map(|assumption| (*assumption).to_string())
            .collect(),
        substitutions: Vec::new(),
        confidence: MaterialConfidence::Curated,
        expansion_policy: MaterialExpansionPolicy::Fixed,
        evidence: evidence(),
    };
    let transparent_colour = |id: &str,
                              canonical_key: &str,
                              name: &str,
                              dye: &str,
                              concentration: f64,
                              medium: &str,
                              de_aliases: &[&str],
                              en_aliases: &[&str]| {
        MaterialRecipe {
        id: id.to_string(),
        version: 1,
        canonical_key: canonical_key.to_string(),
        name: name.to_string(),
        aliases: BTreeMap::from([
            (
                "de".to_string(),
                de_aliases.iter().map(|alias| (*alias).to_string()).collect(),
            ),
            (
                "en".to_string(),
                en_aliases.iter().map(|alias| (*alias).to_string()).collect(),
            ),
        ]),
        basis: MaterialBasis::MassFraction,
        bulk_density: Some(density(1.0)),
        components: vec![component(dye, concentration), component("water", 1.0 - concentration)],
        unresolved_fraction: None,
        physical_form: MaterialPhysicalForm::HomogeneousLiquid,
        roles: if canonical_key.starts_with("food_colour_") {
            vec![MaterialRole::SurfaceColourant {
                srgb: match canonical_key {
                    "food_colour_red" => [210, 35, 55],
                    "food_colour_yellow" => [245, 190, 25],
                    "food_colour_blue" => [35, 90, 210],
                    _ => [128, 128, 128],
                },
            }]
        } else {
            Vec::new()
        },
        preparation: Some(format!(
            "{}% w/w {dye} aqueous {medium}; unbranded transparent optical teaching surrogate",
            concentration * 100.0
        )),
        lot_assumptions: vec![
            "the named chromophore and concentration define this surrogate; it is not a claim about a retail product's ingredients".to_string(),
        ],
        substitutions: Vec::new(),
        confidence: MaterialConfidence::Surrogate,
        expansion_policy: MaterialExpansionPolicy::Fixed,
        evidence: evidence(),
    }
    };
    // These Gaussian bands are illustrative optical surrogates, not measured
    // spectra. Quantize them before export so their JSON representation remains
    // byte-stable after parse/serialize pack round trips on every platform.
    let stable_surrogate_coefficient = |value: f64| {
        if value == 0.0 {
            return 0.0;
        }
        format!("{value:.11e}")
            .parse::<f64>()
            .expect("formatted finite pigment coefficient must parse")
    };
    let pigment_bands = |peaks: &[(f64, f64, f64)]| {
        BAND_NM
            .iter()
            .map(|wavelength| {
                stable_surrogate_coefficient(
                    peaks
                        .iter()
                        .map(|(centre, strength, width)| {
                            let offset = (wavelength - centre) / width;
                            strength * (-0.5 * offset * offset).exp()
                        })
                        .sum(),
                )
            })
            .collect::<Vec<f64>>()
    };
    let acrylic_colour = |id: &str,
                          canonical_key: &str,
                          name: &str,
                          absorption: Vec<f64>,
                          scattering: f64,
                          de_aliases: &[&str],
                          en_aliases: &[&str]| {
        MaterialRecipe {
            id: id.to_string(),
            version: 1,
            canonical_key: canonical_key.to_string(),
            name: name.to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    de_aliases.iter().map(|alias| (*alias).to_string()).collect(),
                ),
                (
                    "en".to_string(),
                    en_aliases.iter().map(|alias| (*alias).to_string()).collect(),
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.1)),
            components: vec![component("water", 0.45)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.55,
                upper: 0.55,
            }),
            physical_form: MaterialPhysicalForm::Suspension,
            roles: vec![MaterialRole::OpaquePigment {
                absorption,
                scattering: vec![scattering; BAND_NM.len()],
            }],
            preparation: Some(
                "waterborne opaque acrylic-paint optical surrogate; effective pigment and binder remain unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "effective K/S coefficients describe this teaching color only; they do not match a brand, named artist pigment, gloss or drying film".to_string(),
                "the layer is treated as optically thick, so the substrate does not affect the computed swatch".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        }
    };
    document.material_recipes.extend([
        MaterialRecipe {
            id: "household/isopropanol-70-percent-vv".to_string(),
            version: 1,
            canonical_key: "isopropanol_70_percent".to_string(),
            name: "70% isopropanol".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "rubbing alcohol 70%".to_string(),
                        "isopropyl alcohol 70%".to_string(),
                        "rubbing_alcohol_70%".to_string(),
                    ],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Isopropanol 70%".to_string(),
                        "Isopropylalkohol 70%".to_string(),
                        "Isopropanol_70%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::VolumeFraction,
            bulk_density: None,
            components: vec![component("isopropanol", 0.70), component("water", 0.30)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("70% v/v isopropanol in water at room temperature".to_string()),
            lot_assumptions: vec![
                "component volumes use the explicit runtime additive-volume teaching approximation; real mixing contracts slightly".to_string(),
                "denaturants, fragrance and brand-specific additives are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/chlorine-bleach-5-percent".to_string(),
            version: 1,
            canonical_key: "chlorine_bleach_5_percent".to_string(),
            name: "5% chlorine bleach".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["household bleach".to_string(), "bleach_5%".to_string()],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Chlorreiniger 5%".to_string(),
                        "Chlorbleiche 5%".to_string(),
                        "Chlorreiniger_5%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.08)),
            components: vec![component("NaOCl", 0.05), component("water", 0.95)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("5% w/w sodium-hypochlorite teaching solution".to_string()),
            lot_assumptions: vec![
                "stabilisers, sodium chloride and brand-specific additives are not resolved in v1".to_string(),
                "never mix real bleach with acids or ammonia cleaners".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/ammonia-cleaner-5-percent".to_string(),
            version: 1,
            canonical_key: "ammonia_cleaner_5_percent".to_string(),
            name: "5% ammonia cleaner".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["household ammonia".to_string(), "ammonia_cleaner_5%".to_string()],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Ammoniakreiniger 5%".to_string(),
                        "Salmiakgeist 5%".to_string(),
                        "Ammoniakreiniger_5%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.98)),
            components: vec![component("NH3", 0.05), component("water", 0.95)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("5% w/w aqueous-ammonia teaching solution".to_string()),
            lot_assumptions: vec![
                "surfactants, fragrance and brand-specific additives are not resolved in v1".to_string(),
                "never mix real ammonia cleaner with bleach".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/sparkling-water-surrogate".to_string(),
            version: 1,
            canonical_key: "sparkling_water".to_string(),
            name: "sparkling water".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["carbonated water".to_string(), "fizzy water".to_string()],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Sprudel".to_string(),
                        "Mineralwasser mit Kohlensäure".to_string(),
                        "Sprudelwasser".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: vec![component("CO2", 0.006), component("water", 0.994)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("0.6% w/w CO2-in-water carbonation teaching surrogate".to_string()),
            lot_assumptions: vec![
                "dissolved minerals and brand-specific carbonation are not represented".to_string(),
                "dispensing into an open vessel lets the installed gas-liquid model decide how much CO2 escapes".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/cola-drink-surrogate".to_string(),
            version: 1,
            canonical_key: "cola_drink".to_string(),
            name: "cola drink surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "fizzy cola".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Cola".to_string(), "Colagetränk".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.04)),
            components: vec![
                component("CO2", 0.006),
                component("H3PO4", 0.0005),
                component("water", 0.885),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.1085,
                upper: 0.1085,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "carbonated phosphoric-acid cola teaching surrogate; 10.85% unresolved solids and flavour blend"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "sugar or sweetener identity, caramel colour, caffeine and flavour compounds remain unresolved".to_string(),
                "this is not a nutritional or brand-specific formulation".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/mineralised-tap-water-surrogate".to_string(),
            version: 1,
            canonical_key: "tap_water".to_string(),
            name: "mineralised tap-water surrogate".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["tap water".to_string()]),
                ("de".to_string(), vec!["Leitungswasser".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.998)),
            components: vec![
                component("CaCl2", 0.0002),
                component("MgSO4", 0.0001),
                component("NaHCO3", 0.0004),
                component("water", 0.9993),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "700 mg/kg mineralised hard-tap-water teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "real tap-water composition varies strongly by location, season and treatment".to_string(),
                "trace disinfectant, dissolved gases and organic matter are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/seawater-35-per-mille-surrogate".to_string(),
            version: 1,
            canonical_key: "seawater".to_string(),
            name: "3.5% seawater surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["sea water".to_string(), "salt water".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Meerwasser".to_string(), "Salzwasser".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.025)),
            components: vec![
                component("CaCl2", 0.0012),
                component("KCl", 0.0007),
                component("MgSO4", 0.0017),
                component("NaCl", 0.027),
                component("water", 0.965),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0044,
                upper: 0.0044,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "3.5% w/w simplified seawater teaching surrogate with major installed salts"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "minor ions, alkalinity, dissolved gases and organic matter remain in the explicit unresolved fraction".to_string(),
                "this is not a recipe for biological or analytical artificial seawater".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/hydrogen-peroxide-3-percent".to_string(),
            version: 1,
            canonical_key: "hydrogen_peroxide_3_percent".to_string(),
            name: "3% hydrogen peroxide".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "household peroxide 3%".to_string(),
                        "peroxide_3%".to_string(),
                    ],
                ),
                (
                    "de".to_string(),
                    vec![
                        "Wasserstoffperoxid 3%".to_string(),
                        "Wasserstoffperoxid_3%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.01)),
            components: vec![component("H2O2", 0.03), component("water", 0.97)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("3% w/w aqueous solution at room temperature".to_string()),
            lot_assumptions: vec![
                "stabilisers and trace impurities are not resolved in v1".to_string()
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/white-vinegar-5-percent".to_string(),
            version: 1,
            canonical_key: "white_vinegar_5_percent".to_string(),
            name: "5% white vinegar".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["vinegar".to_string(), "household vinegar".to_string()]),
                (
                    "de".to_string(),
                    vec![
                        "Essig".to_string(),
                        "Haushaltsessig 5%".to_string(),
                        "Haushaltsessig_5%".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.006)),
            components: vec![component("CH3COOH", 0.05), component("water", 0.95)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("5% w/w acetic-acid teaching surrogate".to_string()),
            lot_assumptions: vec![
                "flavour compounds and brand-specific residues are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/dish-soap-surrogate".to_string(),
            version: 1,
            canonical_key: "dish_soap".to_string(),
            name: "dish soap".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["soap".to_string(), "washing-up_liquid".to_string()]),
                (
                    "de".to_string(),
                    vec!["Spülmittel".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![component("water", 0.80)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.20,
                upper: 0.20,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::FoamStabilizer {
                    trapping_efficiency: 0.85,
                    gas_volume_fraction: 0.90,
                    half_life_seconds: 180.0,
                    saturation_amount: 0.4,
                },
                MaterialRole::SurfaceTensionReducer {
                    saturation_amount: 0.10,
                    max_cleared_fraction: 0.90,
                },
                MaterialRole::AqueousEmulsifier {
                    saturation_amount: 0.10,
                    max_dispersed_fraction: 0.92,
                    half_life_seconds: 300.0,
                },
            ],
            preparation: Some(
                "unbranded aqueous dish-soap teaching surrogate; surfactant blend unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "brand-specific surfactants, salts, fragrance, dye and preservatives remain in the explicit unresolved fraction".to_string(),
                "emulsification parameters are a bounded stirred classroom observable, not a universal detergent specification".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/ground-black-pepper-surrogate".to_string(),
            version: 1,
            canonical_key: "ground_black_pepper".to_string(),
            name: "ground black pepper".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Pfeffer".to_string(), "schwarzer Pfeffer".to_string()],
                ),
                ("en".to_string(), vec!["pepper".to_string(), "ground pepper".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: vec![MaterialRole::SurfaceFloater {
                saturation_amount: 0.08,
            }],
            preparation: Some(
                "dry ground black-pepper teaching surrogate for the quiet-water surface demonstration"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "plant composition, grind distribution and brand-specific wetting remain unresolved; only the reviewed floating-grain observable is modeled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/whole-milk-surrogate".to_string(),
            version: 1,
            canonical_key: "whole_milk".to_string(),
            name: "whole milk surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "milk".to_string(),"whole milk".to_string(), "cow's milk".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Milch".to_string(), "Vollmilch".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![component("water", 0.87)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.13,
                upper: 0.13,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::OpaqueLiquidColloid {
                    srgb: [248, 247, 240],
                    opacity_saturation_g_per_litre: 60.0,
                },
                MaterialRole::AcidCurdlingColloid {
                    acid_species: "CH3COOH".to_string(),
                    onset_moles_per_gram: 0.0001,
                    full_moles_per_gram: 0.0005,
                    max_curdled_fraction: 0.28,
                    max_opacity_reduction: 0.85,
                    curd_srgb: [250, 248, 230],
                },
            ],
            preparation: Some(
                "generic unflavoured whole-cow's-milk teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "USDA's approximately 87% water is resolved; milk fat, protein, lactose, minerals and natural variation remain together as conserved unresolved milk solids rather than fictional molecules".to_string(),
                "1.03 g/mL and 60 g/L full-opacity are explicit room-temperature visual geometry parameters, not product specifications".to_string(),
                "acid curdling is a bounded dose response calibrated to the familiar milk-and-vinegar classroom ratio; its 28% aggregate-solids ceiling separates estimated curd solids from wet-curd yield and its opacity response is independent".to_string(),
                "detergent-driven colour motion is a bounded surface-state response; spoilage and fermentation require separate state transitions".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/dry-yeast-catalase-surrogate".to_string(),
            version: 1,
            canonical_key: "dry_yeast".to_string(),
            name: "dry yeast".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["yeast".to_string(), "baker's_yeast".to_string()]),
                (
                    "de".to_string(),
                    vec!["Hefe".to_string(), "Trockenhefe".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("catalase", 0.000_001)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.999_999,
                upper: 0.999_999,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.0002,
                optimum_temperature_k: 308.15,
                temperature_width_k: 18.0,
                requires_hydration: true,
                metabolism: CultureMetabolism::Alcoholic,
            }],
            preparation: Some(
                "dry baker's yeast represented as a catalase activity proxy; hydrate with warm water in the experiment"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "enzyme activity varies strongly by brand and age; the bounded hydration ramp is a teaching surrogate, not a universal activity per gram".to_string(),
                "the sucrose-fermentation rate is an editorial classroom timescale; strain growth, oxygen switching, inhibition and secondary metabolites remain unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/fresh-compressed-yeast-surrogate".to_string(),
            version: 1,
            canonical_key: "fresh_yeast".to_string(),
            name: "fresh compressed yeast".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["fresh yeast".to_string(), "compressed yeast".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Frischhefe".to_string(), "Presshefe".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("water", 0.70), component("catalase", 0.000_000_3)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.299_999_7,
                upper: 0.299_999_7,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "moist compressed block".to_string(),
            },
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.0002,
                optimum_temperature_k: 308.15,
                temperature_width_k: 18.0,
                requires_hydration: false,
                metabolism: CultureMetabolism::Alcoholic,
            }],
            preparation: Some(
                "fresh compressed baker's-yeast surrogate; already hydrated and scaled to 30% solids"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "70% water and 30% solids follow compressed-yeast technical guidance".to_string(),
                "catalase activity is scaled from the dry-yeast teaching surrogate by dry solids; strain, age, storage and brand variation remain unresolved".to_string(),
                "the sucrose-fermentation rate is an editorial classroom timescale shared by equal dry solids; strain growth, oxygen switching, inhibition and secondary metabolites remain unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/granulated-table-sugar".to_string(),
            version: 1,
            canonical_key: "table_sugar".to_string(),
            name: "granulated table sugar".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "sugar".to_string(),"table sugar".to_string(), "granulated sugar".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Haushaltszucker".to_string(), "Kristallzucker".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("sucrose", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("pure crystalline sucrose teaching surrogate".to_string()),
            lot_assumptions: vec![
                "moisture, invert sugar, anti-caking agents and source-crop residues are omitted"
                    .to_string(),
                "bare sugar/Zucker remains unclaimed because glucose, fructose and mixed sugars are distinct materials"
                    .to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/vegetable-oil-surrogate".to_string(),
            version: 1,
            canonical_key: "vegetable_oil".to_string(),
            name: "vegetable oil".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "oil".to_string(),"cooking oil".to_string(), "plant oil".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Pflanzenöl".to_string(), "Speiseöl".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.92)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::AqueousImmiscibleLiquid {
                srgb: [238, 218, 112],
                colour_word: "pale yellow".to_string(),
            }],
            preparation: Some(
                "generic room-temperature vegetable-oil teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "the oil remains a conserved unresolved triglyceride mixture; crop, refining, age, additives and brand are not guessed".to_string(),
                "0.92 g/mL is an explicit representative geometry parameter, not a product specification".to_string(),
                "the bounded role supports an upper oil layer on water; emulsions, oxidation, hydrolysis and combustion remain unmodelled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/baking-soda".to_string(),
            version: 1,
            canonical_key: "baking_soda".to_string(),
            name: "baking soda".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec![
                        "bicarb".to_string(),"bicarbonate of soda".to_string(), "sodium bicarbonate".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Natron".to_string(), "Speisesoda".to_string(), "Backnatron".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("NaHCO3", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("pure sodium-bicarbonate teaching surrogate".to_string()),
            lot_assumptions: vec![
                "anti-caking agents and moisture in retail products are omitted".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/heat-activated-baking-powder-surrogate".to_string(),
            version: 1,
            canonical_key: "baking_powder".to_string(),
            name: "heat-activated baking powder surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "en".to_string(),
                    vec!["baking powder".to_string()],
                ),
                (
                    "de".to_string(),
                    vec!["Backpulver".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("NaHCO3", 0.30), component("starch", 0.25)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.45,
                upper: 0.45,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some(
                "heat-activated baking-powder teaching surrogate with 30% sodium bicarbonate, 25% starch and 45% unresolved acid-salt blend"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "heating uses the installed bicarbonate decomposition route and does not require the unresolved acid salts".to_string(),
                "wet or double-acting activation is not claimed until the acid salts and their dissolution kinetics are installed".to_string(),
                "this is not a formulation or substitution recommendation for food".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/washing-soda".to_string(),
            version: 1,
            canonical_key: "washing_soda".to_string(),
            name: "washing soda".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["soda ash".to_string()]),
                (
                    "de".to_string(),
                    vec!["Waschsoda".to_string(), "Reine Soda".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Na2CO3", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("anhydrous sodium-carbonate teaching surrogate".to_string()),
            lot_assumptions: vec![
                "hydrated crystal soda and brand-specific additives are not represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/cornstarch".to_string(),
            version: 1,
            canonical_key: "corn_starch".to_string(),
            name: "cornstarch".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["corn starch".to_string()]),
                (
                    "de".to_string(),
                    vec!["Speisestärke".to_string(), "Maisstärke".to_string(), "Staerke".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("starch", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("dry purified-starch teaching surrogate".to_string()),
            lot_assumptions: vec![
                "botanical source, moisture, lipids and protein traces are omitted".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "school/lugol-solution-1-percent".to_string(),
            version: 1,
            canonical_key: "lugol_solution_1_percent".to_string(),
            name: "1% Lugol iodine solution".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Lugolsche Lösung 1%".to_string(),
                        "Lugol-Lösung 1%".to_string(),
                        "Lugol-Lösung_1%".to_string(),
                        "Iod-Kaliumiodid-Lösung 1%".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec!["dilute Lugol solution".to_string(), "Lugol 1%".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.02)),
            components: vec![
                component("I2", 0.01),
                component("KI", 0.02),
                component("water", 0.97),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "dilute school-test surrogate: 1% w/w iodine and 2% w/w potassium iodide in water"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "iodide-assisted iodine solubilisation is represented with retained KI and aqueous I2 bookkeeping; individual I3-/polyiodide equilibria are not resolved".to_string(),
                "the starch-complex optical response is calibrated to the broad literature band near 600-650 nm; starch source, amylose fraction, chain length and temperature shift its colour and intensity".to_string(),
                "this is a laboratory test reagent model, not a food or medical product".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/liquid-hand-soap-surrogate".to_string(),
            version: 1,
            canonical_key: "liquid_hand_soap".to_string(),
            name: "liquid hand soap".to_string(),
            aliases: BTreeMap::from([
                ("en".to_string(), vec!["hand wash".to_string()]),
                (
                    "de".to_string(),
                    vec![
                        "Flüssigseife".to_string(),
                        "Fluessigseife".to_string(),
                        "flüssige Seife".to_string(),
                        "Handseife".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![component("water", 0.75)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.25,
                upper: 0.25,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::FoamStabilizer {
                trapping_efficiency: 0.75,
                gas_volume_fraction: 0.88,
                half_life_seconds: 120.0,
                saturation_amount: 0.5,
            }],
            preparation: Some(
                "unbranded aqueous liquid-hand-soap teaching surrogate; surfactant blend unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "bar soap is a different material; brand-specific surfactants, moisturisers, salts, fragrance, dye and preservatives remain unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        transparent_colour(
            "household/food-colour-red-betanin",
            "food_colour_red",
            "red food colouring",
            "betanin",
            0.001,
            "dropper solution",
            &["rote Lebensmittelfarbe", "Lebensmittelfarbe_rot"],
            &["red food color", "red_food_colouring", "red_food_color"],
        ),
        transparent_colour(
            "household/food-colour-yellow-curcumin",
            "food_colour_yellow",
            "yellow food colouring",
            "curcumin",
            0.001,
            "dropper solution",
            &["gelbe Lebensmittelfarbe", "Lebensmittelfarbe_gelb"],
            &["yellow food color", "yellow_food_colouring", "yellow_food_color"],
        ),
        transparent_colour(
            "household/food-colour-blue-indigo-carmine",
            "food_colour_blue",
            "blue food colouring",
            "indigo_carmine",
            0.001,
            "dropper solution",
            &["blaue Lebensmittelfarbe", "Lebensmittelfarbe_blau"],
            &["blue food color", "blue_food_colouring", "blue_food_color"],
        ),
        transparent_colour(
            "school/watercolor-red-betanin",
            "watercolour_red",
            "red watercolor",
            "betanin",
            0.0002,
            "wash",
            &["rote Wasserfarbe", "Wasserfarbe_rot"],
            &["red watercolour", "watercolor_red", "red_watercolor", "red_watercolour"],
        ),
        transparent_colour(
            "school/watercolor-yellow-curcumin",
            "watercolour_yellow",
            "yellow watercolor",
            "curcumin",
            0.0002,
            "wash",
            &["gelbe Wasserfarbe", "Wasserfarbe_gelb"],
            &["yellow watercolour", "watercolor_yellow", "yellow_watercolor", "yellow_watercolour"],
        ),
        transparent_colour(
            "school/watercolor-blue-indigo-carmine",
            "watercolour_blue",
            "blue watercolor",
            "indigo_carmine",
            0.0002,
            "wash",
            &["blaue Wasserfarbe", "Wasserfarbe_blau"],
            &["blue watercolour", "watercolor_blue", "blue_watercolor", "blue_watercolour"],
        ),
        acrylic_colour(
            "school/acrylic-red-surrogate",
            "acrylic_red",
            "red acrylic paint",
            pigment_bands(&[(470.0, 3.5, 65.0), (545.0, 2.0, 45.0)]),
            1.0,
            &["rote Acrylfarbe", "Acrylfarbe_rot"],
            &["red acrylic", "red_acrylic", "acrylic_paint_red"],
        ),
        acrylic_colour(
            "school/acrylic-yellow-surrogate",
            "acrylic_yellow",
            "yellow acrylic paint",
            pigment_bands(&[(440.0, 4.0, 45.0)]),
            1.0,
            &["gelbe Acrylfarbe", "Acrylfarbe_gelb"],
            &["yellow acrylic", "yellow_acrylic", "acrylic_paint_yellow"],
        ),
        acrylic_colour(
            "school/acrylic-blue-surrogate",
            "acrylic_blue",
            "blue acrylic paint",
            pigment_bands(&[(650.0, 4.0, 65.0)]),
            1.0,
            &["blaue Acrylfarbe", "Acrylfarbe_blau"],
            &["blue acrylic", "blue_acrylic", "acrylic_paint_blue"],
        ),
        acrylic_colour(
            "school/acrylic-white-surrogate",
            "acrylic_white",
            "white acrylic paint",
            vec![0.0; BAND_NM.len()],
            4.0,
            &["weiße Acrylfarbe", "weisse Acrylfarbe", "Acrylfarbe_weiss"],
            &["white acrylic", "white_acrylic", "acrylic_paint_white"],
        ),
        acrylic_colour(
            "school/acrylic-black-surrogate",
            "acrylic_black",
            "black acrylic paint",
            vec![8.0; BAND_NM.len()],
            0.2,
            &["schwarze Acrylfarbe", "Acrylfarbe_schwarz"],
            &["black acrylic", "black_acrylic", "acrylic_paint_black"],
        ),
        // K18/th-007: the cold-pack salt. A species on its own is not a
        // bottle a child can pick up, and `ammonium_nitrate` is what the
        // prompt asks for; species synonyms do not survive the export, so
        // the everyday name has to be a material like every other one.
        // aq-068: a bar of soap is not the liquid in the pump. Bar soap is
        // a fatty-acid salt and its solution is alkaline; the liquid is a
        // detergent and is not. The shelf had only the liquid.
        MaterialRecipe {
            id: "household/bar-hand-soap".to_string(),
            version: 1,
            canonical_key: "hand_soap".to_string(),
            name: "bar hand soap".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Seifenstück".to_string(), "Kernseife".to_string()]),
                ("en".to_string(), vec!["bar soap".to_string(), "soap bar".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.85)),
            components: vec![component("Na2CO3", 0.02)],
            unresolved_fraction: Some(FractionRange { lower: 0.98, upper: 0.98 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("moulded bar".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![MaterialRole::FoamStabilizer {
                trapping_efficiency: 0.6,
                gas_volume_fraction: 0.88,
                half_life_seconds: 100.0,
                saturation_amount: 1.0,
            }],
            preparation: Some(
                "a bar of ordinary hand soap: the free alkali resolved, the fatty-acid salt that IS the soap conserved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "sodium stearate and its relatives are what a soap bar is made of and none of them is an installed species, so 98% of the bar is conserved. The 2% resolved as carbonate stands for the free alkali a bar carries — it is why the solution is alkaline, and it is NOT the soap".to_string(),
                "that substitution is the reason this recipe reports a raised pH: a real bar's alkalinity comes mostly from the hydrolysis of the fatty-acid salt, which this bench cannot compute, and the carbonate is a stand-in for the effect rather than the cause. Stated because a pH that is right for the wrong reason is the defect this file keeps finding".to_string(),
                "no lathering, no soap scum with hard water, and no cleaning: the fatty-acid salt that would form the scum is exactly the part that is unresolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // aq-023: a laundry powder softens hard water because it carries
        // carbonate, which takes the calcium out as chalk. That is the
        // reaction the bench already computes; what was missing was the box.
        MaterialRecipe {
            id: "household/laundry-detergent-powder".to_string(),
            version: 1,
            canonical_key: "laundry_detergent".to_string(),
            name: "powdered laundry detergent".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Waschpulver".to_string(), "Waschmittel".to_string()]),
                ("en".to_string(), vec!["washing powder".to_string(), "laundry powder".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.65)),
            components: vec![component("Na2CO3", 0.30)],
            unresolved_fraction: Some(FractionRange { lower: 0.70, upper: 0.70 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: vec![MaterialRole::FoamStabilizer {
                trapping_efficiency: 0.55,
                gas_volume_fraction: 0.85,
                half_life_seconds: 90.0,
                saturation_amount: 2.0,
            }],
            preparation: Some(
                "a powdered laundry detergent as a builder plus an unresolved surfactant blend: the washing soda is resolved because it is the part that softens water"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "30% sodium carbonate is a builder figure for a powder, not a formulation: real products vary widely and many modern ones use zeolites or citrate instead, which would soften water by a mechanism this recipe does not have".to_string(),
                "the surfactants, enzymes, optical brighteners, bleach and perfume are the conserved 70% — none of them is an installed species and none is invented".to_string(),
                "the water-softening this can show is carbonate taking calcium out of solution, and nothing else: no cleaning, no stain removal and no claim about what a wash does to cloth".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // aq-022: a dishwasher powder is more alkaline still, and that is
        // the whole of what "what happens when it meets water" can show here.
        MaterialRecipe {
            id: "household/dishwasher-detergent-powder".to_string(),
            version: 1,
            canonical_key: "dishwasher_detergent".to_string(),
            name: "powdered dishwasher detergent".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Spülmaschinenpulver".to_string(), "Geschirrspülmittel Pulver".to_string()]),
                ("en".to_string(), vec!["dishwasher powder".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.90)),
            components: vec![component("Na2CO3", 0.45)],
            unresolved_fraction: Some(FractionRange { lower: 0.55, upper: 0.55 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some(
                "a powdered machine dishwasher detergent: strongly alkaline, with the carbonate resolved and the rest conserved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "45% sodium carbonate stands for the alkaline builders as a group; silicates, phosphates and percarbonate bleach are in the conserved remainder and each would change the chemistry if resolved".to_string(),
                "this is deliberately NOT the same recipe as washing-up liquid: a machine powder is an alkali and a hand detergent is a surfactant, and treating them alike is the mistake the two bottles exist to prevent".to_string(),
                "no enzyme activity, no bleaching and no claim about what it does to grease".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // aq-020: cocoa floats and clumps before it wets. That is the
        // SurfaceFloater role ground black pepper already uses, and it is
        // the whole of what the question asks.
        MaterialRecipe {
            id: "household/cocoa-powder".to_string(),
            version: 1,
            canonical_key: "cocoa_powder".to_string(),
            name: "cocoa powder".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Kakaopulver".to_string(), "Kakao".to_string()]),
                ("en".to_string(), vec!["cocoa".to_string(), "unsweetened cocoa".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.45)),
            components: vec![component("starch", 0.11)],
            unresolved_fraction: Some(FractionRange { lower: 0.89, upper: 0.89 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: vec![MaterialRole::SurfaceFloater { saturation_amount: 3.0 }],
            preparation: Some(
                "unsweetened cocoa powder: a little starch resolved, and the cocoa solids, fat, fibre and alkalising salts conserved unresolved"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "cocoa's fat, fibre, polyphenols and theobromine have no installed species, so 89% of the powder is conserved rather than invented; only its starch is resolved".to_string(),
                "the floating is the point of this recipe and it is a bounded visual role, not a wetting model: cocoa floats because the fat on each grain resists water, and none of that mechanism is represented".to_string(),
                "Dutch-processed cocoa is alkalised and natural cocoa is acidic; this recipe distinguishes neither, and reports no pH of its own".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // aq-091: sweet tea through a strainer. The answer is that a
        // strainer does nothing to what is dissolved, and `filter` says so.
        MaterialRecipe {
            id: "household/sweetened-tea".to_string(),
            version: 1,
            canonical_key: "sweetened_tea".to_string(),
            name: "sweetened tea".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["gesüßter Tee".to_string(), "Süßtee".to_string()]),
                ("en".to_string(), vec!["sweet tea".to_string(), "sugary tea".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.02)),
            components: vec![component("water", 0.90), component("sucrose", 0.09)],
            unresolved_fraction: Some(FractionRange { lower: 0.01, upper: 0.01 }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("brewed tea with about nine grams of sugar in every hundred".to_string()),
            lot_assumptions: vec![
                "the tea itself — tannins, caffeine, colour — is the conserved remainder; this recipe is a sugar solution that came out of a teapot and makes no claim about tea chemistry".to_string(),
                "brewing strength, milk and lemon are not represented, and the liquid reports no colour of its own".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // aq-092: muddy water through a filter and then evaporated — the
        // three-way separation the question names, with real quartz for
        // the mud and real salt for the dissolved part.
        MaterialRecipe {
            id: "household/muddy-water".to_string(),
            version: 1,
            canonical_key: "muddy_water".to_string(),
            name: "muddy water".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Schlammwasser".to_string(), "trübes Wasser".to_string()]),
                ("en".to_string(), vec!["dirty water".to_string(), "turbid water".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.04)),
            components: vec![
                component("water", 0.945),
                component("SiO2", 0.05),
                component("NaCl", 0.005),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.0, upper: 0.0 }),
            physical_form: MaterialPhysicalForm::Suspension,
            roles: Vec::new(),
            preparation: Some(
                "water carrying five percent suspended mineral solids and a little dissolved salt"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the mud is quartz because quartz is what the registry has: real mud is mostly clay minerals with a plate structure that settles far more slowly, and none of that is represented".to_string(),
                "organic matter, colour and smell are absent; this is a mineral suspension rather than a pond".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        familiar_solid(
            "household/ammonium-nitrate",
            "ammonium_nitrate",
            // The species is already called "ammonium nitrate", and a
            // material may not shadow a canonical species name — so the
            // bottle is named for what it is FOR.
            "cold pack salt",
            "NH4NO3",
            "white prills",
            &["Ammoniumnitrat", "Kältepack-Salz"],
            &["instant cold pack salt", "ammonium nitrate prills"],
            &["fertiliser-grade prills carry anti-caking coatings and this recipe has none; what is dispensed is the salt alone",
              "the bare word nitrate stays unclaimed because it names an ion, not this substance",
              "the cold this makes is the enthalpy of dissolution and nothing else: no pack, no membrane to break, and no re-use"],
        ),
        familiar_solid(
            "household/table-salt",
            "table_salt",
            "table salt",
            "NaCl",
            "granular crystals",
            &["Speisesalz", "Tafelsalz", "Kochsalz"],
            &["cooking salt"],
            &["moisture, iodine additives and anti-caking agents are omitted; the bare words salt and Salz remain unclaimed because they name a chemical class"],
        ),
        MaterialRecipe {
            id: "household/epsom-salt-heptahydrate".to_string(),
            version: 1,
            canonical_key: "epsom_salt".to_string(),
            name: "Epsom salt".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Bittersalz".to_string(),
                        "Epsomsalz".to_string(),
                        "Magnesiumsulfat-Heptahydrat".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec!["magnesium sulfate heptahydrate".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("epsomite", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some(
                "pure magnesium-sulfate-heptahydrate crystal teaching surrogate".to_string(),
            ),
            lot_assumptions: vec![
                "moisture, dehydration, grain size and retail additives are omitted; anhydrous magnesium sulfate is a distinct material".to_string(),
                "the seven waters remain bound in the dry crystal inventory until a phase model dissolves or transforms it".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "school/iron-filings".to_string(),
            version: 1,
            canonical_key: "iron_filings".to_string(),
            name: "iron filings".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Eisenfeilspäne".to_string(), "Eisenspäne".to_string()],
                ),
                ("en".to_string(), vec!["filings".to_string(), "iron powder".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Fe", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("clean elemental-iron filings teaching surrogate".to_string()),
            lot_assumptions: vec![
                "surface oxide, cutting oil, particle-size distribution and alloying are omitted"
                    .to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Curated,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/quartz-sand-surrogate".to_string(),
            version: 1,
            canonical_key: "quartz_sand".to_string(),
            name: "quartz-rich sand surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Sand".to_string(), "Quarzsand".to_string(), "Spielsand".to_string()],
                ),
                (
                    "en".to_string(),
                    vec![
                        "play sand".to_string(), "quartz sand".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("SiO2", 0.95)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.05,
                upper: 0.05,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some(
                "washed quartz-rich play-sand teaching surrogate with a conserved variable mineral fraction"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "natural and retail sands vary widely; clay, feldspar, shell, iron minerals, organics, moisture and coatings remain in the explicit 5% unresolved fraction".to_string(),
                "grain-size distribution and colour are not yet resolved".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        familiar_solid(
            "school/calcium-carbonate-chalk-stick",
            "chalk_stick",
            "calcium-carbonate chalk stick",
            "CaCO3",
            "cylindrical stick",
            &["Kreidestück", "Tafelkreide", "Calciumcarbonat-Kreide"],
            &["chalk stick", "calcium carbonate chalk"],
            &["represented as pure calcium carbonate; gypsum chalk and product binders are different materials and are not implied"],
        ),
        familiar_solid(
            "school/magnesium-ribbon",
            "magnesium_ribbon",
            "magnesium ribbon",
            "Mg",
            "thin ribbon",
            &["Magnesiumband"],
            &["magnesium strip"],
            &["represented as clean elemental magnesium; surface oxide and reaction-rate effects wait for the surface-state model"],
        ),
        familiar_solid(
            "school/zinc-strip",
            "zinc_strip",
            "zinc strip",
            "Zn",
            "metal strip",
            &["Zinkstreifen", "Zinkblech"],
            &["zinc sheet"],
            &["represented as clean elemental zinc; galvanised coatings and alloying are not implied"],
        ),
        familiar_solid(
            "household/iron-nail-surrogate",
            "iron_nail",
            "iron nail surrogate",
            "Fe",
            "nail",
            &["Eisennagel"],
            &["iron nail"],
            &["represented as elemental iron; ordinary steel nails, coatings and corrosion products require alloy and surface-state recipes"],
        ),
        MaterialRecipe {
            id: "household/steel-wool-iron-surrogate".to_string(),
            version: 1,
            canonical_key: "steel_wool".to_string(),
            name: "steel wool surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Stahlwolle".to_string(), "Eisenwolle".to_string()],
                ),
                ("en".to_string(), vec!["steel wool".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Fe", 0.98)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.02,
                upper: 0.02,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("porous bundle of fine metal fibres".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some(
                "fine low-carbon steel fibres represented by 98% resolved iron and 2% conserved unresolved alloy/coating fraction"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "grade, carbon, alloying elements, oil, soap and coatings vary; the 98% iron fraction is a bounded teaching surrogate, not a product specification".to_string(),
                "fibre geometry explains why steel wool can ignite more readily than a nail, but the current ignition operator applies an explicit hot-zone threshold rather than claiming a measured area-dependent rate".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        familiar_solid(
            "school/copper-wire",
            "copper_wire",
            "copper wire",
            "Cu",
            "wire",
            &["Kupferdraht"],
            &["bare copper wire"],
            &["represented as bare elemental copper; insulation, lacquer and alloying are omitted"],
        ),
        familiar_solid(
            "household/aluminium-foil",
            "aluminium_foil",
            "aluminium foil",
            "Al",
            "thin foil",
            &["Alufolie", "Aluminiumfolie"],
            &["aluminum foil", "tin foil"],
            &["represented as elemental aluminium inventory; the native oxide layer and passivation kinetics are not yet modeled, so the engine must retain its explicit reaction boundary"],
        ),
        MaterialRecipe {
            id: "household/paraffin-candle-wax".to_string(),
            version: 1,
            canonical_key: "candle_wax".to_string(),
            name: "candle wax".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Kerzenwachs".to_string(), "Paraffinwachs".to_string()],
                ),
                ("en".to_string(), vec!["paraffin wax".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.90)),
            // KID-12: the wax is paraffin, and paraffin is a species now.
            components: vec![component("paraffin", 0.92)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.08,
                upper: 0.08,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("moulded block".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some(
                "solid paraffin candle wax, resolved as paraffin with the stearic acid, dye and scent left in the conserved remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "candle wax is a variable blend of long-chain alkanes spanning roughly C20 to C40, often with stearic acid, dye and scent. KID-12 resolves 92% of it as paraffin, one representative chain length standing for the blend; the stearic acid, dye and scent stay in the conserved remainder. This is a teaching stand-in and not a claim about any candle".to_string(),
                "melting is not claimed: the installed state model derives its transitions from water's enthalpies of fusion and vaporisation and covers no other substance, so heating named wax must reach the engine's ordinary model boundary instead of a curated melt".to_string(),
                "burning is claimed as of KID-12, and only as far as the heat: a curated heat of combustion, the oxygen the vessel actually holds, and carbon dioxide and water out. A wick, a melt pool, a luminous flame and soot are none of them modelled, so this is what a wax releases rather than what a candle looks like".to_string(),
                "the bare words wax and Wachs remain unclaimed as material names because beeswax, soy wax and paraffin wax are different materials. The species paraffin is installed and can be added directly; asking for it by name gets the pure alkane and not this blend".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // KID-13: the object the dancing-raisin experiment is about. It is
        // deliberately NOT resolved into its sugars, and the reason is not
        // that they are missing — sucrose, glucose and fructose are all
        // installed. A raisin is an OBJECT: its sugar is inside it and does
        // not join the water on the timescale of the demonstration, and a
        // recipe that dissolved it would delete the very thing the learner
        // is watching. If slow leaching is ever modelled, this is the
        // recipe to revisit, and this comment is where to start.
        MaterialRecipe {
            id: "household/raisin".to_string(),
            version: 1,
            canonical_key: "raisin".to_string(),
            // The key and the display name may not normalise to the same
            // string — the validator holds both in one namespace — so a
            // one-word material is named the way the registry names its
            // other disambiguated entries.
            name: "raisin (dried grape)".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Rosine".to_string(), "Rosinen".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["raisins".to_string(), "dried grape".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.35)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("wrinkled dried grape".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![MaterialRole::ConservedUnresolvedSolid {
                srgb: [90, 58, 46],
                colour_word: "dark brown".to_string(),
            }],
            preparation: Some(
                "a seedless dried grape, conserved whole as the object it is"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the sugars a raisin is mostly made of are installed species and are deliberately not resolved: a raisin is an object whose sugar stays inside it over a demonstration, and dissolving it into the water would delete the thing being watched".to_string(),
                "1.35 g/mL is a bulk density for the whole fruit, which is what decides whether it sinks; it is not a claim about the skin, the flesh or the air the wrinkles trap, and a raisin that has soaked for an hour is lighter than one straight from the box".to_string(),
                "the wrinkled surface is what real bubbles nucleate on, and no surface texture, nucleation-site count or bubble size is represented anywhere in this recipe".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/paper-sheet".to_string(),
            version: 1,
            canonical_key: "paper_sheet".to_string(),
            name: "paper".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Papier".to_string(), "Druckerpapier".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["sheet of paper".to_string(), "office paper".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.80)),
            // KID-20/12: paper is cellulose, and cellulose is a species.
            components: vec![component("cellulose", 0.85)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.15,
                upper: 0.15,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("thin sheet".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some(
                "a sheet of ordinary white paper, resolved as cellulose with filler and sizing left in the conserved remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "KID-20's rule caught this one: the sheet was conserved unresolved because \"cellulose is not in the runtime registry\", and cellulose has been in it since. 85% of the sheet is now resolved as cellulose, which is what paper is, and the reason that had expired is gone rather than left reading as current".to_string(),
                "mineral filler, sizing, coatings and optical brighteners vary by grade and stay inside the same unresolved mass; a carbonate-filled office paper and an unfilled newsprint are not distinguished".to_string(),
                "burning is claimed as of KID-12, and only as far as the heat: a curated heat of combustion per anhydroglucose unit, the oxygen the vessel actually holds, and carbon dioxide and water out. Char, smoke, flame spread and the ash that a real sheet leaves are not modelled".to_string(),
                "wetting, tearing and pulping are not claimed, and this material is not the filter paper the filtration apparatus uses".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/wheat-flour-surrogate".to_string(),
            version: 1,
            canonical_key: "wheat_flour".to_string(),
            name: "wheat flour".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Mehl".to_string(),
                        "Weizenmehl".to_string(),
                        "Weißmehl".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec![
                        "flour".to_string(),
                        "plain flour".to_string(),
                        "all-purpose flour".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.55)),
            components: vec![component("starch", 0.70)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.30,
                upper: 0.30,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some(
                "white wheat flour with a resolved 70% starch fraction and a conserved 30% protein, moisture, fibre, lipid and ash remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the resolved fraction is starch, which is why a named iodine indicator finds it; the gluten proteins, sorbed moisture, fibre, lipids and ash stay in the conserved 30% remainder because none of them is an installed species".to_string(),
                "the flour's moisture is deliberately not resolved as liquid water: it is sorbed in the grain, and adding it to the vessel's free liquid would repeat the error the Epsom-salt recipe exists to avoid".to_string(),
                "mill type, ash grade, protein content, malted amylase and ascorbic-acid improvers vary widely; wholemeal, rye and spelt flours are different materials and are not implied".to_string(),
                "no gluten network, dough rheology, starch gelatinisation, enzymatic hydrolysis or baking behaviour is claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/flour-and-water-dough".to_string(),
            version: 1,
            canonical_key: "flour_water_dough".to_string(),
            name: "flour-and-water dough".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Teig".to_string(), "Mehlteig".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["dough".to_string(), "simple dough".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("starch", 0.42)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.58,
                upper: 0.58,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "kneaded soft solid mass".to_string(),
            },
            roles: Vec::new(),
            preparation: Some(
                "a plain kneaded dough of three parts wheat flour to two parts water, resolving the flour's 42% starch and conserving the rest"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the dough's water is held in the flour matrix and is conserved inside the unresolved remainder rather than poured into the vessel's free liquid; the bench has no water-activity or matrix model, so it must not report a pool of water that a real dough would not release".to_string(),
                "no bulk density is claimed, so dough is dispensed by mass; a kneaded mass has no reviewed packing figure and guessing one would be a visible number without provenance".to_string(),
                "gluten development, kneading, viscoelasticity, proving, gelatinisation and baking are not claimed; adding yeast to this material does not make bread".to_string(),
                "hydration ratio, salt, flour grade and resting time vary; this is the plain flour-and-water teaching mixture only".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/apple-juice-surrogate".to_string(),
            version: 1,
            canonical_key: "apple_juice".to_string(),
            name: "apple juice surrogate".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Apfelsaft".to_string()]),
                ("en".to_string(), vec!["apple juice".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.045)),
            // KID-20: two of this recipe's reasons had expired.
            //
            // It carried "most of apple juice's sugar is fructose and
            // glucose, and neither is an installed species" and "malic acid
            // ... is not in the registry". Both were true when they were
            // written and neither is now — fructose, glucose and malic acid
            // are all shipped species with reviewed solubility limits. A
            // recipe that explains an omission by a fact that has since
            // changed is worse than one that never explained it: the
            // sentence reads as current.
            //
            // The cost was visible: a pH map of the kitchen reported nothing
            // at all for apple juice, because a juice with no acid in it has
            // no acidity for the engine to characterise (KIDS.md, K50).
            components: vec![
                component("water", 0.8824),
                component("fructose", 0.059),
                component("glucose", 0.024),
                component("sucrose", 0.013),
                component("malic_acid", 0.005),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0166,
                upper: 0.0166,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "clear unsweetened apple-juice teaching surrogate: 88% water, the sugars in the proportions the cited composition gives them, and the malic acid the tartness is actually made of"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the sugars are resolved in the cited proportions — fructose first, then glucose, then sucrose — rather than relabelled as one sugar; each is an installed species with its own solubility limit".to_string(),
                "malic acid is the acid apple juice's tartness is made of and is resolved as itself, so the pH this juice reports is computed from the right molecule rather than borrowed from a convenient one. The amount is a composition figure, not a titration of any particular juice".to_string(),
                "pectin, minerals, vitamin C, colour and aroma compounds share the conserved unresolved remainder; cloudy, concentrate-reconstituted and fresh-pressed juices differ and are not distinguished".to_string(),
                "no juicing, browning, pasteurisation, fermentation or nutritional claim is made".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // K13: the invisible ink needs something to write with, and the
        // shelf had no lemon juice. Unlike apple juice, this one's acid
        // computes its own pH: minteq.v4 carries citrate, so the tartness
        // is speciated rather than merely conserved — which is the whole
        // difference between a juice that reports an acidity and one that
        // has to say it cannot.
        MaterialRecipe {
            id: "household/lemon-juice-surrogate".to_string(),
            version: 1,
            canonical_key: "lemon_juice".to_string(),
            name: "lemon juice surrogate".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Zitronensaft".to_string(), "Zitronensaft frisch".to_string()],
                ),
                ("en".to_string(), vec!["lemon juice".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![
                component("water", 0.907),
                component("citric_acid", 0.047),
                component("fructose", 0.010),
                component("glucose", 0.010),
                component("sucrose", 0.005),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.021,
                upper: 0.021,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "fresh-squeezed lemon-juice teaching surrogate: about 91% water, the citric acid the sourness is made of, and a little sugar"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "citric acid is resolved as itself and is the acid a lemon's sourness is made of; minteq.v4 defines a citrate species, so unlike this shelf's apple juice the pH here is computed from the acid rather than reported as uncharacterisable".to_string(),
                "4.7% citric acid is a composition figure for fresh juice, not a titration of any particular lemon: variety, ripeness and how hard it was squeezed all move it, and bottled juice is usually weaker and preserved".to_string(),
                "the pH this reports, about 1.9, is the pH of that much free citric acid and is lower than the 2.2-2.4 a real lemon measures. A lemon carries citrate salts as well as the acid, and those buffer it upwards; this recipe resolves the acid and not its salts, so the number is right for what is in the glass and low for the fruit. Correcting it by weakening the acid would move the composition to fix the pH, which is the wrong end to pull".to_string(),
                "vitamin C is the one component a lemon is famous for and it is NOT resolved, which is a deliberate choice against the obvious one. Ascorbic acid is an installed species, but no shipped database defines an ascorbate, so the bench cannot dissolve it: resolving 0.05% of the juice would put an undissolved grain in the glass and describe it as vitamin C. It made the juice read 'very slightly hazy', which a real lemon juice also is — for pulp, which this recipe does not have. A believable observable produced by the wrong mechanism is worse than an absent one, so the vitamin C stays in the conserved remainder until it can dissolve".to_string(),
                "pectin, limonene and the other oils, flavonoids, minerals and colour share the conserved unresolved remainder; pith, pulp and zest are not represented at all".to_string(),
                "no juicing, browning, preservation or nutritional claim is made, and writing with it is a use rather than a property — the ink turning brown under heat is chemistry this bench does not have".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/soda-lime-glass".to_string(),
            version: 1,
            canonical_key: "glass".to_string(),
            name: "soda-lime glass".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Glas".to_string(),
                        "Fensterglas".to_string(),
                        "Glasscherbe".to_string(),
                    ],
                ),
                ("en".to_string(), vec!["window glass".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.5)),
            components: vec![component("SiO2", 0.73)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.27,
                upper: 0.27,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("flat pane fragment".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(1000000000000.0, 10000000000.0, 10000000000000.0, "Ordinary soda-lime window glass at room temperature. Glass does not conduct by electrons at all: what little current it passes is sodium ions creeping through the silicate network, so the resistivity falls by roughly a decade for every 40 to 50 K of heating and hot glass is a far worse insulator than cold glass. The alkali content is what sets it, which is why the borosilicate row sits a decade higher and the fused-silica row four decades higher again. No temperature coefficient is claimed, and neither is the SURFACE resistance, which on damp or dirty glass runs many orders of magnitude below the bulk and is what actually flashes over."),
            ],
            preparation: Some(
                "an ordinary soda-lime glass object: 73% resolved silica with a conserved 27% network-modifier remainder"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; in glass that silica is an amorphous network rather than the quartz grains of play sand, and no polymorph, devitrification or network claim follows from sharing one species key".to_string(),
                "the soda, lime, magnesia and alumina that make a melt workable are network modifiers, not free oxides in a beaker; resolving them as CaO or MgO would invent an alkaline dissolution that glass does not perform, so they stay in the conserved 27% remainder".to_string(),
                "glass is unreactive in the terms the aqueous engine models, which is not the same as inert: hydrofluoric-acid etching and hot-alkali attack are real chemistry the bench does not have and does not claim".to_string(),
                "softening, annealing, thermal shock, breaking and sharp edges are physics this recipe does not claim; borosilicate, lead crystal and fused quartz are different materials".to_string(),
                "this names the material. Laboratory glassware is created with the vessel commands, not dispensed from the shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "laboratory/fused-silica".to_string(),
            version: 1,
            canonical_key: "silica_glass".to_string(),
            name: "fused silica glass".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Quarzglas".to_string(), "Kieselglas".to_string()]),
                ("en".to_string(), vec!["fused quartz".to_string(), "vitreous silica".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.2)),
            components: vec![
                component("SiO2", 1.0),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("thick-walled tube or crucible".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(1e16, 1000000000000000.0, 1e18, "Fused silica, the best insulator on this shelf and one of the best bulk insulators there is: with essentially no alkali there is nothing mobile to carry a current, and what conduction remains is set by defects rather than by composition. The span is wide because a number this large is measurement-limited - leakage across the sample surface and through the apparatus swamps the bulk conduction, so what a laboratory reports is partly a property of its guard ring. No temperature dependence and no particular grade is claimed."),
            ],
            preparation: Some("fused silica: pure silicon dioxide melted and cooled to a glass, resolved in full".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "this is the one glass on the shelf with nothing in the conserved remainder: fused silica really is silica alone, which is why it is dearer and why it takes heat the soda-lime glass will not".to_string(),
                "the amorphous network and the quartz crystal share the SiO2 species and differ in structure, not in composition; the bench models composition, so it cannot tell them apart and does not pretend to".to_string(),
                "glass is unreactive in the terms the aqueous engine models, which is not the same as inert: hydrofluoric-acid etching and hot-alkali attack are real chemistry the bench does not have and does not claim".to_string(),
                "softening, annealing, thermal shock, scratching and breaking are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "laboratory/borosilicate-glass".to_string(),
            version: 1,
            canonical_key: "borosilicate_glass".to_string(),
            name: "borosilicate glass".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Borosilikatglas".to_string(), "Laborglas".to_string()]),
                ("en".to_string(), vec!["Pyrex-type glass".to_string(), "lab glass".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.23)),
            components: vec![
                component("SiO2", 0.81),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.19, upper: 0.19 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("beaker or test-tube wall".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(10000000000000.0, 100000000000.0, 1000000000000000.0, "Laboratory borosilicate at room temperature. It is about a decade more resistive than soda-lime glass for the same reason it takes thermal shock better: far less alkali in the network, so far less of the mobile sodium that carries the current. Temperature dependence, surface leakage and the behaviour of any particular commercial composition are not claimed."),
            ],
            preparation: Some("ordinary laboratory borosilicate: 81% resolved silica with a conserved 19% boria-and-modifier remainder".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "the boron oxide that makes this glass take thermal shock is NOT resolved, and it is the whole reason the material has a separate name: no shipped database defines a borate that would let it dissolve or react, so resolving it would put an invented oxide in the beaker. It stays in the conserved 19%, which means the bench holds the composition of borosilicate without holding the property borosilicate is bought for".to_string(),
                "low thermal expansion, and therefore the resistance to cracking on a hotplate, is the property this recipe cannot express; a run that needs it needs physics this bench does not have".to_string(),
                "glass is unreactive in the terms the aqueous engine models, which is not the same as inert: hydrofluoric-acid etching and hot-alkali attack are real chemistry the bench does not have and does not claim".to_string(),
                "softening, annealing, thermal shock, scratching and breaking are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/coloured-glass".to_string(),
            version: 1,
            canonical_key: "colored_glass".to_string(),
            name: "coloured glass".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Buntglas".to_string(), "farbiges Glas".to_string(), "Braunglas".to_string()]),
                ("en".to_string(), vec!["stained glass".to_string(), "amber glass".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.5)),
            components: vec![
                component("SiO2", 0.72),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.28, upper: 0.28 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("bottle-wall fragment".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(1000000000000.0, 10000000000.0, 10000000000000.0, "A coloured soda-lime bottle glass, read as its uncoloured parent: the few tenths of a percent of transition-metal oxide that make it brown or green change its colour and not its conduction. The window-glass caveats hold unchanged - ionic conduction by the alkali, a strong temperature dependence, and no claim at all about surface leakage."),
            ],
            preparation: Some("a coloured soda-lime glass: 72% resolved silica with a conserved 28% modifier-and-colourant remainder".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "the colourant is a few tenths of a percent of transition-metal oxide and is NOT resolved. Resolving it as, say, an iron or cobalt species would put a dissolvable metal in a beaker that contains a bottle, and would let an acid leach colour out of glass, which is chemistry that does not happen on this timescale".to_string(),
                "the colour is therefore a fact about the object recorded in this note and not a property the bench can compute or see; nothing in a run will report that this glass is brown".to_string(),
                "glass is unreactive in the terms the aqueous engine models, which is not the same as inert: hydrofluoric-acid etching and hot-alkali attack are real chemistry the bench does not have and does not claim".to_string(),
                "softening, annealing, thermal shock, scratching and breaking are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "mineral/quartz-crystal".to_string(),
            version: 1,
            canonical_key: "quartz".to_string(),
            name: "quartz crystal".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Bergkristall".to_string(), "Quarzkristall".to_string()]),
                ("en".to_string(), vec!["rock crystal".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.65)),
            components: vec![
                component("SiO2", 1.0),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("hexagonal prism with terminated point".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(100000000000000.0, 1000000000000.0, 1e16, "Crystalline quartz, and this row carries the sharpest caveat on the shelf: quartz is strongly ANISOTROPIC, and the resistivity measured along the c axis runs roughly two orders of magnitude below the value across it. A single figure for a crystal is therefore a class average and not a measurement of any orientation; impurity level matters as much again. No axis, no grade, and none of the piezoelectric behaviour the crystal is actually used for is claimed here."),
            ],
            preparation: Some("a single crystal of quartz: silicon dioxide, resolved in full".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "composition alone does not distinguish this from fused silica or from a sand grain; what makes it a crystal is the long-range order, which the bench does not represent. The density difference, 2.65 against 2.20, is the only handle a run has on the distinction".to_string(),
                "piezoelectricity, birefringence, cleavage and the habit that makes crystals collectable are physics this recipe does not claim".to_string(),
                "glass is unreactive in the terms the aqueous engine models, which is not the same as inert: hydrofluoric-acid etching and hot-alkali attack are real chemistry the bench does not have and does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "ceramic/porcelain".to_string(),
            version: 1,
            canonical_key: "porcelain".to_string(),
            name: "porcelain (fired china)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Porzellan".to_string()]),
                ("en".to_string(), vec!["china".to_string(), "bone-white ceramic".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.4)),
            components: vec![
                component("SiO2", 0.68),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.32, upper: 0.32 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("fired dish or evaporating basin".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(1000000000000.0, 10000000000.0, 100000000000000.0, "Electrical porcelain at room temperature - the material of an overhead-line insulator, and the reason it is that material. What current such a body does pass is carried by alkali ions in the glassy phase between the mullite crystals, so the number falls sharply on heating and depends on the body's alkali content: a fired ceramic is not a substance with one resistivity the way copper is. And the failure mode of a real insulator is not bulk conduction at all - it is a wet or salted SURFACE, which conducts many orders of magnitude better than anything inside the body. No temperature coefficient, no surface resistance, and no dielectric strength is claimed: the volts per millimetre at which the material breaks down is a different property, and it is the one an insulator is actually specified on."),
            ],
            preparation: Some("fired porcelain: 68% resolved silica with a conserved 32% alumina-and-flux remainder".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "the alumina from the kaolin is the largest single thing in the conserved remainder and is not resolved: no Al2O3 species is installed, and mapping it to aluminium metal would put a reactive metal in a teacup".to_string(),
                "firing is what turned the clay into this, and the bench has neither the reaction nor the temperature history: porcelain and the clay it was made from are separate materials on this shelf, not two states of one".to_string(),
                "softening, annealing, thermal shock, scratching and breaking are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "ceramic/glazed-ceramic".to_string(),
            version: 1,
            canonical_key: "glazed_ceramic".to_string(),
            name: "glazed ceramic".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["glasierte Keramik".to_string(), "Steingut".to_string()]),
                ("en".to_string(), vec!["glazed earthenware".to_string(), "glazed tile".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.3)),
            components: vec![
                component("SiO2", 0.65),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.35, upper: 0.35 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("glazed tile or mug fragment".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                resistivity(100000000000.0, 1000000000.0, 10000000000000.0, "A glazed earthenware body, about a decade below porcelain because the body is more porous and more alkali-rich. Two things this row cannot express: the object is a thin glassy glaze over a porous body and this is one averaged bulk figure for both, and a porous body that has taken up water conducts far better than a dry one - which is why the number is claimed only for the dry object this meter is allowed to touch."),
            ],
            preparation: Some("a glazed earthenware body: 65% resolved silica with a conserved 35% body-and-glaze remainder".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "the glaze is a thin glassy skin over a porous body and the two have different compositions; this recipe reports one averaged composition for the whole object and cannot represent the layer structure, which is the thing that makes a glazed pot hold water and an unglazed one weep".to_string(),
                "lead and cadmium in old glazes are a genuine hazard of real objects and are not represented here; nothing in this recipe should be read as a statement that a glaze is safe to eat from".to_string(),
                "softening, annealing, thermal shock, scratching and breaking are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "mineral/pumice".to_string(),
            version: 1,
            canonical_key: "pumice".to_string(),
            name: "pumice (volcanic froth)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Bimsstein".to_string()]),
                ("en".to_string(), vec!["pumice stone".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.64)),
            components: vec![
                component("SiO2", 0.7),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.3, upper: 0.3 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("porous stone".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("volcanic pumice: 70% resolved silica with a conserved 30% remainder, at the bulk density of the frothed rock".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "0.64 g/mL is a BULK density and is the point of the material: the glass it is made of is denser than water and the stone floats anyway, because most of its volume is trapped gas. Whole-object buoyancy compares this reviewed bulk density with the liquid density and keeps the resolved silica from being drawn again as a loose deposit".to_string(),
                "the pores are also why real pumice slowly waterlogs and sinks; there is no transport into a pore in this bench, so it floats indefinitely".to_string(),
                "abrasiveness, and the vesicular texture behind it, are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "mineral/clay".to_string(),
            version: 1,
            canonical_key: "clay".to_string(),
            name: "clay (kaolin)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Ton".to_string(), "Tonerde".to_string(), "Kaolin".to_string()]),
                ("en".to_string(), vec!["potter's clay".to_string(), "china clay".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.8)),
            components: vec![
                component("SiO2", 0.45),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.55, upper: 0.55 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("damp potter's clay: 45% resolved silica with a conserved 55% alumina-water-and-mineral remainder".to_string()),
            lot_assumptions: vec![
                "the silica is reported as the installed SiO2 species so the object's silicon and oxygen are real inventory; sharing one species key with quartz sand carries no claim about polymorph, grain structure or optical quality".to_string(),
                "kaolinite is Al2Si2O5(OH)4 and only its silica is resolved; the alumina and the structural hydroxyls have no installed species, so more than half this material is conserved mass with no chemistry attached".to_string(),
                "plasticity is the property clay is used for and it is a behaviour of wet platelets sliding, not a composition: the bench has the mass and not the modelling clay".to_string(),
                "firing clay to a ceramic is an irreversible reaction this bench does not have; the fired materials are separate entries on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "metal/stainless-steel".to_string(),
            version: 1,
            canonical_key: "stainless_steel".to_string(),
            name: "stainless steel".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Edelstahl".to_string(), "rostfreier Stahl".to_string()]),
                ("en".to_string(), vec!["inox".to_string(), "18/8 steel".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(7.9)),
            components: vec![
                component("Fe", 0.7),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.3, upper: 0.3 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("spoon, rod or sheet".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("austenitic 18/8 stainless: 70% resolved iron with a conserved 30% chromium-and-nickel remainder".to_string()),
            lot_assumptions: vec![
                "the iron is resolved as the installed Fe species, so displacement chemistry that acts on iron will act on this object's iron".to_string(),
                "the chromium and nickel are NOT resolved because no Cr or Ni species is installed, and this is the most misleading gap in the entry: chromium is exactly what makes stainless steel stainless. The bench therefore holds an object whose iron can be attacked and has no representation of the passive oxide film that stops the attack, so any corrosion result here is a result about plain iron wearing a stainless label".to_string(),
                "a run that needs the difference between this and mild steel needs the chromium, and until Cr is installed this entry cannot supply it".to_string(),
                "alloy phase, work hardening, magnetism and the grades behind 304 and 316 are metallurgy this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "metal/galvanized-steel".to_string(),
            version: 1,
            canonical_key: "galvanized_steel".to_string(),
            name: "galvanised steel".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["verzinkter Stahl".to_string(), "Verzinkung".to_string()]),
                ("en".to_string(), vec!["zinc-coated steel".to_string(), "galvanised iron".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(7.85)),
            components: vec![
                component("Fe", 0.97),
                component("Zn", 0.03),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("sheet or nail with a bright spangled coat".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("hot-dip galvanised steel: iron under a zinc coat, both resolved, at 3% zinc by mass".to_string()),
            lot_assumptions: vec![
                "both metals are installed species, so this is one of the few composite objects on the shelf with nothing in a conserved remainder".to_string(),
                "the zinc is a COATING and this recipe reports it as a bulk fraction: the bench mixes the 3% through the object instead of putting it on the outside. That matters for the thing galvanising is for — the zinc corrodes first and protects the iron underneath, which is a geometry argument about which metal the liquid reaches, and this entry has no geometry to make it with".to_string(),
                "so a displacement or acid run here will consume zinc and iron together in whatever ratio the chemistry prefers, rather than stripping the coat and then attacking the steel".to_string(),
                "sacrificial protection, coating thickness and the spangle pattern are not claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "metal/painted-iron".to_string(),
            version: 1,
            canonical_key: "painted_iron".to_string(),
            name: "painted iron".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["lackiertes Eisen".to_string(), "gestrichenes Eisen".to_string()]),
                ("en".to_string(), vec!["painted steel".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(7.6)),
            components: vec![
                component("Fe", 0.95),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.05, upper: 0.05 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("painted bar or railing section".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("an iron object under a paint film: 95% resolved iron with a conserved 5% paint remainder".to_string()),
            lot_assumptions: vec![
                "the iron is resolved and the paint is not; there is no binder or pigment species and inventing one would put a dissolvable organic in the beaker".to_string(),
                "as with the galvanised entry, the paint is a barrier and this recipe has no geometry to place it: the bench will let a liquid reach the iron directly, which is what happens where paint is chipped and not what happens where it is sound. Results here describe bare iron".to_string(),
                "adhesion, chipping, primer chemistry and rust creeping under a film are not claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "polymer/expanded-polystyrene".to_string(),
            version: 1,
            canonical_key: "expanded_PS".to_string(),
            name: "expanded polystyrene".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Styropor".to_string(), "expandiertes Polystyrol".to_string()]),
                ("en".to_string(), vec!["styrofoam".to_string(), "EPS foam".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.03)),
            components: vec![
                component("PS", 1.0),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("expanded polystyrene foam: the polystyrene species at the bulk density of the blown foam".to_string()),
            lot_assumptions: vec![
                "the polymer is resolved as the installed PS species, so the solvent behaviour of polystyrene applies to this material — acetone really does collapse it, and that is the same chemistry as for solid polystyrene".to_string(),
                "0.03 g/mL is a BULK density: the polystyrene itself is about 1.05 g/mL and would sink. Almost all of this material's volume is air. The bench's float-and-sink test reads the density of each SPECIES a material resolves into, not the material's own bulk density, and no general material-level buoyancy exists: `bulk_density` is consumed only by the raisin bubble-ride. So this figure is recorded, is right, and does not float or sink anything, and what a run sees is the resolved polystyrene, so a foam cup sinks here".to_string(),
                "the collapse of a foam cup in acetone is dramatic because the gas escapes and the volume falls by a factor of fifty; the bench can dissolve the polymer but cannot show the collapse, having no cells to empty".to_string(),
                "cell structure, insulation and the mechanical crush behaviour are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "electronics/intrinsic-silicon".to_string(),
            version: 1,
            canonical_key: "silicon".to_string(),
            name: "intrinsic silicon".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Silizium".to_string(), "reines Silizium".to_string(), "undotiertes Silizium".to_string()]),
                ("en".to_string(), vec!["undoped silicon".to_string(), "pure silicon".to_string(), "semiconductor-grade silicon".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.33)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a polished wafer chip".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                MaterialRole::ConservedUnresolvedSolid {
                    srgb: [96, 99, 104],
                    colour_word: "grey".to_string(),
                },
                resistivity(2300.0, 2000.0, 3000.0, "Intrinsic - undoped - silicon at 300 K. The number is not a property of the crystal the way copper's is: it is fixed by how many electrons thermal energy has lifted across the 1.12 eV band gap, about 1e16 per cubic metre at room temperature, and that count roughly doubles for every 8 K of heating. So this row is strongly temperature-dependent by nature and the span here covers only the ordinary spread of tabulated room-temperature values, not that dependence. Real 'undoped' wafers are rarely this resistive either: parts per billion of a dopant is enough to dominate the carriers, which is the whole point of the doped-silicon row beside this one."),
            ],
            preparation: Some("a chip of undoped single-crystal silicon, conserved whole because no elemental silicon species is installed".to_string()),
            lot_assumptions: vec![
                "no elemental silicon is installed in this registry, so the chip is conserved as named matter rather than dispensed as a species. The SiO2 that the glass and ceramic recipes resolve is silicon DIOXIDE - a different substance with different chemistry - and mapping a silicon wafer onto it would put sand in the beaker".to_string(),
                "the reason this recipe exists is to stand beside doped_silicon on the conductivity meter. The pair is one reviewed measurement each, and the difference between the two readings is the answer; neither row computes anything about the other".to_string(),
                "silicon's own reaction chemistry - the oxide skin it grows in air, its dissolution in hot alkali, its attack by hydrofluoric acid - is not modelled and not claimed".to_string(),
                "band gap, carrier mobility, the temperature dependence of the conductivity, and every optical and mechanical property are physics this recipe does not claim".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "electronics/doped-silicon".to_string(),
            version: 1,
            canonical_key: "doped_silicon".to_string(),
            name: "doped silicon".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["dotiertes Silizium".to_string(), "n-dotiertes Silizium".to_string(), "Siliziumwafer".to_string()]),
                ("en".to_string(), vec!["n-type silicon".to_string(), "phosphorus-doped silicon".to_string(), "silicon wafer".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(2.33)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a polished wafer chip".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                MaterialRole::ConservedUnresolvedSolid {
                    srgb: [86, 89, 96],
                    colour_word: "grey".to_string(),
                },
                resistivity(0.01, 1e-5, 0.1, "An n-type phosphorus-doped silicon wafer of an ordinary 1 ohm.cm grade. THE VALUE IS ONE POINT IN A RANGE THIS RECIPE DOES NOT PIN DOWN: doped silicon as sold spans about 1e-5 to 1e-1 ohm.m, which end a wafer sits at is set by its dopant concentration - roughly 1e21 to 1e25 atoms per cubic metre - and neither the span nor the reading claims a particular wafer. What changed against the intrinsic-silicon row is the CARRIER DENSITY and nothing else: each dopant atom contributes one mobile electron (phosphorus) or one hole (boron), five to eight orders of magnitude more carriers than the thermal ones, while the lattice, the mobility and the band gap are all essentially as they were. No carrier-density model is computed here and none is claimed - these are two reviewed measurements of two objects, and the sentence between them is the mechanism, not a calculation."),
            ],
            preparation: Some("a chip of phosphorus-doped n-type silicon at an ordinary 1 ohm.cm grade, conserved whole because no elemental silicon species is installed".to_string()),
            lot_assumptions: vec![
                "the dopant is about one phosphorus atom in a hundred thousand of silicon and is deliberately NOT resolved as a species: at that level it would be a rounding error in the mass balance and a fiction in the chemistry, while being the entire reason the material conducts. It is stated here, and carried by the resistivity row, and nowhere else".to_string(),
                "this recipe pins ONE grade. Doped silicon as sold spans four orders of magnitude in resistivity, and the resistivity row states that span beside its value so that no run reads the number as a property of doped silicon in general".to_string(),
                "n-type and p-type are not distinguished anywhere the bench can see: a hole and an electron conduct alike here, so the sign of the Hall voltage, the rectifying junction between the two types, and therefore every semiconductor DEVICE are outside this bench entirely".to_string(),
                "silicon's own reaction chemistry is not modelled and not claimed, and neither is the temperature dependence that makes a semiconductor conduct better when it is hot while a metal conducts worse".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/apple".to_string(),
            version: 1,
            canonical_key: "apple".to_string(),
            name: "apple (fresh fruit)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Apfel".to_string()]),
                ("en".to_string(), vec!["eating apple".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.85)),
            components: vec![
                component("water", 0.851),
                component("fructose", 0.062),
                component("glucose", 0.024),
                component("sucrose", 0.021),
                component("cellulose", 0.024),
                component("citric_acid", 0.005),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.013, upper: 0.013 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("whole fruit".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a fresh eating apple: water, its three sugars, cell-wall cellulose and a little fruit acid, resolved; skin, pectin, pigment and aroma conserved".to_string()),
            lot_assumptions: vec![
                "the sugars are resolved separately rather than as one 'sugar' because an apple's sweetness is mostly fructose, and the bench can now dissolve each of them at its own solubility".to_string(),
                "0.85 g/mL is the bulk density of the whole fruit and is below water, which is why an apple bobs — because of the air in the core and between the cells, not because apple flesh is lighter than water, which it barely is. Whole-object buoyancy compares this reviewed bulk density with the liquid density".to_string(),
                "malic acid is the acid an apple actually contains and it is NOT installed; citric acid stands in for it at the same mass. The pH will be about right and the identity is wrong, which is a substitution recorded here rather than hidden".to_string(),
                "browning is the reaction an apple is used to teach and this bench does not have it: no polyphenol oxidase, no phenolic substrate, no quinone. A cut apple here stays the colour it started".to_string(),
                "the cellulose is resolved as the installed species and is the cell wall; it does not dissolve and is not digested here, which is right for water and wrong for a gut".to_string(),
                "cooking is a reaction this bench does not have: this recipe is the raw material, and the cooked one is neither derivable from it nor present on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/potato".to_string(),
            version: 1,
            canonical_key: "potato".to_string(),
            name: "potato (tuber)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Kartoffel".to_string()]),
                ("en".to_string(), vec!["white potato".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.08)),
            components: vec![
                component("water", 0.792),
                component("starch", 0.165),
                component("cellulose", 0.021),
                component("glucose", 0.006),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.016, upper: 0.016 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("whole tuber".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a raw potato: water, storage starch, cell-wall cellulose and a trace of free sugar".to_string()),
            lot_assumptions: vec![
                "the starch is the point of the tuber and is resolved, so an iodine test on this material has something real to bind to".to_string(),
                "1.08 g/mL is above water, and the gap between it and the apple's 0.85 is what the salt-water flotation demonstration turns on. Whole-object buoyancy compares this reviewed bulk density with the liquid density, so a potato sinks in ordinary water".to_string(),
                "gelatinisation is what happens when a potato is boiled and the bench does not have it: the starch here cannot swell, burst or thicken, so a heated potato changes temperature and nothing else".to_string(),
                "solanine, the reason green potatoes are not eaten, is not represented; nothing here should be read as a statement about whether a potato is safe".to_string(),
                "the cellulose is resolved as the installed species and is the cell wall; it does not dissolve and is not digested here, which is right for water and wrong for a gut".to_string(),
                "cooking is a reaction this bench does not have: this recipe is the raw material, and the cooked one is neither derivable from it nor present on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/onion".to_string(),
            version: 1,
            canonical_key: "onion".to_string(),
            name: "onion (bulb)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Zwiebel".to_string()]),
                ("en".to_string(), vec!["cooking onion".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.96)),
            components: vec![
                component("water", 0.89),
                component("glucose", 0.024),
                component("fructose", 0.012),
                component("sucrose", 0.01),
                component("cellulose", 0.017),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.047, upper: 0.047 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("whole bulb of concentric layers".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a raw onion: water, its sugars and cell-wall cellulose resolved; the sulfur compounds conserved".to_string()),
            lot_assumptions: vec![
                "the propanethial S-oxide that makes an onion sting the eyes is the whole interest of the material and is NOT resolved: it is made by an enzyme acting on a sulfoxide the instant the cell is cut, and this bench has neither the enzyme, the substrate nor the cutting".to_string(),
                "so the bench holds an onion that cannot make anyone cry, and the sulfur chemistry sits in the conserved 4.7% as mass".to_string(),
                "the concentric-layer structure is recorded as a shape and carries no consequence; osmosis across an onion cell membrane is transport this bench does not model".to_string(),
                "the cellulose is resolved as the installed species and is the cell wall; it does not dissolve and is not digested here, which is right for water and wrong for a gut".to_string(),
                "cooking is a reaction this bench does not have: this recipe is the raw material, and the cooked one is neither derivable from it nor present on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/cabbage".to_string(),
            version: 1,
            canonical_key: "cabbage".to_string(),
            name: "cabbage (head)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Kohl".to_string(), "Weisskohl".to_string(), "Rotkohl".to_string()]),
                ("en".to_string(), vec!["white cabbage".to_string(), "red cabbage head".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.95)),
            components: vec![
                component("water", 0.922),
                component("glucose", 0.019),
                component("fructose", 0.016),
                component("cellulose", 0.025),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.018, upper: 0.018 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("dense head of leaves".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a raw cabbage head: water, sugars and cell-wall cellulose resolved; anthocyanin and glucosinolates conserved".to_string()),
            lot_assumptions: vec![
                "this is the WHOLE VEGETABLE and not the indicator: the red-cabbage juice already on this shelf is a separate material with the anthocyanin resolved for its colour response. Adding a cabbage to acid will not turn anything pink, because the pigment is in this entry's conserved remainder".to_string(),
                "that split is deliberate. The juice earns its resolved pigment by being an extract made for the purpose; the head has the pigment locked in cells the bench cannot break open".to_string(),
                "the glucosinolates behind the smell of cooking cabbage are conserved and unmodelled".to_string(),
                "the cellulose is resolved as the installed species and is the cell wall; it does not dissolve and is not digested here, which is right for water and wrong for a gut".to_string(),
                "cooking is a reaction this bench does not have: this recipe is the raw material, and the cooked one is neither derivable from it nor present on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/bread".to_string(),
            version: 1,
            canonical_key: "bread".to_string(),
            name: "bread (baked loaf)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Brot".to_string()]),
                ("en".to_string(), vec!["white loaf".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.27)),
            components: vec![
                component("water", 0.37),
                component("starch", 0.45),
                component("cellulose", 0.027),
                component("maltose", 0.012),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.141, upper: 0.141 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("slice of an open-crumb loaf".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a baked wheat loaf: water, starch, a little cellulose and the maltose baking leaves behind".to_string()),
            lot_assumptions: vec![
                "0.27 g/mL is a BULK density and most of a loaf's volume is gas: the crumb is a foam the bench has the mass of and not the structure".to_string(),
                "the starch is resolved, so amylase acts on bread here exactly as it does on starch, which is the chewing demonstration and is the one piece of bread chemistry this bench genuinely has".to_string(),
                "gluten is the protein that made the foam hold and it is conserved and unmodelled; crust, browning and the Maillard reaction that flavours it are absent".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/pasta".to_string(),
            version: 1,
            canonical_key: "pasta".to_string(),
            name: "dried pasta".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Nudeln".to_string(), "Teigwaren".to_string()]),
                ("en".to_string(), vec!["dry noodles".to_string(), "spaghetti".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.31)),
            components: vec![
                component("water", 0.106),
                component("starch", 0.628),
                component("cellulose", 0.031),
                component("sucrose", 0.009),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.226, upper: 0.226 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("dried extruded strand".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("dried durum pasta: mostly starch at 10% residual moisture".to_string()),
            lot_assumptions: vec![
                "dry pasta sinks and cooked pasta is soft, and the difference is water taken up into gelatinised starch: this bench has no uptake and no gelatinisation, so pasta here never cooks".to_string(),
                "the 22.6% conserved remainder is largely the durum protein that gives pasta its bite".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
                "cooking is a reaction this bench does not have: this recipe is the raw material, and the cooked one is neither derivable from it nor present on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/rice".to_string(),
            version: 1,
            canonical_key: "rice".to_string(),
            name: "dried rice grain".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Reis".to_string()]),
                ("en".to_string(), vec!["white rice".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.85)),
            components: vec![
                component("water", 0.12),
                component("starch", 0.72),
                component("cellulose", 0.013),
                component("glucose", 0.002),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.145, upper: 0.145 }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("dried milled rice: starch grains at 12% residual moisture, at the bulk density of poured grain".to_string()),
            lot_assumptions: vec![
                "0.85 g/mL is the density of POURED GRAIN with the air between the grains included; a single grain is denser than water and sinks. The figure is right for a measuring cup and wrong for a grain, and the bench's float-and-sink test reads the density of each SPECIES a material resolves into, not the material's own bulk density, and no general material-level buoyancy exists: `bulk_density` is consumed only by the raisin bubble-ride. So this figure is recorded, is right, and does not float or sink anything".to_string(),
                "that ambiguity is a real limitation of one number standing in for a particulate solid, and it is recorded rather than corrected, because correcting it to the grain density would break every run that measures out a volume of rice".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
                "cooking is a reaction this bench does not have: this recipe is the raw material, and the cooked one is neither derivable from it nor present on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/honey".to_string(),
            version: 1,
            canonical_key: "honey".to_string(),
            name: "honey (clear jar)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Honig".to_string()]),
                ("en".to_string(), vec!["clear honey".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.42)),
            components: vec![
                component("water", 0.172),
                component("fructose", 0.382),
                component("glucose", 0.312),
                component("sucrose", 0.013),
                component("citric_acid", 0.005),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.116, upper: 0.116 }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("honey: a supersaturated fructose-and-glucose syrup at 17% water".to_string()),
            lot_assumptions: vec![
                "honey is SUPERSATURATED in glucose and that is why real honey crystallises in the jar. This bench will try to dissolve the sugars at their handbook solubilities and will report the excess as undissolved solid, so a jar of honey here reads as a grainy syrup from the first moment rather than after six months".to_string(),
                "the number is right and the timescale is absent: crystallisation is a nucleation process with a clock, and this bench has neither".to_string(),
                "gluconic acid is honey's acid and is not installed; citric acid stands in at the same mass, so the pH is about right and the identity is wrong".to_string(),
                "the enzymes, the hydrogen peroxide behind honey's antibacterial reputation, and the pollen that identifies its source are conserved and unmodelled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/butter".to_string(),
            version: 1,
            canonical_key: "butter".to_string(),
            name: "dairy butter".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Suessrahmbutter".to_string(), "Streichbutter".to_string()]),
                ("en".to_string(), vec!["sweet cream butter".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.911)),
            components: vec![
                component("water", 0.16),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.84, upper: 0.84 }),
            physical_form: MaterialPhysicalForm::Other {
                description: "a firm water-in-fat emulsion".to_string(),
            },
            roles: Vec::new(),
            preparation: Some("dairy butter: 16% water resolved, the butterfat conserved".to_string()),
            lot_assumptions: vec![
                "butterfat is a mixture of triglycerides and no triglyceride species is installed, so 84% of this material is conserved mass. That is the largest unresolved fraction of any food on this shelf and it is the food itself".to_string(),
                "butter is a WATER-IN-FAT emulsion, the inverse of milk, and the bench has no emulsion structure: the water is reported as a component and not as droplets held in fat".to_string(),
                "melting is a phase change this recipe does not claim; butter has no single melting point in any case, softening across a range as its different fats melt in turn".to_string(),
                "saponification, rancidity and browning are absent".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/cream".to_string(),
            version: 1,
            canonical_key: "cream".to_string(),
            name: "dairy cream".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Sahne".to_string(), "Rahm".to_string()]),
                ("en".to_string(), vec!["whipping cream".to_string(), "double cream".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.01)),
            components: vec![
                component("water", 0.578),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.422, upper: 0.422 }),
            physical_form: MaterialPhysicalForm::Suspension,
            roles: Vec::new(),
            preparation: Some("whipping cream: 58% water resolved, the butterfat and milk protein conserved".to_string()),
            lot_assumptions: vec![
                "lactose is the sugar of milk and is NOT installed, so cream's sugar joins its fat and its casein in the conserved 42%".to_string(),
                "whipping is the point of cream and it is mechanical: air beaten in, protein unfolding at the bubble surface, fat globules partly coalescing to hold the foam. None of that is in this bench, so cream here cannot be whipped".to_string(),
                "the same fat-globule structure is why cream and milk differ, and the bench distinguishes them only by how much water each has".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/egg-white".to_string(),
            version: 1,
            canonical_key: "egg_white".to_string(),
            name: "egg white (raw)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Eiklar".to_string(), "Eiweiss".to_string()]),
                ("en".to_string(), vec!["raw albumen".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.04)),
            components: vec![
                component("water", 0.88),
                component("glucose", 0.004),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.116, upper: 0.116 }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("raw egg white: 88% water resolved, the albumen proteins conserved".to_string()),
            lot_assumptions: vec![
                "an egg white is water and protein and nothing else of consequence, so with no protein species installed this recipe is water plus a conserved tenth that is the entire point of the material".to_string(),
                "the white turning opaque and solid at about 65 C is THE demonstration this material exists for, and it is a protein unfolding and cross-linking: the bench has neither, so heating an egg white here raises its temperature and leaves it clear and liquid".to_string(),
                "whipping to a meringue is the same protein at an air interface and is equally absent".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/gelatin".to_string(),
            version: 1,
            canonical_key: "gelatin".to_string(),
            name: "gelatine (dry sheets)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Gelatine".to_string(), "Blattgelatine".to_string()]),
                ("en".to_string(), vec!["leaf gelatin".to_string(), "gelatine powder".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.3)),
            components: vec![
                component("water", 0.1),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.9, upper: 0.9 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("dry gelatine: 10% residual moisture, the collagen-derived protein conserved".to_string()),
            lot_assumptions: vec![
                "gelatine is a protein and no protein species is installed, so 90% of this material is conserved mass".to_string(),
                "gelatine sets a gel by cooling: the chains re-form a triple helix below about 35 C and trap the water. This bench has no gelation, so gelatine here dissolves nowhere and sets nothing, and a jelly cannot be made".to_string(),
                "that also means the classic pineapple-and-jelly demonstration cannot run to its conclusion here even with the pineapple present: the protease would need a protein to cut, and the protein is conserved".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/albumin".to_string(),
            version: 1,
            canonical_key: "albumin".to_string(),
            name: "albumin (dried protein)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Eialbumin".to_string(), "Trockeneiweiss".to_string()]),
                ("en".to_string(), vec!["dried egg-white protein".to_string(), "serum albumin".to_string(), "protein".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.36)),
            components: vec![
                component("water", 0.06),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.94, upper: 0.94 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some("dried albumin: 6% residual moisture, the protein conserved".to_string()),
            lot_assumptions: vec![
                "this is the purified protein rather than a food, and with no protein species installed it is 94% conserved mass".to_string(),
                "it exists on this shelf so that a run about protein has something to name; what it cannot do is behave like one. Denaturation by heat, by acid, by alcohol and by salt are four separate demonstrations albumin is used for and the bench has none of them".to_string(),
                "the Biuret and Ninhydrin tests that would identify it are equally absent".to_string(),
                "it answers to the bare word 'protein' as well as to its own name, because it is the only purified protein on this shelf. That alias is a pointer, not a claim of generality: a digestion run that asks for protein gets dried egg-white albumin and its conserved remainder, not a representative average of dietary protein".to_string(),
                "no protein species is installed, so every protein in this food sits in the conserved remainder as mass without chemistry; denaturation, coagulation and the setting of a cooked white are reactions this bench does not have".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "textile/cotton".to_string(),
            version: 1,
            canonical_key: "cotton".to_string(),
            name: "cotton fibre".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Baumwolle".to_string()]),
                ("en".to_string(), vec!["cotton wool".to_string(), "cotton ball".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.08)),
            components: vec![
                component("cellulose", 1.0),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("cotton fibre: cellulose, resolved in full, at the bulk density of loose wadding".to_string()),
            lot_assumptions: vec![
                "cotton is about 95% cellulose and this recipe rounds it to all of it, which makes it one of the few fully resolved materials on the shelf".to_string(),
                "0.08 g/mL is the bulk density of loose wadding and almost all of that volume is air; the fibre itself is about 1.54 g/mL and sinks once wetted, which is why a real cotton ball floats and then does not".to_string(),
                "the bench has neither wetting nor material-level buoyancy: the bench's float-and-sink test reads the density of each SPECIES a material resolves into, not the material's own bulk density, and no general material-level buoyancy exists: `bulk_density` is consumed only by the raisin bubble-ride. So this figure is recorded, is right, and does not float or sink anything, and the cellulose this resolves to carries the fibre density, so cotton here simply sinks".to_string(),
                "burning cotton is the flame test that distinguishes it from a synthetic fibre; the combustion of cellulose is chemistry this bench has, but the smell and the ash that make the test work are not modelled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // KID-14: the two bottles a slime activity needs. School glue is a
        // poly(vinyl alcohol) dispersion; borax is sold as the decahydrate
        // and the ten waters ride in the conserved remainder rather than
        // being relabelled as the anhydrous salt.
        MaterialRecipe {
            id: "school/pva-craft-glue".to_string(),
            version: 1,
            canonical_key: "pva_glue".to_string(),
            name: "PVA craft glue".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Bastelkleber".to_string(),
                        "Weißleim".to_string(),
                        "PVA-Kleber".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec![
                        "glue".to_string(),
                        "white glue".to_string(),
                        "school glue".to_string(),
                        "craft glue".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.05)),
            components: vec![component("PVA", 0.22), component("water", 0.72)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.06,
                upper: 0.06,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some(
                "unbranded white craft glue: a 22% w/w poly(vinyl alcohol) dispersion in water"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "22% w/w PVA is an explicit teaching concentration for an unbranded school glue; retail glues vary and many are poly(vinyl acetate) rather than the alcohol, which does not gel with borate the same way".to_string(),
                "plasticiser, filler, preservative and defoamer share the conserved unresolved remainder rather than being invented as molecules".to_string(),
                "degree of hydrolysis and chain length are what make one glue slime well and another not, and neither is modelled".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "household/borax-decahydrate".to_string(),
            version: 1,
            canonical_key: "borax".to_string(),
            name: "borax powder".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec!["Boraxpulver".to_string(), "Natriumborat".to_string()],
                ),
                (
                    "en".to_string(),
                    vec!["sodium borate".to_string(), "washing borax".to_string()],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: vec![component("Na2B4O7", 0.527)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.473,
                upper: 0.473,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: Vec::new(),
            preparation: Some(
                "household borax as sold: the decahydrate, so a little over half its mass is the anhydrous salt and the rest is water of crystallisation"
                    .to_string(),
            ),
            lot_assumptions: vec![
                "borax is sold as the decahydrate Na2B4O7·10H2O; 52.7% of that mass is the anhydrous salt, and the ten waters stay in the conserved remainder rather than being released as free water, because dehydrating a hydrate is a phase change this recipe does not perform".to_string(),
                "borate speciation is not modelled: no shipped database is asked for it, so this contributes no pH and no ionic strength, and the bench says so".to_string(),
                "borax is a mild irritant and not for eating; the L0 screen classes it by what it does to other substances, which is nothing it knows".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // KID-9: the black felt-tip. A black ink is a mixture, which is the
        // entire point of the experiment: the strip proves it by taking it
        // apart. Three of the shipped dyes, in the proportions that make a
        // dark neutral.
        MaterialRecipe {
            id: "school/felt-tip-ink-black".to_string(),
            version: 1,
            canonical_key: "black_ink".to_string(),
            name: "black felt-tip ink".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "schwarze Tinte".to_string(),
                        "Filzstifttinte".to_string(),
                        "Tinte_schwarz".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec![
                        "marker_ink".to_string(),
                        "ink".to_string(),
                        "black felt tip".to_string(),
                        "felt_tip_ink".to_string(),
                        "black marker ink".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: vec![
                component("indigo_carmine", 0.0018),
                component("betanin", 0.0012),
                component("curcumin", 0.0009),
                component("water", 0.9),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0961,
                upper: 0.0961,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::SurfaceColourant { srgb: [32, 30, 36] }],
            preparation: Some(
                "unbranded water-based black felt-tip ink: three shipped dyes at the ratio that reads as a dark neutral, in a glycol-and-water carrier".to_string(),
            ),
            lot_assumptions: vec![
                "the three named dyes and their proportions define this surrogate; no retail pen's formulation is claimed, and a real black ink may use quite different colourants".to_string(),
                "the humectant and surfactant carrier stays together as a conserved unresolved remainder rather than fictional molecules".to_string(),
                "that a black ink is a mixture at all is the claim worth making here; which three dyes it is made of is a teaching choice".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // KID-8: the jar of cabbage water. The pigment's pH-dependent colour
        // lives in `kerotakis_core::indicator::PIGMENT_LADDERS` with its own
        // provenance; this record is only the bottle it arrives in.
        MaterialRecipe {
            id: "household/red-cabbage-juice-surrogate".to_string(),
            version: 1,
            canonical_key: "red_cabbage_juice".to_string(),
            name: "red-cabbage indicator juice".to_string(),
            aliases: BTreeMap::from([
                (
                    "de".to_string(),
                    vec![
                        "Rotkohlsaft".to_string(),
                        "Blaukrautsaft".to_string(),
                        "Rotkohlindikator".to_string(),
                    ],
                ),
                (
                    "en".to_string(),
                    vec![
                        "red cabbage juice".to_string(),
                        "cabbage indicator".to_string(),
                        "red_cabbage_indicator".to_string(),
                        "purple cabbage juice".to_string(),
                        // bio-092 asks for the same jar by a different word.
                        "red_cabbage_extract".to_string(),
                    ],
                ),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: vec![component("anthocyanin", 0.0003), component("water", 0.98)],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0197,
                upper: 0.0197,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::SurfaceColourant {
                srgb: [107, 63, 160],
            }],
            preparation: Some(
                "chopped red cabbage steeped in hot water and strained: a well-steeped jar, about 0.03% w/w anthocyanin".to_string(),
            ),
            lot_assumptions: vec![
                "the 0.03% w/w anthocyanin fraction is an explicit teaching concentration at the strong end of what a well-steeped jar reaches; it is not a measurement of any cultivar or extraction".to_string(),
                "sugars, other pigments and cell-wall material stay together as conserved unresolved cabbage solids rather than fictional molecules".to_string(),
                "the juice's own pH is not asserted: a real extract is mildly acidic and carries buffering that this surrogate does not model, so the colour a vessel shows is the pH of what it was added to".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-016: an oil-in-water emulsion that does not separate. The
        // stability is a MODELLING CHOICE here, not a computed result: the
        // bench has no coalescence to compute, so what the recipe supplies
        // is a colloid that stays one thing, and the note says so.
        MaterialRecipe {
            id: "food/mayonnaise".to_string(),
            version: 1,
            canonical_key: "mayonnaise".to_string(),
            name: "full-fat mayonnaise".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Majonäse".to_string(), "Salatmayonnaise".to_string()]),
                ("en".to_string(), vec!["mayo".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.91)),
            components: vec![
                component("water", 0.16),
                component("NaCl", 0.012),
                component("CH3COOH", 0.004),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.824,
                upper: 0.824,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::OpaqueLiquidColloid {
                    srgb: [246, 238, 205],
                    opacity_saturation_g_per_litre: 40.0,
                },
            ],
            preparation: Some("ordinary full-fat mayonnaise: the water, salt and vinegar acid resolved, and the oil and egg yolk that are four fifths of the jar conserved unresolved".to_string()),
            lot_assumptions: vec![
                "about 82% of a full-fat mayonnaise is oil and egg yolk. Neither has an installed species - there is no triglyceride and no lecithin on this shelf - so that fraction is conserved as named matter rather than given a stand-in molecule".to_string(),
                "the emulsion's STABILITY is the question this material exists for and it is not computed. Real mayonnaise holds because lecithin from the yolk sits at every oil-water interface and lowers it; this bench has no interfacial model, no droplet size and no coalescence, so what a run shows is a colloid that does not separate because nothing in the model separates it. That is the right observation for the wrong reason, and it should not be read as an explanation".to_string(),
                "the 0.4% acetic acid stands for the vinegar or lemon juice a recipe uses; it makes the material mildly acidic, which is true, but the preservation that acidity provides is not modelled".to_string(),
                "no thickness, no viscosity and no breaking: whipping it, warming it or adding oil too fast are the three ways a real emulsion fails and none of them is represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-017: mustard is an emulsifier, and that is a role this
        // schema already has. The seed mucilage and its proteins are what
        // actually do it; neither is an installed species, so the role
        // carries the bounded observable and the components carry only
        // what is really there.
        MaterialRecipe {
            id: "food/prepared-mustard".to_string(),
            version: 1,
            canonical_key: "mustard".to_string(),
            name: "prepared mustard".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Senf".to_string(), "Tafelsenf".to_string()]),
                ("en".to_string(), vec!["table mustard".to_string(), "yellow mustard".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.05)),
            components: vec![
                component("water", 0.68),
                component("CH3COOH", 0.016),
                component("NaCl", 0.018),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.286,
                upper: 0.286,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "a thick milled-seed paste".to_string(),
            },
            roles: vec![
                MaterialRole::AqueousEmulsifier {
                    saturation_amount: 2.0,
                    max_dispersed_fraction: 0.55,
                    half_life_seconds: 1800.0,
                },
            ],
            preparation: Some("prepared table mustard: water, vinegar acid and salt resolved, and the milled seed that does the emulsifying conserved unresolved".to_string()),
            lot_assumptions: vec![
                "the emulsifying is a bounded dose response, not chemistry. Mustard works because the seed coat's mucilage and its proteins adsorb at the oil-water interface; none of that is an installed species, so the recipe declares the observable and conserves the seed".to_string(),
                "the parameters are teaching values: about half the oil dispersed at a couple of grams of mustard, holding for something like half an hour. They are not a measurement of any mustard and not a claim about droplet size".to_string(),
                "the heat of mustard - the isothiocyanates released when the seed is wetted - is absent, and so is the colour: this recipe reports no pigment of its own".to_string(),
                "the 1.6% acetic acid is the vinegar the seed is milled into, and it is the whole of the acidity claimed here".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-040: the sugar is real and resolved; the SETTING is pectin
        // gelation and is not modelled at all.
        MaterialRecipe {
            id: "food/fruit-jam".to_string(),
            version: 1,
            canonical_key: "jam".to_string(),
            name: "fruit jam".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Marmelade".to_string(), "Konfitüre".to_string()]),
                ("en".to_string(), vec!["fruit preserve".to_string(), "jelly preserve".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.33)),
            components: vec![
                component("sucrose", 0.44),
                component("fructose", 0.09),
                component("glucose", 0.09),
                component("water", 0.3),
                component("citric_acid", 0.005),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.075,
                upper: 0.075,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "a set fruit gel".to_string(),
            },
            roles: Vec::new(),
            preparation: Some("an ordinary set fruit jam at about 62% sugar: the three sugars, the water and the fruit acid resolved, and the pectin and fruit solids conserved unresolved".to_string()),
            lot_assumptions: vec![
                "62% total sugar is the concentration a jam is boiled to and the reason it keeps; it is an explicit teaching figure for a set jam rather than a measurement of any preserve".to_string(),
                "the SETTING is not modelled. A jam sets because pectin chains cross-link once enough sugar has taken the water and the acid has lowered the pH, and this bench has no polymer network, no gelation and no viscosity. The sugar that makes it happen is in the vessel; what it does is not".to_string(),
                "keeping is not modelled either: water activity, the quantity that actually stops microbes growing, has no representation here".to_string(),
                "the split between sucrose and the invert sugars varies with fruit and boiling time and this recipe fixes one point; the pectin and the fruit's own cell-wall solids are the conserved remainder".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-065: solid fat beside liquid oil. The bench holds the
        // difference as a fact in a note and cannot compute it - there is
        // no triglyceride species and therefore no melting point.
        MaterialRecipe {
            id: "food/coconut-fat".to_string(),
            version: 1,
            canonical_key: "coconut_fat".to_string(),
            name: "coconut fat".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Kokosfett".to_string(), "Kokosnussfett".to_string()]),
                ("en".to_string(), vec!["solid coconut fat".to_string(), "coconut butter".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.92)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::Other {
                description: "a firm white fat, solid at room temperature".to_string(),
            },
            roles: vec![
                MaterialRole::ConservedUnresolvedSolid {
                    srgb: [250, 249, 242],
                    colour_word: "white".to_string(),
                },
            ],
            preparation: Some("coconut fat as it sits in the jar: firm, white and wholly unresolved".to_string()),
            lot_assumptions: vec![
                "there is no triglyceride on this shelf, so the fat is conserved whole rather than given an invented molecule. That also means it has NO melting point here, which is exactly the property the question is about: coconut fat is solid at room temperature because its fatty acids are short and saturated, and this bench cannot show that".to_string(),
                "the comparison with a liquid vegetable oil is therefore a comparison of two conserved materials with different densities and different notes, not a computed phase difference".to_string(),
                "nothing about saponification, rancidity or nutrition is claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-107: 70% ethanol. Evaporation and its latent heat are real
        // engine behaviour, so this row gets an answer rather than a note.
        MaterialRecipe {
            id: "household/alcohol-hand-sanitiser".to_string(),
            version: 1,
            canonical_key: "hand_sanitiser".to_string(),
            name: "alcohol hand sanitiser".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Händedesinfektionsmittel".to_string(), "Handdesinfektionsgel".to_string()]),
                ("en".to_string(), vec!["hand sanitizer".to_string(), "hand rub".to_string(), "alcohol gel".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.87)),
            components: vec![
                component("ethanol", 0.7),
                component("water", 0.27),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.03,
                upper: 0.03,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("an alcohol hand rub at 70% ethanol by mass, with the gelling agent and humectant conserved unresolved".to_string()),
            lot_assumptions: vec![
                "70% ethanol is the concentration hand rubs are sold at, and it is the strength this recipe fixes; the remaining 3% stands for the carbomer gel, glycerol and neutraliser that make it a gel rather than a splash".to_string(),
                "the cooling a hand feels is the ethanol's enthalpy of vaporisation and the bench computes it, which is why this material is worth having. The DISINFECTION is not modelled in any way: nothing here kills anything, and the concentration that matters for that is stated as a fact rather than used".to_string(),
                "no gel viscosity, no skin, and no evaporation rate: how fast it goes is not represented, only that it goes and what that costs in heat".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-101: an alcohol solution whose evaporation the bench does
        // compute. What it cannot do is smell, or spread through a room.
        MaterialRecipe {
            id: "household/alcohol-perfume".to_string(),
            version: 1,
            canonical_key: "perfume".to_string(),
            name: "alcohol-based perfume".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Parfüm".to_string(), "Duftwasser".to_string()]),
                ("en".to_string(), vec!["eau de toilette".to_string(), "fragrance spray".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.87)),
            components: vec![
                component("ethanol", 0.8),
                component("water", 0.15),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.05,
                upper: 0.05,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("an alcohol-based perfume: 80% ethanol with water, and the fragrance oils conserved unresolved".to_string()),
            lot_assumptions: vec![
                "the 5% fragrance concentrate is conserved rather than resolved: a perfume's odorants are dozens of terpenes and esters, and none of them is an installed species".to_string(),
                "the question is why a smell crosses a room, and DIFFUSION IS NOT MODELLED HERE. This bench has no gas-phase transport between a vessel and a room; what it can show is that the alcohol evaporates and what that takes in heat. The odorants riding that vapour, and their journey to a nose, are outside it entirely".to_string(),
                "there is no odour entry for any component, so nothing in a run smells of anything".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-105: the point of the row is that this ink's solvent is an
        // alcohol and not water. The three installed dyes stand in for the
        // pigment exactly as the water-based felt-tip ink already does.
        MaterialRecipe {
            id: "school/permanent-marker-ink".to_string(),
            version: 1,
            canonical_key: "permanent_marker_ink".to_string(),
            name: "permanent marker ink".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Permanentmarker-Tinte".to_string(), "Alkoholmarker-Tinte".to_string()]),
                ("en".to_string(), vec!["permanent ink".to_string(), "alcohol marker ink".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.86)),
            components: vec![
                component("ethanol", 0.62),
                component("indigo_carmine", 0.0018),
                component("betanin", 0.0012),
                component("curcumin", 0.0009),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.3761,
                upper: 0.3761,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("a permanent marker's ink: an alcohol solvent carrying dye, with the film-forming resin conserved unresolved".to_string()),
            lot_assumptions: vec![
                "the solvent is ethanol because ethanol is the installed alcohol; a real permanent marker uses n-propanol or n-butanol, which are not on this shelf. The claim the recipe makes is about the CLASS of solvent, and that is the claim the question turns on".to_string(),
                "the three dyes are the same stand-ins the water-based felt-tip ink uses, at the same order of concentration. A real permanent marker's colourant is a resin-bound pigment, not a soluble food dye, and the difference is exactly why one washes out and the other does not - a difference this recipe cannot express".to_string(),
                "the 38% conserved remainder is the resin that makes the mark permanent. It has no installed species, so nothing here binds to paper, and no run will show a mark that resists water".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-115: an essential oil that does not dissolve in water. The
        // immiscible-liquid role is the same one vegetable oil uses, and
        // it is the whole of the answer the bench can give.
        MaterialRecipe {
            id: "food/orange-peel-oil".to_string(),
            version: 1,
            canonical_key: "orange_oil".to_string(),
            name: "orange peel oil".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Orangenschalenöl".to_string(), "Orangenöl".to_string()]),
                ("en".to_string(), vec!["citrus peel oil".to_string(), "sweet orange oil".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.844)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::AqueousImmiscibleLiquid {
                    srgb: [240, 178, 60],
                    colour_word: "orange".to_string(),
                },
            ],
            preparation: Some("cold-pressed orange peel oil: a wholly unresolved essential oil that floats on water".to_string()),
            lot_assumptions: vec![
                "orange peel oil is about 95% d-limonene, a terpene with no installed species, so the whole material is conserved rather than given a molecule. Its density of 0.844 g/mL is a real measured property of the oil and is why it floats".to_string(),
                "the layering is a bounded material role and not a computed liquid-liquid equilibrium: the recipe declares that this liquid stays separate and sits on top, which is what a jar shows, and claims no partition coefficient and no mutual solubility".to_string(),
                "limonene DOES dissolve polystyrene and is used as a solvent for it; that chemistry is absent here, and so is the smell".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-109: soap and grease. `dish_soap` already carries the
        // emulsifier role, so what was missing was something greasy for it
        // to act on.
        MaterialRecipe {
            id: "household/kitchen-grease".to_string(),
            version: 1,
            canonical_key: "grease".to_string(),
            name: "kitchen grease".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Küchenfett".to_string(), "Bratfett".to_string()]),
                ("en".to_string(), vec!["cooking grease".to_string(), "greasy film".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.92)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![
                MaterialRole::AqueousImmiscibleLiquid {
                    srgb: [238, 226, 190],
                    colour_word: "pale yellow".to_string(),
                },
            ],
            preparation: Some("the fat left on a pan or a pair of hands: a wholly unresolved liquid fat that will not mix with water".to_string()),
            lot_assumptions: vec![
                "kitchen grease is a mixture of triglycerides and their breakdown products and none of them is an installed species, so it is conserved whole. What the recipe supplies is the one property the question needs: it does not mix with water".to_string(),
                "the cleaning that follows is the detergent's bounded emulsifier role acting on this layer under stirring. It is a declared dose response, not a micelle model: nothing here computes a critical micelle concentration, a droplet size or a rinse".to_string(),
                "soiled grease also carries particles, and no dirt is represented".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // th-047: hexane is the one installed alkane inside petrol's
        // carbon-number range, and CEA has a record for it - so this row
        // burns through the ordinary equilibrium route rather than a
        // curated fuel table.
        MaterialRecipe {
            id: "fuel/petrol".to_string(),
            version: 1,
            canonical_key: "petrol".to_string(),
            name: "unleaded petrol".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Benzin".to_string(), "Ottokraftstoff".to_string()]),
                ("en".to_string(), vec!["gasoline".to_string(), "motor spirit".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.745)),
            components: vec![
                component("hexane", 0.85),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.15,
                upper: 0.15,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("petrol as a single representative light alkane: 85% resolved hexane with the aromatics, branched isomers and additives conserved unresolved".to_string()),
            lot_assumptions: vec![
                "real petrol is a blend spanning roughly C4 to C12 with a substantial aromatic fraction; hexane is the one alkane in that range this registry installs, and it stands for the whole blend. It is NOT a claim that petrol is hexane".to_string(),
                "octane rating, and therefore knock resistance, is a property of the blend that this single-component surrogate cannot express - which matters because it is the usual reason petrol and diesel behave differently in an engine".to_string(),
                "volatility is likewise not represented: a real puddle of petrol supplies vapour far faster than hexane's own boiling point suggests, and the ignition this bench performs takes a charge rather than a vapour above a pool".to_string(),
                "the conserved 15% is the aromatic and additive fraction; it contributes no chemistry and no energy".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // th-048: the other half of the comparison, and it routes the
        // opposite way. NASA CEA's thermo.inp has no alkane above C10, so
        // neither of these two reaches the equilibrium solver at all;
        // `ignite` answers them through the curated combustion table,
        // whose autoignition temperatures are what make the row's point.
        MaterialRecipe {
            id: "fuel/diesel".to_string(),
            version: 1,
            canonical_key: "diesel".to_string(),
            name: "diesel fuel".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Dieselkraftstoff".to_string(), "Dieselöl".to_string()]),
                ("en".to_string(), vec!["gas oil".to_string(), "automotive gas oil".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.832)),
            components: vec![
                component("dodecane", 0.45),
                component("hexadecane", 0.3),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.25,
                upper: 0.25,
            }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("diesel as a two-component n-alkane surrogate spanning the C10-C16 cut: 45% resolved n-dodecane and 30% resolved n-hexadecane, with the aromatic and cycloalkane quarter conserved unresolved".to_string()),
            lot_assumptions: vec![
                "real diesel is a distillation cut spanning roughly C10 to C22 with a quarter of its mass in aromatics and cycloalkanes; two straight-chain alkanes stand for the whole cut. It is NOT a claim that diesel is dodecane and hexadecane".to_string(),
                "cetane number, and therefore ignition delay in an engine, is a property of the blend that this surrogate cannot express - which matters because it is the usual reason diesel and petrol behave differently in an engine. What the surrogate DOES carry is the direction: n-hexadecane is cetane, the reference fuel the scale is defined against, and its autoignition temperature is below the petrol surrogate's".to_string(),
                "the flash point is not represented at all. Diesel's flash point is about 325 K against petrol's 230 K, and that - not the autoignition temperature - is why a match dropped into diesel goes out and a match dropped into petrol does not. This bench has no flash-point model and no vapour above a pool, so it cannot show that half of the difference".to_string(),
                "the conserved 25% is the aromatic and cycloalkane fraction; it contributes no chemistry and no energy here, so this fuel carries less energy than a real litre of diesel".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // th-060: the water in the log is the part the bench really can
        // do - it is inventory that takes heat. The smoke is not.
        MaterialRecipe {
            id: "fuel/damp-wood".to_string(),
            version: 1,
            canonical_key: "wet_wood".to_string(),
            name: "damp wood".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["feuchtes Holz".to_string(), "nasses Holz".to_string()]),
                ("en".to_string(), vec!["damp firewood".to_string(), "unseasoned wood".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.75)),
            components: vec![
                component("cellulose", 0.42),
                component("water", 0.3),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.28,
                upper: 0.28,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("split log".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a split log of unseasoned wood at about 30% moisture: cellulose and water resolved, and the lignin and hemicellulose conserved unresolved".to_string()),
            lot_assumptions: vec![
                "30% water by mass is what freshly split, unseasoned wood carries, against something under 20% for firewood that has been stacked a year. That water is real inventory in the vessel and it takes heat, which is the half of the question this bench can answer".to_string(),
                "the SMOKE is not modelled. Damp wood smokes because the water cools the flame below the temperature that burns the pyrolysis volatiles, and this bench has neither pyrolysis nor volatiles nor soot. What it burns is the cellulose, straight to carbon dioxide and water, leaving no char and no ash".to_string(),
                "the conserved 28% is lignin and hemicellulose, which are roughly a third of wood's mass and a comparable share of its energy; that energy is therefore absent, so this log carries less than a real one".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-066: the yeast recipe's fermentation role feeds on sucrose,
        // and this is the sugar solution the prompt hands it.
        MaterialRecipe {
            id: "household/sugar-water".to_string(),
            version: 1,
            canonical_key: "sugar_water".to_string(),
            name: "sugar water".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Zuckerwasser".to_string(), "Zuckerlösung".to_string()]),
                ("en".to_string(), vec!["sugar solution".to_string(), "sucrose solution".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.04)),
            components: vec![
                component("sucrose", 0.1),
                component("water", 0.9),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("a ten-percent sugar solution: nothing but sucrose and water, both resolved".to_string()),
            lot_assumptions: vec![
                "10% w/w is an explicit teaching strength - roughly two teaspoons in a glass - and not a measurement of anything".to_string(),
                "this is the one recipe in this tranche with no conserved remainder at all: sugar water really is sugar and water, and both are installed species".to_string(),
                "it is stated as made up rather than as dissolving: the dissolution enthalpy of the sucrose is not applied on adding this material, because what arrives is a solution and not a spoonful of crystals".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-025: a sealed pot of beans. The raised boiling point under
        // pressure is real engine behaviour; the softening is not.
        MaterialRecipe {
            id: "food/dried-beans".to_string(),
            version: 1,
            canonical_key: "beans".to_string(),
            name: "dried beans".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Bohnen".to_string(), "Trockenbohnen".to_string()]),
                ("en".to_string(), vec!["haricot beans".to_string(), "dry beans".to_string(), "pulses".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.8)),
            components: vec![
                component("starch", 0.5),
                component("water", 0.115),
                component("cellulose", 0.155),
                component("sucrose", 0.02),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.21,
                upper: 0.21,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("dried beans as they come from the packet: starch, residual moisture, fibre and a little sugar resolved, with the storage protein conserved unresolved".to_string()),
            lot_assumptions: vec![
                "the composition is an explicit teaching figure for a dried pulse - about half starch, a sixth fibre, a tenth water - and not a measurement of any variety".to_string(),
                "the 21% conserved remainder is the storage protein and the minerals. Beans are a protein food and this bench has no protein species, so the part a cook cares most about is the part that stays unresolved".to_string(),
                "SOFTENING IS NOT MODELLED. A bean softens because heat and water break down the pectin holding its cell walls together, and nothing here does that. What a sealed pot can show is the pressure and the raised boiling temperature, which is the reason a pressure cooker is faster - so the mechanism is half present and the outcome is absent".to_string(),
                "the oligosaccharides that make beans famous, and the lectins that make raw ones unsafe, are both in the conserved remainder and neither is claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-051. `food/meat` gains an enzyme_activity profile so the
        // protease has muscle protein to cut.
        MaterialRecipe {
            id: "food/meat".to_string(),
            version: 1,
            canonical_key: "meat".to_string(),
            name: "raw meat".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Fleisch".to_string(), "rohes Fleisch".to_string()]),
                ("en".to_string(), vec!["muscle meat".to_string(), "raw beef".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.05)),
            components: vec![
                component("water", 0.72),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.28,
                upper: 0.28,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("cut of muscle".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a raw cut of lean muscle: the water resolved, and the protein, fat and connective tissue conserved unresolved".to_string()),
            lot_assumptions: vec![
                "72% water and a 28% conserved remainder is the ordinary composition of lean muscle. Roughly three quarters of that remainder is protein, and the enzyme profile that goes with this recipe uses that share and nothing else".to_string(),
                "TENDERISING IS NOT WHAT IS COMPUTED. What the model reports is how much of a named protein fraction has been hydrolysed; texture, the collagen that actually makes a cut tough, and the difference between cutting myofibrils and cutting connective tissue are all outside it".to_string(),
                "no myoglobin, so no colour and no browning; no fat rendering; and nothing at all about whether the result is safe or pleasant to eat".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-085. Bile salts really are surfactants, so the bounded
        // emulsifier role is the right shape for once - the mechanism the
        // role stands for is the mechanism the question is about.
        MaterialRecipe {
            id: "laboratory/bile-salts".to_string(),
            version: 1,
            canonical_key: "bile_salts".to_string(),
            name: "bile salts".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Gallensalze".to_string(), "Gallensäuren".to_string()]),
                ("en".to_string(), vec!["bile acids".to_string(), "sodium taurocholate".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.2)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: vec![
                MaterialRole::AqueousEmulsifier {
                    saturation_amount: 1.0,
                    max_dispersed_fraction: 0.7,
                    half_life_seconds: 3600.0,
                },
            ],
            preparation: Some("a dry bile-salt powder: wholly unresolved, and carrying the one property digestion uses it for".to_string()),
            lot_assumptions: vec![
                "sodium taurocholate and its relatives have no installed species, so the powder is conserved whole and the emulsifying arrives as a declared dose response rather than as a molecule".to_string(),
                "this is the one recipe here whose role matches its real mechanism: bile salts emulsify fat because they are amphipathic and sit at the interface, which is what the emulsifier role stands for. It is still a bounded observable - no micelle size, no critical micelle concentration, and no bile-salt-lipase pairing".to_string(),
                "the digestion that emulsification exists to enable needs a lipase and a triglyceride, and only the first of those is on this shelf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-086 (a smoke row) and bio-091. Carrying chlorophyll as a
        // resolved component is what makes the alcohol extraction a real
        // dissolution rather than a shrug.
        MaterialRecipe {
            id: "plant/green-leaf".to_string(),
            version: 1,
            canonical_key: "leaf".to_string(),
            name: "green leaf".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Blatt".to_string(), "grünes Blatt".to_string()]),
                ("en".to_string(), vec!["plant leaf".to_string(), "spinach leaf".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.9)),
            components: vec![
                component("water", 0.75),
                component("cellulose", 0.15),
                component("glucose", 0.01),
                component("chlorophyll", 0.002),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.088,
                upper: 0.088,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a single broad leaf".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a fresh green leaf: water, cell-wall fibre, a little sugar and its chlorophyll resolved, with the protein and the rest of the pigments conserved".to_string()),
            lot_assumptions: vec![
                "0.2% chlorophyll by fresh mass is the right order for a green leaf and is an explicit teaching figure; a single species stands for chlorophyll a, chlorophyll b and the carotenoids together".to_string(),
                "PHOTOSYNTHESIS IS NOT MODELLED AT ALL. Nothing on this bench turns light, water and carbon dioxide into sugar. A leaf under a lamp here absorbs energy and does no chemistry, so the rows that ask what a plant makes get a vessel with a leaf in it and no reaction".to_string(),
                "the chlorophyll IS extractable, because the pigment has a reviewed solubility in ethanol and the leaf really does resolve some. Whether the filtrate is then PAINTED green is a separate question and the answer is no: this species carries no absorption spectrum, so the bench knows the pigment has moved and cannot colour the liquid with it".to_string(),
                "stomata, veins, turgor and the whole of leaf anatomy are absent; this is a composition, not a leaf".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-087, bio-088, bio-089: the oxygen-bubble practical. The
        // plant can be weighed into a beaker and lit; nothing more.
        MaterialRecipe {
            id: "plant/pondweed".to_string(),
            version: 1,
            canonical_key: "pondweed".to_string(),
            name: "pondweed shoot".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Wasserpest".to_string(), "Wasserpflanze".to_string()]),
                ("en".to_string(), vec!["waterweed".to_string(), "elodea shoot".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.95)),
            components: vec![
                component("water", 0.88),
                component("cellulose", 0.07),
                component("glucose", 0.008),
                component("chlorophyll", 0.0015),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0405,
                upper: 0.0405,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a submerged shoot".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a shoot of submerged pondweed: mostly water, with fibre, a little sugar and its chlorophyll resolved".to_string()),
            lot_assumptions: vec![
                "the composition is an explicit teaching figure for a soft aquatic shoot and is not a measurement of any species".to_string(),
                "THE BUBBLES ARE THE WHOLE POINT OF THIS PRACTICAL AND THEY DO NOT APPEAR. Counting oxygen bubbles from pondweed under a lamp is the classic photosynthesis measurement, and this bench has no photosynthesis, no light-limited rate and no action spectrum. The three rows this material serves - brighter light, green light, oxygen release - are all questions about a curve that does not exist here".to_string(),
                "irradiating the vessel deposits energy and does no chemistry, so a run will warm rather than photosynthesise".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-097: salt water and germination. Osmosis is not modelled;
        // the salt solution is.
        MaterialRecipe {
            id: "plant/dry-seed".to_string(),
            version: 1,
            canonical_key: "seed".to_string(),
            name: "dry seed".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Samen".to_string(), "Saatkorn".to_string()]),
                ("en".to_string(), vec!["plant seed".to_string(), "sowing seed".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.1)),
            components: vec![
                component("water", 0.1),
                component("starch", 0.55),
                component("cellulose", 0.1),
                component("sucrose", 0.02),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.23,
                upper: 0.23,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("a dry seed as it comes from the packet: stored starch, fibre, a little sugar and 10% residual moisture resolved".to_string()),
            lot_assumptions: vec![
                "about a tenth water and half starch is the ordinary state of an air-dry seed and is an explicit teaching figure".to_string(),
                "GERMINATION IS NOT MODELLED. Nothing here imbibes water, mobilises the starch or grows. A seed in salt water is a solid sitting in a brine whose osmotic potential this bench does not compute, so the row that asks whether salt stops germination gets a correct salt solution and no germination either way".to_string(),
                "the storage protein and the oil are the conserved remainder and neither is claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-096: a sealed jar and a seed respiring in the dark.
        // Respiration is not modelled, so the oxygen stays where it is.
        MaterialRecipe {
            id: "plant/germinating-seed".to_string(),
            version: 1,
            canonical_key: "germinating_seed".to_string(),
            name: "germinating seed".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["keimender Samen".to_string(), "Keimling".to_string()]),
                ("en".to_string(), vec!["sprouting seed".to_string(), "chitted seed".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.05)),
            components: vec![
                component("water", 0.45),
                component("starch", 0.32),
                component("cellulose", 0.08),
                component("glucose", 0.03),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.12,
                upper: 0.12,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a swollen seed with an emerging radicle".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a seed that has taken up water and begun to sprout: nearly half water now, with some of the starch already turned to sugar".to_string()),
            lot_assumptions: vec![
                "the shift from the dry seed's 10% water to 45%, and some starch already converted to glucose, is what imbibition and early mobilisation do; the numbers are explicit teaching figures rather than a measurement".to_string(),
                "RESPIRATION IS NOT MODELLED. A germinating seed in a sealed jar consumes oxygen and makes carbon dioxide, and nothing on this bench does that. The oxygen weighed into the vessel stays there, so the row that asks whether the seed uses it up gets a sealed jar and an unchanged headspace".to_string(),
                "no growth, no heat of respiration and no root".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-098: dye rising up a stalk. Capillary transport through a
        // plant is not something this bench has.
        MaterialRecipe {
            id: "plant/celery-stem".to_string(),
            version: 1,
            canonical_key: "celery_stem".to_string(),
            name: "celery stem".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Selleriestange".to_string(), "Staudensellerie".to_string()]),
                ("en".to_string(), vec!["celery stalk".to_string(), "celery petiole".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.95)),
            components: vec![
                component("water", 0.95),
                component("cellulose", 0.016),
                component("glucose", 0.004),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.03,
                upper: 0.03,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a ribbed stalk".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a stalk of celery: 95% water, with its fibre and a little sugar resolved".to_string()),
            lot_assumptions: vec![
                "celery is about 95% water and that is the only striking thing about its composition; the figure is an explicit teaching value".to_string(),
                "THE DYE DOES NOT RISE. Coloured water climbing a celery stalk is transpiration pulling on a column of water in the xylem, and this bench has no plant, no vessels and no transpiration. What a run holds is a stalk sitting in dyed water, with the dye exactly where it was put".to_string(),
                "the fibrous strings a child pulls off a stalk are the vascular bundles the experiment is about, and they are conserved matter here rather than a structure".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-099: turgor. There is no cell wall here to press against.
        MaterialRecipe {
            id: "plant/wilted-lettuce".to_string(),
            version: 1,
            canonical_key: "wilted_lettuce".to_string(),
            name: "wilted lettuce leaf".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["welker Salat".to_string(), "welkes Salatblatt".to_string()]),
                ("en".to_string(), vec!["limp lettuce".to_string(), "wilted salad leaf".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.9)),
            components: vec![
                component("water", 0.9),
                component("cellulose", 0.014),
                component("glucose", 0.006),
                component("chlorophyll", 0.0004),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.0796,
                upper: 0.0796,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a limp leaf".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("a lettuce leaf that has gone limp: still 90% water, and short of the few percent that would make it crisp".to_string()),
            lot_assumptions: vec![
                "a wilted leaf has lost only a small share of its water; the difference between crisp and limp is a few percent of mass and an enormous difference in stiffness, which is the point and is exactly what a composition cannot express".to_string(),
                "TURGOR IS NOT MODELLED. A leaf goes crisp because water enters each cell by osmosis until the protoplast presses on a rigid cell wall, and this bench has no cells, no walls and no osmotic potential. Soaking this material in fresh water changes nothing about it".to_string(),
                "the chlorophyll is carried so the leaf is green matter rather than anonymous solids, and for no other reason".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-100: plasmolysis. Same missing model as the lettuce, seen
        // from the other side.
        MaterialRecipe {
            id: "plant/onion-epidermis".to_string(),
            version: 1,
            canonical_key: "onion_cells".to_string(),
            name: "onion epidermis".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Zwiebelhaut".to_string(), "Zwiebelepidermis".to_string()]),
                ("en".to_string(), vec!["onion skin cells".to_string(), "onion epidermal strip".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.03)),
            components: vec![
                component("water", 0.89),
                component("glucose", 0.04),
                component("sucrose", 0.02),
                component("cellulose", 0.012),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.038,
                upper: 0.038,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a peeled epidermal strip".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: Vec::new(),
            preparation: Some("the transparent skin peeled from an onion scale: the layer a microscope slide is made from, as a composition".to_string()),
            lot_assumptions: vec![
                "this is the epidermal peel rather than the whole bulb, which is why it is a separate material from the onion already on the shelf: one is a thing you eat and the other is a thing you look through".to_string(),
                "PLASMOLYSIS IS NOT MODELLED. Strong salt solution pulls water out of a plant cell and the protoplast shrinks away from its wall - the observation this row is about - and there is no cell here to shrink. The salt solution outside it is computed correctly and does nothing to the strip".to_string(),
                "microscopy is out of scope generally; nothing on this bench is observed at cell scale".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // mat-025's other half. The thermoplastic is the object that DOES
        // soften, and without it the thermoset's "does not" is a sentence
        // with nothing to be measured against.
        MaterialRecipe {
            id: "polymer/thermoplastic-sheet".to_string(),
            version: 1,
            canonical_key: "thermoplastic".to_string(),
            name: "thermoplastic sheet".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Thermoplast".to_string(), "thermoplastische Folie".to_string(), "Polyethylenplatte".to_string()]),
                ("en".to_string(), vec!["thermoplastic".to_string(), "polyethylene sheet".to_string(), "moulded plastic".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.95)),
            components: vec![component("PE", 1.0)],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a moulded sheet or offcut".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![heat_response(1.82, Some(403.15), 673.15, "A moulded high-density polyethylene article. 130 C is the CRYSTALLINE MELT, and it is the threshold this bench uses because it is the one a hot-air gun or a pan of oil actually reaches and the one that lets the object be reshaped. Polyethylene's glass transition is far below room temperature and is not modelled at all; for the amorphous thermoplastics - polystyrene, PVC, acrylic - it is the glass transition rather than a melt that does this job, and this row does not speak for them. Crystallinity, molecular weight and any plasticiser move the melt by tens of kelvin. The 400 C figure is the onset of thermal decomposition and not a boiling point; no degradation product is named, and no viscosity, no rate of flow and no mould-filling behaviour is claimed.")],
            preparation: Some("a moulded high-density polyethylene article, resolved in full as the installed PE species".to_string()),
            lot_assumptions: vec![
                "polyethylene HAS a repeat unit, so unlike the cured thermoset beside it this object resolves entirely into the installed PE species and its solvent and reaction chemistry is that species'. The recipe adds the one thing a species record does not carry: what heat does to the ARTICLE, which is a question about chains sliding rather than about a molecule".to_string(),
                "high-density polyethylene is the stand-in for the whole thermoplastic family here. The amorphous thermoplastics soften at a glass transition rather than at a crystalline melt, and reading this row as a figure for polystyrene or PVC would be wrong by a hundred kelvin in either direction".to_string(),
                "softening changes the sentence and nothing else: there is no shape on this bench, so a softened object can be described as mouldable and cannot be moulded".to_string(),
                "recycling by melting is exactly what this threshold is, and the bench does not model the degradation that limits how many times it can be done".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // mat-025: the thermoset has no melting point, and that is a
        // claim rather than a gap - a cured network has none in reality
        // either, because it decomposes instead.
        MaterialRecipe {
            id: "polymer/thermoset-resin".to_string(),
            version: 1,
            canonical_key: "thermoset_resin".to_string(),
            name: "cured thermoset resin".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Duroplast".to_string(), "ausgehärtetes Harz".to_string()]),
                ("en".to_string(), vec!["cured resin".to_string(), "thermosetting plastic".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.2)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange {
                lower: 1.0,
                upper: 1.0,
            }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("a cured moulded block".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![
                MaterialRole::ConservedUnresolvedSolid {
                    srgb: [214, 198, 160],
                    colour_word: "off-white".to_string(),
                },
                heat_response(1.5, None, 573.15, "A cured epoxy or phenolic network. It has NO softening point, and that is a claim rather than a gap: the chains are joined to one another by covalent cross-links, so there are no separate chains left to slide and the object cannot melt at any temperature. It does have a glass transition - a cured epoxy's usually falls between 100 and 200 C - above which the network turns rubbery, and this bench deliberately does not model that, because a rubbery thermoset still holds its shape and still cannot be moulded, which is the distinction the row exists to make. The 300 C figure is the onset of thermal decomposition; the char, the smoke and the volatiles that come off it are real products this bench has no formulas for and does not claim, so the ledger is untouched and only the sentence changes."),
            ],
            preparation: Some("a block of cured thermosetting resin: cross-linked, wholly unresolved, and with no melting point to reach at any temperature".to_string()),
            lot_assumptions: vec![
                "an epoxy or phenolic network has no repeat unit that could be dispensed as a species, so the block is conserved whole. That is the honest reading of a material whose identity IS its cross-linking".to_string(),
                "the absence of a melting point here is correct rather than convenient: a cured thermoset does not melt, it decomposes, and the reviewed heat-response row says so as an absence - no softening temperature at all - rather than as a very large number".to_string(),
                "the glass transition a cured epoxy does have, usually between 100 and 200 C, is not modelled. Above it the network turns rubbery and still holds its shape and still cannot be moulded, so it changes nothing this row is about".to_string(),
                "the char, the smoke and the loss of strength that follow decomposition are real and are not modelled: the bench names the decomposition and leaves the ledger alone, having no formula for the products".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // th-101. Nucleation is the whole mechanism and there is none.
        MaterialRecipe {
            id: "laboratory/boiling-chips".to_string(),
            version: 1,
            canonical_key: "boiling_chips".to_string(),
            name: "boiling chips".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Siedesteinchen".to_string(), "Siedeperlen".to_string()]),
                ("en".to_string(), vec!["boiling stones".to_string(), "anti-bumping granules".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.6)),
            components: vec![
                component("SiO2", 0.92),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.08,
                upper: 0.08,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("porous silica boiling chips: mostly silica, with the pore structure that matters conserved as nothing at all".to_string()),
            lot_assumptions: vec![
                "the composition is the least interesting thing about a boiling chip. What it does is trapped air in its pores giving vapour somewhere to form, and POROSITY IS NOT A PROPERTY THIS BENCH HAS".to_string(),
                "BUMPING IS NOT MODELLED either, so there is nothing for the chips to prevent. Superheated water on this bench boils when it is disturbed, which is a separate mechanism the corpus already exercises without chips; adding chips changes no run".to_string(),
                "the chips are therefore present, correct in composition, and inert - and the row they close is closed against the parser rather than against the question".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // mat-072: the cooling is real; the cell reaction it is supposed
        // to slow is not here at all.
        MaterialRecipe {
            id: "household/alkaline-battery-electrolyte".to_string(),
            version: 1,
            canonical_key: "battery_electrolyte".to_string(),
            name: "alkaline battery electrolyte".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Batterieelektrolyt".to_string(), "Alkalielektrolyt".to_string()]),
                ("en".to_string(), vec!["potassium hydroxide electrolyte".to_string(), "cell electrolyte".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.29)),
            components: vec![
                component("KOH", 0.3),
                component("water", 0.7),
            ],
            unresolved_fraction: None,
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: Vec::new(),
            preparation: Some("the electrolyte inside an alkaline cell: 30% potassium hydroxide solution, resolved in full".to_string()),
            lot_assumptions: vec![
                "30% w/w potassium hydroxide is the ordinary strength of an alkaline cell's electrolyte and it is resolved completely, so the vessel really does hold a strong base and reports its pH".to_string(),
                "IT IS NOT A BATTERY. There is no zinc, no manganese dioxide, no separator and no circuit, so nothing discharges and there is no reaction for cooling to slow. What a run shows is a strong alkali getting colder, which is arithmetic on a heat capacity and not electrochemistry".to_string(),
                "this material is deliberately NOT called a battery: naming it for the liquid is what keeps the missing cell visible".to_string(),
                "it is corrosive, and the safety screen sees that through the hydroxide it resolves".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-006. Setting is protein coagulation and starch
        // gelatinisation; the bench has neither.
        MaterialRecipe {
            id: "food/cake-batter".to_string(),
            version: 1,
            canonical_key: "cake_batter".to_string(),
            name: "cake batter".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Kuchenteig".to_string(), "Rührteig".to_string()]),
                ("en".to_string(), vec!["sponge batter".to_string(), "raw cake mixture".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: vec![
                component("water", 0.28),
                component("starch", 0.24),
                component("sucrose", 0.2),
                component("cellulose", 0.01),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.27,
                upper: 0.27,
            }),
            physical_form: MaterialPhysicalForm::Suspension,
            roles: Vec::new(),
            preparation: Some("a plain sponge batter before baking: water, flour starch, sugar and a little fibre resolved, with the egg, fat and raising agent conserved".to_string()),
            lot_assumptions: vec![
                "the proportions are an explicit teaching figure for a simple sponge and not a recipe worth following".to_string(),
                "SETTING IS NOT MODELLED. A cake firms because its egg protein coagulates and its starch gelatinises, in that order, and this bench has neither transition. Heating this batter raises its temperature and boils off water; it never becomes a cake".to_string(),
                "the raising agent is in the conserved remainder, so no carbon dioxide is released and nothing rises. The baking-powder recipe already on the shelf does release gas, and pointing this batter at it would be a composition claim this recipe is not making".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        // bio-022. The water that does the bursting is real inventory;
        // the hull that contains it is not a thing this bench has.
        MaterialRecipe {
            id: "food/popcorn-kernel".to_string(),
            version: 1,
            canonical_key: "popcorn_kernel".to_string(),
            name: "popcorn kernel".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Maiskorn".to_string(), "Puffmaiskorn".to_string()]),
                ("en".to_string(), vec!["corn kernel".to_string(), "popping corn".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.85)),
            components: vec![
                component("water", 0.135),
                component("starch", 0.62),
                component("cellulose", 0.03),
            ],
            unresolved_fraction: Some(FractionRange {
                lower: 0.215,
                upper: 0.215,
            }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: Vec::new(),
            preparation: Some("a whole popping-corn kernel: its starchy endosperm, its hull fibre and the 13.5% moisture that makes it pop, resolved".to_string()),
            lot_assumptions: vec![
                "13.5% moisture is the narrow window a kernel has to be in to pop at all - drier and wetter kernels both fail - and getting that number into the vessel is the most this recipe can do for the question".to_string(),
                "THE BURSTING IS NOT MODELLED. A kernel pops because its hull holds the steam in until about 180 degC and 9 atmospheres, and then fails. This bench has no hull, no containment and no fracture, so the water simply boils off. The pressure that does the work never builds".to_string(),
                "the starch that expands into foam is resolved and the protein is conserved; neither undergoes anything".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "prepared/naked-egg-surrogate".to_string(),
            version: 1,
            canonical_key: "naked_egg".to_string(),
            name: "naked egg membrane surrogate".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Nacktes Ei".to_string()]),
                ("en".to_string(), vec!["peeled egg".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.04)),
            components: vec![component("water", 0.88), component("glucose", 0.004)],
            unresolved_fraction: Some(FractionRange { lower: 0.116, upper: 0.116 }),
            physical_form: MaterialPhysicalForm::CompositeObject { geometry: None },
            roles: vec![
                MaterialRole::CoherentObject,
                MaterialRole::OsmoticMembrane { internal_osmolarity_mol_per_litre: 0.30 },
            ],
            preparation: Some("egg-white composition retained behind a semipermeable membrane; shell, yolk and tissue mechanics are outside this bounded surrogate".to_string()),
            lot_assumptions: vec!["only water transfer from an osmolarity contrast is computed; ion selectivity, elasticity and final equilibrium mass are not".to_string()],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "prepared/cut-apple".to_string(),
            version: 1,
            canonical_key: "cut_apple".to_string(),
            name: "freshly cut apple".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Apfelscheibe".to_string()]),
                ("en".to_string(), vec!["apple slice".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(0.85)),
            components: vec![
                component("water", 0.851), component("fructose", 0.062),
                component("glucose", 0.024), component("sucrose", 0.021),
                component("cellulose", 0.024), component("citric_acid", 0.005),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.013, upper: 0.013 }),
            physical_form: MaterialPhysicalForm::CompositeObject { geometry: None },
            roles: vec![MaterialRole::CoherentObject, MaterialRole::BrowningSurface],
            preparation: Some("fresh cut surface exposed when the material is added".to_string()),
            lot_assumptions: vec!["visible browning is a bounded oxygen/time response with ascorbate inhibition; enzyme turnover, texture, flavour and food safety are not computed".to_string()],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "prepared/fatty-soap-equivalent".to_string(),
            version: 1,
            canonical_key: "fatty_soap".to_string(),
            name: "sodium fatty-soap teaching equivalent".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Fettsäureseife".to_string()]),
                ("en".to_string(), vec!["sodium soap equivalent".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::FattySoapEquivalent { moles_per_gram: 1.0 / 306.46 }],
            preparation: Some("declared sodium-stearate-equivalent reagent for bounded hard-water precipitation; not a commercial detergent formulation".to_string()),
            lot_assumptions: vec!["two fatty-carboxylate equivalents bind each calcium or magnesium ion; aggregate mass follows the consumed sodium-soap equivalent plus bound Ca/Mg minus sodium returned to solution".to_string()],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/pineapple".to_string(),
            version: 1,
            canonical_key: "pineapple".to_string(),
            name: "pineapple (fresh fruit)".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Ananas".to_string(), "frische Ananas".to_string()]),
                ("en".to_string(), vec!["fresh pineapple".to_string(), "pineapple flesh".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.01)),
            components: vec![
                component("water", 0.86),
                component("sucrose", 0.0599),
                component("fructose", 0.0212),
                component("glucose", 0.0173),
                component("citric_acid", 0.006),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.0356, upper: 0.0356 }),
            physical_form: MaterialPhysicalForm::CompositeObject {
                geometry: Some(MaterialGeometry {
                    shape: Some("fresh fruit flesh".to_string()),
                    surface_area_m2: None,
                    characteristic_length_m: None,
                }),
            },
            roles: vec![MaterialRole::EnzymeSource {
                enzyme: "bromelain".to_string(),
                catalyst_equivalent_per_gram: 0.05,
                denatures_above_k: 343.15,
            }],
            preparation: Some("fresh pineapple flesh: water, its three sugars and its fruit acid resolved; fibre, protein and the bromelain the fruit is famous for conserved, with the enzyme's ACTIVITY declared separately".to_string()),
            lot_assumptions: vec![
                "the sugars follow the raw-pineapple composition and are resolved separately for the same reason the apple's are: the bench dissolves each at its own solubility".to_string(),
                "citric acid is the fruit acid entered, at 0.6% of the flesh. Real pineapple also carries malic acid and buffering citrate salts, neither of which is installed, so the pH this recipe produces in a small volume is LOWER than a fresh pineapple measures. The acid mass is right and the buffering is missing".to_string(),
                "0.05 g of catalyst-equivalent activity per gram of fruit is an ACTIVITY EQUIVALENT in the bounded enzyme model's own dose units, not a bromelain content. Nothing here claims to know how much bromelain a pineapple holds; the number is an editorial classroom calibration, chosen so that a 20 g piece of fresh fruit visibly cuts a 10 g gelatine dose within an hour, which is the timescale the kitchen demonstration runs on".to_string(),
                "70 degC as the denaturation point is why cooked and canned pineapple set a jelly and fresh pineapple does not. Above it the carried enzyme is gone for good and cooling the beaker does not bring it back".to_string(),
                "the gelatine it acts on is conserved unresolved protein, so what the bench reports is hydrolysed protein MASS. It still has no gelation model: the jelly that fails to set is the consequence, and this bench cannot show it".to_string(),
                "bromelain is a mixture of cysteine proteases rather than one enzyme, and the recipe declares one activity for the mixture; no strain, cultivar, ripeness or stem-versus-flesh difference is claimed".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "school/sodium-polyacrylate-instant-snow".to_string(),
            version: 1,
            canonical_key: "instant_snow_powder".to_string(),
            name: "superabsorbent instant-snow powder".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Instant-Schnee-Pulver".to_string(), "Superabsorber".to_string()]),
                ("en".to_string(), vec!["superabsorbent_polymer".to_string(), "instant snow".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::Powder,
            roles: vec![MaterialRole::ConservedUnresolvedSolid {
                srgb: [245, 245, 250],
                colour_word: "white".to_string(),
            }],
            preparation: Some("dry cross-linked sodium-polyacrylate teaching powder".to_string()),
            lot_assumptions: vec![
                "the commercial formulation remains conserved and unresolved; cross-link density, additives and particle-size distribution are not guessed".to_string(),
                "the runtime reports equilibrium retained water with a conservative editorial capacity of 100 g/g; it does not predict swelling time, volume, texture, salinity or pH response".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "school/luminol-glow-solution".to_string(),
            version: 1,
            canonical_key: "luminol_glow_solution".to_string(),
            name: "prepared luminol glow solution".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Luminol-Leuchtlösung".to_string()]),
                ("en".to_string(), vec!["luminol_solution".to_string(), "luminol".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.0)),
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::HomogeneousLiquid,
            roles: vec![MaterialRole::ConservedUnresolvedLiquid],
            preparation: Some("a prepared alkaline luminol solution containing a suitable catalyst, activated separately with 3% hydrogen peroxide".to_string()),
            lot_assumptions: vec![
                "luminol, alkali and catalyst remain a conserved unresolved formulation; the model does not invent concentrations, products or photon yield".to_string(),
                "the Arrhenius-shaped intensity and half-life are a bounded comparison calibrated at 20 C, not a fit to a commercial product".to_string(),
                "commercial glow sticks normally use peroxyoxalate chemistry, not this luminol demonstration".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/yoghurt-culture".to_string(),
            version: 1,
            canonical_key: "yoghurt_culture".to_string(),
            name: "yoghurt starter culture".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Joghurtkulturen".to_string(), "Joghurt-Starterkultur".to_string()]),
                ("en".to_string(), vec!["yogurt culture".to_string(), "yoghurt starter".to_string(), "live yoghurt".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::Granules,
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.000_01,
                optimum_temperature_k: 316.15,
                temperature_width_k: 15.0,
                requires_hydration: false,
                metabolism: CultureMetabolism::Homolactic,
            }],
            preparation: Some("a homofermentative lactic starter, conserved entirely as unresolved culture; what it DOES is the declared metabolism and nothing else".to_string()),
            lot_assumptions: vec![
                "the metabolism is the homolactic route, one balanced equation: lactose or sucrose plus water gives four lactic acids and NO gas. That is why a yoghurt pot does not rise and a sourdough does, and it is the whole of what this recipe claims".to_string(),
                "43 degC as the optimum is the incubation temperature yoghurt cultures are described as working at; the Gaussian around it is what makes a refrigerator roughly two orders of magnitude slower, which is the food-preservation fact rather than a measured rate".to_string(),
                "0.00001 per second per gram of culture at the declared optimum is one editorial classroom timescale shared by the three cultures added with it; what differs between them is the temperature envelope and the dose, not a measured activity. Cell growth, a lag phase, inhibition by the acid the culture itself makes, nutrient depletion and strain variation are all absent".to_string(),
                "NO SPECIES AND NO STRAIN IS NAMED. Real yoghurt is two organisms in symbiosis and which of them dominates decides whether the acid is the L-(+) or the D-(-) form; this recipe names neither and the acid it makes carries no stereochemistry".to_string(),
                "the acid is made and the yoghurt is not: coagulation of casein into a gel is what turns milk into yoghurt, and this bench has no protein gelation. Texture, thickness, flavour and aroma are all outside it".to_string(),
                "NOTHING HERE IS A FOOD-SAFETY MODEL. There is no pathogen, no spoilage organism and no competing culture on this bench, so a finished run says an acid was made and never that the result is safe to eat".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/acetic-acid-bacteria".to_string(),
            version: 1,
            canonical_key: "acetobacter".to_string(),
            name: "acetic-acid bacteria culture".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Essigsaeurebakterien".to_string(), "Essigmutter".to_string()]),
                ("en".to_string(), vec!["acetic acid bacteria".to_string(), "vinegar mother".to_string(), "mother of vinegar".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: None,
            components: Vec::new(),
            unresolved_fraction: Some(FractionRange { lower: 1.0, upper: 1.0 }),
            physical_form: MaterialPhysicalForm::Other {
                description: "live cellulose-and-culture mat".to_string(),
            },
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.000_01,
                optimum_temperature_k: 303.15,
                temperature_width_k: 12.0,
                requires_hydration: false,
                metabolism: CultureMetabolism::Acetic,
            }],
            preparation: Some("an acetic-acid bacteria mat, conserved entirely as unresolved culture".to_string()),
            lot_assumptions: vec![
                "the metabolism is one balanced oxidation: ethanol plus oxygen gives acetic acid and water. It is an OXIDATION, which is why a vinegar jar is covered with cloth rather than a lid, and here it stops the moment the vessel's oxygen runs out".to_string(),
                "the oxygen is the oxygen IN THE VESSEL. This bench does not feed room air into a reaction, so an uncovered jar with no oxygen added ferments nothing here — which is a boundary of the wiring, not of the chemistry".to_string(),
                "30 degC as the optimum follows the range acetification is described as running fastest over".to_string(),
                "0.00001 per second per gram of culture at the declared optimum is one editorial classroom timescale shared by the three cultures added with it; what differs between them is the temperature envelope and the dose, not a measured activity. Cell growth, a lag phase, inhibition by the acid the culture itself makes, nutrient depletion and strain variation are all absent".to_string(),
                "the acetaldehyde the oxidation really goes through is not modelled: this is the aggregate equation, not the pathway, and no intermediate is claimed".to_string(),
                "vinegar is more than its acid — the aroma esters, the residual sugars and the colour of a wine or cider vinegar are all absent, and what this makes is acetic acid in water".to_string(),
                "NOTHING HERE IS A FOOD-SAFETY MODEL. There is no pathogen, no spoilage organism and no competing culture on this bench, so a finished run says an acid was made and never that the result is safe to eat".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
        MaterialRecipe {
            id: "food/sourdough-starter".to_string(),
            version: 1,
            canonical_key: "sourdough_starter".to_string(),
            name: "sourdough starter".to_string(),
            aliases: BTreeMap::from([
                ("de".to_string(), vec!["Sauerteig".to_string(), "Anstellgut".to_string()]),
                ("en".to_string(), vec!["sourdough".to_string(), "sourdough sponge".to_string(), "levain".to_string()]),
            ]),
            basis: MaterialBasis::MassFraction,
            bulk_density: Some(density(1.05)),
            components: vec![
                component("water", 0.5),
                component("sucrose", 0.02),
            ],
            unresolved_fraction: Some(FractionRange { lower: 0.48, upper: 0.48 }),
            physical_form: MaterialPhysicalForm::Suspension,
            roles: vec![MaterialRole::FermentationCulture {
                reference_rate_per_second_per_gram: 0.000_01,
                optimum_temperature_k: 300.15,
                temperature_width_k: 15.0,
                requires_hydration: false,
                metabolism: CultureMetabolism::Heterolactic,
            }],
            preparation: Some("a mature flour-and-water starter: half water, a little free sugar resolved, and the flour solids and the mixed culture conserved".to_string()),
            lot_assumptions: vec![
                "the metabolism is the heterolactic route, one balanced equation: sucrose plus water gives two lactic acids, two ethanols and two carbon dioxides. Acid AND gas out of the same sugar is exactly what makes a sourdough sour and risen, and it is what separates this culture from the yoghurt one".to_string(),
                "THE FREE SUGAR IS ENTERED AS SUCROSE AND A REAL STARTER'S IS MOSTLY MALTOSE. The substitution is recorded rather than hidden: the fermentation model consumes disaccharides, sucrose is the disaccharide it books gas against, and entering maltose would make carbon dioxide that no event reports. The mass is right and the identity is wrong".to_string(),
                "2% free sugar is an editorial figure for a mature starter, not a measured one, and it is the ONLY fermentable substrate here: this bench does not saccharify starch, so the flour beside the starter contributes none and the fermentation stops when the starter's own sugar is gone. A real starter keeps making maltose from flour amylases for as long as it is fed".to_string(),
                "0.00001 per second per gram of culture at the declared optimum is one editorial classroom timescale shared by the three cultures added with it; what differs between them is the temperature envelope and the dose, not a measured activity. Cell growth, a lag phase, inhibition by the acid the culture itself makes, nutrient depletion and strain variation are all absent".to_string(),
                "27 degC as the optimum is the room-temperature range a starter is kept at".to_string(),
                "a sourdough starter is a community of wild yeasts and lactic bacteria and no organism is named; what is claimed is one aggregate equation, not who runs which half of it".to_string(),
                "no gluten, no dough, no rise and no crumb: the gas is booked as carbon dioxide in the vessel, and whether a loaf holds it is a mechanical question this bench does not ask".to_string(),
                "NOTHING HERE IS A FOOD-SAFETY MODEL. There is no pathogen, no spoilage organism and no competing culture on this bench, so a finished run says an acid was made and never that the result is safe to eat".to_string(),
            ],
            substitutions: Vec::new(),
            confidence: MaterialConfidence::Surrogate,
            expansion_policy: MaterialExpansionPolicy::Fixed,
            evidence: evidence(),
        },
    ]);
}

fn export_species(document: &mut RegistryDocument, species: &SpeciesData) -> Result<(), String> {
    let source_id = match species.key {
        "isopropanol" => ISOPROPANOL_SOURCE.to_string(),
        "sucrose" => SUCROSE_SOURCE.to_string(),
        "Fe2O3" => IRON_III_OXIDE_SOURCE.to_string(),
        "epsomite" => EPSOMITE_SOURCE.to_string(),
        "chalcanthite" => CHALCANTHITE_SOURCE.to_string(),
        "SiO2" => SILICA_SOURCE.to_string(),
        _ => format!("legacy/{}", species.key),
    };
    let curated_isopropanol = species.key == "isopropanol";
    let curated_sucrose = species.key == "sucrose";
    let curated_iron_iii_oxide = species.key == "Fe2O3";
    let curated_epsomite = species.key == "epsomite";
    let curated_chalcanthite = species.key == "chalcanthite";
    let curated_silica = species.key == "SiO2";
    let runtime_source = curated_isopropanol
        || curated_sucrose
        || curated_iron_iii_oxide
        || curated_epsomite
        || curated_chalcanthite
        || curated_silica;
    document.sources.push(SourceRecord {
        id: source_id.clone(),
        citation: species.provenance.to_string(),
        licence: match species.key {
            "isopropanol" => "LicenseRef-US-Public-Domain",
            "sucrose" => "AGPL-3.0-or-later",
            "Fe2O3" => "LicenseRef-US-Public-Domain",
            "epsomite" => "LicenseRef-US-Public-Domain",
            "chalcanthite" => "LicenseRef-US-Public-Domain",
            "SiO2" => "LicenseRef-US-Public-Domain",
            _ => LEGACY_LICENCE,
        }
        .to_string(),
        lane: if runtime_source {
            SourceLane::Runtime
        } else {
            SourceLane::BuildOracle
        },
        origin: Some(match species.key {
            "isopropanol" => "https://pubchem.ncbi.nlm.nih.gov/compound/3776".to_string(),
            "sucrose" => "crates/kerotakis-registry-export/src/lib.rs".to_string(),
            "Fe2O3" => "vendor/nasa-cea/thermo.inp".to_string(),
            "epsomite" => "vendor/iphreeqc/database/wateq4f.dat".to_string(),
            "chalcanthite" => "vendor/iphreeqc/database/wateq4f.dat".to_string(),
            "SiO2" => "https://pubchem.ncbi.nlm.nih.gov/compound/24261".to_string(),
            _ => "crates/kerotakis-core/src/species.rs".to_string(),
        }),
        revision: match species.key {
            "isopropanol" => Some("CID 3776".to_string()),
            "sucrose" => Some("v1".to_string()),
            "Fe2O3" => Some("NASA CEA Fe2O3(cr), PubChem CID 14833".to_string()),
            "epsomite" => Some("USGS WATEQ4F Epsomite, PubChem CID 24843".to_string()),
            "chalcanthite" => Some("USGS WATEQ4F Chalcanthite".to_string()),
            "SiO2" => Some("PubChem CID 24261".to_string()),
            _ => None,
        },
        retrieved: runtime_source.then(|| {
            if curated_chalcanthite {
                "2026-08-29".to_string()
            } else {
                "2026-08-27".to_string()
            }
        }),
    });

    let mut identifiers = BTreeMap::new();
    if !species.inchikey.is_empty() {
        identifiers.insert("inchikey".to_string(), species.inchikey.to_string());
    }
    document.identities.push(IdentityRecord {
        id: species.key.to_string(),
        canonical_key: if species.inchikey.is_empty() {
            format!("legacy:species/{}", species.key)
        } else {
            species.inchikey.to_string()
        },
        name: species.name.to_string(),
        identifiers,
        synonyms: Vec::new(),
        evidence: evidence(&source_id),
    });

    let formula = parse_formula(species.formula)
        .map_err(|error| format!("{} formula '{}': {error}", species.key, species.formula))?;
    document.compositions.push(CompositionRecord {
        id: format!("composition/{}", species.key),
        species_id: species.key.to_string(),
        formula: species.formula.to_string(),
        elements: formula
            .counts
            .into_iter()
            .map(|(element, count)| ElementAmount {
                element,
                count: exact_number(count, "1", Dimension::Dimensionless, &source_id),
            })
            .collect(),
        net_charge: exact_number(formula.charge, "1", Dimension::Dimensionless, &source_id),
        evidence: evidence(&source_id),
    });

    let phase = phase(species.standard_phase);
    for (suffix, property, value, symbol, dimension) in [
        (
            "molar-mass",
            PhaseProperty::MolarMass,
            species.molar_mass,
            "g/mol",
            Dimension::MolarMass,
        ),
        (
            "heat-capacity",
            PhaseProperty::MolarHeatCapacity,
            species.heat_capacity,
            "J/(mol.K)",
            Dimension::MolarHeatCapacity,
        ),
        (
            "density",
            PhaseProperty::MassDensity,
            species.density,
            "g/mL",
            Dimension::MassDensity,
        ),
    ] {
        document
            .phase_thermodynamics
            .push(PhaseThermodynamicRecord {
                id: format!("{suffix}/{}", species.key),
                species_id: species.key.to_string(),
                phase,
                property,
                quantity: imported_number(value, symbol, dimension, phase, &source_id),
            });
    }
    // EXP-33: melting/boiling ride the typed properties; sublimation,
    // decomposition and dehydration ride `Other`, because the schema has no
    // variant for them and inventing one would claim a dimension check the
    // schema cannot make. Every record carries the same transition source.
    if let Some(t) = species.transitions {
        for (suffix, property, value) in [
            (
                "melting-point",
                PhaseProperty::MeltingTemperature,
                t.melting_k,
            ),
            (
                "boiling-point",
                PhaseProperty::BoilingTemperature,
                t.boiling_k,
            ),
            (
                "sublimation-point",
                PhaseProperty::Other("sublimation_temperature".to_string()),
                t.sublimation_k,
            ),
            (
                "decomposition-point",
                PhaseProperty::Other("decomposition_temperature".to_string()),
                t.decomposition_k,
            ),
            (
                "dehydration-point",
                PhaseProperty::Other("dehydration_temperature".to_string()),
                t.dehydration_k,
            ),
        ] {
            let Some(value) = value else { continue };
            document
                .phase_thermodynamics
                .push(PhaseThermodynamicRecord {
                    id: format!("{suffix}/{}", species.key),
                    species_id: species.key.to_string(),
                    phase,
                    property,
                    quantity: NumericRecord {
                        value,
                        unit: Unit {
                            symbol: "K".to_string(),
                            dimension: Dimension::Temperature,
                        },
                        conditions: Applicability {
                            phase: Some(phase),
                            pressure: Some(Interval {
                                lower: 101_325.0,
                                upper: 101_325.0,
                                unit: Unit {
                                    symbol: "Pa".to_string(),
                                    dimension: Dimension::Pressure,
                                },
                            }),
                            notes: t.boundary.map(str::to_string),
                            ..Applicability::default()
                        },
                        uncertainty: Uncertainty::NotReported,
                        source_id: PHASE_TRANSITION_SOURCE.to_string(),
                        method: Method::Curated(
                            "EXP-33 curated transition tranche; value corroborated against \
                             the handbook literature named in the source citation"
                                .to_string(),
                        ),
                    },
                });
        }
    }
    if let Some(value) = species.dissolution_enthalpy_kj {
        document
            .phase_thermodynamics
            .push(PhaseThermodynamicRecord {
                id: format!("dissolution-enthalpy/{}", species.key),
                species_id: species.key.to_string(),
                phase,
                property: PhaseProperty::EnthalpyOfDissolution,
                quantity: NumericRecord {
                    value,
                    unit: Unit {
                        symbol: "kJ/mol".to_string(),
                        dimension: Dimension::MolarEnergy,
                    },
                    conditions: Applicability {
                        phase: Some(phase),
                        notes: DISSOLUTION_NOTES
                            .iter()
                            .find(|(key, _)| *key == species.key)
                            .map(|(_, note)| (*note).to_string()),
                        ..Applicability::default()
                    },
                    uncertainty: Uncertainty::NotReported,
                    source_id: DISSOLUTION_SOURCE.to_string(),
                    method: Method::Curated(DISSOLUTION_METHOD.to_string()),
                },
            });
    }

    // The same escape the transition temperatures take, for the same
    // reason: the schema has no `ElectricalResistivity` property and
    // inventing one would claim a dimension check the schema cannot make.
    if let Some((_, value, note)) = RESISTIVITY.iter().find(|(key, _, _)| *key == species.key) {
        document
            .phase_thermodynamics
            .push(PhaseThermodynamicRecord {
                id: format!("electrical-resistivity/{}", species.key),
                species_id: species.key.to_string(),
                phase,
                property: PhaseProperty::Other("electrical_resistivity".to_string()),
                quantity: NumericRecord {
                    value: *value,
                    unit: Unit {
                        symbol: "Ohm.m".to_string(),
                        dimension: Dimension::Other("electrical_resistivity".to_string()),
                    },
                    conditions: Applicability {
                        temperature: Some(Interval {
                            lower: 293.15,
                            upper: 293.15,
                            unit: Unit {
                                symbol: "K".to_string(),
                                dimension: Dimension::Temperature,
                            },
                        }),
                        phase: Some(phase),
                        notes: Some((*note).to_string()),
                        ..Applicability::default()
                    },
                    uncertainty: Uncertainty::NotReported,
                    source_id: RESISTIVITY_SOURCE.to_string(),
                    method: Method::Curated(RESISTIVITY_METHOD.to_string()),
                },
            });
    }

    export_optical(document, species, &source_id, phase);
    export_model_parameters(document, species, &source_id);
    Ok(())
}

fn export_optical(
    document: &mut RegistryDocument,
    species: &SpeciesData,
    source_id: &str,
    phase: Phase,
) {
    if species.appearance.is_none()
        && species.flame_colour.is_none()
        && species.colour.is_none()
        && species.spectrum.is_none()
    {
        return;
    }
    let spectrum = species
        .spectrum
        .copied()
        .into_iter()
        .flat_map(|values| BAND_NM.into_iter().zip(values))
        .map(|(wavelength, molar_absorptivity)| SpectralSample {
            wavelength: imported_number(wavelength, "nm", Dimension::Wavelength, phase, source_id),
            molar_absorptivity: imported_number(
                molar_absorptivity,
                "L/(mol.cm)",
                Dimension::MolarAbsorptivity,
                phase,
                source_id,
            ),
        })
        .collect();
    document.optical.push(OpticalRecord {
        id: format!("optical/{}", species.key),
        species_id: species.key.to_string(),
        phase,
        appearance: species.appearance.map(str::to_string),
        flame_colour: species.flame_colour.map(str::to_string),
        reflective_srgb: species.colour.map(rgb_hex),
        spectrum,
        evidence: evidence(source_id),
    });
}

fn export_model_parameters(
    document: &mut RegistryDocument,
    species: &SpeciesData,
    source_id: &str,
) {
    document.model_parameters.push(ModelParameterRecord {
        id: format!("dissolves-without-speciation/{}", species.key),
        subject: ModelSubject::Species(species.key.to_string()),
        model: "legacy-runtime".to_string(),
        parameter: "dissolves-without-speciation".to_string(),
        quantity: exact_number(
            if species.dissolves_without_speciation {
                1.0
            } else {
                0.0
            },
            "1",
            Dimension::Dimensionless,
            source_id,
        ),
    });
    if let Some(solubility) = species.aqueous_solubility_g_per_100_ml {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("aqueous-solubility/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "bounded-room-temperature-dissolution".to_string(),
            parameter: "aqueous-solubility-g-per-100-ml".to_string(),
            quantity: imported_number(
                solubility,
                "g/100mL",
                Dimension::MassConcentration,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if let Some(hot) = species.aqueous_solubility_g_per_100_ml_at_100c {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("aqueous-solubility-100c/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "two-point-temperature-dependent-dissolution".to_string(),
            parameter: "aqueous-solubility-g-per-100-ml-at-100c".to_string(),
            quantity: imported_number(
                hot,
                "g/100mL",
                Dimension::MassConcentration,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if let Some(colour) = species.colour {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("legacy-tint-strength/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "legacy-tint".to_string(),
            parameter: "strength".to_string(),
            quantity: imported_number(
                colour.strength,
                "L/(mol.cm)",
                Dimension::MolarAbsorptivity,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if let Some(temperature) = species.forms_only_above_k {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("forms-only-above/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "legacy-metastability".to_string(),
            parameter: "forms-only-above".to_string(),
            quantity: imported_number(
                temperature,
                "K",
                Dimension::Temperature,
                phase(species.standard_phase),
                source_id,
            ),
        });
    }
    if species.magnetic {
        document.model_parameters.push(ModelParameterRecord {
            id: format!("magnetic/{}", species.key),
            subject: ModelSubject::Species(species.key.to_string()),
            model: "legacy-runtime".to_string(),
            parameter: "magnetic".to_string(),
            quantity: exact_number(1.0, "1", Dimension::Dimensionless, source_id),
        });
    }
}

fn imported_number(
    value: f64,
    symbol: &str,
    dimension: Dimension,
    phase: Phase,
    source_id: &str,
) -> NumericRecord {
    NumericRecord {
        value,
        unit: Unit {
            symbol: symbol.to_string(),
            dimension,
        },
        conditions: Applicability {
            phase: Some(phase),
            ..Applicability::default()
        },
        uncertainty: Uncertainty::NotReported,
        source_id: source_id.to_string(),
        method: Method::Imported(IMPORT_METHOD.to_string()),
    }
}

fn exact_number(value: f64, symbol: &str, dimension: Dimension, source_id: &str) -> NumericRecord {
    NumericRecord {
        value,
        unit: Unit {
            symbol: symbol.to_string(),
            dimension,
        },
        conditions: Applicability::default(),
        uncertainty: Uncertainty::Exact,
        source_id: source_id.to_string(),
        method: Method::Derived("parsed from the verbatim legacy declaration".to_string()),
    }
}

fn evidence(source_id: &str) -> Evidence {
    Evidence {
        source_id: source_id.to_string(),
        method: Method::Imported(IMPORT_METHOD.to_string()),
    }
}

fn phase(value: LegacyPhase) -> Phase {
    match value {
        LegacyPhase::Solid => Phase::Solid,
        LegacyPhase::Liquid => Phase::Liquid,
        LegacyPhase::Aqueous => Phase::Aqueous,
        LegacyPhase::Gas => Phase::Gas,
    }
}

fn rgb_hex(colour: Colour) -> String {
    format!("#{:02X}{:02X}{:02X}", colour.r, colour.g, colour.b)
}
