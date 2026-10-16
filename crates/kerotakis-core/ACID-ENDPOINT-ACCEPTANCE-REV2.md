# Acid endpoint acceptance: operator-boundary fixture revision 2

Prepared before executing the revised fixture, 2026-10-16. The original
`a03113a2` source and first failure remain immutable.

[CI38863384721, job116850587674](https://github.com/CrispStrobe/kerotakis/actions/runs/38863384721/job/116850587674)
failed `the_prewarmed_cache_covers_the_lessons` during isolated endpoint
acceptance. The saved output has MnII0.002mol and target O2 0.005mol. Its
ordinary first-answer event cools the open fixture from298.07260663471266K to
297.99521306177974K before ReactionOccurred. The operator snapshot still holds
raw molecular stock at298.15K. The fixture had prepared an aqueous state and
then characterized it again using that same raw preparation snapshot; its
whole-wrapper temperature comparison included ordinary preparation heat.
This source-only diagnosis does not assign a measured heat or calibration.

Revision2 leaves the original reagent amounts, phases, boundary, target
stoichiometry, comparison function and tolerances unchanged. It explicitly
separates setup from the tested operator: characterize the original molecular
fixture, then capture the tested operator's StepStart from the prepared state.
The shared native/browser wrapper then runs once from that prepared snapshot.
The missing-current-question warmer uses the identical preparation and new
operator boundary, and still never warms its transformed endpoint answer.

Each original acceptance condition now has a separate named result. Failure
output includes both complete states, both temperatures and the events; success
output also records before/after temperatures. The endpoint continuation's
strict thermal, material, gas and trusted-answer checks remain unchanged.

Only acceptance preparation/diagnostics change. Chemistry admission, acid
ownership/debits, gas events, continuation transactions, readback and thermal
policies retain the original product source. Hosted native, prewarm, cache,
Wasm and real-browser checks must execute this revised head. Original F12
replay38864486610 remains bound to its original executable; no new physical
execution or scientific verdict is established by this fixture repair.
