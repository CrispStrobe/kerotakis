//! WORLD-003 — the runtime catalog contract.
//!
//! One answer to "what can this learner reach, and why", joined from the
//! things that decide it: the verbs the engine actually parses, the species
//! the registry actually holds, the packs actually installed, and the
//! learner's own Story progress.
//!
//! Three properties are the whole point.
//!
//! **The rules live here, once.** Availability used to be computed in the
//! browser from a table the browser owned, which meant the desktop shell and
//! the CLI either duplicated it or did without. A rule with two copies is a
//! rule that will eventually disagree with itself. The browser's copy is
//! gone as of the client migration, and so is the fixture that pinned the
//! two to each other while both existed.
//!
//! **Sandbox is derived, never stored.** A Sandbox save does not serialize
//! thousands of `unlocked = true` flags that go stale the moment a pack
//! changes; Sandbox availability is computed as full from the installed
//! inventory, every time.
//!
//! **No prose crosses the wire.** Every reason is a stable tag plus its
//! parameters, so an English and a German client render the same state from
//! the same response. The catalog says `locked` and `minimum_completed: 3`;
//! what that sentence looks like is the client's business.

use serde::{Deserialize, Serialize};

/// Which save namespace is asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogMode {
    Story,
    Sandbox,
}

/// What kind of thing this row is. Clients group the cabinet by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogKind {
    /// A material drawn from the shelf.
    Reagent,
    /// A verb the bench performs with apparatus.
    Apparatus,
    /// A measurement, addressed as `measure:<token>`.
    Instrument,
}

/// Why a row is reachable, or why it is not.
///
/// A tagged id with parameters rather than a sentence: `locked` carries the
/// count that would unlock it, and the client writes the sentence in its own
/// language. Adding a variant is additive; renaming one is a protocol break.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum CatalogReason {
    /// Sandbox reaches everything installed, by derivation.
    Sandbox,
    /// Story progress has passed this row's milestone.
    Earned { minimum_completed: u32 },
    /// A closed case granted it permanently.
    Awarded,
    /// The active mission supplies it for the duration of the mission.
    Loaned,
    /// Available only in Sandbox or while an authored mission supplies it.
    MissionOnly,
    /// Not yet: this many completed missions would earn it.
    Locked { minimum_completed: u32 },
}

/// One catalog row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogItem {
    /// Stable id: a verb (`filter`), an instrument (`measure:ph`), or a
    /// registry species key (`NaCl`). Never a localized name.
    pub id: String,
    pub kind: CatalogKind,
    /// Completed missions that would earn this row on progress alone.
    pub minimum_completed: u32,
    pub available: bool,
    pub reason: CatalogReason,
}

/// What the client knows and the engine does not: who is asking, and how far
/// they have got.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogRequest {
    pub mode: Option<CatalogMode>,
    /// How many missions this learner has completed.
    #[serde(default)]
    pub completed: u32,
    /// Ids permanently granted by closed cases.
    #[serde(default)]
    pub awarded: Vec<String>,
    /// Ids the active mission supplies for its own duration.
    #[serde(default)]
    pub mission_kit: Vec<String>,
}

/// The joined answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub mode: CatalogMode,
    pub completed: u32,
    pub items: Vec<CatalogItem>,
    /// Installed packs, echoed so one response answers "what is installed"
    /// and "what can I reach" together rather than in two round trips.
    pub packs: Vec<String>,
}

/// What the caller knows about one shelf material. Passed in rather than
/// looked up, because hazard classification lives in `kerotakis-safety`,
/// which depends on this crate — the join belongs to the host that has both.
#[derive(Debug, Clone)]
pub struct ReagentFacts<'a> {
    pub key: &'a str,
    /// GHS-style labels as the safety matrix reports them.
    pub hazards: &'a [&'a str],
    /// False where the species has no safety-matrix row at all.
    pub assessed: bool,
}

