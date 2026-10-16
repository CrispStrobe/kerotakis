# Complete English bench command admission

[PR815](https://github.com/CrispStrobe/kerotakis/pull/815) adds seven missing
verbs: remove, discard, stock, smell, waft, zoom and voltmeter. Its separate
[review](review.json) records19 lightweight API/admission-store/packaging test
groups passing, including exact inventory checks against all35 canonical bench
verbs and eight synonyms/spellings. Multiword ingredient aliases are checked
against the engine before gateway parameter-position accounting; submitted
scripts remain unchanged.

`GET /api/capabilities` reports the enforced command/limit policy. Session words
without JSON batch handlers refuse explicitly; admitting a no-op would not add
functionality. Filesystem operations and uncanonicalized language changes remain
excluded. Chemical support is not implied by admission.

The narrow branch starts from actual main and retains PR814 as its exact
prerequisite. Full CI, native sandbox acceptance, full own fleet and bound Vercel
controls gate readiness. CI38918403545, REST38918403439 and the single manually
dispatched full fleet38918540101 are source-bound to735c523d. Neither production
activation nor a new scientific pass is claimed.
