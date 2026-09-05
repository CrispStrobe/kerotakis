use std::path::PathBuf;

use kerotakis_codex::curiosity::{load_manifest, ActionFamily, AgeBand, Disposition};
use kerotakis_core::script::ParseErrorKind;

fn corpus_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/coverage/curiosity-v1/manifest.toml")
}

#[test]
fn curiosity_v1_is_complete_and_structurally_sound() {
    let corpus = load_manifest(&corpus_path()).expect("load curiosity-v1");
    let problems = corpus.lint();
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    let inventory = corpus.inventory();
    assert_eq!(inventory.prompts, 500);
    assert!(inventory.complete);
    assert!(inventory.smoke_prompts >= 16);

    // Every route a prompt can REQUIRE is exercised somewhere in the corpus.
    //
    // `Missing` is deliberately not among them: `expected` is a requirement,
    // and nothing requires the engine to stay silent — `lint` refuses it, and
    // the assertion above would have failed first if one slipped in. What the
    // engine actually does is the baseline's business, not this field's.
    for disposition in Disposition::ALL {
        if disposition == Disposition::Missing {
            assert_eq!(
                inventory
                    .by_expected
                    .get(&disposition)
                    .copied()
                    .unwrap_or(0),
                0,
                "`missing` cannot be a requirement"
            );
            continue;
        }
        assert!(
            inventory.by_expected[&disposition] > 0,
            "missing {disposition:?}"
        );
    }
    // And stating no requirement is the common case, not an edge one: most
    // prompts are questions worth asking that nobody has committed a route
    // for. If this ever reaches zero, the field has quietly become mandatory
    // again.
    assert!(
        inventory.without_requirement > 0,
        "no prompt states an absent requirement — has `expected` become mandatory?"
    );
    for action in ActionFamily::ALL {
        assert!(inventory.by_action[&action] > 0, "missing {action:?}");
    }
    for age_band in AgeBand::LEARNER_BANDS {
        assert!(inventory.by_age_band[&age_band] > 0, "missing {age_band:?}");
    }
    // `UnknownSpecies` still has corpus rows and should: substances this
    // shelf does not carry are a standing state, not a defect.
    //
    // `UnknownReaction` no longer has one. bio-064 and bio-080 were the
    // only two, and BRD-023/BRD-052 closed both by curating the reactions
    // they asked for — which is the outcome this corpus exists to produce.
    // What the assertion was actually protecting is that the `react` verb
    // still vets its names and still says so in a TYPED way, so that is
    // asserted against the parser directly. Keeping a row permanently
    // broken to feed a test would be the corpus serving the test.
    assert!(
        corpus
            .prompts
            .iter()
            .any(|prompt| prompt.parse_boundary == Some(ParseErrorKind::UnknownSpecies)),
        "corpus does not exercise UnknownSpecies"
    );
    assert_eq!(
        kerotakis_core::script::parse_op_typed("react v1 transmutation")
            .expect_err("an uncurated reaction name must not parse")
            .kind,
        ParseErrorKind::UnknownReaction,
        "the react verb no longer vets its names typed"
    );
}
