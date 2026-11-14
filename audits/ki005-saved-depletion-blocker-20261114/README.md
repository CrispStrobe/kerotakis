# KI005: authenticated post-split depletion blocker

**Held; no new qualification.** The bounded saved-output review verifies proportional enzyme/substrate transfer, but the original design has already consumed 99.999999% of its peroxide before splitting. Both post-split amounts are below the pinned consuming-rate gate. Equal zero loss over the next20s cannot establish equal fractional reaction rates. The original script, forecast, criteria and earlier assessment are preserved. No dose, time, phase, tolerance or runtime floor is changed.

| Quantity | Saved value |
| --- | --- |
| Initial peroxide |0.001mol|
| Peroxide immediately before split, after5s |1e-11mol|
| Post-split peroxide, remainder/aliquot |7.5e-12 /2.5e-12mol|
| Post-split fractional loss over20s |0 /0|
| Pinned consuming-rate gate |strictly greater than1e-11mol|
| First reaction extent / released heat |0.000499999995mol /98.199999018J|

The two unchanged criteria in [original.json](original.json) are assessed separately in [review-result.json](review-result.json):

- **KI-005-C1:** proportional transfer passes. Water, peroxide and aqueous catalase follow the0.75/0.25 split; retained oxygen stays in the donor. The surrogate enzyme remains aqueous.
- **KI-005-C2:** arithmetic fractional-loss comparison is performed, but scientific qualification is held. Both rates are switched off by depletion before the20s comparison. A zero/zero result cannot identify the original intensive-versus-extensive rate scaling. It is not a chemical negative or a measured kinetic agreement.

The [pinned integrator](pinned-kinetics_integrator.txt) defines `DEPLETION_EVENT = DEPLETED * 10.0`; [the pinned reaction source](pinned-kinetics.txt) defines `DEPLETED = 1e-12`. The consuming direction requires a non-intermediate reactant strictly above that event floor; otherwise its forward rate is zero. Peroxide is consumed and not produced by this irreversible decomposition, so it is not an intermediate exemption. This source trace explains the saved residual arms without asserting that a current-source repair has run. It does not justify lowering the floor or changing the original waits.

All27 represented H/O/charge checks across nine observations close below4.1e-16mol/eq under the unchanged1e-10 bound. The catalase entry has a placeholder C composition, not a real enzyme formula: its effective surrogate pool is preserved separately at all nine checkpoints, with strict proportional-transfer checks. No real protein elemental or empirical enzyme calibration claim is made. Inspect leaves the entire bench unchanged; the second wait preserves the complete material inventory, emits no reaction event and advances both vessel clocks by20s. The first reaction heat matches its declared extent and196400J/mol-extent convention; this is not total apparatus or ambient-cooling heat closure.

Twelve mutations reject missing enzyme, insoluble enzyme, unequal substrate transfer, missing water/oxygen, invented heat/loss/pending ownership, solver failure, changed wait, invented active reserve and changed inspection inventory. These are reader controls on copies, not new engine executions. The first reader run succeeds; [stdout](first-review.stdout) and [stderr](first-review.stderr) are retained.

## Identity and reproducibility

The [saved response](response.json) SHA256 is `2aa3edc46ba7d83c8fd306127a8bbc2d2d78491d91a4dd68896bb90ff3f0919e`, authenticated against the existing production-run manifest. The [original script](original.lab) SHA256 is `ea151e5084526ddb42d9822ad88952b1f5d76e29c080acc363fde1cdca6dc85a`. [The request receipt](request-receipt.json) binds source `abf4e9a34eb16e51f605a2009e542b62002a30ea`, binary `6101bfbdcfc2c5f95cf771e44c549cf3ae390ee0a1cf37af06fbf9651b952544` and producer lock `c25e1ccd68ff9ef61dde11a62224117cf4f25ea61cce196d3e066ff5d4bb60bf`. Source excerpts and registry compositions are authenticated against their actual Git blobs by [review.py](review.py). These historical identities are not new native execution or current-release acceptance.

The saved reader accepts an exclusive output path:

```console
python -B audits/ki005-saved-depletion-blocker-20261114/review.py --out /tmp/ki005-review.json
```

All434 catalog objects remain unchanged:430/468 originals qualify and38 remain. KI005 is not added. EC009's failed0aa/5271 candidates, skipped v2 original continuation and unexecuted solved-water proposal remain separate. EC004 historical chemistry/binary-null versus later readout and KI011 gross-versus-surviving iodine/unexhausted sinks/unspent pending CO2 retain their exact earlier scopes. PR882/PR883 full acceptance is not earned here.

## User-directed park

The user's subsequent direction stops owner replacement and new validation work. Kero is parked with this scientific blocker and EC009's separate closed resource gate: the existing12GiB fast-disk reserve remains required. No heavy build/native execution, new validation job, dataset copy, `ci:full`, tolerance change, claim-only PR or owner replacement is issued. Cold archival, if needed before future heavy work, must first respect the protected evidence/worktree inventory; no protected artifact is removed or moved by this review. The exact host measurements and private proposal/cache locations remain in the private operating handover.
