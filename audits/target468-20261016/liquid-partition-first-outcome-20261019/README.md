# Liquid transfer first outcome: upstream phase detection still panics

Tests-only baseline `7e18b221` genuinely fails three assertions and passes its ordinary-drain control. Candidate `5cdf8a6b` passes extraction’s typed refusal and all five earlier app controls, but two liquid tests still panic: drain enters `layered_pair` and the unchecked UNIFAC phase-stability calculation before reaching its new transfer handler. Formatter and Clippy passed; the actual failing test output is retained. This is a source defect, not a scientific pass or accepted implementation.

The source successor adds fallible phase stability through every original scan/bisection evaluation, checked layer detection and operational callers, plus direct refusal/first-failed-evaluation controls. No coefficient or phase result is fabricated on error. The combined source on delivered main must pass full thermo/core/native/locale/all33 product and protected-platform acceptance before integration. Original fixture assertions remain unchanged.

The first observer’s metadata incorrectly says version2; the actual issued provider version is1, bound by the push response and producer source/worker bytes. This presentation error is recorded without rewriting the first metadata or outcomes. No provider or original experiment is repeated by this saved publication.
