# First-failure source binding clarification

This addendum preserves the first failures recorded in `FORECAST.md` and corrects their source attribution before any diagnostic execution.

The captured Ubuntu and macOS native CI logs identify the tested checkout as merge commit `37bb699`, with the log line `Merge 6e44feab03a646b412c123b21b275e3b95942931 into ccf30b2aeac4042ffe79942d4c8746ec10a553ef`. Thus the failures named in the forecast were observed on that merge commit, not on the exact `6e44feab` commit alone. The available CI trace does not establish that all other source in the merge is identical to `6e44feab`; no such equivalence is claimed here.

The diagnostic source is intentionally based on the exact `6e44feab` commit and adds only observational logging. Its later bounded run of the two existing tests records actual outcomes, whether or not either failure reproduces. The CI failure values remain historical observations on `37bb699`; they are not predicted outcomes or acceptance predicates for the exact-commit diagnostic.
