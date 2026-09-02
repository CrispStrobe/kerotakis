//! EXP-0's acceptance, bench half: the demo quest completes through
//! the real REPL with the full solver stack — sealed unknown masked in
//! every rendered line, identified only by name at the end, the pH
//! value-claim read from the actual solved state, and two command
//! orders both reaching the star.

use std::io::Write;
use std::process::{Command, Stdio};

fn run_repl(script: &str) -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kero"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("kero repl spawns");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("repl exits");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Everything a learner does *before* naming the unknown: the whole
/// observation half of ORDER_A, and the part the mask has to survive.
///
/// Kept as its own script rather than as a prefix of the output, and that
/// is not tidiness. The leak assertions used to run on
/// `out.split("quest answer").next()` — but `quest start` prints its own
/// instructions, ending "name one with `quest answer <alias> <species>`",
/// so the split landed on the instruction line and the assertions saw
/// only the two lines of the start banner. They were passing on an empty
/// haystack. Running the observation half as a script and checking all of
/// its output has no boundary to get wrong.
const OBSERVATIONS: &str = "quest start the-white-unknown\n\
    add v1 water 250mL\n\
    add v1 unknown-a 1g\n\
    inspect\n\
    particles v1\n\
    explain v1\n\
    measure v1 ph\n";

const ORDER_A: &str = "quest start the-white-unknown\n\
    add v1 water 250mL\n\
    add v1 unknown-a 1g\n\
    inspect\n\
    particles v1\n\
    explain v1\n\
    measure v1 ph\n\
    quest answer unknown-a NaCl\n\
    quest status\n";

const ORDER_B: &str = "quest start the-white-unknown\n\
    add v1 water 250mL\n\
    quest answer unknown-a NaCl\n\
    add v1 unknown-a 1g\n\
    measure v1 thermometer\n\
    measure v1 ph\n\
    quest status\n";

#[test]
fn order_a_completes_with_the_unknown_sealed() {
    let out = run_repl(ORDER_A);
    assert!(out.contains("quest started"), "start line: {out}");
    assert!(
        out.contains("★ quest complete"),
        "the star at the end: {out}"
    );
    assert!(out.contains("COMPLETE"), "status agrees: {out}");
}

/// The mask holds in every rendered line while the quest runs — the x-ray
/// views included: `inspect`, `particles` and `explain` once printed the
/// vessel's truth unmasked, and a census row reading `Na+` answers the
/// quest no less than one reading `NaCl`.
#[test]
fn nothing_names_the_unknown_before_it_is_named() {
    let out = run_repl(OBSERVATIONS);
    for leak in ["sodium", "chloride", "NaCl", "Na+", "Cl-"] {
        assert!(
            !out.contains(leak),
            "sealed until answered, but '{leak}' printed: {out}"
        );
    }
    assert!(out.contains("unknown-a"), "the alias speaks instead: {out}");
    // The haystack is real — this is what the old `split` boundary lost.
    assert!(
        out.lines().count() > 10,
        "the observation half must actually have run: {out}"
    );
}

#[test]
fn order_b_also_completes() {
    let out = run_repl(ORDER_B);
    assert!(
        out.contains("★ quest complete"),
        "a different order reaches the same star: {out}"
    );
}

#[test]
fn the_learners_own_bottle_is_not_masked_but_poured_unknown_travels_sealed() {
    // Two vessels: v1 holds the sealed sample, v2 the learner's own NaCl
    // — same substance, different knowledge. The covers follow the alias,
    // not the species: v2's census names its ions openly, and only after
    // matter is poured out of the sealed vessel does its destination go
    // behind the covers too.
    let script = "quest start the-white-unknown\n\
        add v1 water 250mL\n\
        add v1 unknown-a 1g\n\
        new\n\
        add v2 water 100mL\n\
        add v2 NaCl 1g\n\
        particles v2\n\
        new\n\
        add v3 water 100mL\n\
        decant v1 v3 0.5\n\
        particles v3\n\
        register lv1\n\
        particles v1\n\
        register lv2\n";
    let out = run_repl(script);
    // v2 — the learner's own bottle — speaks plainly.
    assert!(
        out.contains("Na+"),
        "the learner's own NaCl census is not masked: {out}"
    );
    // v1's and v3's censuses wear the alias.
    let v2_census = out
        .split("v2 — what the particles are doing:")
        .nth(1)
        .unwrap()
        .split("particles")
        .next()
        .unwrap();
    let sealed_censuses = out
        .split("v3 — what the particles are doing:")
        .nth(1)
        .unwrap();
    // v1's census runs at lv1, where the renderer speaks in plain
    // registry names — the mask must cover that spelling too.
    for leak in ["Na+", "Cl-", "sodium", "chloride"] {
        assert!(
            !sealed_censuses.contains(leak),
            "poured unknown stays sealed in v3 and v1, but '{leak}' printed: {sealed_censuses}"
        );
    }
    assert!(
        sealed_censuses.contains("unknown-a"),
        "the alias speaks in the sealed censuses: {sealed_censuses}"
    );
    assert!(
        v2_census.contains("Na+"),
        "v2's own census names its ions: {v2_census}"
    );
}

