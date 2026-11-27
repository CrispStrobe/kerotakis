//! Activity-model routes: a reviewed (metal, ligand) family whose complexes
//! at high ionic strength exist, with a consistent activity model, only in
//! a database the default route does not use.
//!
//! The default inorganic route (wateq4f, WATEQ Debye-Hückel) is reliable to
//! about I = 1 mol/kgw, and for cobalt chloride it defines only the first
//! chloro complex. sit.dat carries the full CoCl+ .. CoCl4-2 set together
//! with its SIT interaction coefficient eps(Co+2, Cl-). Solving a
//! concentrated cobalt chloride on wateq4f cannot produce the tetrachloro
//! complex at all, however strong the chloride.
//!
//! This module is a TABLE. A new family is one [`ActivityRoute`] row; the
//! selector and its tests are generic over the rows. No constant lives
//! here: every complex a row names must already be defined by the routed
//! database itself, which a test asserts, so a row can never stand in for
//! missing data.

use crate::derived;
use crate::ligand_routing::base;

/// One reviewed activity-model route.
#[derive(Debug, Clone, Copy)]
pub struct ActivityRoute {
    /// Base element of the metal, e.g. `Co`.
    pub metal: &'static str,
    /// Base element of the ligand, e.g. `Cl`.
    pub ligand: &'static str,
    /// The route applies strictly above this estimated ionic strength,
    /// mol/kgw. Below it the default dataset's activity model is in its
    /// own domain and stays the answer.
    pub min_ionic_strength: f64,
    /// Database tag the route selects (a [`crate::native_namespace`] tag).
    pub db: &'static str,
    /// Complexes the routed database defines for this family and the
    /// default route lacks — the reason the row exists. Asserted present.
    pub complexes: &'static [&'static str],
    /// Valence-state element names the other routes book that the routed
    /// database writes as a plain element, `(booked, routed)`. sit.dat
    /// defines `Co` with master species `Co+2` and no `Co(2)`, so the
    /// booked `Co(2)` total is posed as `Co`. Valid only because the routed
    /// master carries that valence — asserted by a test.
    pub element_aliases: &'static [(&'static str, &'static str)],
    /// Where the routed rows are, in the pinned vendor database.
    pub source: &'static str,
}

/// The reviewed rows. Order is preference order.
pub const ACTIVITY_MODEL_ROUTES: &[ActivityRoute] = &[ActivityRoute {
    metal: "Co",
    ligand: "Cl",
    min_ionic_strength: 0.5,
    db: "sit",
    complexes: &["CoCl+", "CoCl2", "CoCl3-", "CoCl4-2"],
    element_aliases: &[("Co(2)", "Co")],
    source: "vendor/iphreeqc/database/sit.dat: SOLUTION_SPECIES Co+2 + n Cl- (81TUR/WHI, 06BLA/IGN) and SIT eps(Co+2, Cl-) = 0.160",
}];

/// Charge of a master species written PHREEQC-style: `Co+2` → 2,
/// `Cl-` → -1, `H2O` → 0.
fn species_charge(species: &str) -> i32 {
    let Some(pos) = species.rfind(['+', '-']) else {
        return 0;
    };
    let sign = if species[pos..].starts_with('+') {
        1
    } else {
        -1
    };
    let digits = &species[pos + 1..];
    if digits.is_empty() {
        sign
    } else {
        digits.parse::<i32>().map_or(0, |n| sign * n)
    }
}

/// Ionic strength estimated from analytical totals, each element counted
/// as the routed database's own master species: I = ½ Σ m z².
///
/// An estimate for ROUTING, not a result: complexation lowers the true
/// ionic strength, and H/O (and the proton of an acid) are not counted, so
/// it does not over-state a dilute solution into the route.
pub fn estimated_ionic_strength(db: &str, totals: &[(String, f64)], kgw: f64) -> f64 {
    if !(kgw.is_finite() && kgw > 0.0) {
        return 0.0;
    }
    let index = derived::index_for(db);
    let sum: f64 = totals
        .iter()
        .filter(|(element, _)| !matches!(base(element), "H" | "O"))
        .map(|(element, moles)| {
            let z = index
                .masters
                .get(element.as_str())
                .or_else(|| index.masters.get(base(element)))
                .map_or(0, |master| species_charge(&master.species));
            moles.max(0.0) * f64::from(z * z)
        })
        .sum();
    0.5 * sum / kgw
}

