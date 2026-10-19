# Original swept-ammonia experiment: forecast fails, separate boundary gap

Unchanged original AB031 was issued once on independently accepted source `8c584e20`, binary `13a4d134`, actual producer lock `66886813`. [Frozen design](design.json), raw response and [independent saved review](assessment.json) retain commands, forecasts and actual observations. No retry or catalog selection follows from exit0.

Both arms start at pH9.286315883367. After sweep and global120s wait, the sealed control is9.286276077177 and the swept arm9.287908348168: the original lower-pH criterion fails. The sweep exports initial trapped NH3 (6.92153283e-7mol) and also strips CO2. Chloride is retained. Per-vessel atoms/charge over sweep and subsequent histories close within the declared1e-8mol/eq review tolerance. Trapped N2 and reduced NH3/NH4 nitrogen are accounted separately; the saved raw response includes both histories and actual temperatures.

Source inspection finds that NH3(g) is admitted for finite headspaces only, while the swept reservoir covers other gases. Exporting initial trapped NH3 is not demonstrated continued transfer from dissolved ammonia. This is a separate coverage gap; CO2 removal also confounds the original pH forecast. Do not force the pH sign, invent unobserved nitrogen exports, ignore carbon or declare an empirical flow model validated.

Reproduce without engine/network calls from the repository root:

```console
python3 audits/target468-20261016/ammonia-sweep-original-20261019/review_saved.py --out ammonia-sweep-review.json
```

[Source repair and acceptance lane](../../../docs/post-hydrolysis-gap-lanes-20261019.md#2-swept-ammonia-boundary-and-competing-carbon-dioxide) scopes the generic boundary implementation and independent controls. This experiment remains unselected; catalog248 /fixed468244 is unchanged.
