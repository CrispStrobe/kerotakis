# Sixth independent fifty: original evidence

[Run 37762502850](https://github.com/CrispStrobe/kerotakis/actions/runs/37762502850) completed all 72 original CLI processes at the frozen source `7bc43e089753483710507133ccc8377591bb3490`: **39 ordinary passes, one model qualification, ten unmet expectations**. Every input/stdout/stderr hash, inspection record, original check and aggregate outcome was reverified from raw evidence. The [review receipt](experiments-receipt.json) and [hosted artifact bindings](artifact-bindings.json) preserve that result. The original forecast is unchanged.

The ten mismatches do not establish ten engine defects. Raw inventories explain why their original checks were insufficient:

| Cases | Original oracle problem | Separately frozen correction |
| --- | --- | --- |
| S04, S05, S08, S12, S45 | Dissolved NaCl/KCl are represented by their owned cations and chloride; counting only the original salt reports zero. | Count each conserved salt element independently, including any intact salt. Preserve original doses and tolerances. |
| S13 | Calcium carbonate partially dissolves; counting intact carbonate alone misses transferred dissolved calcium. | Include owned calcium ions across donor and receivers while retaining the solid-retention and solvent-transfer checks. |
| S46, S48 | Sealing traps ambient nitrogen in addition to the supplied charge. | Add an uncharged sealed control; check the supplied increment and heating conservation. |
| S47 | Atmospheric pressure remains present, so total-pressure ratios do not follow the supplied-charge ratio. | Compare excess pressure against matched uncharged sealed controls at both volumes. |
| S50 | Carbon distributes between gas, dissolved CO2, bicarbonate and carbonate; sealing also imports ambient CO2. | Count all represented carbon forms and subtract a matched sealed-water control. |

S07 originally passed a salt comparison of zero against zero. It is included in the correction replay with positive, independently conserved sodium and chloride quantities. This is an oracle gap, even though it was not an original failing case.

S09 preserved copper and water during magnetic separation but reported an explicit boundary for copper in contact with liquid. Its model qualification remains visible; preserving inventory does not validate unmodeled dissolution chemistry.

The [semantic replay](semantic-replay.json) freezes **11 cases and 19 CLI processes** at `631881c4` before adapted execution, SHA256 `9296a251a40c432155f841eae8dba48e1ab4bf8c358be4cd4ef47b1f401acd66`. [The adaptation receipt](semantic-adaptation.json) records changed checks and added variants. [Run 37775481924](https://github.com/CrispStrobe/kerotakis/actions/runs/37775481924) is queued at harness `359c0bdc`. It uses the exact original executable. These are disclosed post-result oracle corrections, not new independent experiments or retroactive changes to the original score. The stronger checks still need execution and review.

The general lesson is to test conserved inventories across chemical representations and explicit external boundaries, and to include positive quantity checks so absent fields cannot produce a vacuous comparison. Source-informed follow-ups must remain distinct from independently forecast experiments. Further engine repairs require a reproduced remaining mismatch after that distinction is resolved.
