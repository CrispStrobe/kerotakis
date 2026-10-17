//! A parsed operation must consume its complete command, apart from comments.
use kerotakis_core::{
    ops::{Compare, Endpoint},
    script::parse_op,
    Operator,
};

#[test]
fn titration_endpoints_keep_valid_default_and_explicit_step_limits() {
    for (endpoint, expected) in [
        ("ph 7", Endpoint::Ph),
        (
            "pe > 8",
            Endpoint::Pe {
                compare: Compare::Above,
                value: 8.0,
            },
        ),
        ("colour persists", Endpoint::ColourPersists),
    ] {
        for (suffix, steps) in [("", 100), (" max 4", 4)] {
            let input = format!("titrate v1 NaOH 1M 1mL until {endpoint}{suffix}");
            match parse_op(&input).unwrap().unwrap() {
                Operator::Titrate {
                    endpoint,
                    max_steps,
                    ..
                } => {
                    assert_eq!(endpoint, expected);
                    assert_eq!(max_steps, steps);
                }
                other => panic!("unexpected operation: {other:?}"),
            }
        }
    }
}

#[test]
fn titration_rejects_unconsumed_or_incomplete_step_limit_tails() {
    for endpoint in ["ph 7", "pe > 8", "colour persists"] {
        for suffix in [" max 4 ignored", " max 4 max 9", " max", " ignored"] {
            let input = format!("titrate v1 NaOH 1M 1mL until {endpoint}{suffix}");
            assert!(parse_op(&input).is_err(), "silently accepted: {input}");
        }
    }
}

#[test]
fn mix_rejects_trailing_tokens_and_preserves_comment_syntax() {
    let valid = "mix v1 0.5 v2 0.25 into v3";
    assert!(matches!(
        parse_op(valid).unwrap(),
        Some(Operator::Mix { .. })
    ));
    assert!(matches!(
        parse_op(&format!("{valid} # observation")).unwrap(),
        Some(Operator::Mix { .. })
    ));
    for suffix in [" ignored", " into v1", " wait 1s"] {
        assert!(
            parse_op(&format!("{valid}{suffix}")).is_err(),
            "silently accepted suffix: {suffix}"
        );
    }
}

#[test]
fn every_documented_operation_rejects_an_unconsumed_tail() {
    for (verb, example) in kerotakis_core::script::VERBS {
        assert!(
            parse_op(example).unwrap().is_some(),
            "invalid example for {verb}"
        );
        let malformed = format!("{example} __unexpected_tail__");
        assert!(
            parse_op(&malformed).is_err(),
            "silently accepted: {malformed}"
        );
    }
}

#[test]
fn optional_forms_are_consumed_without_losing_valid_syntax() {
    for valid in [
        "heat v1 10kJ on candle",
        "heat v1 10kJ candle",
        "distil v1 v2 0.5 stages 4",
        "centrifuge v1 3000rpm 60s 8cm 100g",
        "transport v1 v2 from v3 to v4 steps 3 courant 0.5",
    ] {
        assert!(
            parse_op(valid).unwrap().is_some(),
            "valid optional form refused: {valid}"
        );
        assert!(
            parse_op(&format!("{valid} __unexpected_tail__")).is_err(),
            "silently accepted optional-form tail: {valid}"
        );
    }
    for malformed in [
        "heat v1 10kJ on",
        "heat v1 10kJ on candle hotplate",
        "transport v1 from v2 ignored to v3 steps 3",
        "transport v1 from v2 to v3 ignored steps 3",
        "transport v1 from v2 to v3 steps 3 courant 0.5 courant 0.1",
    ] {
        assert!(
            parse_op(malformed).is_err(),
            "silently accepted: {malformed}"
        );
    }
}
