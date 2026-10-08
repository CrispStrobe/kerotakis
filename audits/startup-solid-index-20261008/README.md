# Startup solid index — reviewed acceptance

[PR #766](https://github.com/CrispStrobe/kerotakis/pull/766) merged as `b4455b7871889d0f1e1423390a3bc33a3ec9cdbd` after exact-head required checks and archived comparison review. It caches immutable compiled registry solid formulas, retaining first-match order, composition and hydrate matching. Native engine construction and failure behavior remain covered.

[Run 37775230032](https://github.com/CrispStrobe/kerotakis/actions/runs/37775230032), artifact `solid-index-comparison-37775230032`, compared baseline `c7f405bb` and repair `80ed1c36` under the same generated lock. The frozen comparison and public offline reviewer are on [the evidence branch](https://github.com/CrispStrobe/kerotakis/tree/0fc8d6d4/tools/startup-solid-index). [The reviewed receipt](review.json) binds source trees, executable hashes, raw records and timings. All 96 native processes across six workloads preserve complete parsed JSON output. Both sources pass two registry identity controls and ten derivation unit controls.

| Workload | Baseline median seconds | Repair median seconds | Observed reduction |
| --- | ---: | ---: | ---: |
| F19-a | 0.166144 | 0.120417 | 27.52% |
| F19-b | 0.166578 | 0.124139 | 25.48% |
| F39-a | 0.257904 | 0.216193 | 16.17% |
| F45-a | 0.232710 | 0.188873 | 18.84% |
| water-setup | 0.162952 | 0.117899 | 27.65% |
| neutral-salt | 0.165382 | 0.121219 | 26.70% |

F19a's separate Callgrind comparison drops from 1,380,526,585 to 850,152,111 instructions (38.42%). Native medians improved by 16.17–27.65% on this hosted runner. Seven interleaved samples per executable do not establish tail latency or gains on other machines. Whole CLI timing includes startup, engine initialization and serialization. Explicit manifest hash files were not collected; exact source/tree and clean-status records bind their contents.

Repair `80ed1c36` and merge `b4455b78` have the identical complete Git tree `cf3766ba38223c674de733f1615570b1c9498d86`. This bounded equivalence/performance evidence does not replace historical full-audit integration or certify later changes.
