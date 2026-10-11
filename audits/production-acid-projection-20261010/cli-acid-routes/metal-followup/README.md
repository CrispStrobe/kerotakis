# First acid-route results and separate metal follow-up

Run38149577918 completed on pinned harness3904977d and reused the independently accepted sourcedbe86521 executable. [Independent review](../review-38149577918.json) accepts collection lineage and all twelve raw process bindings. It does **not** accept the complete six-case warning slice.

| Original protocol | Observed category | What this establishes |
| --- | --- | --- |
| Bleach positive | Qualifying, original partner consumed | One prospective warning in JSON/text; no surviving-final exposure proof |
| Bleach benign | Qualifying benign counterpart | No primary warning |
| Carbonate positive | Qualifying, original partner consumed | One prospective warning in JSON/text; no surviving-final exposure proof |
| Carbonate benign | Qualifying benign counterpart | No primary warning |
| Metal positive | Does not exercise the original strict contract | Two setup capability notices violate the global no-refusal prerequisite |
| Metal benign | Does not exercise the original strict contract | MgCl2 is unregistered; both processes exit1 before titration |

The magnesium positive nevertheless completes the acid step: exactly one reviewed Caution warning, a full10mL dose and magnesium consumption. Its two `not_yet_modeled` events occur during the earlier magnesium–water setup, not the titration. Setup uses the intentional analytic solvent-only route, so the original requirement for PHREEQC at both setup and final state also does not fit this protocol. No acid warning defect is demonstrated by these outputs. The original freeze, scripts and classifications remain unchanged.

The [separate follow-up freeze](freeze.json) declares the same Mg positive and a registered MgSO4 benign counterpart before corrected-pair execution. Only the exact two declared Mg setup notices may occur at step1. Positive setup must use the source's analytic solvent-only route; benign setup and both final acid states must use PHREEQC. Both acid operations require no refusal, the complete declared dose, solved strong acidity, exact warning metadata/count/severity and matching ordinary-text narration. Retained raw setup notices remain visible. Consumed Mg can prove prospective narration only.

Source evidence on the pinned head includes the [species registry](../../../../crates/kerotakis-core/tests/golden/registry.json) and existing registered-salt controls in [CLI perturbation tests](../../../../crates/kerotakis-cli/tests/perturbation.rs). Unsupported species in an authored probe is a prerequisite error; adding chemistry or changing reporting to make that probe pass is not justified. Wider magnesium–water kinetics, new final-only exposure and intermediate products remain separate tasks.

[Independent follow-up review](review-local-20261011.json) accepts all four separately collected local processes using the identical accepted hosted executable. The Mg positive qualifies as prospective-only with consumed Mg; MgSO4 qualifies as benign. Two independent reviews produce byte-identical receipts. The reviewer retains both the original collector failure and the separately scoped notice/backend checks, without rewriting raw provenance. Eight focused tests cover altered notices, text-only refusals, warning metadata/severity/dose, raw stream hashes, process vectors and duplicate/nonfinite JSON.

This follow-up is **local executable reuse**, distinct from the original hosted collection and production integration. No Rust build or engine change was needed. The public receipt binds every raw evidence file, the frozen inputs, reviewer and exact inherited binary/source/lock; physical evidence locations are private.

```sh
python3 tools/metal-acid-followup-review.py \
  --artifact FOLLOWUP_ARCHIVE --parent-artifact ACCEPTED_REPLAY_ARCHIVE \
  --parent-run-metadata PARENT_FINAL_RUN_JSON --report NEW_REPORT_JSON
```

Keep the report outside immutable artifact directories. For this local review the recorded executable copy must still be available so its bytes can be independently checked. Next: consume the remaining exact-head gates/fleet. The full original six-case contract remains unaccepted. Complete exact-head CI/fleet gates are still required before PR777 integration.
