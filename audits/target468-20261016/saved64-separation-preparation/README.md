# Saved separation selection preparation

The [reader](verify_metrics.py) reproduces [all12 original separation cases](metrics.json),
including43 operations with whole-vessel ownership before and after, actual
phase/species amounts, represented element and formal-charge subtotals, declared
mass properties, composition fractions and original measurement events. It
preserves whole state across partial inspect rows; omitted vessels are not erased.
Original source, registry, archive, scripts and raw streams are authenticated.
No experiment/API execution, new scientific selection or empirical calibration
is claimed.

```console
python3 audits/target468-20261016/saved64-separation-preparation/verify_metrics.py
```

The ethanol fraction is explicitly relative to water plus ethanol, not every
solute or every phase in an arbitrary mixture. Mass is labelled known-property
mass and any missing molar masses are listed. Gravimetric AgCl is computed from
owned AgCl and the pinned molar mass, independently of a displayed balance value;
the original0.002mol gives0.28663330686g. Actual measured conductivity and
chromatographic peak/outside-method events are retained rather than invented.

SP003's unsupported higher-stage request, SP011's unresolved reduction/extraction
prerequisite and SP012's absent compound peaks remain held. Existing whole-bench
material closure does not prove stage performance, partition calibration or
energy-budget semantics. Finish per-criterion ownership and energy-domain review
before selecting candidates. Catalog stays200 / fixed468 selected196.
