# Diagnostic strict-lint follow-up

The bounded preflight on `ea3a9d4d` reaches a single remaining strict-lint error in the new crystal-amount refusal test: `err().expect()` must use `expect_err`. [Raw excerpt](CLIPPY-EXCERPT.log) and [source/producer bindings](FIRST-FAILURE.json) retain the result. All selected test commands remain unexecuted because lint runs first.

The successor changes only the assertion idiom. Rejected negative/nonfinite amounts, the specific amount-versus-solvent reason, raw detail and warnings-as-errors remain unchanged. The newly added assertion sites were checked for the same issue; existing sites whose success type lacks `Debug` retain their valid idiom. No pass or scientific qualification is inferred from this failure.
