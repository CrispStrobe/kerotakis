# Authenticated CPU-provider native controls

The corrected chromatography source `8e2fe429` passes all8 core and3 native
controls, including actual dissolved glucose beside the retained ethanol peak.
Its original source `7b5f62d1` retains the first native setup failure.
Phase source `106dd5f3` passes Clippy but fails reverse cooling while partial
melting, above/below budgets and oxidation controls pass. The new correction
`92d439d4` has separate pending execution; these results cannot accept it.

Receipts bind actual source trees, compiler, worker hashes, commands, raw test
stdout and generated producer-lock bytes. Full stderr hashes and provider
manifest hashes preserve linkage to retained original logs. Native test results
are not CLI corpus outcomes; no catalog selections or empirical calibration are
claimed. Protected GitHub contexts and full own-fleet acceptance remain required.

`92d439d4` passes all80 CEA tests but its new core test stops on the
water-only table lookup; no physical assertion was weakened to correct that
setup. Exact corrected head `5b054e6c` passes Clippy,31 targeted core tests and
all80 CEA native tests:111 in total, with zero failures/ignored tests. Water
remains included through its actual solvent-state owner. Full own-fleet and
protected acceptance are still pending; these are test counts, not new corpus
selections.
