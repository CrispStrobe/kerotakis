//! BRD-041: the gas-phase mechanism packs the bench ships, compiled once
//! and spoken in registry keys.
//!
//! The packs live as data in `data/mechanisms/` and are written in the
//! formulas their evaluations used. Here they become networks the slow
//! clock can run: parsed, renamed onto the registry's keys where the
//! registry has one (`CH4` → `methane`, `H2O` → `water`), compiled into
//! an arena that lives as long as the process, and paired with the
//! standard enthalpies of formation of their species so a burn can heat
//! the vessel it happens in.
//!
//! What a pack does NOT get here: a registry identity for its radicals.
//! `H`, `O`, `OH`, `HO2` and `CO` are not registry species; they enter
//! the ledger under their own names, at the nanomole populations a chain
//! carries them, and leave again as the chain closes. CO is the one that
//! matters — a rich flame's product — and it is a registry gap named in
//! BREADTH, not a species this module invents.

use std::sync::OnceLock;

use super::mechanism::{parse_yaml, MechanismArena};
use super::{KineticReaction, ReactionNetwork};
use crate::species::Phase;
use crate::vessel::Vessel;

/// A shipped pack, ready to run.
pub struct ShippedPack {
    /// The pack's own name, from its `description`.
    pub id: &'static str,
    pub network: ReactionNetwork<'static>,
    /// Every species the network names, after renaming, in document order.
    pub species: Vec<String>,
}

/// Pack text, in the order the README lists them.
const PACKS: &[(&str, &str)] = &[
    (
        "h2-o2-skeletal-v1",
        include_str!("../../../../data/mechanisms/h2-o2-skeletal-v1.yaml"),
    ),
    (
        "co-h2-wet-v1",
        include_str!("../../../../data/mechanisms/co-h2-wet-v1.yaml"),
    ),
    (
        "hydrocarbon-global-v1",
        include_str!("../../../../data/mechanisms/hydrocarbon-global-v1.yaml"),
    ),
];

/// Pack formula → registry key, for every pack species the registry knows
/// under another name. Anything not listed keeps its formula, which for
/// `H2`, `O2`, `N2`, `CO2` and `H2O2` IS the registry key.
pub const REGISTRY_NAMES: &[(&str, &str)] = &[
    ("CH4", "methane"),
    ("C3H8", "propane"),
    ("C4H10", "butane"),
    ("H2O", "water"),
];

/// Standard enthalpy of formation at 298.15 K, J/mol, by the name the
/// network uses after renaming — with the citation `vendor/nasa-cea/
/// thermo.inp` carries on the record it was read from. The vendored
/// NASA dataset is the one source on this bench that names its own
/// provenance per species; a burn's heat here is the difference of these
/// numbers over the stoichiometry, nothing more.
pub const FORMATION_ENTHALPY_J_PER_MOL: &[(&str, f64, &str)] = &[
    (
        "H2",
        0.0,
        "reference element; NASA thermo.inp: Gurvich,1978 pt1 p103 pt2 p31",
    ),
    (
        "O2",
        0.0,
        "reference element; NASA thermo.inp: Gurvich,1989 pt1 p94 pt2 p9",
    ),
    (
        "N2",
        0.0,
        "reference element; NASA thermo.inp: Gurvich,1978 pt1 p280 pt2 p207",
    ),
    ("Ar", 0.0, "reference element; NASA thermo.inp"),
    (
        "water",
        -241_826.0,
        "NASA thermo.inp H2O: Cox,1989 (CODATA). Woolley,1987. TRC(10/88) tuv25",
    ),
    (
        "H",
        217_998.828,
        "NASA thermo.inp: D0(H2) Herzberg,1970. Moore,1972. Gordon,1999",
    ),
    (
        "O",
        249_175.003,
        "NASA thermo.inp: D0(O2) Brix,1954. Moore,1976. Gordon,1999",
    ),
    (
        "OH",
        37_278.206,
        "NASA thermo.inp: D0(H-OH) Ruscic,2002. Gurvich,1978 pt1 p110 pt2 p37",
    ),
    (
        "HO2",
        12_020.0,
        "NASA thermo.inp: Hills,1984 & NASA data. Jacox,1998 p153",
    ),
    (
        "H2O2",
        -135_880.0,
        "NASA thermo.inp: Gurvich,1989 pt1 p127. Gurvich,1978 pt1 p121",
    ),
    (
        "CO",
        -110_535.196,
        "NASA thermo.inp: Gurvich,1979 pt1 p25 pt2 p29",
    ),
    (
        "CO2",
        -393_510.0,
        "NASA thermo.inp: Gurvich,1991 pt1 p27 pt2 p24",
    ),
    (
        "methane",
        -74_600.0,
        "NASA thermo.inp CH4: Gurvich,1991 pt1 p44 pt2 p36",
    ),
    (
        "propane",
        -104_680.0,
        "NASA thermo.inp C3H8: TRC(10/85) w1350. Chao,1973",
    ),
    (
        "butane",
        -125_790.0,
        "NASA thermo.inp C4H10,n-butane: TRC(10/85) w1350. Chen,1975",
    ),
];

