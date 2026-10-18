# Kerotakis current state — 2026-10-18

Read the [fresh-agent handover](next-agent-handoff.md),
[ordered agenda](improvement-agenda-20261014.md) and
[PR delivery policy](PR-WORKFLOW.md). This maintained summary supersedes older
pending notes. Preserve original forecasts, first outcomes and exact source,
producer-lock and binary associations. Refresh live GitHub state before acting.

## Execution and scientifically qualified coverage

All **668 supplied records** have authenticated native baseline results:
557 zero exits and111 nonzero, without parsing errors or retries. Fixed468
is465/3 and the separate curriculum200 is92/108. Execution is not a science pass.
The first API audit made668 requests:555 completed,111 CLI refusals and two
infrastructure failures. All666 engine-bearing responses matched saved native
results; the two separately bound budget follow-ups completed. Do not rerun an
unchanged corpus.

The [latest working catalog](../audits/working-experiments-20261018-reviewed-gas.json) has
**204 source-qualified entries, including200 of fixed468**. The fixed468 target401
needs201 further earned selections. Earlier entries retain their own tested
bindings; this is not204 reruns on current production or empirical calibration.
The unchanged previous catalog remains preserved. All64 saved cases have
[reproducible preparation](../audits/target468-20261016/saved64-review-preparation/README.md):
Three original gas protocols now have [final criterion review](../audits/target468-20261016/saved-gas-final-review-20261018/README.md), authenticating all six original criteria without new simulator calls. The41 other older-source candidates still need final domain/criterion review. The20 original holds remain historical; KN-007 now has a separate
repaired-source selection. No other candidate was automatically promoted. The previous201-entry catalog is preserved unchanged.

The [four unchanged deployed-source diagnostic replays](../audits/target468-20261016/deployed-shared-diagnostics-20261018/README.md)
produce one selection and three continuing failures:

- **KN-007 passes:** catalyst remains0.001mol, filtering retains donor oxygen,
  whole H/O/Mn/charge closes, and the second peroxide charge decomposes with
  matched oxygen formation. This is a bounded teaching-kinetics result, without
  measured rate calibration or an assertion that the two rates are equal.
- **GS-010 fails:** brine creates approximately2.767e-7mol carbon and1.107e-6eq
  charge. The traced cause is ideal-pressure initialization into a native
  real-gas model. A distinct direct-solver control supplies0.001mol but receives
  0.0010003390707956424mol; exact native component moles close carbon to1.3e-17mol.
  PR833 has524 passing native tests on its first source; current combined-source
  acceptance remains pending.
  A plausible salinity trend does not excuse the original imbalance.
- **OR-006 fails:** the named hydrolysis request leaves ester unchanged and
  produces no acetate/ethanol. Analytical basis is not usable base capacity.
- **TH-016 fails on deployed source:** all0.7647598654mol zinc is liquid though
  the pulse pays only0.07395386004mol melting; source-bound phase energy exceeds
  the input by5056.699959J. The separately prepared repair is not deployed yet.

## Integrated source and actual production

Latest verified main is **`b3c18f5c`**, the normally protected PR829 merge after
25 CI successes, both reviewed native-platform logs and all33 original logical
fleet jobs (25 GitHub +8 bounded native). Its [parents/tree receipt](../audits/target468-20261016/accepted-pr-integration/pr829-integration.json)
verifies the actual tree against the reviewed candidate. PR830 integrated as
`0fe97e76` after25 CI and33 logical fleet jobs, with its separate retained
[receipt](../audits/target468-20261016/accepted-pr-integration/pr830-integration.json).
PR826 integrated as `ed7c452d`; its [receipt](../audits/target468-20261016/accepted-pr-integration/pr826-integration.json)
remains preserved. PR822/823/824/825/827
remain integrated as `bc472fb4`, `d6020643`, `76fe144f`, `050aea7f`, `e63a0824`.
Their retained receipts cover finite phase inventory, conservative MIX fallback,
complete command parsing, finite quantities and live-engine deployment.

