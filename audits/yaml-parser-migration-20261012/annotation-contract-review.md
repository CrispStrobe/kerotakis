# Annotation admission contract review and next lane

The preserved [twenty-case first outcomes](supplement-20/paired-first-outcomes.json) report **18 pass and 2 fail on each parser**, with identical individual outcomes. `ignored_custom_scalar_tag` and `ignored_custom_sequence_tag` forecast successful document-level notes, but both sources returned `MechanismError::Yaml` during flattened deserialization. These are unmet prospective expectations. They do not establish a migration regression, a promised feature, or an implemented fix. Their [fixture and freeze](supplement-20/freeze.json) remain immutable.

Source binding: accepted main `a125c88d5e4e97cfc55eb2f51e0490e9cb0ed21b`, tree `cbe987d1fba5eaeaecbd7a23232fb2105eee8684`, and maintained-parser candidate `1acdda4696e565be5dee13f0598eb2aa8081304e`. The parser implementation bytes are identical on those sources; SHA256 `b074b5f1c16508765cdff6e2db280537e37c586ea7f424a08760c905edac9640`. This review performs source and existing-evidence analysis only; it executes no additional controls.

## What the public contract establishes

[The mechanism pack documentation](../../data/mechanisms/README.md#the-supported-subset) calls this a portable Cantera YAML subset. It states that named annotations are “parsed and ignored” and that unsupported constructs receive typed refusals. [BRD-040 section 3.4](../../provenance/brd-040-cantera-audit.md#34-unknown-keys-were-dropped-rather-than-refused) identifies six raw structures with unmatched-key capture and short allowlists. Neither document promises arbitrary application-specific YAML tag syntax or universal YAML conformance.

[The source](../../crates/kerotakis-core/src/kinetics/mechanism.rs) describes auditable key allowlists, not a generic metadata extension mechanism. Its `Value` payload permits successful deserialization of ordinary nested annotation values and keeps values available for selector diagnostics. It does not itself guarantee that Serde's flattened content buffer can admit every tagged node that a standalone YAML `Value` could represent. The first failures specifically name the flattened enum-input limitation. That implementation explanation is not an application-level decision to support or reject all custom tags.

There are **seven `#[serde(flatten)]` attributes, but six annotation owners**. The seventh belongs to `RawPressureRate` and flattens modeled rate fields. Treating it as another allowlisted metadata owner would introduce a new contract.

## Annotation owners, modeled fields and defaults

All six `extra` fields currently use `BTreeMap<String, serde_yaml_ng::Value>`. A missing annotation is absent from that map; an admitted annotation value is stored, then ignored by the corresponding allowlist check. Annotation values are not retained in `ParsedMechanism` or serialized as persistent mechanism metadata.

| Owner | Modeled fields and defaults | Extra keys admitted and ignored | Unknown-key refusal after successful YAML admission |
|---|---|---|---|
| Document (`RawMechanism`) | `description` defaults to empty text; `units` defaults to `RawUnits`; `phases`, `species`, `reactions` default to empty vectors. A blank description ultimately becomes `runtime-mechanism`; empty required collections receive `MissingPhases`, `MissingSpecies`, `MissingReactions`. | `generator`, `input-files`, `cantera-version`, `date`, `note`, `references`, `elements` | `UnsupportedSection { section }` |
| Units (`RawUnits`) | Optional `length`, `time`, `quantity`, `energy`, `activation-energy`, `pressure`, `temperature`. Defaults: metres, seconds, kilomoles, joules, pascals; activation scale derives from energy/quantity. An explicit temperature scale must be `K`. | `mass`, `current` | `UnsupportedField { owner: "units", field }` |
| Phase (`RawPhase`) | Required `name`, `thermo`; optional `species`, `reactions`, `kinetics`. Absent/null selectors mean all declared species/reactions. Species also accepts `all` or a sequence of names. A present reactions selector must be `all`. Absent kinetics enables this loader's network; present kinetics accepts `gas` or `bulk`. Thermo must be `ideal-gas`. | `elements`, `note`, `state`, `transport`, `adjacent-phases`, `skip-undeclared-elements`, `skip-undeclared-third-bodies`, `explicit-third-body-duplicates` | `UnsupportedField { owner: "phase '<name>'", field }`; unsupported thermo is checked first and yields `UnsupportedPhase` |
| Species (`RawSpecies`) | Required `name`, typed string-to-f64 `composition`; optional `thermo` defaults absent. Element counts must be finite and positive. | `note`, `transport`, `critical-parameters`, `equation-of-state` | `UnsupportedField { owner: "species '<name>'", field }` |
| Species thermo (`RawThermo`) | Required `model`, `temperature-ranges`, `data`; optional `reference-pressure` defaults absent and normalizes to 101325 Pa. Model/range/coefficient validation remains separate. | `note` | `UnsupportedField { owner: "species '<name>' thermo", field }` |
| Reaction (`RawReaction`) | Required `equation`; `type` defaults `elementary`; rate/high-rate/low-rate/Troe options default absent; pressure-rate list, efficiencies and orders default empty; `default-efficiency` defaults 1; `negative-orders` defaults false. `Troe` and high/low pressure field spelling remain case-sensitive as declared in source. | `note`, `id`, `duplicate` | `UnsupportedField { owner: "reaction <one-based index>", field }` |

`reject_unknown_fields` examines captured keys in the `BTreeMap` order and returns the first non-allowlisted key. It does not inspect ignored values. The entire typed raw document must deserialize first: a YAML/type error in an annotation or another field may therefore occur before a named unknown-field refusal. This matters when specifying a future custom visitor.

`species` and `reactions` phase selectors are model-bearing `Value` fields, not annotations. Their unsupported forms receive `UnsupportedFieldValue` with owner, field and compact rendered value. Rate scalars are an untagged numeric/text union: quoting affects admission and field-specific refusal. Neither selector nor rate tag behavior may be broadened by an annotation-only change.

## Other nested rate structures

`RawPressureRate` requires `P: Scalar` and flattens `RawRate`, which requires `A: Scalar` and `Ea: Scalar` while `b: f64` defaults to zero. `RawTroe` requires `A`, `T3`, `T1`; optional `T2` defaults absent. These structures have no annotation allowlists and no unmatched-key capture or `deny_unknown_fields` attribute.

Consequently, the six-owner rejection mechanism does not prove a corresponding nested-rate unknown-key guarantee. The code leaves the default Serde unknown-field behavior in place for these structures. This is a separate source-indicated scope question, potentially more consequential than decorative metadata tags because these structures carry modeled rate data. Freeze original controls for unknown fields in ordinary rate constants, high/low pressure rate blocks, pressure-rate entries and Troe before calling it a demonstrated defect or changing admission. Keep this inquiry out of an annotation-only patch.

## Duplicate and merge evidence without overgeneralizing

The preserved twenty independently establish these distinctions on both sources:

- Repeated outer document `note` entries are admitted and leave the full parsed mechanism unchanged.
- Repeated known `A` fields in a rate mapping receive a YAML refusal.
- Repeated composition `H` entries retain the final typed-map value in the tested document.
- Repeated keys inside a generic nested annotation mapping receive a YAML refusal.
- Ordinary `<<` keys on document, phase and species retain their named section/field refusals.
- An alias and ordinary merge key wholly inside an ignored note are admitted and leave the mechanism unchanged.

These are specific witnesses, not a universal duplicate-key policy. A repeated reaction list entry and its ignored `duplicate:` assertion are another concept: the source compiles each reaction independently and does not enforce the assertion. An annotation visitor must preserve these distinctions rather than implement global last-wins, duplicate rejection, merge expansion or tag stripping.

## Recommended decision and priority

Keep the maintenance repair scoped to preserved admission behavior. The evidence supports documenting an **observed custom-tagged annotation boundary** now: document-level custom-tagged scalar and sequence notes are refused by both existing parsers. No support promise has been established for that syntax. Record the two frozen positive forecasts as unresolved contract expectations; do not relabel them as passing negative controls or declare a parser defect from those forecasts alone.

This documentation/matrix lane follows the unchanged dependency-policy gate and previously demonstrated correctness repairs. An additional source-based nested-rate unknown-key matrix deserves separate prioritization because silent model-bearing key loss would affect scientific meaning if confirmed. General custom-tag support is an optional explicit admission expansion, not a prerequisite for the maintenance replacement.

A fresh agent can complete the annotation lane as follows:

1. Read the owner table above, the public subset documentation, original fifty and preserved supplement twenty. Record a policy decision in a new receipt: either retain/document an explicit boundary for application-specific annotation tags, or justify support with a concrete supported authoring need. Do not infer support solely from an allowlisted field name or `Value` type.
2. Freeze a new owner-by-owner matrix before execution. Cover all six actual annotation owners, every allowlisted key at least once, and scalar/sequence/mapping tags at the owner root and nested within otherwise ordinary metadata. Include ordinary null/bool/string/numeric/nonfinite values, numeric/complex keys, aliases, duplicates, ordinary merge keys, unknown outer keys and malformed YAML. Keep model-bearing selector/rate/type-tag controls separate and unchanged. Use a valid NASA7 witness for thermo and appropriate simple/falloff/pressure-rate witnesses for nested modeled structures.
3. If retaining the boundary, make new refusal expectations explicit and independently verify every owner claimed by documentation. Preserve the original positive forecasts and their failures as a distinct historical record. Do not state that all owner/tag combinations are refused until observed.
4. If expanding support, first write a reviewed proposal limited to keys already allowlisted. Consider a private map visitor that deserializes known fields through their existing typed paths and handles only allowed metadata through a bounded ignore/capture visitor. Preserve required-field defaults, known-field duplicate errors, named unknown-field refusals, parsing order where material, typed string keys, complex annotation keys, nonfinite scalar categories, selector rendering and ordinary merge behavior. A value tree for the whole document, global tag stripping or global key stringification would change more than this policy permits.
5. Replacing an `extra` value with `IgnoredAny` alone is not a demonstrated fix: derived flattening still buffers content before that value visitor, and a parser may reject the node first. Inspect the actual backend's public APIs and preserve syntax/budget errors rather than bypassing validation. No new public metadata type or persistence format is required. The dependency migration's public `MechanismError::Yaml` payload-type change is already a separate recorded issue.
6. Preserve commands, exact source/tree and lock, all raw outcomes and independent reviews in new evidence destinations. Require original fifty, immutable supplement twenty, the new matrix, complete affected core/pack libraries and all fifteen CLI controls. Any code expansion needs its own source-associated CI/native/fleet and guarded integration; matching prior failures does not claim a new repair passed.

Completion means an evidence-backed, explicitly documented annotation policy and independently verified owner matrix. It does not require inventing arbitrary tag semantics or converting the frozen twenty to an unqualified 20/20 claim.