/// ΔH°f for a network species, J/mol.
pub fn formation_enthalpy_j_per_mol(species: &str) -> Option<f64> {
    FORMATION_ENTHALPY_J_PER_MOL
        .iter()
        .find(|(name, _, _)| *name == species)
        .map(|(_, value, _)| *value)
}

impl ShippedPack {
    /// The standard reaction enthalpy of one step, J per mole of the
    /// equation as written: Σ ν·ΔH°f over products minus reactants. `None`
    /// where a species has no tabulated value — the heat is then not
    /// applied, and `every_shipped_species_has_a_formation_enthalpy` is
    /// the test that keeps that branch dead for the shipped packs.
    pub fn reaction_enthalpy_j_per_mol(&self, reaction: &KineticReaction<'_>) -> Option<f64> {
        let mut total = 0.0;
        for term in reaction.stoichiometry {
            total += term.coefficient * formation_enthalpy_j_per_mol(term.species)?;
        }
        Some(total)
    }

    /// How many of this pack's species stand in the vessel's gas phase.
    pub fn present_count(&self, vessel: &Vessel) -> usize {
        self.species
            .iter()
            .filter(|name| {
                vessel
                    .contents
                    .iter()
                    .any(|p| p.phase == Phase::Gas && p.species.0 == **name && p.moles.0 > 0.0)
            })
            .count()
    }

    /// Whether this pack has anything to say about the vessel: at least
    /// two of its species stand in the gas phase. The rate laws decide the
    /// rest — a cold mixture, or one with no radical to carry a chain,
    /// integrates to nothing in a few evaluations.
    pub fn matches(&self, vessel: &Vessel) -> bool {
        self.present_count(vessel) >= 2
    }
}

/// The one pack that speaks for a vessel this interval: the pack covering
/// the most of its gas species, the earlier-listed pack on a tie.
///
/// One, not every matching pack. The wet-CO pack contains the hydrogen
/// pack's chemistry, so running both on a hydrogen flame would count the
/// same steps twice — and, with the first pack's heat already applied,
/// the second would run at flame temperature and dissociate the water
/// the first had just made. That is a real effect at 3000 K, and it is
/// CEA's to settle over the equilibrated state, not a kinetic pack's to
/// half-do at the end of its interval.
pub fn pack_for(vessel: &Vessel) -> Option<&'static ShippedPack> {
    let mut best: Option<(&'static ShippedPack, usize)> = None;
    for pack in shipped() {
        let present = pack.present_count(vessel);
        if present < 2 {
            continue;
        }
        if best.is_none_or(|(_, count)| present > count) {
            best = Some((pack, present));
        }
    }
    best.map(|(pack, _)| pack)
}

