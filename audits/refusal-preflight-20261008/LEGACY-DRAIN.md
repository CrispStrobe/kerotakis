# Legacy Drain fixture adaptation

Mac native CI [run 37737767183, job 113181501188](https://github.com/CrispStrobe/kerotakis/actions/runs/37737767183/job/113181501188) compiled and passed all 30 new refusal/preflight controls and 20 inherited safety-veto controls. The overall job failed in two older Drain assertions. This is executed targeted evidence, not full CI acceptance. The checkout was merge revision `05eb16de03c8236285c07b03be8e8135f9157b1b`, whose source tree is identical to PR head `faea153bc77a1e0093e480c367ebdd341915796b` (GitHub compare reports no changed files).

`liquid_extraction::draining_outside_an_empirical_partition_temperature_moves_no_material` expected an empty receiver to be created after an explicit temperature-domain refusal. Its adaptation now requires the complete Bench except log to remain unchanged and no VesselCreated event; the original source preservation and refusal diagnostic assertions remain.

`proportional_transfer::a_transfer_creates_the_vessel_it_is_aimed_at` exercised Drain on water and solid chalk, a single-liquid-phase refusal. The accepted Drain case now adds hexane and asserts an actual layered state before requiring the original receiver creation event/count. Filter and Decant cases remain unchanged.

No production change accompanies this adaptation. Exact original full files are preserved in `legacy-drain-originals/`; original/adapted hashes and precise changes are in `legacy-drain-adaptation.json`. Source-bound runtime outcomes/log hash are in `mac-native-faea153b.json`. Frozen new refusal/preflight contracts remain unchanged. Adapted legacy controls and refreshed full CI remain pending.
