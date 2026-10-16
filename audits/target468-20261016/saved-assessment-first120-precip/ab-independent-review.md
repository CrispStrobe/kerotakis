# Independent saved acid–base assessment review

Disposition: no source/evidence blocker found for the **21 proposed bounded selections** in the reviewed acid–base assessment. This is an independent review of existing observations and the offline reader; it adds no catalogue case and makes no empirical, unrestricted phase, kinetic or full heat claim.

Exact reviewed files:

| Repository file | SHA-256 |
| --- | --- |
| `audits/target468-20261016/saved-assessment-first120-ab/review_saved.py` | `941aca6d60bb0e5c8741047fa6ba6f52483c5d10a1d6fb7b1acad7342032aef5` |
| `audits/target468-20261016/saved-assessment-first120-ab/assessment.json` | `6aca2726467a5ff74193714aee98b816b08408e76ca28b57f1c89a9a0fdfa010` |
| `audits/target468-20261016/saved-assessment-first120-ab/README.md` | `19aec55e07f5ed70dc9a6dc024f5f4f811f46ee1aecc7b38d69890000c57d871` |

The original dataset and artifact hashes were independently authenticated in the accompanying precipitation assessment. All 24 acid–base forecasts and 48 original criterion objects were additionally compared directly with that original dataset. Every final vessel snapshot and every reported pH observation was independently matched with the raw ZIP streams, retaining earlier owners when a later targeted inspection omits them. An offline execution of the Python reader reproduced the assessed JSON byte for byte. No simulator, API, native test, browser or build was executed.

The review checked the original criterion correspondence across AB-001–024. Twenty-two cases meet both bounded criteria; AB-018 remains held despite its two literal criteria being met. The proposed set is AB-001, AB-003–016, AB-019–024. The three holds are correctly distinct:

- AB-002-C2 lacks the specified single equal-size addition well before equivalence. The earlier half-millimole comparator ends at equivalence, while early increments are quarter-millimole. A normalized slope cannot replace that original control.
- AB-017-C1 lacks a sodium-acetate-only comparator with the same sealed boundary. Its intermediate ammonium-only state and the open AB-016 example do not supply both corresponding controls.
- AB-018 records the actual ferric/calcium acidity comparison and absence of an external hydroxide addition, but the named excluded iron phases prevent unrestricted selection. The actual small Fe(OH)3 inventory is not evidence of an added base bottle.

Material accounting uses canonical analytical portions, not free-ion diagnostics, solver-species tables or historical addition lots. `base_equivalents` supplies the represented OH basis and negative charge; measured free H/OH and `solute_charge` are not extra portions. Ammonia-family nitrogen is distinguished from trapped N2. The sealed initial atmosphere uses the recorded trapped-air amount with N2/O2/CO2 fractions 0.7901/0.2095/0.0004. No `gas_absorbed` event appears in this cohort, so the reader's general gas-input branch does not double-book an explicit gas dose here. Actual CO2 export events are counted once.

AB-012's decant and AB-022's mix use declared fractions of the previous liquid/aqueous portions. Final per-vessel ledgers supplement the whole-script ledger; both full-mix donors are empty. These checks support final represented material and charge closure, not per-step closure or enthalpy conservation. The existing 1e-8 mol audit reporting resolution is explicitly qualified and does not become a new physical forecast tolerance. All reported final residuals are much smaller, with the maximum approximately 2.204e-12 mol in AB-003.

The corrected snapshot integrity check permits omission of a known vessel only for the exact original targeted `inspect vN`, publishing that singleton unchanged with no events. Other operations must retain all previously known vessels. Thus a final inspection cannot silently erase the other control vessel from accounting. The quantitative direction comparisons now have assertions or explicit conditional outcomes. Original qualitative words such as “similar”, “near” and activity-qualified comparison remain visible manual semantic judgments with concrete values and no invented numerical pass band. AB-010 compares the actual 1.83332 pH difference with log10(81)=1.90849; this is the stated approximate comparison, not calibration.

Shared design authentication qualification: the original repository `native-screen-design.json` raw SHA is `e89c2f91b07ecb639436d75bae7417b5af9d3febfecb977519a2742fde692ea0`, matching `binding.design_sha256`. The archived `design.json` raw SHA is `8809b1558840709aed6f046087727a3d8b0a37fe031be1cf3a1e472c532ed05c`. Their complete parsed JSON values are identical; the archive uses ASCII Unicode escapes. Re-serialization with the original Unicode setting exactly reproduces the original bytes. Raw archive-design byte identity is not claimed. Binary and Cargo.lock identity remain producer-attested because their raw bytes are absent from this output archive.
