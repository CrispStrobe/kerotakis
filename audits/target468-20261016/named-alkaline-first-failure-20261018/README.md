# Named hydrolysis: authentic first baseline and source outcomes

These immutable outputs bind test-only baseline `2157e5fa` (unchanged production
runtime `d3883757`) and repair `03aab10c`, with byte-identical chemical control
files and actual producer lock `66886813`. All60 producer evidence hashes and
collected file bytes were independently authenticated. There were no original
corpus replays or scientific selections.

The baseline executed nine core tests: three passed and six failed. The native
baseline executed three tests: the actual alkali-cleavage and neutralization-order
controls failed; the weak-base/refusal control passed. Preserve the original
failure states and assertions. This demonstrates the runtime gap, rather than a
compilation or transport failure.

On the repair, all-target Clippy passed. The full core run passed2136 tests,
failed one and ignored zero. All nine new chemical controls passed, including
base/ester limits, potassium, proton reserve, unknown-pool refusal and serialization.
The unchanged composed-locale control caught a missing French key for the new
strong-base refusal message. The worker stopped there, before changed-source
native execution. An expected caught-unwind panic in the kinetics library is
not an additional failed test; the actual test-result summaries are retained.

Correction `bf03c11a` adds the missing French message and a meaningful English
sentence, preserving both chemical control files. Its separate validation must
still pass full core/native controls and independent chemical review. Full33,
platform/protected integration, delivery and original OR006 replay remain required.
No native or scientific acceptance is assigned by this first-failure evidence.

`raw/SHA256.json` authenticates the original raw files; `frozen-plan.json` binds
the uploaded worker and source. Do not normalize raw whitespace or replace the
failed logs with a later result.
