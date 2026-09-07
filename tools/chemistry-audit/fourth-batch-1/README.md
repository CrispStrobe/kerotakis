# Fourth fleet: 36 runs, 108 passing checks

All cases 123–158 exited successfully with no solver failures. All 108
predeclared scientific checks pass. The separate native inventory checker
passes 49/49 comparisons across 197 rows, with no internal-alias leaks.

See the sibling `fourth-batch-1-checks.json` and
`fourth-batch-1-native-inventory.json` reports. `PREEXECUTION.md` pins the
generator, analyzer, scope/tolerances and native executable hashes. No
expectations or tolerances were changed after outputs were observed.

These are tests of declared model behavior and conservation, not independent
experimental validation of every property or a guarantee beyond documented
model limits. Full combined CI and coordinated integration remain pending.
