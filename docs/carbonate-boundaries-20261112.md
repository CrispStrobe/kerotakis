# Carbonate boundary repairs — 2026-11-12

PR881 remains a draft. These changes require their own hosted acceptance before normal merge,
one production deployment and affected-original review. Production remains `731931f0`;
qualification remains 402/468, spanning explicitly bound historical sources.

## Separate three owners

An instantaneous operator has no diffusive time. Additions, filtering, decanting and mixing share
the same bubble-pressure boundary; native MIX must not bypass it. The first selectivity failures
exported exactly the apparent missing carbon as CO2 during filtering. Preserve their original
contents-conservation checks and separately reconcile every actual gas crossing.

The finite-time film clock now proposes signed inward/outward CO2 parcels. Outward transfer uses
the existing Cole and Caraco (1998) still-surface coefficient, vessel mouth geometry and Henry
relation, bounded by actual neutral `CO2(aq)` inventory including already reserved parcels. The
aqueous owner applies and announces each parcel once. A dedicated step-local CO2 boundary keeps
the same solve from also emptying the vessel to atmospheric equilibrium. Sealed and swept
boundaries retain their existing owners. The coefficient remains a low-wind lake estimate used
as an upper-bound analogy for a beaker, with no beaker calibration, chemical enhancement or
new temperature-dependent transport correlation claimed.

The old acid fallback instead relaxed the whole supersaturated inventory whenever final bulk
pH fell below pK1. It missed low acid doses and discharged preexisting CO2 at higher doses. The
replacement is an explicitly bounded **curated endpoint approximation**, not measured
instantaneous transport or an order-independent quantitative yield: a recognized acid/carbonate addition must form new neutral dissolved
CO2 in a complete native answer. The parcel cannot exceed either that new neutral population
or available fresh reagent capacity, after reconciling first-pass gas exports. A cloned second
pass applies the parcel through the existing conservative external-carbon owner, with a fresh
incremental thermal snapshot and no second heat-input dose. Both the confirming closed pose and
the parcel pose invalidate inherited diagnostics and require a typed receipt bound to the exact
new question and returned complete answer, element/charge closure and the actual announced gas
parcel. No-op, partial, stale-receipt and merely-cleared-pending adapters must fail atomically.
Failure retains the first closed state and commits no endpoint gas. EN/DE/FR notices state that acid-front nucleation, escape
times and calibrated instantaneous yield remain unmodelled.

The admission domain is a simple water/alkali/carbonate/acetate/strong-acid medium. Opaque
material, interfaces, competing unreviewed buffers and reactive nitrate/iodide/bromide pairs
are excluded. HCl's deliberate generic reserve exception is preserved. The [RSC bicarbonate
experiment](https://edu.rsc.org/resources/bubble-volcanoes/2026.article) supports the qualitative
reaction; it does not supply an escape-rate constant. Do not promote this approximation into
an empirical prediction merely because an event-order assertion passes.

## Independent contract correction

[The preserved first failures and strict successor](../audits/carbonate-law-correction-20261112/README.md)
show why closed NaHCO3 pH need not rise monotonically with dose. All original scripts and frozen
manifests survive. The versioned successor replaces only that invalid ordering with independent
Na/C/H/O/charge, activity and mass-action checks against the actually identified WATEQ4F data.
Four saved cases pass and 40 corrupted variants fail; this is saved-predicate validation,
not execution of the newly edited runtime.

## Acceptance and remaining work

The first patched-source hosted probe passed32 native controls, including the new instantaneous
transfer check, Warder titrations and order invariance. Its core unit build failed on a missing
`Liters` qualification in a new test; another new test used unsupported CLI `wait 0s` instead of
the typed zero-time operator. Both test-authoring errors are corrected, with original output
preserved. That producer predates the new curated endpoint and cannot accept it.

The next exact-source hosted run must run core tests, existing overlap/Warder/mixing/clock/
order controls, low-dose gas-budget and competing-base negatives, affected gas/carbonate/
selectivity fleet laws, locale/presentation checks and source-integrity checks. Keep genuine
producer lock and binary identities. Do not bless chemistry goldens or weaken a physical bound.
After those pass, complete applicable protected acceptance through the existing PR without
adding `ci:full`, dispatching full workflows or making placeholder PRs.

The independent aq-023 review distinguishes the old repaired addition-order solvent-mass defect
from an unproved process-to-process auxiliary drift allegation. Capture eight serial repetitions
of the identical script with the same binary and lock, then compare primary and Pitzer auxiliary
coordinates, native input identities, temperature and all conserved ledgers. Canonical primary
refresh currently leaves auxiliary `solvent_activity` unchanged; trace the first divergence
before editing that ownership path. A stale historical tolerance is not reproducibility evidence.
