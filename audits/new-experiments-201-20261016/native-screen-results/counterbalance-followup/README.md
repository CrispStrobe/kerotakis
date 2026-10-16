# CO-005 counterbalance follow-up

One separately frozen production request completed HTTP 200/exit 0 and reached the
intended centrifuge. Counterbalance **39.91 g** matches the actual sample
39.91055040329605 g within the unchanged 0.10 g admission bound. The original
native failure at 100 g remains preserved.

[Design](design.json) binds original, corrected native and API-adapted scripts
separately. The only physical change is the counterbalance. API transport removes
`register lv3` and spells `look v3` as its canonical equivalent `measure v3 eyes`;
all other commands, quantities, times and original forecast remain unchanged.
Fresh health matched production c446/binary `670bd3d8` before this single POST.
The producer lock remains qualified as reported, without raw build-lock rehashing.

The [raw response](counterbalance.response.json),
[execution](counterbalance.execution.json) and [assessment](assessment.json) show:

- The centrifuge reports octacalcium-phosphate particles settling, using an
  assumed 100 µm particle size. Their recorded suspended fraction changes from
  one to zero; no matter is removed/exported.
- Betanin remains 0.000001 mol. The unresolved milk pool is identical before and
  after. Resolved elemental/charge residuals are below 1e-8 mol.
- Pigment-only absorbance is 1.56001 at 525 nm; the mixture reports 1.61133 at
  525 nm before and after centrifugation (change −8.62e-12). The milk blank
  reports zero at 405 nm. These different-wavelength peak readings do not
  establish total scattering or justify ordinary blank subtraction.
- The final appearance remains purple/cloudy with a phosphate deposit.
  Analytical-charge and excluded supersaturated-phase limits remain explicit.

The procedure correction is demonstrated. **Milk clarification and quantitative
scattering removal are not validated**, and no physical/catalogue pass is assigned.
The original forecast's caution is preserved; it was not replaced by an easier
conditional forecast. Source's instrument reports peak Beer-Lambert absorbance;
the unresolved dairy material is not a measured turbidity-removal channel.

The [saved reader](review_saved.py) reconstructs these checks without requests;
use a new `--report` destination. There were no retries, source/model changes,
local builds, extra experiments or deployment changes. Authorization remained in
a private temporary mode-0600 config, was scanned out of returned evidence, and
was removed afterward.
