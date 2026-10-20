# KI-010: independent actual-production saved response

The unchanged original was issued once against accepted source `dacf76513328c2226e2f5133a79fb9af84343431`, binary `f6aef37ddadfa0f1163e2aff9f436ec560eae767d6d86203bfa850c04f0c9955` and actual producer lock `41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb`. Independent saved review proposes a **bounded pass**, independently reproduced and accepted by the parent. No new engine/API request, job or automatic selection was performed.

```sh
python3 audits/target468-20261016/sulfur-filter-beneficiary-review-20261020/read_responses.py \
  --repo "$REPOSITORY" --design "$EVIDENCE/design.json" \
  --results-dir "$EVIDENCE/results" \
  --artifact-receipt "$REPOSITORY/audits/target468-20261016/partition-release-acceptance-20261020/artifact-acceptance.json" \
  --production-receipt "$REPOSITORY/audits/target468-20261016/partition-release-acceptance-20261020/production-acceptance.json" \
  --out "$NEW_ASSESSMENT"
```

Use this directory's `evidence/` bundle for `$EVIDENCE`; `--out` must be new. Compare parsed output with `evidence/independent-assessment-bounded.json`. `evidence-manifest.json` binds the safe bounded raw design, runner, issuance, requests, responses and assessments. Production/artifact receipts bind actual producer bytes; the mathematical reader binds exact source/tree, original script/forecast/criteria and every raw response hash.

Original criterion C1 is met: immediately after filtration receiver retains1.904118345mmol thiosulfate and1.808236689mmol analytical acid. Original C2 is met: receiver initially contains **zero old solid sulfur**; old aqueous sulfur0.311920563µmol is separately transferred and retained. After30s receiver forms219.639288270µmol solid sulfur, matching actual precursor loss, separately owned SO2, twice the consumed analytical acid and newly formed water. Old donor solid95.569734878µmol remains with its **95.881655441µmol SO2 gas**; that gas was silently lost by the historical c446 filter. No current gas loss occurs in this saved response.

Decimal45 ledgers cover every step/vessel and all declared atoms/charge, gas/filter phase ownership, carrier water and noncontents material reservoirs. Maximum residual is9.27e-16mol-equivalent. Actual clocks, temperatures and pH are included; transferred old aqueous sulfur cannot masquerade as the newly produced solid. Pending external CO2 credits below1µmol are reported separately; exact source parks those unapplied credits until the shared transfer floor, rather than already owning extra canonical carbon. No measured atmospheric uptake is invented.

The disclosed finite-acid interval approximation remains explicit. This qualifies the original acid/precursor retention and fresh-sulfur separation contrast, not continuously coupled acid/base equilibrium, buffered proton replenishment, calibrated absolute rate or isothermal kinetics. First-order source parameters are authenticated; sulfur/acid/water/SO2 quantities are independently measured from before/after inventories rather than accepted from event labels.

First reader failure (`KeyError: scene`) and code are preserved: actual API projection omits the full CLI scene, while canonical phase inventories required by the original criteria remain available. No rendered-appearance claim is made. The first completed reader also returned a mechanical hold because it treated every named limitation as disqualifying. That reader and assessment are preserved separately. Final criterion-level review explicitly classifies only `not-modeled.finite-acid-inventory` as compatible with this bounded ownership question; any other missing-model/failure event still disqualifies. All original words, raw outputs and earlier holds remain unchanged.

Parent acceptance is recorded in `parent-adjudication.json`, with identical raw-safe `evidence/parent-assessment.json`. Canonical SO2 gas retention establishes material/phase bookkeeping only, not validated physical open-vessel mass transfer, venting or release rate. Final manifest binds the parent assessment; its predecessor remains in `evidence-manifest-before-parent-review.json`. The separate successor selection preserves all321 earlier catalog entries exactly.