#[test]
fn a_wrong_answer_is_spoken_and_nothing_locks() {
    let script = "quest start the-white-unknown\n\
        add v1 water 250mL\n\
        add v1 unknown-a 1g\n\
        quest answer unknown-a CuSO4\n\
        measure v1 ph\n\
        quest answer unknown-a NaCl\n";
    let out = run_repl(script);
    assert!(
        out.contains("is not CuSO4"),
        "the refusal names the guess: {out}"
    );
    assert!(
        out.contains("★ quest complete"),
        "the right answer still completes afterwards: {out}"
    );
}

/// The same script as a file, through `kero run … --json`.
///
/// `--json` is a `run` flag rather than a REPL one: the stream is a
/// contract for a host driving a script, and that is the surface the
/// masking has to hold on.
fn run_json(script: &str, name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("kero-quest-json-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join(format!("{name}.lab"));
    std::fs::write(&lab, script).unwrap();
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .current_dir(root)
        .output()
        .expect("kero runs the script");
    assert!(
        out.status.success(),
        "kero run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The mask on the wire, not only on the screen.
///
/// This was a KNOWN LIMIT on the record: `--json` carried true species
/// keys while a sealed unknown was on the bench, because hosts key
/// rendering off those ids and rewriting them is a change hosts must be
/// told about. But a mask that holds in the REPL and not in the stream is
/// not a mask; it is a mask plus a way around it, and `--json` is the
/// easier of the two to read. The same script as `ORDER_A`, through the
/// same REPL, with `--json`.
#[test]
fn the_json_stream_seals_the_unknown_too() {
    let out = run_json(OBSERVATIONS, "observations");
    let before_answer = out.as_str();
    // Every spelling the text mask covers: the key, the registry name,
    // and the dissociation ions that have no alias of their own.
    for leak in ["sodium", "chloride", "NaCl", "Na+", "Cl-"] {
        assert!(
            !before_answer.contains(leak),
            "the --json stream leaked '{leak}':\n{before_answer}"
        );
    }
    // Every line is still a JSON object — the mask must not cost the
    // contract it rides on — and the placeholder speaks in it.
    let mut sealed_lines = 0;
    for line in before_answer.lines() {
        if !line.starts_with('{') {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("not JSON: {e}\n{line}"));
        if let Some(sealed) = value.get("sealed") {
            sealed_lines += 1;
            // The additive field declares placeholders, never the mapping:
            // telling a host what `unknown-a` really is would be the leak
            // with an extra step.
            let declared = sealed["placeholders"].as_array().expect("placeholders");
            assert!(
                declared.iter().any(|p| p == "unknown-a"),
                "the placeholder is declared: {sealed}"
            );
            assert!(
                !sealed.to_string().contains("NaCl"),
                "the declaration must not carry the answer: {sealed}"
            );
        }
    }
    assert!(
        sealed_lines > 0,
        "no line declared its placeholders:\n{before_answer}"
    );
    assert!(
        before_answer.contains("unknown-a"),
        "the alias speaks instead:\n{before_answer}"
    );
}

/// Nothing is masked when nothing is sealed — the stream a host without a
/// quest sees is the stream it always saw, `sealed` field included (it is
/// absent, not empty).
#[test]
fn the_json_stream_is_untouched_without_a_quest() {
    let out = run_json("add v1 water 250mL\nadd v1 NaCl 1g\ninspect\n", "no-quest");
    assert!(
        out.contains("NaCl"),
        "an unsealed bench names its salt: {out}"
    );
    for line in out.lines().filter(|l| l.starts_with('{')) {
        let value: serde_json::Value = serde_json::from_str(line).expect("JSON");
        assert!(
            value.get("sealed").is_none(),
            "no quest, no sealed field: {line}"
        );
    }
}
