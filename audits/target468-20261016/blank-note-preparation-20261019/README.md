# Neutral-water pH note localization — focused preparation

Source `b703b695d6fea2efdf0f0ac6ff7fd172e8b49cc1` starts from actual main
`0b579699`. Tests-only baseline `7595b3e1be2576e0ec72900d42bdac84ad2f9477`
retains that runtime unchanged. [Frozen plan](frozen-plan.json),
[exact CPU worker](worker.py) and [actual issue binding](issued.json) preserve
source/tree, identical test-fixture SHA, seeded dependency-lock bytes and the
provider's actual kernel reference. A title/slug warning did not trigger a retry.

The pH meter's already admitted plain-water/glucose blank carried a long English
`note` and no typed `note_reason`. The repair uses a `Phrase` and adds German and
French catalogue entries. Its English legacy note, pH7, temperature/admission
conditions and numerical model remain unchanged. The real renderer already uses
`note_reason`; no English-text matching or new renderer fallback was added.

Four controls exercise actual measurement events: German and French LV2/LV3
transcripts for plain water and glucose, exact English legacy note/value, and
continued refusal for warm water, an uncharacterized acid and a dry vessel.
The source inspection forecasts two baseline assertion failures and two positive
passes; these are not observed results yet. The worker must distinguish compiler
failure from those assertions before testing the repair. It then runs locale
lint, composed-locale controls and core all-target Clippy on the fixed source.

Local standalone formatting, both catalogue TOML parses and diff/privacy checks
pass. No local Rust build was attempted. Focused hosted execution is issued;
raw results and independent review remain pending. No chemistry experiment,
full fleet, deployment or scientific selection was issued by this worker.

Keep PR841's active exact head and its existing collectors untouched. After its
accepted integration/delivery, refresh this narrow branch with actual main and
validate the combined source, applicable full core/native/product fleet and
protected/platform checks before exact-head normal auto-merge. Authenticate the
changed served browser transcripts afterward. Do not treat this focused run as
complete source or production acceptance. Public chemical-domain and dynamic
family/vendor-message i18n lanes remain open.

The first issued worker subsequently stopped during dependency setup, before any
test execution. [Preserved first stop](../blank-note-first-setup-stop-20261019/README.md)
and [separate corrected worker](../blank-note-submodule-preparation-20261019/README.md)
retain unchanged runtime, fixture, forecasts and lock. Neither is yet acceptance.
