# Independent syntax-only design

This design contains exactly 50 original, short experiments. Every expected observation was authored from the designer’s independent chemistry and physics knowledge before any execution. The designer has not run Kero, searched the web, read a registry inventory, consulted repository source or tests, read any existing experiment corpus or audits, or consulted prior outcomes. Ingredient identifiers are plausible guesses and may be unsupported. Unsupported ingredients or mechanisms must not be treated as evidence that the independent physical expectation is wrong.

The sole external language-information read was `../syntax-only.md`. The task instruction supplied the destination, fresh-vessel numbering, resource constraints, and the requirement to freeze the design. No other existing file was read. The only other reads used for freezing were the files newly authored in this design itself.

Each `scripts/NN.lab` is intended for its own fresh CLI process. Observation command numbers count executable commands, excluding comments and blank lines. `expectations.json` records the pre-execution prediction, rationale, confidence, observation points, and pass/fail/unsupported/inconclusive distinctions. A scientific result requires an observable identity, trend, quantitative comparison, or conservation bound; avoiding a crash is insufficient.

Defaults and command semantics not specified by the syntax reference are explicitly treated as assumptions. These include initial thermal state, equilibration, phase representations, headspace conventions, titration increments, and assay oxygen availability. Operator evaluation should record discrepancies with these assumptions rather than silently modifying expectations.

`freeze.json` records SHA-256 hashes of all 50 scripts, `expectations.json`, and this README. It was written after the full design was completed, before any execution. No authored file may be rewritten after this freeze. The freeze manifest itself is excluded from its own hash to avoid a circular definition.