/// The shipped packs, compiled on first use and kept for the life of the
/// process. The arenas are leaked on purpose: a network borrows its
/// strings and slices from the arena that compiled it, and these
/// networks are consulted by every `wait` on the bench.
pub fn shipped() -> &'static [ShippedPack] {
    static PACKS_COMPILED: OnceLock<Vec<ShippedPack>> = OnceLock::new();
    PACKS_COMPILED.get_or_init(|| {
        PACKS
            .iter()
            .map(|(id, text)| {
                let mut mechanism = parse_yaml(text).unwrap_or_else(|error| {
                    panic!("shipped mechanism pack {id} does not parse: {error}")
                });
                for (from, to) in REGISTRY_NAMES {
                    mechanism.rename_species(from, to);
                }
                let species = mechanism.species_names().map(str::to_string).collect();
                let arena: &'static MechanismArena = Box::leak(Box::new(MechanismArena::default()));
                let network = mechanism.compile_in(arena);
                ShippedPack {
                    id,
                    network,
                    species,
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_packs_compile_and_speak_registry_keys() {
        let packs = shipped();
        assert_eq!(packs.len(), 3);
        let hydrocarbons = packs
            .iter()
            .find(|p| p.id == "hydrocarbon-global-v1")
            .expect("the hydrocarbon pack ships");
        for key in ["methane", "propane", "butane", "water", "O2", "CO2"] {
            assert!(
                hydrocarbons.species.iter().any(|s| s == key),
                "{key} missing"
            );
        }
        assert!(!hydrocarbons
            .species
            .iter()
            .any(|s| s == "CH4" || s == "H2O"));
        // The renamed species reach the compiled stoichiometry too.
        assert!(hydrocarbons
            .network
            .reactions
            .iter()
            .any(|r| r.stoichiometry.iter().any(|t| t.species == "methane")));
        // Every species is a registry key or one of the named radicals.
        let radicals = ["H", "O", "OH", "HO2", "CO", "Ar"];
        for pack in packs {
            for name in &pack.species {
                assert!(
                    crate::species::lookup_key(name).is_some() || radicals.contains(&name.as_str()),
                    "{}: {name} is neither a registry key nor a named radical",
                    pack.id
                );
            }
        }
    }

    #[test]
    fn one_pack_speaks_for_a_vessel_and_the_larger_cover_wins() {
        use crate::units::{Kelvin, Liters, Moles};
        use crate::vessel::{Headspace, Vessel, VesselId};
        use crate::SpeciesId;
        let mut hydrogen = Vessel::new(VesselId(0), "h2");
        hydrogen.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        hydrogen.temperature = Kelvin(1200.0);
        for (k, n) in [("H2", 2e-3), ("O2", 1e-3), ("N2", 4e-3)] {
            hydrogen.deposit(SpeciesId::new(k), Moles(n), Phase::Gas);
        }
        // Both hydrogen packs cover the same three species: the earlier
        // listed, smaller one speaks.
        assert_eq!(pack_for(&hydrogen).map(|p| p.id), Some("h2-o2-skeletal-v1"));
        hydrogen.deposit(SpeciesId::new("CO"), Moles(1e-3), Phase::Gas);
        assert_eq!(pack_for(&hydrogen).map(|p| p.id), Some("co-h2-wet-v1"));
        let mut methane = Vessel::new(VesselId(1), "ch4");
        for (k, n) in [("methane", 1e-2), ("O2", 3e-2), ("N2", 0.1)] {
            methane.deposit(SpeciesId::new(k), Moles(n), Phase::Gas);
        }
        assert_eq!(
            pack_for(&methane).map(|p| p.id),
            Some("hydrocarbon-global-v1")
        );
        let mut lone = Vessel::new(VesselId(2), "o2");
        lone.deposit(SpeciesId::new("O2"), Moles(1e-2), Phase::Gas);
        assert!(pack_for(&lone).is_none());
    }

    #[test]
    fn every_shipped_species_has_a_formation_enthalpy() {
        for pack in shipped() {
            for name in &pack.species {
                assert!(
                    formation_enthalpy_j_per_mol(name).is_some(),
                    "{}: no ΔH°f for {name}",
                    pack.id
                );
            }
            for reaction in pack.network.reactions {
                assert!(
                    pack.reaction_enthalpy_j_per_mol(reaction).is_some(),
                    "{}",
                    reaction.id
                );
            }
        }
    }

    #[test]
    fn methane_combustion_releases_its_textbook_heat() {
        let pack = shipped()
            .iter()
            .find(|p| p.id == "hydrocarbon-global-v1")
            .unwrap();
        let methane = pack
            .network
            .reactions
            .iter()
            .find(|r| r.equation.starts_with("CH4 "))
            .expect("the methane step");
        // CH4 + 2 O2 → CO2 + 2 H2O(g): −802.3 kJ/mol (lower heating value).
        let dh = pack.reaction_enthalpy_j_per_mol(methane).unwrap();
        assert!((dh + 802_562.0).abs() < 500.0, "ΔH = {dh} J/mol");
    }
}
