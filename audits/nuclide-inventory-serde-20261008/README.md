# Reviewed owned-nuclide JSON diagnostic

[Run 37797436300](https://github.com/CrispStrobe/kerotakis/actions/runs/37797436300) completed successfully. Independent archive review verified exact source trees, clean checkouts, identical historical fixture, one generated lock, dispatched harness `ec003cc9`, raw logs and evidence hashes. [Machine-readable review](review.json) preserves every outcome.

Baseline `0028f3a7`: **seven passes / four failures**. Repair `b42732c1`: **11 passes / zero failures**. All **604 inherited core library tests pass on both sources**. Both compile the same public-API fixture; missing-API errors are not behavioral evidence.

The repair serializes nonempty owned inventories with canonical `El-A` / `El-Am` keys, retaining individual nuclide objects. Identity/amount validation and duplicate-key rejection protect round-trips; zero and smallest-positive amounts survive. This establishes bounded JSON persistence, not isotope existence, decay data or runtime nuclear accounting. The 11 controls are unchanged historical contracts, not another independent experiment batch.

[PR #769](https://github.com/CrispStrobe/kerotakis/pull/769) still requires final matching-head integration gates. Its later main-ancestry merge is separate from the source pinned by this diagnostic.
