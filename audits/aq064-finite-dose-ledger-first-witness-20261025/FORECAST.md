# Finite-dose uptake is not a second material inlet

The unchanged aq-064 protocol actually completes without SolverFailed on headde8, tested merge6139f978 with the same tree. [Exact raw output](aq-064.stdout), [native/selector trace](aq-064.stderr), [script](aq-064.kero) and [bindings](BINDING.json) are frozen here. Exit0 is not a full physical verdict or a new original qualification.

The CO2 addition delivers0.01mol. That same operation reports0.003394783903533mol absorbed into the liquid and0.006605216096467mol evolved/vented. The remaining bulk carbon is0.003394783903533064mol. Dose minus outflow closes carbon within6.4e−17mol. The source poses an explicit gas reagent as `ExternalGasKind::Dose`, removes the transient gas portion, and reports its dissolved share and vented remainder. Its absorbed share has already been counted in the original reagent addition.

## Prospective native-test correction

The new endpoint test incorrectly adds both the full reagent dose and its absorbed share as independent material inlets. It would expect0.006789567807066mol carbon after that operation. This defect is independently reproduced from actual saved output; no native test failure is claimed before its execution.

Correct only the new test ledger. Recognize the matching finite CO2 Add that is already included in the inlet inventory; require its total absorbed share to be finite, nonnegative and at most the original dose. Omit only that duplicate inlet. Still debit every real gas outflow; inward gas on other operations remains an inlet. Retain exact original/nearby commands, full H/O/C/charge budgets1e−10, no-SolverFailed and zero-time stock/temperature controls. Do not change chemistry or raise a residual bound. Validate the actual corrected test and full current source; keep this witness immutable.

No qualified count is increased:338/468. Normal integration/product assessment, further unchanged-original review and runtime diagnostic i18n remain required before closing the ordered task.
