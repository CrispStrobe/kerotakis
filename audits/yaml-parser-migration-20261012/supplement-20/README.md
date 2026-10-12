# Twenty prospective YAML edge controls

These independently authored controls extend the separate [frozen fifty](../freeze.json). They were frozen before any execution on accepted main or a proposed replacement parser. Their synthetic rates exercise admission behavior; they provide no empirical chemistry evidence.

The [fixture](fixtures/yaml_mechanism_edge_contracts.rs) uses only public `parse_yaml`, `ParsedMechanism` equality and `MechanismError` variants. The [freeze](freeze.json) binds accepted main, source tree, fixture bytes, all twenty names, categories, witnesses and rationales. Supported cases compare the complete parsed mechanism with an independently authored ordinary document. Refusals preserve field-specific numerical/text categories, owners and offending values where relevant.

The cases cover quoted numbers and nonfinite strings versus numeric nonfinite values, ignored custom tags versus model-bearing tags/selectors, ordinary merge keys at three schema levels, an alias/merge inside ignored metadata, and three distinct duplicate-key situations. Annotation-tag and duplicate expectations are prospective: a baseline failure is evidence to preserve, not permission to rewrite the forecast.

A fresh runner must verify the freeze and fixture hashes, obtain exactly the frozen source and tree, inject this fixture at the declared repository destination, and execute all twenty tests without filters or ignored cases. Preserve commands, actual source and lock, full logs, exit code and individual results in new evidence destinations. Run the same immutable bytes on the candidate afterward. Keep original baseline failures and these new observations separate from the existing fifty. Do not claim completion before actual execution and independent review.

## First execution and actionable follow-up

[The preserved paired outcomes](paired-first-outcomes.json) are **18 pass / 2 fail
on each source**, unfiltered, with identical per-case results. The failing new
positive forecasts are `ignored_custom_scalar_tag` and
`ignored_custom_sequence_tag`. Both report a YAML refusal because untagged and
internally tagged enums do not support enum input in the flattened annotation
path. Neither frozen fixture nor forecast was changed. The first candidate
resource gate refused execution; a separate candidate-only continuation completed
its twenty cases without rerunning the original first failures.

A fresh agent should first inspect the published annotation contract and every
`#[serde(flatten)]` annotation owner in
`crates/kerotakis-core/src/kinetics/mechanism.rs`. Establish whether custom tags
in ignored metadata are intended supported syntax or an explicit boundary.
Record that decision and its evidence separately from these preserved forecasts.
If support is warranted, freeze a bounded matrix across document, phase, species,
reaction and rate owners before changing production code. Include scalar,
sequence and mapping tags; nested ignored metadata; complex annotation keys;
unknown-key diagnostics; duplicate known fields; duplicate annotations; ordinary
merge keys; and model-bearing tagged scalars.

Avoid global key stringification, tag stripping, or preconverting the whole
document to a value tree: those can change typed keys and duplicate/refusal
semantics. If a custom map visitor is necessary, compare its behavior across all
annotation owners and keep model-bearing field admission unchanged. Require the
original fifty controls, this preserved first supplement, the new owner matrix,
complete affected libraries and all fifteen pack tests. Document any intentional
policy expansion and validate its own candidate source. This is a separate
follow-up to the maintenance replacement; it must not silently expand PR783's
contract or claim that all twenty original forecasts passed.
