# Native lesson observation stream admission

Before the43-control continuation, move only the three new lesson-test observation macros from `println!` to `eprintln!`. The frozen worker uses `--nocapture` and validates exact standard libtest stdout rows; test-authored stdout can interleave with those rows. Diagnostic JSON belongs in the separately retained stderr stream so the original strict result parser remains unchanged. All observation fields, commands, numeric assertions and physical predicates remain byte-for-byte identical after normalizing that macro prefix. This is a prospective test-harness admission adjustment, not an observed chemistry result or a numerical repair.

The actual9abc lesson test source was compiled but never executed because of the earlier list stop. Preserve its source hash1e79801a and the original stop; execute the continuation only after its new source/worker/lock/control bindings are frozen.