/// Materials a learner starts with. Water and table salt are not a reward.
const STARTER_STOCK: &[&str] = &[
    "water", "NaCl", "CH3COOH", "NaHCO3", "CaCO3", "MgSO4", "CaCl2",
];

/// Materials which must never become permanent Story stock. Progress is a
/// learning gate, not a licence to handle a cryogen without supervision.
const MISSION_ONLY_STOCK: &[&str] = &["liquid_nitrogen"];

/// Apparatus verbs and the progress that earns them. A verb absent here is
/// deliberately last (4): a new verb should be unreachable until someone
/// decides where it belongs, rather than silently free on day one.
const APPARATUS_MILESTONES: &[(&str, u32)] = &[
    ("burette", 0),
    ("stir", 0),
    ("heat", 0),
    ("centrifuge", 0),
    ("dilute", 0),
    ("grind", 0),
    ("filter", 0),
    ("decant", 0),
    ("mix", 0),
    ("bunsen", 1),
    ("evaporate", 1),
    ("drain", 1),
    ("magnet", 1),
    ("react", 1),
    ("regulate", 2),
    ("irradiate", 2),
    ("electrolyse", 3),
    ("cell", 3),
    ("distil", 4),
    ("transport", 4),
    ("sweep", 4),
];

/// Instruments, addressed the way a client names them.
const INSTRUMENT_MILESTONES: &[(&str, u32)] = &[
    ("measure:eyes", 0),
    ("measure:thermometer", 0),
    ("measure:ph", 0),
    ("measure:balance", 0),
    ("measure:smell", 1),
    ("measure:volume", 1),
    ("measure:conductivity", 1),
    ("measure:pressure", 2),
    ("measure:calorimeter", 2),
    ("measure:uvvis", 3),
    ("measure:chromatograph", 3),
    ("measure:geiger", 4),
    ("measure:melting_point", 4),
    ("measure:boiling_point", 4),
];

/// Verbs that are real operations but not cabinet apparatus, so they carry no
/// milestone. Three groups: vessel handling a learner always has (`new`,
/// `remove`, `add`, `stock`), observation that costs nothing to own
/// (`measure`, `smell`, `test`, `chromatograph`, `particles`), and bench
/// controls that are not equipment (`open`, `seal`, `wait`, `cool`,
/// `ignite`, `titrate`).
///
/// `measure` and `chromatograph` are here because the catalog tiers the
/// INSTRUMENT (`measure:ph`, `measure:chromatograph`), not the verb that
/// reads it; `titrate` because the burette is the tiered thing. Only verbs
/// the parser actually knows may be listed — a stale entry would hide a verb
/// that has since become apparatus, so the test checks this list both ways.
///
/// `particles` is the one entry here that is not obviously either. It draws
/// the vessel's own census — Johnstone's submicroscopic level — and no
/// cabinet in any school contains an instrument that shows you ions. It is a
/// way of looking at a result the bench has already computed rather than a
/// way of obtaining one, so tiering it would gate the representation instead
/// of the measurement. The instruments that produce the numbers underneath
/// it are tiered where they belong.
pub const NOT_CABINET: &[&str] = &[
    "new",
    "remove",
    "add",
    "stock",
    "open",
    "seal",
    "wait",
    "cool",
    "ignite",
    "measure",
    "smell",
    "test",
    "titrate",
    "chromatograph",
    "particles",
];

/// The last tier. Anything unclassified sits here rather than at zero.
pub const LAST_TIER: u32 = 4;

/// Progress that earns one apparatus verb or instrument id.
pub fn equipment_requirement(id: &str) -> u32 {
    APPARATUS_MILESTONES
        .iter()
        .chain(INSTRUMENT_MILESTONES.iter())
        .find(|(name, _)| *name == id)
        .map(|(_, tier)| *tier)
        .unwrap_or(LAST_TIER)
}

