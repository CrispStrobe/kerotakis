# Isolated live REST-client browser verification

This validation branch must never merge. Its dispatch-only Vercel workflow is
replaced on this ref with browser verification; it cannot deploy anything.
The default-main deployment workflow is unchanged.

Before dispatch, bind the accepted deployed source, native hash and exact client
bytes in `release-binding.json`. The harness refuses a missing binding. Record
the resulting harness commit and workflow run separately from the engine source.

Actual Chromium checks the served Wasm revision and an uncached aqueous solve
through real deployed Rust/Emscripten modules. Compare that solution with the
separately saved authenticated native response using the repository bridge
tolerances. This establishes bounded cross-platform consistency, not empirical
calibration or a new catalog selection. Bind all five actual deployed bridge
asset hashes to the preserved frontend archive before dispatch.

It also checks the production and Pages clients, live invalid-key
401/CORS behavior, request wiring, text rendering, endpoint validation, redirects,
offline behavior and privacy. UI response fixtures are explicitly labelled and
are not chemical execution or scientific evidence. No owner credentials enter
the runner. Real authenticated native controls have separate deployment evidence.
The unapproved-origin document is controlled test content, not publication.

Preserve failed setup, assertions and reports. Do not weaken checks, fabricate
observations or rerun native experiments to repair a browser harness assertion.
Download the actual report/screenshots and independently review them before
claiming acceptance. Browser success is not chemistry calibration.
