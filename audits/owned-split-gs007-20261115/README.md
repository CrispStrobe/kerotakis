# Owned split planning and exact GS007 diagnosis

**Coverage remains431/468 originals,435 entries,37remaining. No SP003 or GS007 qualification is added; PO002 is not credited again.** Signed source `6a2a4fc916e311aadeb96abad25222b6d5aac9d7`, tree `8ac9981b82aa2d0ba8f6d203d9bee384437fe3a3`, adds an isolated planning API and the unchanged GS007 fixture. It changes no still/aqueous algorithm, vessel writer, scientific tolerance, gas boundary or equilibrium datum. No PR or merge is requested for this experimental design branch.

## SP003: smallest representation and the exact unsafe writer boundary

`ComponentSplit` owns **both** retained and transferred compensated `Amount`s; it never derives retained authority by subtracting a rounded overhead scalar. `scalar_write_plan` binds the actual donor and refuses any low limb that scalar source/receiver writes would discard. The new actual-SP003-step629 control preserves its next positive retained amount and transferred correction but rejects scalar write authority. An independent dyadic1−2⁻⁷⁰ control does the same. Representable source/receiver plans succeed; stale donors, overdraw, invalid amounts and receiver rounding refuse. Four authority controls and15 unchanged Amount controls actually pass.

The exact unsafe API invariant is public `Portion.moles: Moles(f64)`. `deposit` merges scalar moles; `withdraw_phase` subtracts scalar moles; persistence and existing native/physical writers continue to read/write that field. A second residue field or independent correction map would create competing authorities and can be erased by those writers. The smallest honest next integration is an authoritative amount for each affected species/phase, with all its transfer/persistence writers preserving both limbs; a still return-value change alone is insufficient. The implemented planner is **not** a migrated writer, a completed still cut or SP003 qualification. The SP003 original was not rerun.

## GS007 unchanged original and complete reader

Exact original `new120:gs-007` script SHA256 `aee4f444f4bc8c20870b424b4a9c64d5ed6b30b5efa652432289c54acb08cce9`. One native PHREEQC/full-stack run completes every original command and exits101 at unchanged C2. The complete independent reader actually ran: **C1PASS/C2FAIL**, qualificationFALSE. CO2-rich solution dissolves3.961162380331e-4molCa, retaining1.038837619669e-4molcalcite. After sweep+60s calcite is0, while0.0005molCa remains aqueous. Rich state298.2014743281659K/112418.98012693944Pa; final298.04112821167905K/100000Pa.

The first independent reader failed with `KeyError: base_equivalents`; its exact source and failure are preserved. The corrected reader binds `species::basis_species_data` at the tested source: this runtime basis carrier explicitly borrows the registry OH stoichiometry (H1/O1/charge−1) for analytical closure. This is **not** an invented physical species or measured free hydroxide. The observation and registry data are not modified. All11 C/H/O/N/Ca/charge ledgers close below1.7763568394002505e-15mol/eq, all observations are read-only, and four reader mutations reject. Pending carbon is exactly0 at every frame.

## Every actual gas crossing and its owner

| Command | Species | Outward mol | Owner |
|---|---|---:|---|
| sweep | N2 |0.016147291289336348| core `vent_headspace` |
| sweep | O2 |0.004281556163923509| core `vent_headspace` |
| sweep | CO2 |0.002241891385439| core `vent_headspace` |
| sweep | CO2 |0.001265954140717| native reservoir projection |
| wait60s | CO2 |1.063249319833e-10| native reservoir projection |

The first three exactly equal the pre-sweep gas portions. The open-only `GasExchangeClock` explicitly returns for Swept and supplies no competing parcel/event. Native `ExternalGasKind::Reservoir` owns the other two CO2 events; `complete_basis` incorporates each actual gas loss in carbon and H/O, then reconstructs the analytical acid/base coordinate. Base equivalents rise8.485176850973886e-6→0.0009993471721120408 during sweep and0.00099934738369256 after wait. Final measured free hydroxide is separately0.0009539820797277511mol. These quantities are not interchangeable.

Independent decimal accounting closes swept carbon to2.45e-20mol and the signed acid/base coordinate to1.13e-18eq; the wait closes still more closely. Two actual Dissolved events consume the entire remaining calcite,1.2987770658116097e-5+9.08959913087839e-5mol. There is no dropped carbon/basis owner or pending-parcel explanation in these data.

## Source bug or model limit?

**Source-trace inference: scoped swept-reservoir model limitation, not a demonstrated conservation defect.** `partition` maps Swept to `SWEPT_LOG_PARTIAL_PRESSURE=-12` and the actual native input offers `CO2(g) -12 0` with Calcite equilibrium. This near-zero CO2 sink has no finite supplied gas inventory or flow/time budget. The coupled equilibrium therefore removes carbon while dissolving the remaining mineral; the analytical base coordinate closes the resulting alkaline solution. It does not model a finite60-second gas-flow trajectory or guarantee the original scale-recovery forecast.

No `NotYetModeled` warning is emitted for that missing transient interpretation: this is a separate honesty gap, explicitly retained rather than fabricated as a runtime event. A causal next change would separate a finite swept-gas transfer owner from the existing equilibrium reservoir, with justified transfer/domain parameters and coupled carbonate settlement, while exposing the equilibrium-only boundary. Tuning−12, changing the original criterion, ignoring basis or inventing a flow to make C2 pass is not supported.

Native provenance is the existing USGS `wateq4f.dat` plus the existing USBM reference-complex and Sander HBr slices; mol/kgw native concentrations, mol inventories, K/Pa vessel coordinates and log10-atm gas boundary. The pinned primary-data hashes/licence references are supplied. No data was imported. The native carbon-overdraw refusal control also passes.

## Exact evidence and delivery separation

One bounded cached CPU job, no dataset copy, no SP003/PO002/EC009 original retry. The reader verifies57 small producer files, exact source/tree/compiler/producer lock and three independently stream-hashed executable artifacts. Public logs are labelled excerpts with full raw hashes. Full private producer evidence remains in the operating handoff; reproducing the reader requires that bundle. Source/runtime excerpts and the first reader failure are retained here.

PR884 remains a distinct draft at5db4a1260 with its44 core passes and existing normal CI/fleet; full acceptance and merge authorization remain separate. ROOT owns TestFlight; no release call belongs to this work. Existing owner stays ACTIVE.
