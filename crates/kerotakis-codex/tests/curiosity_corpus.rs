use std::path::PathBuf;

use kerotakis_codex::curiosity::{load_manifest, ActionFamily, AgeBand, Disposition};

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

    for disposition in Disposition::ALL {
        assert!(
            inventory.by_expected[&disposition] > 0,
            "missing {disposition:?}"
        );
    }
    for action in ActionFamily::ALL {
        assert!(inventory.by_action[&action] > 0, "missing {action:?}");
    }
    for age_band in AgeBand::LEARNER_BANDS {
        assert!(inventory.by_age_band[&age_band] > 0, "missing {age_band:?}");
    }
}
