# Transparent blue/yellow teaching wash revision 2

This candidate changes only the blue and yellow named watercolor surrogates to
0.002% dye by mass, a tenfold dilution of revision 1. Spectra, vessel optical
paths, food-color dropper recipes and red watercolor are unchanged. The strength
is an editorial teaching choice, not a measured retail paint formulation.

Before native execution, the controls declare these expectations for an empty
beaker followed by 100 g blue and 100 g yellow wash:

- Current mixture: green channel above 80 and at least 40 above red and blue.
- Archived revision-1 mixture: maximum channel below 40, preserving the dense,
  nearly black baseline rather than reinterpreting historical experiments.
- Blue primary remains blue; yellow primary remains yellow at the declared dose.
- Addition order and a subsequent one-second wait preserve the mixture colour.
- Dilution increases transmission while retaining dye; optical path still scales
  absorbance. No RGB clamping or fixed colour-blending rule is introduced.
- New input pins revision 2; historical parsing pins revision 1. Old and new
  finite bottles debit independently and preserve their units and compositions.

`crates/kerotakis-core/tests/watercolour_revision.rs` implements seven native
control groups. These are predictions and prepared controls; no native outcome
is claimed until actual source-bound execution. Promotion requires the archived
recipe/session/stock prerequisite and native, app, protocol and browser acceptance.
Standalone unversioned script imports select current defaults unless explicitly
read through historical parsing. Arbitrary real blue and yellow dyes need not
produce the same colour as these declared teaching washes.