/// An element name as `route`'s database writes it.
pub fn routed_element<'a>(route: &ActivityRoute, element: &'a str) -> &'a str {
    route
        .element_aliases
        .iter()
        .find(|(booked, _)| *booked == element)
        .map_or(element, |(_, routed)| routed)
}

/// Rename aliased element totals in place for posing on `route`'s
/// database. Amounts are untouched; two totals that alias to one name are
/// summed so no element is booked twice.
pub fn apply_aliases(
    route: &ActivityRoute,
    elements: &mut Vec<String>,
    totals: &mut Vec<(String, f64)>,
) {
    for element in elements.iter_mut() {
        *element = routed_element(route, element).to_string();
    }
    elements.sort();
    elements.dedup();
    let mut merged: Vec<(String, f64)> = Vec::with_capacity(totals.len());
    for (element, moles) in totals.drain(..) {
        let name = routed_element(route, &element).to_string();
        match merged.iter_mut().find(|(e, _)| *e == name) {
            Some(entry) => entry.1 += moles,
            None => merged.push((name, moles)),
        }
    }
    *totals = merged;
}

/// The first row whose metal and ligand are both present, whose database
/// defines every element present, and whose ionic-strength trigger the
/// solution clears.
pub fn select(
    elements: &[String],
    totals: &[(String, f64)],
    kgw: f64,
) -> Option<&'static ActivityRoute> {
    ACTIVITY_MODEL_ROUTES.iter().find(|route| {
        let present = |symbol: &str| elements.iter().any(|e| base(e) == symbol);
        if !(present(route.metal) && present(route.ligand)) {
            return false;
        }
        let index = derived::index_for(route.db);
        if !elements
            .iter()
            .all(|e| index.has_element(routed_element(route, e)))
        {
            return false;
        }
        let routed_totals: Vec<(String, f64)> = totals
            .iter()
            .map(|(e, n)| (routed_element(route, e).to_string(), *n))
            .collect();
        estimated_ionic_strength(route.db, &routed_totals, kgw) > route.min_ionic_strength
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbindex::ActivityModel;

    /// Whether a database's own bytes define `species` by a reaction
    /// (`... = species`), read from the text rather than any index.
    fn defines(db: &[u8], species: &str) -> bool {
        String::from_utf8_lossy(db).lines().any(|line| {
            let line = line.split('#').next().unwrap_or("");
            line.split_once('=')
                .and_then(|(_, rhs)| rhs.split_whitespace().next())
                .is_some_and(|name| name == species)
        })
    }

    fn database_bytes(tag: &str) -> &'static [u8] {
        match tag {
            "sit" => crate::databases::sit(),
            other => panic!("route row names a database without a test arm: {other}"),
        }
    }

    fn elements(list: &[&str]) -> Vec<String> {
        list.iter().map(|e| e.to_string()).collect()
    }

    #[test]
    fn every_row_names_complexes_its_database_defines() {
        for route in ACTIVITY_MODEL_ROUTES {
            let bytes = database_bytes(route.db);
            for complex in route.complexes {
                assert!(
                    defines(bytes, complex),
                    "{} does not define {complex}; a route row must never stand in for data",
                    route.db
                );
            }
        }
    }

    #[test]
    fn sit_index_declares_the_sit_activity_model() {
        assert_eq!(derived::index_for("sit").activity_model, ActivityModel::Sit);
        // The default route is unchanged.
        assert_ne!(
            derived::index_for("wateq4f").activity_model,
            ActivityModel::Sit
        );
    }

    #[test]
    fn default_route_lacks_the_higher_cobalt_chloro_complexes() {
        // The reason the Co-Cl row exists: wateq4f cannot form CoCl4-2.
        assert!(!defines(crate::databases::wateq4f(), "CoCl4-2"));
        assert!(defines(crate::databases::sit(), "CoCl4-2"));
    }

    #[test]
    fn every_alias_lands_on_a_master_of_the_booked_valence() {
        for route in ACTIVITY_MODEL_ROUTES {
            let index = derived::index_for(route.db);
            for (booked, routed) in route.element_aliases {
                let valence: i32 = booked
                    .split_once('(')
                    .and_then(|(_, v)| v.trim_end_matches(')').parse().ok())
                    .expect("an alias maps a valence state");
                let master = index
                    .masters
                    .get(*routed)
                    .unwrap_or_else(|| panic!("{} has no master for {routed}", route.db));
                assert_eq!(species_charge(&master.species), valence);
                assert!(!index.has_element(booked));
            }
        }
    }

    #[test]
    fn booked_cobalt_two_routes_and_is_renamed_for_sit() {
        // The engine books dissolved cobalt as Co(2) (derived.rs).
        let mut elements = elements(&["Cl", "Co(2)"]);
        let mut totals = vec![("Co(2)".to_string(), 1.0), ("Cl".to_string(), 2.0)];
        let route = select(&elements, &totals, 1.0).expect("concentrated Co(2)-Cl routes");
        apply_aliases(route, &mut elements, &mut totals);
        assert_eq!(elements, vec!["Cl".to_string(), "Co".to_string()]);
        assert!(totals.contains(&("Co".to_string(), 1.0)));
        assert!(totals.contains(&("Cl".to_string(), 2.0)));
    }

    #[test]
    fn charges_parse_from_master_species_names() {
        assert_eq!(species_charge("Co+2"), 2);
        assert_eq!(species_charge("Cl-"), -1);
        assert_eq!(species_charge("Na+"), 1);
        assert_eq!(species_charge("CO3-2"), -2);
        assert_eq!(species_charge("H2O"), 0);
    }

    #[test]
    fn concentrated_cobalt_chloride_routes_to_sit() {
        // 1 mol/kgw CoCl2: I = ½(1·4 + 2·1) = 3 mol/kgw.
        let totals = vec![("Co".to_string(), 1.0), ("Cl".to_string(), 2.0)];
        let route = select(&elements(&["Co", "Cl"]), &totals, 1.0)
            .expect("concentrated Co-Cl must take the SIT route");
        assert_eq!(route.db, "sit");
        assert_eq!(route.metal, "Co");
        assert!(estimated_ionic_strength("sit", &totals, 1.0) > 2.9);
    }

    #[test]
    fn cobalt_in_strong_hydrochloric_acid_routes_to_sit() {
        // Dilute cobalt, chloride from the acid: the chloride alone clears
        // the trigger (protons are not counted, so this is conservative).
        let totals = vec![("Co".to_string(), 0.01), ("Cl".to_string(), 6.02)];
        let route = select(&elements(&["Co", "Cl"]), &totals, 1.0);
        assert_eq!(route.map(|r| r.db), Some("sit"));
    }

    #[test]
    fn dilute_cobalt_chloride_stays_on_the_default_route() {
        // 1 mmol/kgw CoCl2: I = 0.003 mol/kgw, far inside Debye-Hückel.
        let totals = vec![("Co".to_string(), 0.001), ("Cl".to_string(), 0.002)];
        assert!(select(&elements(&["Co", "Cl"]), &totals, 1.0).is_none());
        let booked = vec![("Co(2)".to_string(), 0.001), ("Cl".to_string(), 0.002)];
        assert!(select(&elements(&["Co(2)", "Cl"]), &booked, 1.0).is_none());
    }

    #[test]
    fn just_below_the_trigger_stays_on_the_default_route() {
        // 0.1 mol/kgw CoCl2: I = 0.3 mol/kgw < 0.5.
        let totals = vec![("Co".to_string(), 0.1), ("Cl".to_string(), 0.2)];
        assert!(select(&elements(&["Co", "Cl"]), &totals, 1.0).is_none());
    }

    #[test]
    fn concentrated_chloride_without_cobalt_is_not_this_route() {
        let totals = vec![("Na".to_string(), 3.0), ("Cl".to_string(), 3.0)];
        assert!(select(&elements(&["Na", "Cl"]), &totals, 1.0).is_none());
    }

    #[test]
    fn concentrated_cobalt_without_chloride_is_not_this_route() {
        let totals = vec![("Co".to_string(), 1.0), ("N(5)".to_string(), 2.0)];
        assert!(select(&elements(&["Co", "N(5)"]), &totals, 1.0).is_none());
    }
}
