# Independent unchanged AB029 CLI replay review

**Recommend PR809 proceed through normal exact-head readiness/auto-merge**, subject to root's current head/base/protected-check refresh. The independently reviewed head remains `673233df07deda6f311c474e913e9733a971853d`; tested merge `7fca297b73ed888e154c40c83811a57d4995e861`, tree `9f12cf9ec52a9929b92febf273d1d33639649861`. The previously pending exact CLI replay gate is now met. No additional nonprotected acceptance run is identified for the narrow Filter/Decant ownership repair.

**Recommend unchanged AB029 as a source-bound working protocol** on this tested merge and explicitly recorded compatible runtime. Both original criteria are met; the former c446 solver failure remains historical evidence, not rewritten success. This does not establish latest-main/production execution or empirical calibration.

The [separate review receipt](cli-replay-independent-review.json) verifies the frozen design, single issued marker, raw stdout/stderr and supervisor envelope, original script, actual producer binary/raw lock bytes, and the five replacement runtime files. The [runtime correction design](cli-replay-design-runtime-correction.json) SHA-256 is `0195e83b9215202eda20585622d282c7bbf2e0206648466dcbd3b5c708d38b73`; unchanged script SHA-256 is `cf7a5085df974af447178e328821b2ec41c883e27092929b790d25d670e12c9c`. Actual binary and lock hashes match `ccb09303ed954df3f07490cc4502efb5669eed27b30f8972df464741b315aec7` and `82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229`.

Saved stdout hash is `2d3fe03dce14c93b3b7c154444f2a788cba20eca1c9e099b3bf26b9d8ea2fb17`. All14 observations parse, exit is0 and stderr is empty. The existing saved assessment reproduces exactly without executing chemistry. Independent snapshot/event checks establish:

| Frozen requirement | Saved observation |
| --- | --- |
| Initial dissolved-state pH comparison | Both requested readings10.39568327534 before acid |
| Suspension consumes solid reserve while filtrate acidifies more | Suspension Mg(OH)₂ falls from1.986794373093 to1.494349940131 mmol; final pH9.671205408243 versus filtrate2.049931955107 |
| Fresh filtrate answer without transport-only shift | Filter gives complete solution; ΔT=0 K, ΔpH=0, retained-water difference0 mol |
| Correct solid separation | Filtrate contains no Mg(OH)₂; donor retains1.986794373093 mmol |
| Honest execution/scope | Four requested pH readings, zero solver_failed and zero not_yet_modeled events |

The final pH contrast is7.621273453136 units. Each challenged vessel receives the original1 mmol HCl. These observations satisfy the original qualitative criteria directly; they do not depend on inventing a new tolerance or interpreting exit0 as chemistry success. All additional already frozen native regression bounds also hold unchanged.

Independent decimal accounting checks cumulative operator feeds against retained contents in the entire bench plus actual exported gas at **each of14 observations**. Every represented H/O/Mg/Cl/formal-charge ledger is within the existing1e-8 mol audit bound; maximum residual is7.14656e-16 mol with decimal accumulation (the existing float reader reports its own roundoff). Exact-source `species.rs` derives `base_equivalents` from OH⁻, justifying its H/O/charge composition; measured free-ion fields, speciation distributions and lots are not counted again as matter. No surfaces or unresolved materials require an additional ledger here. This is material/formal-charge closure, not a full thermal/energy proof.

The matching Ubuntu24.04 loader, libc, libm, libstdc++ and libgcc byte hashes agree with [the runtime provenance](cli-replay-runtime-provenance.json). Static `readelf` inspection independently confirms every direct/transitive NEEDED name resolves within that five-file set. No target binary was executed locally. The preceding loader failure produced zero chemistry observations and remains separate; the SDK setup failure had zero CLI invocations. This successful attempt is a prospective changed-runtime acceptance, not a relabeling of either earlier failure.

**Manifest qualification:** the immutable raw manifest inherits base-snapshot `build_image` and `rustc` fields. They do not attest the separately hosted CLI producer or replacement runtime. Use the exact source/tree/binary/raw-lock keys and explicit runtime/design hashes; make no new build-image/compiler identity claim. This bookkeeping limitation does not invalidate the authenticated binary/library identities or physical observations and requires no chemistry rerun. SDK destroy return likewise does not independently prove VM deletion.

The earlier [final source review](final-source-review.md), original forecasts, c446 failure, test-access failure and all raw first outcomes remain unchanged. Existing25 successful CI jobs and33 successful fleet jobs, plus this separate CLI acceptance, cover the named narrow repair. No convergence thresholds, iterations, chemistry inputs, rate/heat parameters or forecasts were changed. Broader transfer routes, empirical heat/rate/solubility calibration and production deployment remain outside this acceptance.
