//! A supported Cu(II) displacement result must not claim that every dissolved
//! copper form has disappeared. Forecasts frozen before narration repairs.
use kerotakis_core::*;

struct Characterized;
impl Equilibrator for Characterized {
    fn name(&self) -> &'static str { "characterized-noop" }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        vessel.solution = Some(serde_json::from_value(serde_json::json!({"ph":7.0,"ionic_strength":0.01})).unwrap());
        Ok(Vec::new())
    }
}

fn initial(copper_one: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("Fe"), Moles(0.002), Phase::Solid);
    vessel.deposit(SpeciesId::new("Cu+2"), Moles(0.001), Phase::Aqueous);
    if copper_one > 0.0 {
        vessel.deposit(SpeciesId::new("Cu+1"), Moles(copper_one), Phase::Aqueous);
    }
    vessel
}

fn spent(events: &[Event]) -> &Event {
    events.iter().find(|event| matches!(event, Event::Inert { species, spent: Some(ion), .. } if species.0 == "Fe" && ion.0 == "Cu+2")).expect("represented Cu(II) pathway completion")
}

fn check_scoped_rendering(event: &Event) {
    for locale in i18n::Locale::available() {
        for register in [Register::LV1, Register::LV2, Register::LV3] {
            let text = render::render_event_in(event, register, locale);
            assert!(text.contains("Cu+2"), "supported ionic pathway must be named in every register/locale: {} {text}", locale.code());
            if locale.is_english() {
                assert!(text.contains("supported"), "bound the conclusion to the modeled pathway: {text}");
                assert!(!text.contains("all the copper") && !text.contains("all out of the water"), "do not claim other dissolved copper forms disappeared: {text}");
            }
        }
    }
}

#[test]
fn copper_one_remaining_after_copper_two_displacement_is_not_declared_gone() {
    let mut vessel = initial(3.54e-6);
    let events = displacement::over(&mut Characterized, &mut vessel).unwrap();
    assert_eq!(vessel.moles_of(&SpeciesId::new("Cu+1")).0, 3.54e-6);
    assert!(vessel.moles_of(&SpeciesId::new("Cu")).0 > 0.0009);
    assert!(vessel.moles_of(&SpeciesId::new("Fe")).0 > 0.0009);
    check_scoped_rendering(spent(&events));
}

#[test]
fn ordinary_copper_two_completion_still_reports_the_supported_pathway() {
    let mut vessel = initial(0.0);
    let events = displacement::over(&mut Characterized, &mut vessel).unwrap();
    assert!(vessel.moles_of(&SpeciesId::new("Cu")).0 > 0.0009);
    let event = spent(&events);
    let Event::Inert { computed, why, .. } = event else { unreachable!() };
    assert!(*computed);
    assert!(why.contains("plated out") && why.contains("nothing left to displace"));
    check_scoped_rendering(event);
}

#[test]
fn below_observation_copper_two_is_a_bound_not_an_exact_exhaustion_claim() {
    let mut vessel = initial(3.54e-6);
    vessel.withdraw_phase(&SpeciesId::new("Cu+2"), Moles(0.001), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Cu+2"), Moles(1e-8), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Cu"), Moles(0.001), Phase::Solid);
    Characterized.equilibrate(&mut vessel).unwrap();
    let before = serde_json::to_value(&vessel).unwrap();
    let events = displacement::bystanders(&vessel, &["Cu"]);
    check_scoped_rendering(spent(&events));
    assert_eq!(serde_json::to_value(vessel).unwrap(), before);
}
