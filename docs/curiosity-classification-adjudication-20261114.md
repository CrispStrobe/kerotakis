# Curiosity classification adjudication — 2026-11-14

These are coverage provenance corrections, not new experiment qualifications. Prompt scripts,
questions and required grades remain unchanged. The original
[baseline at `378761da7`](https://github.com/CrispStrobe/kerotakis/blob/378761da7c926ebf90de6829bb8afccf9e1f8960/tests/coverage/curiosity-v1/baseline.toml)
and the [first curiosity failures](../audits/curiosity-cbab-first-failure-20261113/README.md)
remain historical evidence. The maintained baseline records the reviewed successor behavior;
it does not turn an unsupported question into a scientific pass.

The hosted `5b1be7d35767282229d2b557697ac4127363233b` report contains the complete
per-step solver routes. Its JSON stdout SHA-256 is
`764fa19e1b6da791e1aa1ae4e9c28c96c2787819d7e59639978eafbe5e025981`.
The same producer's unchanged `mat-032` replay stdout SHA-256 is
`0df19b025e89dbed5a87d23cf518daa171e194f3b3625d8a2b30dd6960c3323e`.
These readbacks establish the classification causes below. They do not validate the
later classifier or cuprous-oxide successor: the hosted coverage unit run failed the
first PE classifier's expected Qualitative/actual Curated assertion, and copper still
failed its energy solve on that producer. Exact-successor hosted validation is required.

## Polyethylene: an observed melt and an unsupported burn

The unchanged `mat-032` script adds 2 g PE, heats by 2 kJ, then attempts ignition.
The replay reports 0.07129108148570615 mol changing from solid to liquid at 414.6 K;
2 kJ is delivered, with 1606.9112 J assigned to sensible heating. The remaining
393.0888 J pays the installed grade's fusion heat. The final heating temperature is
740.1136 K. The ignition step then emits `not-modeled.combustion-no-solver` and no
combustion result. Melting is observed; burning and its products are not established.

The first classifier retained the melt but let a two-event, **nonchemical** Curated
`phase-routes` success promote the partial observation. The successor distinguishes
physical provenance from successful chemistry. An explicit combustion refusal plus
a positive finite phase-change amount for the same directly added substance supplies
only a partial qualitative observation. Only O2/N2 additions in the same vessel may
accompany that substance; another reagent, material recipe, solution, vessel, unrelated
transition, or action family does not earn this exception. Successful computed or
curated chemistry retains its existing treatment. Ordinary melting without a
combustion refusal retains its existing computed/curated treatment.

Both PE cases retain the explicit reason
`partial-phase-observation-combustion-unmodeled`. The JSON coverage report additionally
retains actual `NotYetModeled` events in `limitations`, beside the complete route list.
This is partial evidence of melting, not a claim that the melting-versus-burning
comparison or polymer combustion has been modeled. Tests inspect the original PE
replay's phase event, combustion refusal, absence of released combustion energy, and
serialized limitation. Unrelated solvent and invalid-amount controls remain in place.

| Prompt | Previous baseline | Reviewed successor | Reason |
| --- | --- | --- | --- |
| `mat-032` | Qualitative / typed-observation | Qualitative / partial-phase-observation-combustion-unmodeled | Keeps the actual paid melt and makes the unsupported combustion explicit. |
| `th-061` | Curated / curated-route | Qualitative / partial-phase-observation-combustion-unmodeled | Its hotter PE script adds oxygen and observes afterwards, but has no successful combustion route. Physical heating and melting do not earn a quantitative burn claim. |

## Mixed carbonate provenance

All ten newly changed carbonate cases below have both a committed Curated
`carbonate-endpoint` contribution and successful Computed `phase-coupled` chemistry.
The endpoint route is created only by its typed commit notice paired with positive
same-step, same-vessel CO2 evolution. A warning alone, another gas, another vessel,
a later step, a nonpositive amount or an unrelated limitation cannot create it.

The conservative summary is Curated / `curated-route`, because part of the answer
uses that bounded endpoint. Every Computed route remains in the result. This summary
neither makes all contributions curated nor claims a fully calculated gas yield.
The typed bounded-endpoint limitation remains in the transcript and now also in JSON
`limitations`. The pressure-gauge routes in `th-099`/`th-100` likewise remain Computed.

| Prompts | Previous baseline | Reviewed successor |
| --- | --- | --- |
| `aq-034`, `aq-060` | Computed / computed-route | Curated / curated-route |
| `bio-004`, `bio-083`, `bio-084` | Computed / computed-route | Curated / curated-route |
| `mat-052`, `mat-076`, `mat-124` | Computed / computed-route | Curated / curated-route |
| `th-099`, `th-100` | Computed / computed-route | Curated / curated-route |

Tests check both contributions in all ten cases and in the already-Curated controls
`aq-059`, `aq-122`, `bio-114`. The original three controls keep their baseline records.
Spectator-warning controls retain their independent Computed answers.

## Thermal route and silica corrections in the `5b1be7d3` readback

| Prompts | Previous baseline | Reviewed successor | Scope of the evidence |
| --- | --- | --- | --- |
| `mat-004`, `mat-013`, `th-066` | Curated / curated-route | Computed / computed-route | The actual successful owner is `cea-thermal`, with chemistry=true and zero emitted result events. This records the solver that evaluated the thermal state; it is not an independently validated oxidation yield, blackening rate or alloy model. |
| `mat-049`, `mat-092` | Solver failure / solver-failed-cea-thermal | Computed / computed-route | The corrected in-range silica phase is accepted by CEA. The duplicate clay-heating scripts retain their resolved SiO2 and their unresolved clay portion. No pottery firing, sintering or clay chemistry is established. |
| `mat-054` | Solver failure / solver-failed-cea-thermal | Missing / not-yet-modeled | Heating reaches only the 1773.15 K burner ceiling, below the supported silica melt. The script then requests 100 kJ cooling, exceeds its available sensible energy and receives the absolute-zero cooling limit. No melt or glass crystallization is demonstrated. |

The zero-event thermal chemistry route is retained explicitly, rather than invented
from visual output or a successful process exit. Its provenance category cannot itself
prove that the scientific question was answered. The preserved silica/clay replays make
that distinction visible.

`th-042` keeps its original Computed / `computed-route` baseline. Its first unclosed
copper energy failure is not accepted by refreshing that baseline. The sourced Cu2O
successor requires its own element/enthalpy and complete hosted acceptance before
that original record can be considered met. No energy tolerance or original required
grade is changed here.

## Exact `3f2e44928` readback: polypropylene and heated copper

The separate hosted release-binary producer built
`3f2e4492828cd8ddd17f62dd34f342750081468c`. Its complete curiosity JSON stdout
SHA-256 is `2940fc254f1f3e6322fa96093eb64e1bf3b76cb75660c2eeac9c12c1506967fc`.
All 500 prompts executed with zero solver/runtime failures and zero unmet required
grades. Two baseline drifts remained; neither is accepted without the following
route and source review. This producer does not establish complete workspace or
fleet acceptance of this source or the later baseline/test successor.

| Prompt | Baseline before this readback | Reviewed correction | Actual source-bound evidence |
| --- | --- | --- | --- |
| `th-062` | Curated / curated-route | Qualitative / partial-phase-observation-combustion-unmodeled | The PP/O2 ignition script has a two-event, nonchemical Curated phase route and an explicit combustion-no-solver limitation. It has no successful combustion chemistry route. Its question about combustion products remains unsupported. |
| `th-066` | Computed / computed-route | Curated / curated-route | The sourced Cu2O successor now emits both a two-event nonchemical Curated phase contribution and a four-event Computed CEA chemistry contribution. The conservative summary retains both routes and acknowledges its curated contribution. |

Unlike the PE heating scripts, `th-062` applies no separate heat dose. Its phase
evidence arises during the ignition-zone exposure; withdrawing the unsupported
spark restores the earlier temperature and refreezes its transient melt. The
qualitative category records that partial physical observation, not persistent
melting, polypropylene combustion or its products. The combustion limitation remains
serialized. Its original script, lack of a required grade, and scientific limitations
are unchanged. A dedicated original-script control checks PP retention, restored
solid phase/temperature, no CO2/water inventory or emitted combustion products, no
released combustion energy, the phase observation and the serialized refusal.

For `th-066`, the earlier `5b1be7d3` zero-event CEA observation above is preserved as
history. It must not stand in for the new producer, whose reachable Cu2O chemistry
changes the actual thermal path and its phase-route contributions. A dedicated
classification control requires **both** successful positive-event routes. The
Curated category does not independently validate a blackening rate or imply every
contribution is curated. Copper energy/element acceptance belongs to the separate
CEA controls; the original `th-042` Computed baseline remains unchanged.

The two baseline corrections change no prompt script, forecast, required grade,
energy tolerance, chemistry route or fleet assertion. Their new controls and the
complete curiosity gate require hosted execution on the successor.