/// Progress that earns one material.
///
/// Hazard is the ladder: what a learner can be trusted with grows with what
/// they have done. An UNASSESSED species sits at the top — not because it is
/// known to be dangerous, but because it is not known to be safe, and the
/// catalog must not promote silence to a clearance.
pub fn reagent_requirement(facts: &ReagentFacts) -> u32 {
    if STARTER_STOCK.contains(&facts.key) {
        return 0;
    }
    if !facts.assessed {
        return LAST_TIER;
    }
    let has = |label: &str| facts.hazards.contains(&label);
    if has("cryogen") || has("asphyxiant") {
        return LAST_TIER;
    }
    if has("toxic") || has("corrosive") {
        return 3;
    }
    if has("flammable") || has("oxidizer") {
        return 2;
    }
    1
}

/// Decide one row: is it reachable, and which fact decided it?
///
/// Order matters and is itself the contract. Sandbox first, because it
/// derives everything. Then progress, then a permanent award, then a mission
/// loan — the most durable reason a thing is reachable is the one worth
/// reporting, so a learner is told "you earned this" rather than "your
/// mission lent it to you" for something they own outright.
pub fn decide(
    mode: CatalogMode,
    completed: u32,
    minimum_completed: u32,
    awarded: bool,
    loaned: bool,
) -> (bool, CatalogReason) {
    if mode == CatalogMode::Sandbox {
        return (true, CatalogReason::Sandbox);
    }
    if completed >= minimum_completed {
        return (true, CatalogReason::Earned { minimum_completed });
    }
    if awarded {
        return (true, CatalogReason::Awarded);
    }
    if loaned {
        return (true, CatalogReason::Loaned);
    }
    (false, CatalogReason::Locked { minimum_completed })
}

