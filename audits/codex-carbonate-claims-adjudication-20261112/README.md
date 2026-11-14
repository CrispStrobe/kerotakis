# Codex carbonate claims adjudication — 2026-11-12

The [macOS source66d job](https://github.com/CrispStrobe/kerotakis/actions/runs/43316391517/job/131525505849)
passes its Rust tests, then fails `kero codex lint` on three old pH claims.
Linux independently reports the same three failures. Original complete codex
source/DE/FR files are frozen as gzip files, with original per-entry data and
translation projections. The original scripts and event predicates remain intact.
`macos-first-failure-excerpts.txt` preserves the observed failures; `bindings.json`
pins both original complete log hashes, source, native binary and genuine lock.

## Mechanism and observed outputs

The old entries describe instantaneous atmospheric equilibrium. The
[reviewed carbonate boundary repair](../../docs/carbonate-boundaries-20261112.md)
instead preserves zero-time carbon below the bubble-pressure threshold, assigns
finite-time surface-film transfer a separate owner, and limits the acid endpoint
to newly formed neutral CO2 and fresh reagent capacity. It does not discharge all
carbon merely because a beaker is open or bulk pH is low.

Three bounded native diagnostics execute the unchanged original scripts once
using authenticated source66d CLI bytes, SHA256 `04aefaa6bccf91bfd08584e9991c69c19dad88c29ae7d31ab44034c9a3b61b2a`,
producer lock `a43bbe9a6ce7be1b5ecbbfbf091e25124c60122b08bd2ba003c8e250d3262bdd`.
These come from artifact13988427289 on CI merge source
`c6f74c6ed07cb1734e7343835b44f6e70d28c5fc` for reviewed head
`66d0d9704d5d443684acfefdadf0395f14c791c9`. Each command is
`kero run experiment.lab --json`, with 30s CPU/2GiB address-space bounds and
exit0. Complete compressed raw stdout, stderr, exact input and receipt survive
for each. These are new diagnostics, separate from the earlier fleet results.

| Entry | Original pH claim | Actual pH | Successor pH claim | Independent interpretation |
| --- | --- | --- | --- | --- |
| baking-soda-vents-carbon-dioxide | 9.5–9.9 | 8.037131019204 | 7.9–8.2 | One hour exports0.000119077800350367mol CO2; most carbon remains. Liquid pCO2 is0.043975atm, still supersaturated relative to room air. |
| bicarbonate-plays-both-sides | 5.4–5.8 | 4.535776323530 | 4.4–4.7 | Acid vessel exports0.009822560677763293mol CO2 and retains0.000177439322mol dissolved carbon; it is not a carbon-free salt solution at room-air equilibrium. |
| the-antacid-runs-on-arithmetic | 9.0–9.4 | 7.997491346967 | 7.9–8.1 | Addition exports0.010000000000017509mol CO2 and retains0.001903769924mol dissolved carbon, predominantly bicarbonate. |

The amphiprotic entry's second vessel ends at pH11.2232008454, retains0.01mol
carbon and exports no gas; about95% is carbonate. This supports its unchanged
correct choice: bicarbonate donates a proton to hydroxide without a gas export.
It contradicts the old prose claiming0.003mol carbon had already self-vented
before lye was added. The arithmetic entry's correct choice remains the mildly
basic outcome, with its stated number changed from about9 to about8.
The first entry's correct choice remains finite gas loss without acid; its
incorrect equilibrium-with-air distractor now distinguishes transport time from
aqueous equilibrium. All three correct answer indices remain unchanged.

`review.py` independently accounts for all known Na/Cl/C/H/O and charge from
input lots, actual gas imports/exports and final stocks. Maximum residual is
1.4211e-14mol/eq against the retained1e-10 budget. No unknown chemistry appears in
these scripts. The reader verifies original scripts/events/answer indices,
strictly narrower pH intervals, rejection by the original intervals and explicit
typed acid-endpoint model-limit notices. This is physical bookkeeping validation
of model observations, not empirical validation of their pH or gas amounts.

## Reviewed content changes and validation

Only the three entries in `codex/aqueous.toml` and their affected DE/FR strings
change. Questions, distractor diagnoses and register prose now describe retained
carbon and finite transport. False claims of complete carbon loss, room-air
equilibrium after one hour and acid/protons themselves escaping as CO2 are
removed. The acid cases disclose the bounded curated endpoint where learners
see the outcome: acid-front nucleation and escape times remain unmodelled, and
the exported parcel is not a calibrated instantaneous yield. The antacid analogy
does not claim clinical efficacy or dosing authority. Existing linked references
remain consistent with bicarbonate's mildly basic character.

The first edited-content lint fails only because its equation annotation is not
accepted by the chemistry parser. Its complete input hashes, changed source
copies, stdout, stderr and receipt remain under `lint-input` / `updated-codex-lint`.
The successor equation retains the balanced chemistry and gas-arrow marker,
while prose carries the finite-time qualification. No linter rule changes.

One successor whole-codex lint runs the same authenticated binary against an
immutable copy of the revised codex files, with45s CPU/2GiB bounds: exit0 in18.35s,
131 entries replayed,114 questions and222/222 diagnosed distractors. Its exact
input hash manifest and changed source copies are preserved as `lint-input-v2`;
raw output/receipt are `updated-codex-lint-v2.*`. It also reports468 non-blocking
prose-number warnings across the whole existing codex; a clean prose-number audit
is not claimed. The source's locale lint passes with1452/1452 DE and FR paths.

Run `python3 review.py` to review saved evidence without executing chemistry.
No native source, original forecast, event predicate, linter or physical tolerance
is weakened. The final committed source still needs its protected acceptance;
no release, empirical gas-transfer calibration or new original qualification is
earned by this content adjudication.
