//! Wire, renderer and production propagation contracts; no chemistry result is inferred.
use kerotakis_core::{
    aqueous_continuation,
    i18n::Locale,
    render::{render_event_in, Register},
    solve::{Equilibrator, SolveError, SolverStack},
    Event, Vessel, VesselId,
};

fn policy_error() -> SolveError {
    SolverStack::with_required_conservation(Vec::new(), f64::NAN)
        .err()
        .expect("a nonfinite conservation policy must be refused")
}

fn event_from(error: &SolveError) -> Event {
    Event::SolverFailed {
        vessel: VesselId(0),
        solver: "diagnostic-contract-fixture".into(),
        detail: error.to_string(),
        reason: error.owned_reason().cloned().map(Box::new),
    }
}

#[test]
fn old_solver_failure_json_round_trips_without_a_new_field() {
    let old = serde_json::json!({
        "event": "solver_failed", "vessel": 0,
        "solver": "native-fixture", "detail": "unknown raw label owner_ä"
    });
    let event: Event = serde_json::from_value(old.clone()).unwrap();
    assert!(matches!(&event, Event::SolverFailed { reason: None, .. }));
    assert_eq!(serde_json::to_value(event).unwrap(), old);
}

#[test]
fn owned_reason_round_trips_and_does_not_change_error_display() {
    let error = policy_error();
    let raw = "solver-stack could not solve this state: required conservation tolerance must be finite and between zero and one";
    assert_eq!(error.to_string(), raw);
    let value = serde_json::to_value(event_from(&error)).unwrap();
    assert_eq!(value["detail"], raw);
    assert_eq!(value["reason"]["key"], "solver-reason.stack.tolerance");
    let decoded: Event = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
}

struct RefusingPolicy;
impl Equilibrator for RefusingPolicy {
    fn name(&self) -> &'static str {
        "owned-reason-policy-probe"
    }
    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Err(policy_error())
    }
}

#[test]
fn actual_policy_refusal_crosses_the_production_solver_stack() {
    let mut vessel = Vessel::new(VesselId(0), "unknown-label_ä");
    let before = format!("{vessel:?}");
    let mut stack = SolverStack::new(vec![Box::new(RefusingPolicy)]);
    let events = stack.equilibrate(&mut vessel).unwrap();
    assert_eq!(format!("{vessel:?}"), before);
    let failures: Vec<_> = events
        .iter()
        .filter(|event| matches!(event, Event::SolverFailed { .. }))
        .collect();
    assert_eq!(failures.len(), 1);
    let event = failures[0];
    let Event::SolverFailed { reason, detail, .. } = event else {
        unreachable!()
    };
    assert_eq!(
        reason.as_ref().unwrap().key,
        "solver-reason.stack.tolerance"
    );
    assert_eq!(*detail, policy_error().to_string());
    for tag in ["en", "de", "fr"] {
        let locale = Locale::parse(tag);
        let phrase = reason.as_ref().unwrap();
        if tag != "en" {
            assert!(locale.lookup(&phrase.key).is_some());
            assert_ne!(phrase.render(locale), phrase.render(Locale::EN));
        }
        for register in [Register::LV2, Register::LV3] {
            let line = render_event_in(event, register, locale);
            assert!(line.contains(&phrase.render(locale)));
            assert!(line.contains(detail));
            println!("OWNED-POLICY-REFUSAL [{tag}] {line}");
        }
        let without = Event::SolverFailed {
            vessel: VesselId(0),
            solver: "owned-reason-policy-probe".into(),
            detail: detail.clone(),
            reason: None,
        };
        assert_eq!(
            render_event_in(event, Register::LV1, locale),
            render_event_in(&without, Register::LV1, locale)
        );
        assert!(!render_event_in(event, Register::LV1, locale).contains(detail));
    }
}

#[test]
fn actual_missing_adapter_capability_has_localized_owned_reason() {
    let mut adapter = RefusingPolicy;
    let mut vessel = Vessel::new(VesselId(0), "capability-fixture");
    let error = aqueous_continuation::characterize_via(&mut adapter, &mut vessel)
        .expect_err("an adapter without a trusted readback must be refused");
    assert_eq!(
        error.owned_reason().unwrap().key,
        "solver-reason.continuation.adapter-unavailable"
    );
    assert_eq!(
        error.to_string(),
        "aqueous-continuation could not solve this state: equilibrator has no trusted aqueous readback adapter"
    );
    for tag in ["de", "fr"] {
        let locale = Locale::parse(tag);
        let phrase = error.owned_reason().unwrap();
        assert!(locale.lookup(&phrase.key).is_some());
        assert_ne!(phrase.render(locale), phrase.render(Locale::EN));
        let line = render_event_in(&event_from(&error), Register::LV3, locale);
        assert!(line.contains(&phrase.render(locale)));
        assert!(line.contains(&error.to_string()));
        println!("OWNED-ADAPTER-CAPABILITY [{tag}] {line}");
    }
}

