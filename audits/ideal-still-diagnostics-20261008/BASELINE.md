# Additional-solvent operator baseline

This baseline retains pre-repair production behavior from `73d231b3` and the exact forecast test at `a5377136`. Only a private test-module hook exists in the thermo source. The targeted core integration test builds thermo as a dependency without its private test module, so the proposed checked API is not required to compile this operator baseline.

The new checked-kernel API contracts remain separately frozen and unexecuted until implementation; a missing API symbol is not a behavioral failure. The operator test preserves complete state and checks distinct reasons; a failed table iteration does not establish outcomes for later entries. The success control must remain supported.

The dispatch-only workflow is confined to this baseline branch and must never merge. It binds tracked manifests and source/test bytes before testing and captures the generated dependency lock and raw result afterward, including failure. No outcome is inferred from dispatch.