/// Join everything into one response.
pub fn catalog(
    request: &CatalogRequest,
    reagents: &[ReagentFacts<'_>],
    packs: &[String],
) -> CatalogResponse {
    let mode = request.mode.unwrap_or(CatalogMode::Story);
    let awarded: Vec<&str> = request.awarded.iter().map(String::as_str).collect();
    let kit: Vec<&str> = request.mission_kit.iter().map(String::as_str).collect();

    let mut items = Vec::new();
    {
        let mut push = |id: String, kind: CatalogKind, minimum: u32| {
            let (available, reason) = decide(
                mode,
                request.completed,
                minimum,
                awarded.contains(&id.as_str()),
                kit.contains(&id.as_str()),
            );
            items.push(CatalogItem {
                id,
                kind,
                minimum_completed: minimum,
                available,
                reason,
            });
        };

        for (verb, tier) in APPARATUS_MILESTONES {
            push((*verb).to_string(), CatalogKind::Apparatus, *tier);
        }
        for (id, tier) in INSTRUMENT_MILESTONES {
            push((*id).to_string(), CatalogKind::Instrument, *tier);
        }
    }
    for facts in reagents {
        let minimum = reagent_requirement(facts);
        if MISSION_ONLY_STOCK.contains(&facts.key) && mode != CatalogMode::Sandbox {
            let loaned = kit.contains(&facts.key);
            items.push(CatalogItem {
                id: facts.key.to_string(),
                kind: CatalogKind::Reagent,
                minimum_completed: minimum,
                available: loaned,
                reason: if loaned {
                    CatalogReason::Loaned
                } else {
                    CatalogReason::MissionOnly
                },
            });
        } else {
            let id = facts.key.to_string();
            let (available, reason) = decide(
                mode,
                request.completed,
                minimum,
                awarded.contains(&id.as_str()),
                kit.contains(&id.as_str()),
            );
            items.push(CatalogItem {
                id,
                kind: CatalogKind::Reagent,
                minimum_completed: minimum,
                available,
                reason,
            });
        }
    }

    CatalogResponse {
        mode,
        completed: request.completed,
        items,
        packs: packs.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts<'a>(key: &'a str, hazards: &'a [&'a str], assessed: bool) -> ReagentFacts<'a> {
        ReagentFacts {
            key,
            hazards,
            assessed,
        }
    }

    #[test]
    fn sandbox_derives_everything_as_full() {
        let none: Vec<&str> = vec![];
        let reagents = vec![facts("HCl", &none, true)];
        let response = catalog(
            &CatalogRequest {
                mode: Some(CatalogMode::Sandbox),
                completed: 0,
                ..Default::default()
            },
            &reagents,
            &[],
        );
        assert!(response.items.iter().all(|item| item.available));
        assert!(response
            .items
            .iter()
            .all(|item| item.reason == CatalogReason::Sandbox));
        // The milestone is still REPORTED in Sandbox: a client showing "this
        // would take three missions in Story" needs the number even where it
        // does not gate.
        let distil = response.items.iter().find(|i| i.id == "distil").unwrap();
        assert_eq!(distil.minimum_completed, 4);
    }

    #[test]
    fn story_locks_what_progress_has_not_reached_and_says_what_would() {
        let response = catalog(
            &CatalogRequest {
                mode: Some(CatalogMode::Story),
                completed: 1,
                ..Default::default()
            },
            &[],
            &[],
        );
        let distil = response.items.iter().find(|i| i.id == "distil").unwrap();
        assert!(!distil.available);
        assert_eq!(
            distil.reason,
            CatalogReason::Locked {
                minimum_completed: 4
            }
        );
        let filter = response.items.iter().find(|i| i.id == "filter").unwrap();
        assert!(filter.available);
        assert_eq!(
            filter.reason,
            CatalogReason::Earned {
                minimum_completed: 0
            }
        );
    }

    #[test]
    fn an_award_outranks_a_loan_and_both_beat_a_lock() {
        // Earned first: something already reached reports as earned even when
        // a mission also lends it.
        assert_eq!(
            decide(CatalogMode::Story, 3, 3, true, true).1,
            CatalogReason::Earned {
                minimum_completed: 3
            }
        );
        // Not reached, but granted permanently by a closed case.
        assert_eq!(
            decide(CatalogMode::Story, 0, 3, true, false).1,
            CatalogReason::Awarded
        );
        // Not reached and not granted, but this mission supplies it.
        assert_eq!(
            decide(CatalogMode::Story, 0, 3, false, true).1,
            CatalogReason::Loaned
        );
        assert_eq!(
            decide(CatalogMode::Story, 0, 3, false, false),
            (
                false,
                CatalogReason::Locked {
                    minimum_completed: 3
                }
            )
        );
    }

    #[test]
    fn hazard_sets_the_reagent_ladder_and_silence_is_not_a_clearance() {
        let none: Vec<&str> = vec![];
        let toxic = vec!["toxic"];
        let flammable = vec!["flammable"];
        assert_eq!(reagent_requirement(&facts("water", &none, true)), 0);
        assert_eq!(reagent_requirement(&facts("NaCl", &toxic, true)), 0);
        assert_eq!(reagent_requirement(&facts("AgNO3", &none, true)), 1);
        assert_eq!(reagent_requirement(&facts("ethanol", &flammable, true)), 2);
        assert_eq!(reagent_requirement(&facts("HCl", &toxic, true)), 3);
        assert_eq!(
            reagent_requirement(&facts("liquid_nitrogen", &["cryogen", "asphyxiant"], true)),
            LAST_TIER
        );
        // Unassessed is not safe. It is unknown, and unknown sits last.
        assert_eq!(reagent_requirement(&facts("mystery", &none, false)), 4);
    }

    #[test]
    fn liquid_nitrogen_is_loaned_by_a_mission_but_never_earned_or_awarded() {
        let hazards = vec!["cryogen", "asphyxiant"];
        let reagents = vec![facts("liquid_nitrogen", &hazards, true)];
        for awarded in [Vec::new(), vec!["liquid_nitrogen".to_string()]] {
            let response = catalog(
                &CatalogRequest {
                    mode: Some(CatalogMode::Story),
                    completed: u32::MAX,
                    awarded,
                    ..Default::default()
                },
                &reagents,
                &[],
            );
            let nitrogen = response
                .items
                .iter()
                .find(|item| item.id == "liquid_nitrogen")
                .unwrap();
            assert!(!nitrogen.available);
            assert_eq!(nitrogen.reason, CatalogReason::MissionOnly);
        }

        let response = catalog(
            &CatalogRequest {
                mode: Some(CatalogMode::Story),
                completed: 1,
                mission_kit: vec!["liquid_nitrogen".to_string()],
                ..Default::default()
            },
            &reagents,
            &[],
        );
        let nitrogen = response
            .items
            .iter()
            .find(|item| item.id == "liquid_nitrogen")
            .unwrap();
        assert!(nitrogen.available);
        assert_eq!(nitrogen.reason, CatalogReason::Loaned);
    }

    #[test]
    fn an_unknown_verb_is_last_rather_than_free() {
        assert_eq!(equipment_requirement("nothing-like-this"), LAST_TIER);
        assert_eq!(equipment_requirement("measure:ph"), 0);
        assert_eq!(equipment_requirement("electrolyse"), 3);
    }

    #[test]
    fn every_parsed_verb_is_either_tiered_or_deliberately_not_cabinet() {
        // Drift in either direction is a real defect: a verb the bench can
        // perform but the catalog never tiers is unreachable through the UI
        // for no stated reason, and a stale exclusion hides a verb that has
        // since become apparatus. So the split is declared, and both halves
        // are checked against the parser.
        let declared: Vec<&str> = APPARATUS_MILESTONES.iter().map(|(v, _)| *v).collect();
        let known: Vec<&str> = crate::script::VERBS.iter().map(|(v, _)| *v).collect();

        let untiered: Vec<&str> = known
            .iter()
            .copied()
            .filter(|verb| !NOT_CABINET.contains(verb) && !declared.contains(verb))
            .collect();
        assert!(
            untiered.is_empty(),
            "verbs the parser knows but nothing tiered: {untiered:?} — \
             give each a milestone, or list it in NOT_CABINET with a reason"
        );

        let stale: Vec<&str> = NOT_CABINET
            .iter()
            .copied()
            .filter(|verb| !known.contains(verb))
            .collect();
        assert!(
            stale.is_empty(),
            "NOT_CABINET names verbs the parser dropped: {stale:?}"
        );
    }

    #[test]
    fn the_response_carries_no_prose() {
        let response = catalog(
            &CatalogRequest {
                mode: Some(CatalogMode::Story),
                completed: 0,
                ..Default::default()
            },
            &[],
            &["core-aqueous".to_string()],
        );
        let json = serde_json::to_string(&response).unwrap();
        // Reasons are tags with parameters. If a sentence ever appears here,
        // German clients render English, and this is where it is caught.
        // WORLD-007: no string value anywhere in the response is a
        // sentence. Ids in this protocol never contain a space; prose
        // always does, which makes the rule exact and cheap.
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        fn no_prose(value: &serde_json::Value, context: &str) {
            match value {
                serde_json::Value::String(s) => assert!(
                    !s.contains(' '),
                    "{context}: response field carries prose, not an id: {s:?}"
                ),
                serde_json::Value::Array(items) => {
                    items.iter().for_each(|item| no_prose(item, context))
                }
                serde_json::Value::Object(fields) => fields
                    .iter()
                    .for_each(|(key, item)| no_prose(item, &format!("{context}.{key}"))),
                _ => {}
            }
        }
        no_prose(&value, "catalog");
        assert!(json.contains(r#""reason":"locked""#));
        assert!(json.contains(r#""minimum_completed""#));
        assert!(!json.to_lowercase().contains("complete "));
        assert!(json.contains("core-aqueous"));
    }
}
