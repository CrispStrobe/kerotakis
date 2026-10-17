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
