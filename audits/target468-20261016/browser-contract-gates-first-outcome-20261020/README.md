# Browser contract remaining gates: completed stages and first Clippy stop

The source-bound remaining-gates run on `6b637e4d7a00847865990b259171a8f4c6c11c1e`, tree `48a68818a1aadabf54caa6eb4b5d815331f0c8de`, **passed canonical formatting, all 39 transaction controls, all 18 composed-locale controls, the locale check and Wasm-target compilation**. It then stopped at Clippy. This is acceptance of those completed stages only; the whole focused run, full source, browser deployment and original experiments remain unaccepted by this evidence.

The [completed-stages receipt](stages-accepted.json), [unchanged raw files](raw/browser-contract-gates-evidence/), [collection receipt](collected.json), [producer](producer.py) and [publication map](publication.json) retain the actual first results. The exact dependency lock is `668868136fa9ee1ebd6433d59eb68d5fa46224d874fd96c4036f7614b3c4cd85`. Both prerequisite acceptance receipts remain byte-bound: the earlier phase-transfer full acceptance and the preceding [nine baseline failures/three positives/twelve repaired controls](../browser-contract-controls-outcome-20261020/README.md). This remaining-gates job reran neither baseline nor those twelve controls.

The transaction result contains exactly one borrowed-policy control and all 38 existing transaction controls, without ignored tests. The composed-locale result contains all 18 named controls, including the expected-panic control. The static locale result concerns reachable engine catalogues and does not prove that every application string is translated.

## The actual stop and source correction

Clippy's sole originating error was the unused private constructor `PathResult::new` at `crates/kerotakis-phreeqc/src/aqueous.rs:412`, with the native engine feature disabled. The final compiler error reports that one earlier error; there is no reported runtime test failure in this stage. Preserve this nonzero result rather than treating the completed checks as whole-run success.

Independently reviewed successor commit `101ea2787048787d7b1dc021f3098317c9314ec4` adds only `#[cfg(feature = "engine")]` to that private constructor. All four callers are within the already engine-gated `compare_paths` method. Public `PathResult` data and rendering remain available without the engine. The correction aligns conditional compilation with actual ownership and does not broadly suppress warnings or change chemical evaluation. These stopped-run records remain bound to **6b637e4d**, not the successor. The changed source needs its own Clippy and applicable fixed-source acceptance; this publication claims none of those future results.

## Reproduce the bounded review

Run the [portable partial reader](review_completed_stages.py) against this complete public directory:

```sh
python3 audits/target468-20261016/browser-contract-gates-first-outcome-20261020/review_completed_stages.py \
  --run-dir audits/target468-20261016/browser-contract-gates-first-outcome-20261020
```

It authenticates every collected file and original raw manifest, exact producer bytes, source/tree/lock/fixture binding, both prerequisite receipts, actual commands and exit codes, every named test and count, and the exact Clippy stop. It also accepts the original collected-bundle directory through the same argument. It prints a bounded receipt and makes no provider request or engine execution.

**No test executable archives were produced by this job.** The producer originally retained executables after all gates, and stopped before that retention step. There is no executable manifest or compressed test binary to independently rehash here. The completed test-stage evidence consists of authenticated source-bound raw output; it must not be described as independently rehashed 39-test executable evidence. The separately completed twelve-control job retains its own actual executable archives and acceptance under its own source.

This publication includes all 53 collected small files (206,457 bytes), omits no produced executable archive, and changes no original raw bytes. No experiment is scientifically selected by these host-contract checks.
