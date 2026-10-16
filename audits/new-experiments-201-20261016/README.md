# Additional 201 experiment designs: source-only intake

The supplied batches contain **101 batch-2 and 100 batch-3 designs**. All remain
unexecuted. This intake preserves their original scripts and forecasts; it adds
four narrowly corrected haloalkane procedure candidates. No chemistry pass,
novelty certification, model repair or official-parser execution is claimed.

Read [the manifest](manifest.json), [all 201 records](intake.json),
[the priority queue](rewrite-queue.json) and [four proposed corrections](protocol-rewrites.json).
Exact supplied files are in [originals](originals/); individual exact scripts are
in [scripts](scripts/), with proposed revisions separately in [corrected](corrected/).
The supplied reports and README are historical author assertions preserved byte
for byte, rather than independently reproduced validation receipts.

## Scope and source binding

The review uses integrated source `c44692053a94c59dcf62e7fb67549f84c7d18f83`.
The manifest pins SHA-256 and Git blob IDs for the parser, CLI help, dispatcher,
input lookup, enzyme families and registry. The authors referenced older commit
`159fa6e08fa4ebea3105ac7e17d5b8350bc91189`; that original association remains intact.
No executable or binary/lock binding exists for this source-only intake.

Independent checks decoded both datasets, verified all 201 supplied script hashes,
unique IDs and unevaluated status, and checked vessel creation/reference order
and input identity against the reviewed source. There are **3,002 command lines**.
All declared input keys have a registry identity/material recipe or named dynamic
enzyme-family route. No alias or counterion substitutions are proposed.

The delivered JSON Schema, custom validator and standalone original script package
were not supplied. Their reported passes cannot be reproduced from these eight
files. JSON decoding and structural checks here are not full JSON Schema validation.
The reports explicitly say no official parser, engine or chemistry acceptance ran.
Their novelty checks do not establish semantic independence or an exhaustive
comparison with the upstream catalogue. README's generated total of 321 includes
an earlier 120-entry batch absent here; this intake contains only the supplied 201.

## Concrete protocol correction

Batch-3 SO-001, SO-002, SO-003 and SO-004 describe selectivity-table reactions but
never request them. In reviewed source, only `Operator::React` invokes
`selectivity::dispatch`; the exact verb is **`react v1 haloalkane`**. Addition and
observation alone cannot request this table. Each corrected script inserts the
appropriate call after all additions and before observations, for v1 alone in
SO-002 and both vessels in the other three. No identities, amounts, temperatures,
boundaries or physical forecasts change. Original/corrected hashes and verbatim
forecasts are recorded in `protocol-rewrites.json`.

These are explicit bounded table exercises. Actual solvent temperature,
nucleophile ownership, refusal messages, product inventories and material closure
still need acceptance. A successful call would not establish a timed reaction,
measured selectivity ratio or reaction heat.

Source references: [parser](../../crates/kerotakis-core/src/script.rs),
[selectivity table and verb](../../crates/kerotakis-core/src/selectivity.rs),
[React dispatcher](../../crates/kerotakis-core/src/bench.rs).

## Procedure and coverage review

The command families, instrument words, positive quantity spellings, explicit
concentrations, endpoint forms, counterbalance argument and transport form were
compared with parser/help source. No additional concrete syntax correction was
identified by this bounded review. This is not a substitute for official parsing.
Per-record flags distinguish questions needing acceptance design:

- 12 titration designs: preserve endpoint semantics, actual delivered increments,
  titrant solvent volume and max-step termination. The rounded-volume forecast
  in QA-011 is already conditional on interpolation; do not force a preferred result.
- 34 sealed/regulated designs: budget trapped gas and later exchange explicitly.
  Sealing is not evacuation. Pressure alone does not identify gas conversion.
- 24 energy-dose designs: `heat`/`cool` deliver energy; `@` sets an addition's
  temperature. Equal doses do not establish matched final temperatures.
- 39 multi-vessel waits and 17 stirring designs: `wait` advances all live vessels;
  `stir` advances its target vessel. Specify actual per-vessel histories before
  interpreting reaction rates, aging or matched controls.
- Eight transport designs: the inlet is repeatedly sampled composition, not a
  depleted finite bottle. Equal positive solvent volumes and adiabatic admission
  must survive composed chemistry; receiver contents accumulate effluent.
- Four BD designs intentionally probe model boundaries. In reviewed source,
  irradiation emits `photolysis_coupled=false`; no photochemical conversion is
  not a measurement of zero quantum yield. SO-005 preserves withdrawn numerical
  provenance, while SE-008 includes the separate known nitrate-solubility question.

Flags overlap and are review requirements, not a count of defective designs.
Original limitations often already state these issues; preserve those qualifications.
For peroxide cases, retain total closure including bulk gas and distinguish
`GasProduced` from external-transfer events. The existing
[MODEL-005 intake](../reaction-dataset-200-20261015/model-issues-20261016/MODEL-005.json)
asks whether persistent open-vessel gas is an intentional residence/accounting
contract. Do not assume missing mass or silently require export before that review.

## Next steps

Follow the ranked lanes in `rewrite-queue.json`: review the four proposed calls,
then freeze small source-bound acceptance batches with explicit ledgers/tolerances,
then composed transport and known coverage/provenance investigations. Each future
run needs its own exact source/binary/lock binding, original raw output and physical
assessment. Keep parser refusal, unsupported domain, inconclusive observable and
failed physical forecast distinct. No runtime or model work was performed here.
