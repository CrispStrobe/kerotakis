# Validated provisional CLI profile

[Run 37751230970](https://github.com/CrispStrobe/kerotakis/actions/runs/37751230970) finished profiling the original executable, source `7bc43e089753483710507133ccc8377591bb3490`. The [offline review](profile-review.json) reverified archived inputs, outputs, protocol metadata and every repeated sample against the original frozen contract. Profile source/executable/generated-lock bindings match the preserved [build receipt](build-receipt.json). The profiling job's green status alone was not used as acceptance.

| Workload | Variant | Median native CLI seconds | Qualification |
| --- | --- | ---: | --- |
| F19 | a | 0.160845 | Original checks passed; Callgrind output also validated. |
| F19 | b | 0.162487 | Original checks passed. |
| F34 | a | 0.159787 | Original gas-model qualification retained. |
| F34 | b | 0.162624 | Original gas-model qualification retained. |
| F39 | a | 0.252138 | Original checks passed; Callgrind output also validated. |
| F45 | a | 0.227927 | Original checks passed; Callgrind output also validated. |

F26 was excluded after its original author identifier error. Its time is not an accepted chemistry workload. The later syntax replay is a separate result and does not replace the original profiling input.

These are whole-process timings: startup, native database initialization, commands, JSON serialization and pipe collection. Seven samples on one runner do not establish a stable tail estimate or a speedup. Instrumented instruction counts are separate from native wall time; inclusive call costs overlap and must not be added.

F19 recorded 1,380,544,962 instructions in total. `derived::registry_solid_matching` accounted for 533,395,842 inclusive instructions (38.64%): it reparses immutable solid registry formulas while matching database phases. Native database construction accounted for a separate substantial cost; its initialization and failure contract are preserved.

[PR #766](https://github.com/CrispStrobe/kerotakis/pull/766) caches parsed solid formulas while preserving lookup order, composition and hydrate matching. [Comparison run 37775230032](https://github.com/CrispStrobe/kerotakis/actions/runs/37775230032) builds exact baseline and repair sources under one dependency resolution, checks complete JSON output equivalence, interleaves seven native samples per binary and measures F19 instructions. No optimization acceptance or speedup is claimed until that comparison and current-head PR checks are reviewed.
