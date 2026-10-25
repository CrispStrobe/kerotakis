# SP003 binary-distillation precision lane —2026-10-23

Complete current phase/source/product acceptance before this lane. This is a source audit and executable task scope, not a newly observed original experiment failure.

## Scope and binding

Read-only source diagnosis for `new120:sp-003`. No original script was replayed; no engine, build, test, provider, or source edit was run.

- Current narrow source: commit `ed7d8c5ed517725cdaa69ed87a9a3de7a5a2f89a`, tree `6987cafbcd77f9123f560fd60fac9ee32f68be55`.
- Accepted main comparison: commit `4eee0d636475393d6c5628e81432d45168a8c4da`, tree `31b0f4ef03f0ff00fb518dfb1d8f50cc92b946da`.
- Distillation caller `crates/kerotakis-core/src/bench.rs` has the same Git blob in current and accepted main: `efc4a35353cc261d28c5efbdc6447463b65d5b45`.
- Additional-solvent routing `crates/kerotakis-core/src/volatility.rs` has the same blob: `565c872e1e644fae6b60600eb48a3cf9d6b166ea`.
- Binary still `crates/kerotakis-thermo/src/vle.rs` is blob `3ad600507df1d75d2808fed09ce2302b48c9277b` in both current and accepted main. The blob is also unchanged from original execution source `c44692053a94c59dcf62e7fb67549f84c7d18f83`.
- Batch still `crates/kerotakis-thermo/src/batch.rs` is blob `2d172987a4363cd4b6cbc3c281dc28c80752ac92` in current, accepted main, and original execution source `c44692053a94c59dcf62e7fb67549f84c7d18f83`.
- `audits/target468-20261016/common-engine-completion/scripts/new120_sp-003.lab` SHA-256: `203a94f77d183e9be5adfd54dcb62a1024bd25979e94d036cba16e97c52df085`.
- `audits/target468-20261016/remaining-ten-readiness-20261022/ranked-originals.json` SHA-256: `1304c2f2fe986f18f2fa436fb1e00ed0d87f28a28d361bd54ee6cc5e22412c9b`.
- Historical result reference in `audits/target468-20261016/common-engine-completion/results/verified-results.json` binds the same script to stdout SHA-256 `54123256e498c021885b08e30a764bc2ebab3f5d116eaa11551a78b94e3d6af2`, source `c44692053a94c59dcf62e7fb67549f84c7d18f83`, tree `f17216653825f3149bb7a0e46946834ee5c7e3df`, binary SHA-256 `670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f`, and lock SHA-256 `82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229`. This saved result records one `not-modeled.still-condensate-precision` limit and one successful `distilled` event. It is a historical observation, not a present execution.

## Original and acceptance boundary

The immutable protocol is register LV3; add 100 g water and 10 g ethanol; split 50% of v1 into v2; distil 10% from v1 with one stage into v3; distil 10% from v2 with four stages into v4; inspect. It does not state explicit pressure or temperature, so a future exact run must record the actual vessel conditions produced by the unchanged protocol. The original forecast expects greater enrichment of the more volatile component for the four-stage cut at equal cut fraction. The two frozen criteria are:

- `SP-003-C1`: compare product compositions at equal cut fractions from the split feed.
- `SP-003-C2`: increasing stage count must not create ethanol; inventory closure includes both residues.

The original selection and readiness record explicitly say historical failure is not itself proof of a current failure and require real material, not a nominal cut. Keep the saved first refusal and its exact hashes immutable.

## Current source diagnosis

The SP003 feed contains only water and ethanol. `additional_solvent_cut` returns `None` when its inventory keys are only water/ethanol (`crates/kerotakis-core/src/volatility.rs`, lines 118–123), so the new additional-solvent/batch implementation is not the route under test. The caller proceeds to `ethanol_water_still_checked` with source pressure and the requested stage count (`crates/kerotakis-core/src/bench.rs`, lines 3382–3429). Thus the exact historical refusal maps to the binary VLE integrator, not a third-solvent property table.

In the binary integrator, a four-stage cascade can make a positive component's top fraction extremely small. The current implementation refuses when a component withdrawal is zero or cannot increment its accumulated overhead (`crates/kerotakis-thermo/src/vle.rs`, lines 2139–2143; the energy-limited branch repeats this guard at 2152–2160). That all-or-none refusal is safer than dropping matter, but the relevant VLE source is byte-identical to the source used in the historical run. The new source therefore has not repaired the demonstrated binary path at source level. This is a concrete still-present numeric-support gap for the requested four-stage conditions; whether the unchanged original still triggers it on a current binary/lock remains unverified. Do not label it a new current runtime failure or a pass without a fresh authorized execution.

The batch still also has positive-component accumulation guards, but its code is likewise unchanged since the original execution; they are immaterial to this binary-only protocol. Core receiver transfer is only reached after a successful cut. On a refusal, `bench.rs` marks the operation unchanged and emits a model-boundary event before source withdrawal or receiver deposit. For a successful binary cut it withdraws water/ethanol from the source and deposits the cut in the receiver (`bench.rs`, lines 3432 onward); the event energy is latent heat for withdrawn condensate, with the source code expressly excluding reflux/reboiler circulation and sensible heating. This is not a complete apparatus-energy ledger.

Current LV3 coverage is the implemented water/ethanol binary model at the actual vessel pressure and its supported phase domain. The caller documents a total-reflux stage cascade as a separation approximation, not a physical column; it does not predict activity-coefficient shifts, azeotropes, or real apparatus recovery. Other volatile liquids use a separate ideal-liquid route with curated approximate properties and an explicit ±40 K interval. SP003 has no additional volatile component, so those extensions do not widen its binary validation.

## Independent controls needed before adjudication

1. Preserve the exact original script and historical refusal. First freeze and execute a separate generic current-source one/four-stage matched-feed control, recording the positive-component accumulation guard and independent source/receiver/residue budgets. Diagnose and repair a demonstrated generic source gap before replaying the unchanged original as a repaired-source beneficiary. Never change its stages, split, cut or amounts; record actual vessel conditions and every retained/vented phase.
2. Independently measure each product's ethanol fraction at the exact 10% cut. Verify the one-stage and four-stage feeds are composition-matched after the split; criterion C1 cannot be inferred from a `Distilled` event alone.
3. Close ethanol and water across v1/v2 sources, v3/v4 receivers, and all residual phases. Check no positive component was silently lost and no ethanol was created. Any precision refusal is an unsupported result for the requested stage/cut, not a zero-size successful cut.
4. Use a separate generic supported-domain control with an ordinary, non-extreme binary composition and the same pressure/cut at one and four stages. It should show finite products and stage-dependent enrichment with independent inventory closure. Add a near-boundary positive-trace control to retain the all-or-none precision refusal behavior. These are controls for implementation support; they do not substitute for or qualify the unchanged original.
5. Keep energy claims bounded to the code's reported condensate latent heat unless an independent burner/reflux/sensible/residual energy ledger is supplied. No full apparatus-energy or empirical recovery claim is supported here.

## Conclusion

Source review finds no source-level repair of SP003's historical precision path between original execution source `c4469205`, accepted main `4eee0d63`, and current narrow source `ed7d8c5e`: the exact binary integrator is unchanged across all three. The original's current execution outcome remains unverified. The next evidence must be the separate current-source generic control above, followed by a source repair only if its independent budgets demonstrate a defect. An unchanged original replay comes after accepted repair delivery and must meet both frozen criteria. The historical refusal remains preserved and cannot be promoted to a current result.

Parent source check confirms the same binary/batch VLE blobs on `c8171c4e`; the intervening phase-coupling changes do not repair this separate path.