**Last accepted production remains `ed7c452d`, binary `f9358b91`, producer lock
`66886813`.** [Release evidence](../audits/target468-20261016/production-release-ed7c452d-20261018/README.md)
authenticates the actual artifact, runtime libraries and frontend assets:

- All11 frozen API controls pass, including finite stocks, solid/gas MIX,
  unchanged Fe(OH)2 waits/explanation, command consumption and duration policy.
- Authentication/CORS controls pass; paired concurrency is one200 and one429.
- All37 unchanged live-browser checks pass, including actual served asset hashes,
  a real uncached Wasm solve, native/browser pH/I agreement and interpreter
  arithmetic. Labelled UI fixtures test interface behavior, not chemistry.
- The browser ran on a CPU provider with exact source/binding/package-lock
  receipts. No protected GitHub context or GitHub browser success was fabricated.

First harness failures are preserved: public engine identity has three fields,
whereas the full authenticated artifact manifest has more; a text-only explain
row has no inventory snapshot. The corrected reader retained every physical
criterion and assessed saved responses without repeating experiments.

The [new phase-source production release](https://github.com/CrispStrobe/kerotakis/actions/runs/39143554661) is dispatched exactly once for `0a5a92f1`. Its eventual deploy success is not artifact/API/browser acceptance.

The previous production404 engine-asset gap is closed on Vercel. **Pages still
has a confirmed separate profile gap:** actual published source `76fe144`
reports BASIC disabled and refuses USER_PUNCH. Pages REST client/UI checks pass;
that does not certify Pages chemistry. Follow
[lane H](saved64-review-lanes-20261018.md#h-match-the-actual-pages-payload-to-its-reviewed-interpreter-profile).

## Prepared repairs: implemented is separate from integrated

| PR | Current reviewed head | Behavior and actual focused evidence | Remaining acceptance |
| --- | --- | --- | --- |
| [828](https://github.com/CrispStrobe/kerotakis/pull/828) | `5b054e6c` | Preserve supported paid solid/liquid coexistence across unchanged CEA readback. Correct freezing/condensation energy when phase Cp changes. Clippy,31 targeted core and80 CEA tests pass on authenticated CPU execution, including reverse cooling/pouring and water's actual solvent owner. | Integrated as `0a5a92f1` after25 CI, both native platforms and all33 original logical jobs. A production release is dispatched; actual delivered artifact/API/browser acceptance and separate TH-016 replay remain pending. General reacting multiphase HP energy remains open. |
| [829](https://github.com/CrispStrobe/kerotakis/pull/829) | `8e2fe429` | Report missing neutral chromatography parameters beside supported peaks; retain ionic exclusions. Actual8 core and3 native controls pass with real mixing/dissolution before PHREEQC and an explicit aqueous-glucose precondition. | Integrated as `b3c18f5c` after25 CI, both native platforms and all33 original logical fleet jobs. Missing retention parameters remain unsupported; SP012 is not selected. |
| [830](https://github.com/CrispStrobe/kerotakis/pull/830) | `f3517e08` | Cancellation-sensitive aggregates preserve original assertions/artifacts. Normal-cancellation control,25 CI and33 logical fleet jobs pass. | Integrated as `0fe97e76`; preserve the actual merge receipt. |
| [832](https://github.com/CrispStrobe/kerotakis/pull/832) | `09aa2b01` | Select the existing reviewed MY-BASIC profile for the Pages build and probe the actual final payload before all existing browser gates. | Actual build, applicable acceptance and normal integration, then independent published Pages interpreter/uncached solution parity. |
| [833](https://github.com/CrispStrobe/kerotakis/pull/833) | `e9cc8782` | Initialize finite gas with native raw component moles. First source `8eece0b1` passes Clippy and all524 PHREEQC native tests; current source includes accepted phase/energy repair. | Current combined-source Clippy,604 PHREEQC/CEA tests,31 core controls and all33 fleet jobs pass on authenticated CPU execution; protected/platform acceptance and normal integration remain. A separate deployed GS-010 replay follows. |
| [834](https://github.com/CrispStrobe/kerotakis/pull/834) | `2750df2d` | Couple finite NH3 gas and reduced-nitrogen protonation in the same native solve; preserve inert carriers and core/dry fallback. Explicitly refuse a routed database lacking NH3(g) before mutation. | Four new native controls and the full PHREEQC suite are running on CPU. Source review, combined finite-gas prerequisite, full applicable fleet, protected/platform checks, integration and original GS005 replay remain pending. No scientific selection. |

[Full original PR828 fleet evidence](../audits/target468-20261016/cpu-full-fleet-828-20261018/README.md) authenticates source, binary, producer lock and every original command. A redundant GitHub producer build occurred during transfer; no GitHub experimental Record step started. The cancelled workflow is not a GitHub success.

[Authenticated focused CPU evidence](../audits/target468-20261016/cpu-native-controls-20261018/README.md)
retains original failures, actual compiler/producer locks and raw stdout. PR828's
first lint failure and later reverse-cooling failure remain preserved. PR829's
first native test omitted neutral-solid dissolution; its correction uses the
real solver and adds a precondition, without dropping an assertion. The newer
water test likewise keeps water and its original assertions through the correct
solvent-state owner. Test counts are not corpus experiment counts.

Never merge the broad audit branch or isolated browser validation ref. Once all
applicable acceptance is met, enable normal auto-merge for the exact reviewed
head without another permission round. Preserve branch protection and inspect
actual merge parents/tree afterward.

## Next executable work

1. Finish PR833 current-head protected/platform acceptance and normal integration; PR828/829/830 are integrated. Retain
   exact provider qualification and first failures; do not reset valid runs for
   optional changes. Document any held resource gate or active-run conflict.
2. Deploy changed runtime source only after acceptance. Freeze a separate
   TH-016 replay; never relabel the current deployed failure as repaired.
3. Repair the demonstrated brine finite-carbon/charge cause and establish a
   safe analytical strong-base capacity/atomic applier for hydrolysis. Neither
   positive basis nor instantaneous free OH alone proves a stoichiometric reserve.
4. Close the now measured Pages interpreter-profile gap using the reviewed
   backend and actual served-payload acceptance. Continue the
   [scoped lanes](saved64-review-lanes-20261018.md), including staged separation,
   thermal domains, boundary semantics and final41 candidate review.

Host paths, credentials, resource thresholds, session inventories and operating
controllers belong only in the ignored private handoff. Legacy Hermes workers
are closed; any independently approved reviewer is supervised and its conclusions
need root review. No worker report alone earns a scientific selection.


The [first gas native acceptance](../audits/target468-20261016/cpu-native-controls-20261018/finite-gas-moles-first-acceptance/README.md)
retains actual524-test results and producer lock. The combined worker's first
setup failure is preserved: missing recursive submodules stopped Cargo before
compilation or experiments. Its correction initializes the original source
submodules before unchanged native commands. No failing assertion is waived.

Current accepted-source receipts: [PR829 fleet](../audits/target468-20261016/hybrid-own-fleet-20261018/pr829/README.md), [verified integration](../audits/target468-20261016/accepted-pr-integration/pr829-integration.json), and [PR833 combined-source native/fleet acceptance](../audits/target468-20261016/cpu-combined-gas-833-20261018/README.md). Scientific catalog remains204 overall /200 fixed468. A separate current-accepted-equivalent ammonia control reproduces inconsistent neutral-gas/protonation coupling; lane J remains open, with no new scientific selection.

[Ammonia first-control evidence](../audits/target468-20261016/ammonia-coupled-first-control-20261018/README.md) preserves the failed current-equivalent forecast, all three raw arms and the rejected first reader. PR834 is a proposed source repair undergoing actual execution, not an accepted fix.
