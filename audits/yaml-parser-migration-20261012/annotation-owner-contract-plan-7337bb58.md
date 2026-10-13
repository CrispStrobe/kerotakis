# Ignored annotation policy and owner matrix on accepted main

This is a source-informed planning lane bound to accepted main `7337bb58ce48285c54be5885dbe07afca1adbb7e`, tree `8613e46420a8f8d76f39732c5a50d3dcd8417811`. [The companion source binding](annotation-owner-contract-source-7337bb58.json) pins the implementation, public contract documents and preserved evidence. No new controls were executed or executable fixture frozen here. Expectations below are prospective engineering expectations, not blind experiments or chemistry results.

The public entry point is `kinetics::mechanism::parse_yaml`. It deserializes a complete typed `RawMechanism` before checking allowlists and model validity. Admitted annotation values do not survive into `ParsedMechanism`. A malformed or unsupported YAML value can therefore fail with `MechanismError::Yaml` before a named unknown-field refusal. Preserve that distinction.

## Policy evidence and present limits

Accepted main's [pack README](../../data/mechanisms/README.md#the-supported-subset) says named annotations are parsed and ignored within a portable Cantera subset. Its shipped text does not promise arbitrary application-specific tag syntax. The published audit-side clarification in that same document states that application-specific YAML tags lie outside the documented annotation subset; that clarification is separate from the accepted-main document bytes. The [agenda's lane 0b](../../docs/chemistry-work-agenda-20261012.md#0b-adjudicate-tagged-ignored-annotations-as-a-separate-admission-lane) calls for policy adjudication before an expansion.

The original [supplement twenty](supplement-20/paired-first-outcomes.json) and [independent repaired-source review](supplement-independent-review-51e99750.json) preserve **18 pass / 2 fail**: `ignored_custom_scalar_tag` and `ignored_custom_sequence_tag` expected admission but received YAML refusals on both earlier parsers, then the repaired source. Those positive forecasts remain unmet. There is no new main-source execution in this plan, no migration regression established by those witnesses, and no promise of all-owner tag support. The parser implementation on accepted main is byte-identical to the reviewed repaired source; source equality is not a postmerge test run.

Recommended first step: retain the documented tag boundary, measure its actual owner coverage, and make the public wording precise. Treat tag admission as an optional, explicitly approved subset expansion if a concrete mechanism-authoring need justifies it. Do not turn the old two positive forecasts into passing refusal controls. New boundary controls need distinct identities and a new pre-execution freeze.

## Actual owners and key meanings

There are **nine** flatten attributes: six allowlisted annotation owners, two strictly validated modeled extra owners, and the modeled `RawPressureRate` flatten. The earlier [annotation review](annotation-contract-review.md) describes the historical seven-attribute implementation; its nested-rate gap is closed by the accepted repair and must not be used as a current-source inventory.

| Annotation owner | Keys captured then ignored | Meaning and refusal after successful YAML admission |
|---|---|---|
| Document | `generator`, `input-files`, `cantera-version`, `date`, `note`, `references`, `elements` | Provenance and declarations not used by this loader. Other keys: `UnsupportedSection { section }`. |
| Units | `mass`, `current` | Unit dimensions unused by this kinetics subset. Other keys: `UnsupportedField { owner: "units", field }`. |
| Phase | `elements`, `note`, `state`, `transport`, `adjacent-phases`, `skip-undeclared-elements`, `skip-undeclared-third-bodies`, `explicit-third-body-duplicates` | Declarations, initial-state/transport/interface settings and assertions unused by this loader. Other keys: `UnsupportedField` naming `phase '<name>'`; invalid phase thermo can take precedence. |
| Species | `note`, `transport`, `critical-parameters`, `equation-of-state` | Notes and properties unused by the accepted ideal-gas kinetics path. Other keys: `UnsupportedField` naming `species '<name>'`. |
| Species thermo | `note` | Annotation only. Other keys: `UnsupportedField` naming `species '<name>' thermo`. |
| Reaction | `note`, `id`, `duplicate` | Provenance/identifier and a duplicate-reaction assertion. Entries compile independently; the flag is not enforced. Other keys: `UnsupportedField` naming the one-based reaction index. |

These **25 keys** are ignored by the current API, but are not all decorative in other Cantera applications. In particular, phase state/transport/interface settings, species equation-of-state and duplicate assertions may matter elsewhere. This lane must not turn other model modifiers into annotations.

Keep the following modeled fields on their existing typed paths: document description and collections; supported unit scales; phase name/thermo/kinetics and species/reaction selectors; species composition; NASA7 model/ranges/coefficients/reference pressure; reaction equation/type/orders/efficiencies; and every rate block. `description` affects the resulting mechanism name even though it is not a rate parameter. Phase selectors are model-bearing `Value` fields, not ignored extras.

`RawRate` requires `A` and `Ea`, defaults `b` to zero, and rejects every extra key. `RawTroe` requires `A`, `T3`, `T1`, permits optional `T2`, and rejects every extra key. `RawPressureRate` consumes required `P`, then flattens `RawRate`. Ordinary, high/low, PLOG and Troe extras are validated before reaction-type dispatch, including unused declared blocks. No annotation allowlist applies to those owners. Preserve exact spelling, defaults, missing-field behavior and refusal precedence.

## Focused first matrix: 50 prospective controls

A fresh agent should create a new fixture/manifest/freeze, with source, tree, file hashes, exact test identities, expected error categories and rationales recorded **before execution**. Do not add controls to the old fifty, twenty or their freezes.

Use a small synthetic balanced irreversible mechanism (`A => B`, both compositions `{H: 1}`, finite numeric Arrhenius inputs, ideal-gas phase). Give the thermo-owner witness a valid one-interval NASA7 block. Synthetic coefficients are admission witnesses, not calibrated chemistry. Every positive control compares the entire `ParsedMechanism` against its otherwise identical annotation-free baseline.

| Group | Count | Construction and prospective expectation |
|---|---:|---|
| Plain admitted key coverage | 25 | One control per exact allowlisted key above, with a small ordinary value appropriate to that key. Entire parsed result unchanged. |
| Owner-root custom tags | 18 | Each of the six owners crossed with custom-tagged scalar, sequence and mapping values. Use `note`, except `mass` for units. Under the retained boundary, forecast a YAML refusal; only the two historical document witnesses are already observed. |
| Nested custom tags | 6 | One per owner, inside an ordinary ignored mapping. Rotate scalar/sequence/mapping payloads across owners. Forecast refusal; record this as incomplete nested owner-by-shape coverage. |
| Tagged unknown field | 1 | An unknown document key carrying a custom-tagged value. Forecast YAML refusal during deserialization, rather than successful admission or a guaranteed later `UnsupportedSection`. |

Use application-specific `!AnnotationProbe` tags. Standard YAML tags, ordinary quoted strings that contain `!`, anchors and aliases are separate dimensions. If a baseline contradicts any prospective forecast, retain that first result and adjudicate it; do not revise the freeze to manufacture a clean score.

The fifty cover key admission and root-tag ownership, not universal YAML behavior. Before implementing a broad metadata visitor, add a separately frozen follow-up for **each owner** covering outer annotation duplicates, duplicates inside ignored mappings, numeric and sequence/mapping keys inside metadata, nested aliases/ordinary `<<`, and outer ordinary `<<`. Include unknown ordinary keys and malformed YAML. Probe nested scalar/sequence/mapping tags for the remaining owner combinations. Existing twenty witnesses remain the compatibility evidence for their exact tested locations, not substitutes for untested owners.

Keep modeled controls distinct: custom tags and quoted/unquoted finite/nonfinite values on selectors, composition counts, `A`, `Ea`, `b`, `P`, Troe and NASA7 fields; duplicate known fields; required/defaulted fields; and unknown nested rate keys. Preserve existing observed categories rather than assigning every modeled tag the same error. Ordinary merge keys must not become automatic merge expansion. Outer annotation duplicate admission does not establish which discarded value wins; nested mapping duplicate refusal and typed-map last-value behavior are different contracts.

## Feasible implementation choices after the matrix

1. **Retain and document the boundary.** No parser change is required solely by the two unmet forecasts. Publish only owner/value boundaries supported by new actual controls; keep unobserved combinations qualified. This is the smallest completion lane.
2. **Explicitly expand allowed metadata.** If support is chosen, prototype a private map visitor for the six annotation owners. Known fields must use their existing typed deserialization paths; only exact allowlisted extras receive a direct, recursively validating ignore/capture visitor. Unknown keys still refuse by owner, and malformed YAML/alias/depth/backend limits still apply. First prove this can handle tagged scalar/sequence/mapping metadata without changing modeled value admission or duplicate behavior. No public metadata type or persistence format is needed because extras are discarded.
   This means replacing only the six annotation flatten paths with explicit owner deserialization, not merely deleting their `extra` fields and allowing Serde to drop unknown keys. Direct `Value` capture for allowlisted metadata is a concrete prototype because the backend value type represents tags, but its compatibility must be demonstrated. Leave the strict `RawRate`, `RawTroe` and modeled `RawPressureRate` flatten paths intact.
3. **Targeted fallback while retaining the ordinary parser.** A second private owner visitor could be attempted after the existing YAML refusal, reading the original input and accepting a result only when it identifies custom tags exclusively inside allowlisted discarded metadata. It needs the same typed known-field paths and strict unknown/model-bearing tag checks as option 2. Preserve the original error if fallback cannot prove that narrow scope. This retains the common path but adds a second admission path and precedence risk; tag-error text alone, a generic whole-document `Value`, or blind retry is not sufficient. Include aliases that cross from metadata into modeled fields before accepting this design.
4. **Backend/API investigation if the direct visitor cannot preserve contracts.** Inspect the currently pinned maintained parser's source and buffering semantics before proposing another dependency or adapter. Replacing flattened `Value` with `IgnoredAny` alone is not proven sufficient: flattened Serde content can reject enum/tag input before that visitor is reached. Avoid global tag stripping, key stringification, whole-document normalization or automatic merge expansion; these change the admission of modeled and unsupported fields.

A change must preserve all existing owner names and field diagnostics, selector rendering, known-field duplicate errors, the tested composition-map duplicate behavior, complex ignored keys, numeric/nonfinite categories, required/defaulted fields, nested-rate pre-dispatch validation, whole-document validation and bounded YAML failure behavior. An annotation-support expansion should not silently loosen tagged modeled inputs or unknown extras.

## Compatibility gates and handoff

Read the source binding and preserved evidence first. Keep original fifty, supplement twenty and their first outcomes immutable, including the historical 18/2 distinction. Before a code change is accepted, require the original fifty, nested-rate thirteen functions, unchanged supplement twenty, new prospective owner matrix, complete affected core/data/pack libraries, all fifteen shipped-pack tests and supported CLI controls (original fifteen plus nested twelve). A new source needs its own CI/native/fleet and guarded integration evidence; old-source successes do not transfer merely because the patch looks small. Record actual lock/binary/source associations and all first failures in new evidence destinations. Builds and executions remain subject to the operating resource gate.

Completion is an evidence-backed annotation policy and an owner matrix that supports its wording. A separately justified tag-admission feature can follow. This plan accepts no new test result, parser fix or integration.
