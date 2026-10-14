# Mixed-spelling CLI acceptance controls

Two source-informed controls compare canonical water inputs with mixed `H2O`
and `water` inputs. Both must debit one finite bottle, retain 0.5 mol after two
accepted additions, refuse the 0.75 mol overdraw without inventory mutation, and
preserve canonical stock/inventory identities through later observations.

[Expectations](expectations.json) and [protocol](protocol.json) were frozen before
execution at PR787 source `e4fb5e33`, bound to its verified own CLI artifact.
Current state: **0/2 executed**. A resource gate refused runtime preparation before
any case ran. Preserve that refusal and the original frozen inputs; resume only
after checking saved evidence and current gates. Runtime locations are private.

The [nine native alias regressions passed on macOS](../water-alias-native-validation.json).
These CLI controls remain separate from the original blind fifty and normalized
forty, and do not establish physics calibration or current integration acceptance.
