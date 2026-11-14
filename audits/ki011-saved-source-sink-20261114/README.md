# KI011: gross iodine, surviving iodine and competing sink consumption

The two **unchanged original inspection criteria** pass at the authenticated saved production source `abf4e9a34eb16e51f605a2009e542b62002a30ea`. The original larger-sink/delayed-iodine forecast remains **unconfirmed**: neither sink is exhausted and no surviving iodine appears during the100s window. No induction time, twofold delay, empirical rate or absorbance calibration is qualified.

The original criteria are preserved verbatim in [original.json](original.json): “Distinguish gross iodine production from surviving iodine inventory” and “Compare sink exhaustion and precursor consumption, not colour onset alone.” A comparison that finds both sinks unexhausted is recorded as such; it is not relabelled an observed exhaustion. The [prior assessment](prior-assessments.json) remains immutable and correctly identifies the forecast as unsupported in that window. This review separately assesses the actual two inspection criteria, using source/sink amounts rather than treating an unavailable colour observation as a rate measurement.

## Actual saved comparison

| Time | Gross iodine v1/v2, µmol | Surviving iodine, µmol | Remaining thiosulfate v1/v2, µmol | Remaining peroxide v1/v2, mmol |
| --- | --- | --- | --- | --- |
|10s|3.129399 /3.105514|0 /0|89.266888 /184.907481|1.863424 /1.863446|
|40s|11.178985 /11.027843|0 /0|61.841380 /145.955019|1.507324 /1.507440|
|100s|22.717304 /22.201537|0 /0|24.309160 /90.466689|0.984785 /0.985088|

Gross iodine is the sum of actual `iodide-peroxide-clock` reaction extents, independently cross-checked against retained tetrathionate. Each iodine amount is immediately scavenged, returning iodide; gross production is not additional surviving I2 and must not be added to the iodine atom inventory. Actual retained iodide remains0.002mol per vessel. At100s the gross-source ratio v2/v1 is0.9772962885731138, not an inferred half-speed source. The forecast does not specify exact equality and this comparison is not empirical kinetic calibration.

The initial thiosulfate amounts are100/200µmol. Actual acid decomposition consumes30.256232/65.130237µmol by100s, separately cross-checked against total retained sulfur and SO2. Iodine scavenging consumes45.434608/44.403074µmol thiosulfate. Their sum reconciles the actual remaining24.309160/90.466689µmol. Neither is exhausted. Peroxide consumption is separately split between2×retained O2 from decomposition and the gross iodine-source extent; it is not all attributed to iodine production. All three matched times are reviewed. Four UV/visible measurements explicitly refuse unparameterised sulfur absorbance; no colour onset is invented.

## Conservation and the unspent CO2 handoff

All147 known represented H/O/S/K/I/Na/charge checkpoints across21 saved observations close within3.56e-15mol/eq, under the unchanged1e-10 bound. No solver failure or material loss is accepted. Measurement and inspection leave the entire bench exactly unchanged. Twelve reader mutations reject lost iodine/sink/tetrathionate/SO2/water/peroxide, invented surviving iodine or gross production, solver failure, invented exhaustion and invalid pending ownership.

The first reader's assertion that every pending-CO2 field must be zero failed. [That reader](review.first.py), its [first failure](first-review.stderr), subsequent review and [correction record](reader-correction.json) are preserved. The producer's actual schema repeats the step index across non-consuming `inspect`; the corrected reader checks the source's operation index while keeping every observation as a distinct ledger checkpoint.

The pending-CO2 field is **not zeroed or hidden**. Final values are2.9117260384357037e-8 /2.91172367539558e-8mol. [The pinned clock owner](pinned-gas-exchange-owner.txt), [spending-floor contract](pinned-transfer-floor.txt), [vessel owner](pinned-pending-owner.txt) and [native boundary check](pinned-aqueous-transfer-spending.txt) establish this as an owned parked handoff below the existing1e-6mol application floor. The native validator requires zero applied transfer while below that floor; no carbon appears in canonical portions, no carbon-absorption event is emitted and no completed aqueous carbon uptake is claimed. The final reader checks every recorded handoff for finite nonnegative below-floor ownership, unchanged values outside `wait`, and accumulation only at `wait`. The complete counter history is preserved in [the actual result](review-v3-result.json). It is separate from canonical stock closure and from iodine/peroxide production, not an extra ionic portion or a dropped source/sink. This is not a waiver of1e-10 conservation and does not raise or lower either runtime bound. Carbon uptake remains unqualified.

The source's finite-acid warnings explicitly hold the activity coefficient at each interval's starting value while proton concentration depletes, then re-solve between intervals; continuous acid/base coupling and empirical kinetics are not claimed. Unpriced KI/thiosulfate heat warnings remain explicit; no calorimetric closure or calibration is qualified.

## Provenance

[The exact saved response](response.json) SHA256 is `5243dfbbd7f4da7f8ddaf1d3deb0db476f475206c1537d17b83679adbb1917f5`, matching the previously published production-run manifest. The [unchanged original script](original.lab) SHA256 is `8ee57b2e0bca4ecd54eba3bc8c7a622b89ab7c329a932b9be933f3fc5f61467c`. The response's actual deployment engine binding matches [the request receipt](request-receipt.json): sourceabf4e9a3, tree108961f6824cef5a3adb61a923a0279e7c4e3c90, binary6101bfbdcfc2c5f95cf771e44c549cf3ae390ee0a1cf37af06fbf9651b952544, producer lockc25e1ccd68ff9ef61dde11a62224117cf4f25ea61cce196d3e066ff5d4bb60bf. These are authenticated saved deployment identities; no new binary execution or binary hash measurement is claimed in this review. Exact source-composition records and [the source reaction row](pinned-iodine-source-row.txt) are pinned separately.

No new job, build, chemical prefix or protocol is replayed. EC004's4bd historical chemistry/binary-null and4151791fa readout remain separate provenance; no identity is substituted here. EC009 remains blocked by its actual−1.0985317866e-6eq charge defect. PR882/PR883 full source/platform/fleet and correctly matched selector/floor acceptance remain separate delivery gates.
