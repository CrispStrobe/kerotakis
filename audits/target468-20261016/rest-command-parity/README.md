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

## Native preview controls

[Two bound native command controls](preview-followup-results.json) completed on
735c523d through its directly deployed Vercel preview. They cover the seven new
verbs, finite stock, particle/voltmeter aliases and vessel lifecycle, plus four
live over-budget/session refusals. These are exposure controls, not empirical
chemistry validation.

The [first lifecycle forecast](preview-first-protocol-failure.json) failed
because the protocol attempted to remove the final vessel, which the CLI
correctly refuses. Its `gateway_revision` is the actual735c523d binding; the
misnamed `api_revision` field in that first receipt contains the engine identity.
The separate [corrected protocol](preview-followup-design.json) adds another
vessel before removal. The guard, original script and first outcome remain.
The voltmeter control was unsubmitted at that first failure and then ran once.

[PR816](https://github.com/CrispStrobe/kerotakis/pull/816) at828c09b9 reports native
signals before parsing partial JSON and wires engine grammar/identity changes
to the REST inventory checks. The [independent review](independent-review.md)
retains exact-source conclusions. Full applicable CI and own-fleet acceptance
still gate integration; no branch protection or test is waived.
