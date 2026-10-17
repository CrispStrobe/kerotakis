# Finite-solid inventory repair

## Demonstrated failure and bounded repair

A water/ferrous-hydroxide protocol (`add v1 water 100mL`,
`add v1 Fe(OH)2 0.01mol`, followed by `inspect v1`, `explain v1` and
successive `wait 1s`/inspection steps) on the recorded c446 engine alternates
between zero solid and approximately 0.009999 mol solid. Reported pH alternates
between approximately 9.333 and 10.783 at the same reported temperature.
The original first diagnostic had no live solid and was inconclusive for the
positive-solid comparison forecast; the separate wait follow-up supplied that
precondition. Neither run establishes empirical solubility accuracy.

The direct input builder frees positive non-native anhydrous phases into solution
before consulting reviewed alternate/foreign phase admission. A subsequent solve
can admit the same phase as a zero-amount precipitation candidate. Independently,
the comparison builder previously removed every non-native phase, including
positive supplied inventory, while describing the result as the same question.

The candidate admits already-reviewed alternate/foreign positive phases before
that dissolution fallback. Comparisons preserve admitted phase amounts and
saturation targets, and explicitly decline datasets that cannot represent a
positive supplied phase. Unavailable zero-amount precipitation candidates remain
optional. Coupled-redox and typed-surface solves retain their existing native-only
admission domain. No phase constants, polymorph equivalence rules or thermodynamic
sources are added by this repair.

## Acceptance still required

The candidate is unaccepted until exact-head native CI and the full own fleet
pass. The native regression uses the actual engine and repeats unchanged finite
inventory equilibrations, checking retained solid, represented iron, canonical
solvent, analytical charge baseline, enthalpy, temperature and pH, then compares
live and same-dataset phase/pH observations. Helper controls distinguish trace
positive inventory from zero candidates and retain the coupled/surface
exclusions. German and French CLI diagnostics explain refused comparisons.
These checks establish regression consistency, not complete empirical validation
or a general H/O/charge closure proof.

After source/binary/lock binding, replay the unchanged diagnostic separately on
the repaired engine and retain the original responses and first test failures.
Do not rerun the whole unchanged corpus or increment the working catalog from
these diagnostic controls.

## Remaining implementation lanes

1. **Unsupported positive solids:** inventory the remaining analytical-dissolution
   fallback across routes and consumers, then define typed domain limits without
   turning absent mineral data into unrestricted physical dissolution. Preserve
   compatibility with existing coupled and interface ledgers.
2. **Phase identity and ledgers:** test alternate-name collisions and hydrated
   phases, and verify complete element, water, charge and energy accounting across
   reconstruction, transfer, MIX and cache replay. Refuse domains whose extra
   inventory pools cannot be represented.
3. **Finite nitrate solubility:** follow the [KNO3 lane](kno3-finite-solubility-lane.md)
   for source admission, temperature dependence and concentrated activity models.
   Reusing an existing hydroxide definition does not repair nitrate solubility.
4. **Deployment and observations:** deliver the accepted full frontend/native
   snapshot, verify its identity and repeat only changed-source controls. Keep
   CLI/API completion separate from physical success.
