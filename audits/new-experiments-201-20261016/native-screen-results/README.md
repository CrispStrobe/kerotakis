# Native 201 saved-output review

The authenticated hosted screen completed 201 single attempts: **198 zero exits,
three nonzero exits, no JSON parsing errors and no retries**. This is an engineering
result, not 198 physical passes. The scripts comprise 197 unchanged originals and
four separately corrected explicit haloalkane requests.

[Assessment](assessment.json) and [all case records](cases/) preserve original
physical forecasts/checks, actual operator sequences, final available vessel states,
all measurement/process observations and contextual model-limit/solver events.
The independent reader rehashed every script/stdout/stderr and checked every
per-case execution receipt. Source c446, binary `670bd3d8`, input checkout and
producer-lock qualifications remain attached. Root publishes the authenticated raw
archive/completion separately; these derived records do not replace raw evidence.

| Saved-output category | Cases |
| --- | ---: |
| Completed, physical assessment still required | 111 |
| Completed with named model limits | 85 |
| Completed with solver failure | 2 |
| Nonzero process admission refusal | 3 |

These categories describe output, not universal chemistry support. There are 637
named-limit event occurrences across the batch. Repeated event counts are not
independent defects or case verdicts. Zero physical passes are assigned here.

## Prioritized findings

1. **AB-029:** filtration transfers the finite aqueous fraction, then PHREEQC's
   coupled temperature solve fails after 64 passes (residual 0.003402842 K). The
   initial filtrate pH is honestly withheld; later acid addition obtains a fresh
   pH. Combined H/O/Mg/charge closes across filtering, but that does not establish
   a converged thermal answer. See
   [MODEL-008](../../reaction-dataset-200-20261015/model-issues-20261016/MODEL-008.json).
2. **D3-AN-001:** NaBr ignition reports CEA nonconvergence and no fuel, with the
   whole bench restored. NaCl/KCl provide flame-test colours; NaBr's registry
   flame-colour field is absent. Separate hot-equilibrium admission from missing
   optical-property coverage. Do not call the sodium-colour forecast a pass. See
   [MODEL-009](../../reaction-dataset-200-20261015/model-issues-20261016/MODEL-009.json).
3. **D3-CO-005:** the supplied 100 g counterbalance is 60.09 g away from the
   actual 39.91 g simulated sample. Source admits at most 0.10 g imbalance.
   [The separate candidate](counterbalance-candidate.json) changes only the
   counterbalance to 39.91 g, retaining the original optical forecast.
4. **D3-TR-003 / TR-007:** uniform-volume admission refuses the first transport.
   Carbonate aqueous bookkeeping shifts the proxy by 1.57096 ppm, beyond the
   1 ppm gate; gypsum releases stoichiometric hydrate water, changing the first
   cell by about 564 ppm. No transport balance was completed, so no conservation
   defect is demonstrated. See
   [MODEL-007](../../reaction-dataset-200-20261015/model-issues-20261016/MODEL-007.json).
   Investigate hydraulic-domain ownership; do not relax tolerance, delete hydrate
   water or remove the intended chemistry to make these scripts pass.
5. **D3-SO-001 / SO-003:** native records preserve the strong-base selectivity gap
   already in [MODEL-006](../../reaction-dataset-200-20261015/model-issues-20261016/MODEL-006.json).

[Exact before/after diagnostics](diagnostics/) and
[ledger checks](diagnostic-ledger-checks.json) qualify the claims. Equal
before/after temperatures or excess-enthalpy fields alone are not a full energy
proof. Full-batch element/energy acceptance remains case-specific, especially for
unresolved materials, surfaces, gas exchange and transport reservoirs.

## Reproduce the saved review

Run from a checkout with the original prospective design and an extracted copy of
the authenticated artifact. No simulator process is started:

```sh
python audits/new-experiments-201-20261016/native-screen-results/review_saved.py \
  --artifact EXTRACTED_ARTIFACT_DIRECTORY \
  --frozen-design audits/new-experiments-201-20261016/native-screen-design.json \
  --out NEW_REVIEW_DIRECTORY
```

Use a fresh output directory: the reader refuses overwrites. The runner
reserialized its artifact copy of the design. The reader therefore verifies the
original frozen byte hash **and** equality of the artifact copy's complete JSON
values; both byte hashes are retained separately. Public diagnostic displays
redact hosted script locators while preserving the raw stderr hash and original
bytes in the archive.

Follow [the review queue](review-queue.json). Model intakes remain separate from
protocol corrections and are not source repairs. No blind reruns, local builds,
model edits, catalogue/status changes or commits were performed by this review.

A separately authorized [single counterbalance follow-up](counterbalance-followup/README.md)
now reaches the intended centrifuge on the same production source. It preserves
pigment and unresolved dairy pools and reports settling phosphate particles;
it does not validate milk-scattering removal. No original native result changed.


Summed serial CLI process wall time was47.94s, with median0.118s and maximum8.59s
in this hosted run. These figures exclude CI queue/artifact setup and do not
measure VPS or API performance. They establish no chemical calibration.

The [separate QA-001–003 saved assessment](quantitative-saved-review/README.md)
checks original quantitative criteria without rerunning cases. Limestone and
acid-spike checks are met; the aliquot dose ratio is met but its requested
liquid-volume measurement remains unsupported by the selected gas instrument.
Original screen classifications and the working catalog remain unchanged.

A [separate QA-003 scene-volume supplement](quantitative-saved-review/scene-volume-followup.md)
checks liquid volumes already present in the raw measurement-step scene. They
support the concentration forecast; the original gas-meter refusals still stand.
The first reader and assessment omitted those scene values and remain preserved.

The [QA-011 endpoint-dose assessment](endpoint-dose-review/README.md) verifies
that the refined final increment is physically dispensed. It adds one unchanged,
source-qualified CLI protocol to the working catalog (now150); the previous149
entries and original200 outcomes remain unchanged. No experiment was rerun.