struct RawUpstream;
impl Equilibrator for RawUpstream {
    fn name(&self) -> &'static str {
        "opaque-upstream-protocol-fixture"
    }
    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Err(SolveError::NotConverged {
            solver: self.name().into(),
            detail: "vendor fixture: unknown external label Label_ä".into(),
            reason: None,
        })
    }
}

#[test]
fn raw_upstream_failure_is_not_given_a_manufactured_owned_reason() {
    let mut vessel = Vessel::new(VesselId(0), "opaque-upstream-fixture");
    let mut stack = SolverStack::new(vec![Box::new(RawUpstream)]);
    let events = stack.equilibrate(&mut vessel).unwrap();
    let event = events
        .iter()
        .find(|event| matches!(event, Event::SolverFailed { .. }))
        .expect("the actual upstream error must remain visible");
    let Event::SolverFailed { reason, detail, .. } = event else {
        unreachable!()
    };
    assert!(reason.is_none());
    assert_eq!(
        detail,
        "opaque-upstream-protocol-fixture could not solve this state: vendor fixture: unknown external label Label_ä"
    );
    assert!(serde_json::to_value(event).unwrap().get("reason").is_none());
    for tag in ["de", "fr"] {
        let line = render_event_in(event, Register::LV3, Locale::parse(tag));
        assert!(line.contains(detail));
        assert!(line.contains("Label_ä"));
        println!("OPAQUE-UPSTREAM-NONE-PROTOCOL-FIXTURE [{tag}] {line}");
    }
}

#[derive(serde::Deserialize)]
struct OwnedReasonCatalogContract {
    key: String,
    en: String,
    placeholders: Vec<String>,
}

fn placeholder_names(text: &str) -> std::collections::BTreeSet<String> {
    let mut result = std::collections::BTreeSet::new();
    let mut remaining = text;
    while let Some(open) = remaining.find('{') {
        let tail = &remaining[open + 1..];
        let close = tail.find('}').expect("a catalog placeholder must close");
        let name = &tail[..close];
        assert!(!name.is_empty());
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        result.insert(name.to_string());
        remaining = &tail[close + 1..];
    }
    result
}

#[test]
fn all_new_owned_reason_catalog_keys_resolve_with_equal_placeholders() {
    use kerotakis_core::phrase::{Phrase, Slot};
    let contracts: Vec<OwnedReasonCatalogContract> =
        serde_json::from_str(include_str!("fixtures/owned_solver_reason_catalog.json")).unwrap();
    // A floor, not an exact count: lanes add owned reasons over time, and each
    // row below is checked individually (unique key, equal placeholders,
    // every locale). The floor still catches a truncated fixture.
    assert!(contracts.len() >= 199, "catalog shrank to {}", contracts.len());
    let mut keys = std::collections::BTreeSet::new();
    for row in contracts {
        assert!(keys.insert(row.key.clone()), "duplicate key {}", row.key);
        let expected: std::collections::BTreeSet<_> = row.placeholders.into_iter().collect();
        assert_eq!(placeholder_names(&row.en), expected);
        let slots = expected
            .iter()
            .map(|name| (name.clone(), Slot::text(format!("value-for-{name}"))))
            .collect();
        let phrase = Phrase::new(&row.key, &row.en, slots);
        for tag in ["de", "fr"] {
            let locale = Locale::parse(tag);
            let translated = locale.lookup(&row.key).expect(&row.key);
            assert!(!translated.trim().is_empty());
            assert_ne!(translated, row.en);
            assert_eq!(placeholder_names(translated), expected, "{} {tag}", row.key);
            let rendered = phrase.render(locale);
            for name in &expected {
                assert!(rendered.contains(&format!("value-for-{name}")));
                assert!(!rendered.contains(&format!("{{{name}}}")));
            }
        }
    }
}
