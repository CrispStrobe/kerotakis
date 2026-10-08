# Explicit safety-veto repair status

[Twenty source-informed contracts](FORECASTS.md) were committed before production edits. [The freeze receipt](freeze.json) binds their unchanged test bytes; [the prepared repair receipt](prepared-repair.json) binds the main-based source and minimal repair. Hosted baseline and repair execution are pending. Formatting and diff checks pass; this is not an accepted runtime result.

The repair marks seven explicit refusal paths unchanged so the existing bench loop logs the refusal and exits before contact/characterization changes or solver settlement. Decant and Filter additionally remove only their newly auto-created receiver and its creation event on veto. Existing receivers remain untouched, and accepted creation/warning/success order is preserved. Missing Mix receivers retain their existing error. Accidental spill evidence continues with a hazard notice and actual material movement.

This branch is based on main's Amount-foundation checkpoint. It does not import the separate compensated-stock or binary-still repairs. No public event variants, operator fields or save schema change. It does not certify general invalid-input rollback, every unsupported-model branch, screen interior mutability, or whole solver-stack transaction atomicity.

## Executed unchanged-production baseline

[Run 37733252264](https://github.com/CrispStrobe/kerotakis/actions/runs/37733252264), source `530cf03e000bafa627c6d969053970136fbb586d`, compiled 20 tests: 11 passed and 9 failed. [baseline.json](baseline.json) preserves the source, log hash, lock and individual outcomes. 20 frozen controls:11pass9fail; observed downstream solver calls on explicit safety vetoes. Fresh receiver refusals stop at solver-call assertion before later graph assertions. Failing table loops do not exercise subsequent rows. Repaired-source validation remains pending.

## CI lint correction

CI rejected the boolean match predicate in the supported-event test before test execution. [Lint adaptation](lint-adaptation.json) preserves original bytes and replaces it with equivalent `matches!`; inputs and assertions remain unchanged. Original baseline execution above remains valid evidence. Repaired-source CI requires a fresh head.
