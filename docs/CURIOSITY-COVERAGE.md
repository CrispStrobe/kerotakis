# Curiosity coverage

The corpus contains 500 prompts. Its `expected` value is a minimum evidence
grade, not an exact category: computed and curated results share the quantified
top grade, while boundary refusals compare only with boundary requirements.

## Current audited result

Against the native solver stack on 2026-09-13:

| disposition | prompts |
|---|---:|
| computed | 320 |
| curated | 64 |
| qualitative | 55 |
| boundary | 60 |
| missing | 1 |

All declared requirements are met. There are no solver/runtime failures. The
single missing row has no asserted capability requirement.

The baseline is an audited observation record. Update only rows explained by a
reviewed engine, provenance, corpus, or expectation change; never regenerate it
blindly to make CI green.

Eight rows moved `computed -> curated` on 2026-09-13 — `th-030 th-048 th-051
th-058 th-059 bio-008 bio-009 bio-044` — and no chemistry changed for any of
them. `curated-combustion` had never declared a `route_kind` and so took the
trait default, `Computed`, while running entirely on the curated `FUELS` and
`GAS_AUTOIGNITION` tables; it now declares `Curated`, which is what the
provenance rule settled on 2026-09-11 always said it was. Those eight are the
only rows in the five hundred with a succeeded `curated-combustion` route, and
none of them had another succeeded curated route that was already counting
them, so the move is exactly eight rows wide. The totals above are the only
other thing it changed.

## Route attribution

An observation does not erase a stronger model result beside it. Classification
therefore considers successful routes before the generic `handle_and_inspect`
observation shortcut. Smell, flame, and unsupported observation paths remain
qualitative unless their own operation records stronger provenance.

Direct model-backed operations record routes only after producing a valid
result:

- classical gas tests use their curated threshold route;
- pressure-gauge readings use the computed headspace route;
- named organic `react` operations use their curated reaction route;
- the starch–iodine optical result uses its curated appearance route.

A refusal records no successful route. Electrochemical `Inert` events carry a
machine-readable computed flag at their source; observational insolubility does
not. This avoids inferring authority from English prose or audit IDs.

## Corpus repairs

The ammonia gas-test script owns a finite sealed headspace, allowing the generic
Henry-law pass to partition dissolved ammonia before damp litmus reads it. The
starch test includes iodide, which is required to solubilize iodine and form the
diagnostic polyiodide complex. The carbon-dioxide and hydrogen venting scripts
measure pressure before and after opening. The empty-vessel ignition row expects
a qualitative no-fuel observation rather than overclaiming a computation.

Comparative magnesium scripts contain both control and treatment vessels. They
expose, rather than close, the remaining limitation: acid–metal displacement is
an instantaneous equilibrium in the current engine, so surface-area and elapsed-
time effects need reviewed kinetic parameters before Kerotakis can quantify
them.

## Reproduction

```sh
cargo run -p kerotakis-cli -- coverage curiosity --check
```

The chemistry-audit workflow separately runs frozen process-isolated fleets and
retains raw stdout, stderr, scripts, final benches, hashes, and law-check reports.
