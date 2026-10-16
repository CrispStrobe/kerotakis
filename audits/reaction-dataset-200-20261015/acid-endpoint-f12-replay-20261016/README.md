# Unchanged original F12 on the bounded endpoint candidate

[The assessment](assessment.json) reviews the original unmodified acidic
permanganate/peroxide stock protocol on candidate `a03113a2`. Its script and
qualitative forecast were frozen in the original [design](original-design.json).
The authenticated [binding](binding.json) was written before the single bounded
CLI execution. Actual generated [Cargo.lock bytes](producer-Cargo.lock), raw
[stdout](stdout.jsonl), [stderr](stderr.txt) and [process receipt](process.json)
are retained. This was hosted execution of an existing build, with no VPS build
or CLI process. The validation-only harness ref must never merge.

The expected principal response appears: 2 mmol permanganate and 5 mmol peroxide
are consumed; 2 mmol Mn(II) and 5 mmol exported oxygen appear; 8 mmol water forms,
K/sulfate remain and the inventory is stable after ten seconds. Independent
material/charge residuals are recorded. Supersaturated manganese-oxide phases
remain explicitly outside the final phase model; heat/rate calibration is absent.

This is a source-bound follow-up, not a new blind experiment or full chemistry
pass. PR805 remains unmerged. Its prewarm probe separately failed because an
ordinary setup solve was repeated within the compared operator; the minimal
harness correction and fresh complete product acceptance remain pending. No
production/API, final-phase or current-main claim follows from this replay.
