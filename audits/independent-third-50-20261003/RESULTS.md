# Third independent fifty: E101–E150

Original forecasts were committed as `cebb3365` before any of these scripts ran. Their SHA-256 is `14f28baca8ebac00c7b3e808490440e9f65f104b253a73dc6e70ac71bb2d2511`. Two fresh agents authored fifteen each from parser syntax only; the root authored twenty and retained knowledge from earlier repair rounds. No existing corpus selected the cases.

## Baseline evidence

Hosted run [37097623666](https://github.com/CrispStrobe/kerotakis/actions/runs/37097623666) executed all fifty scripts in both CLI modes using the hash-verified executable from successful validation run 37095659202, source `59c75fcd`. Forty-nine scripts completed; E147 explicitly refused unknown MgCl2. A zero process status was never treated as chemistry agreement. Raw outputs and hash-bound receipt are in `baseline/`; original case assessments are in `assessment.json`.

The current checker records 853 baseline checks, with 11 failed checks across six defect groups. These include the optical coverage checks added after reviewing the captured outputs; original forecasts remain unchanged. Capability and fixture qualifications are separate.

## General defects and repairs

| Trigger | General defect | Repair and regression scope |
|---|---|---|
| E105: partial pouring of settled chalk in brine | Undamped chemistry/temperature iteration oscillated, exhausting 64 passes and leaving the source uncharacterized. | Safeguarded signed-residual bracket and secant/bisection steps. Preserve strict temperature tolerances and genuine discontinuity refusal. Test both transfer sides and elemental ownership. |
| E128: 1 pmol HCl | A display-sized cutoff removed positive chlorine inventory during solver booking. | Preserve positive element/redox/gas/solid/interface quantities and acid/base conservation coordinates independently of display cutoffs. Repeated acid/salt solves span both sides of the old cutoff and dilution. |
| E129: equal concentration at 100-fold sample scale | Density coverage depended on absolute mole inventory. | Missing dissolved-ion volume data is reported for every positive ionic amount. Scale and pure-water controls. The missing density data itself remains missing. |
| E113/E123: change an existing sealed headspace | Text always claimed an open-to-sealed transition and called swept nitrogen trapped air. | Record actual prior boundary, render actual gas basis, preserve old event deserialization without inventing an origin. Exercise open/sealed/piston/swept states and all registers. |
| E110: four-stage partial ethanol cut | Fixed integration steps clipped a present component to exactly zero; endpoint equality was called an azeotrope. | Component-limited adaptive integration, explicit completion/precision refusal, relative composition comparison with at least two active components. Positive small cuts, scale and split-path consistency, pure/dilute non-azeotrope controls. |
| E107/E108: dissolved iodine optical readings | Missing neutral-solute spectrum coverage allowed zero numerical absorbance and omitted the appearance caveat. | Shared coverage recognizes positive dissolved I2 independently of native solution; numerical reading refuses and appearance warns. Water/hexane, microscopic amounts, clear/solid controls and bounded Lugol/starch tests. Existing qualitative colour surrogate is retained with a caveat. |

## Additional defects found by regression expansion

The first hosted repair validation showed that four-decimal native input temperatures were coarser than the required temperature residual. Eight-decimal solution and sealed-gas temperature input now preserves the question the thermal solver is actually solving. The safeguard also preserves the existing fast path when the fixed point contracts; the second run passed both settled transfer and the established native call budget.

The second validation passed 1,241 Rust tests but failed a new microscopic precipitation preparation. Investigation found that fixed nine-decimal solvent mass input serialized a positive picogram stock as zero. All three direct/MIX input builders now serialize solvent mass scientifically. The positive native control uses neutral AgNO3/NaCl feed at the established dilute concentration, checks repeated positive precipitation and strict relative Ag/Cl/Na/N budgets at two microscopic scales. A separate readback control prevents duplication of a positive trace solid into the aqueous budget. The original concentrated bare-ion preparation remains a diagnostic requiring conservation on completion or complete state preservation on explicit numerical refusal.

Failed validation receipts and logs are preserved in `validation-first-attempt/`, `validation-second-attempt/`, `validation-third-attempt/` and `validation-fourth-attempt/`. These failures are not accepted executable provenance.

The third completed repair validation passed 1,247 Rust tests and failed two native controls. Positive solvent serialization alone did not resolve the native microscopic matrix conditioning: nitrate had a roughly 0.09% relative residual. A diagonal-scaling attempt did not close the microscopic failure: native initialization already forces that setting. Review found an absolute native water floor and linear-solver tolerance, so the replacement repair normalizes tiny extensive native problems and returns their extensive outputs to the original scale before booking. Concentrations and nonlinear stopping tolerances are preserved. The old order-invariance assertion also relied on fixed-decimal mass rounding to conceal a tiny difference already present in committed water inventory. It now compares characterized kg per represented water mole at the original strict 1e-9 relative bound, retaining independent inventory, pH/activity and call-budget checks. This change is explicit rather than pretending two nonidentical inventories have an identical extensive mass.

Two intermediate validation runs were cancelled when additional related source fixes became ready (`37100906417`, `37101010230`); neither supplies accepted executable evidence.

The normalization activates below 1e-8 kg solvent and uses a 1 kg native reference. Every extensive phase, gas-volume and interface coordinate uses the same factor; MIX uses one shared factor. Intensive quantities and nonlinear stopping criteria retain their meaning. Extensive selected outputs return to the physical basis before redox search and caching. Exported physical caches load directly; raw live hooks use the normalization contract. Required selected columns, nonfinite input and unrepresentable readback fail closed. New controls compare positive precipitation at ordinary and microscopic scale, native MIX with direct preparation including initial H/O budgets, and serialized microscopic cache replay without an engine call.

## Validation

Focused hosted run [37104263790](https://github.com/CrispStrobe/kerotakis/actions/runs/37104263790) passed 72 native and 64 cache-only unit/integration test executions on source `e06111ca`. Its receipt is diagnostic-only; native positive precipitation, initial H/O MIX budgets, microscopic cache replay and the live-hook path all completed. Logs and receipt are in `native-scale-proof/`. Full hosted validation [37104654649](https://github.com/CrispStrobe/kerotakis/actions/runs/37104654649) passed all ten stages, with 1,318 Rust test executions, WASM compilation, original-fifty CLI replay and its assessment. Exact source: `4061a2804be6003fddf5b6ed1b76ce1d1a1a2e4a`; executable SHA-256: `80c4bf46f603a2a09cb2eea15eaae1e65a0e482819e3121bf1cf864864b3f4ff`. Logs, source-hash receipt and resolved Cargo.lock are in `validation/`; executable remains on the NAS. Repaired new-fifty replay [37105690481](https://github.com/CrispStrobe/kerotakis/actions/runs/37105690481) used that exact executable: 898 checks passed, zero failures, with nine separately recorded qualifications. All fifty original scripts ran in text and JSON modes (100 invocations), followed by two frozen positive controls in both modes (four invocations). Raw streams, hash-bound execution receipt and verification are in `repaired/`.

E110 now completes the one-stage cut but explicitly refuses its large four-stage cut, preserving the entire source and receiver state. This closes the silent clipping defect; it does **not** fulfill the original comparative physical forecast. F01 independently completes both one-stage and four-stage smaller cuts, conserves components and confirms enrichment with positive residues. F02 completes pure-water four-stage distillation without falsely reporting an azeotrope. E147 still explicitly rejects unknown MgCl2. Iodine numerical optical readings now disclose unavailable spectral data, and appearance carries its qualitative-model caveat.

Previous next-fifty replay [37105852199](https://github.com/CrispStrobe/kerotakis/actions/runs/37105852199) used the same executable and passed all 410 repair-contract checks across fifty scripts plus thirteen controls (126 CLI invocations). Its two declared limitations remain separate. The execution receipt and verification are in `prior-suite-replay/`; raw streams remain on the NAS. These passing contracts include explicit capability limits and atomic refusal, so they are not a claim that every original scientific forecast was achieved.

## Remaining limits

Unknown MgCl2 requires curated anhydrous/hydrate identity, measured solid density and heat capacity before registry addition; hydrate water and dissolution enthalpy must not be copied from calcium chloride. The Mg/Cl ionic masters already exist, so rejection currently occurs at the identity registry. Dissolved-ion partial volumes, several reaction heats and optical/retention data remain incomplete. Organic-extracted iodine uses the existing `aqueous` phase enum as generic dissolved-solute storage, which is an intentional schema convention. Review did reveal a separate missing-spectrum coverage bug: dissolved iodine could yield a zero absorbance reading without disclosing absent data. Appearance separately applies a polyiodide surrogate even to extracted iodine without the documented water/iodide preparation; that extrapolation is retained as a qualitative surrogate with an explicit incomplete-colour caveat. An explicit shared coverage guard now reports this limitation; no spectrum is inferred from solid colour. Resealing changes geometry without an explicit work protocol. Unnameable oxidation-state fractions also retain an absolute native noise tolerance; its relation to genuine trace budgets needs separate systematic study. None of these are claimed solved.

E106 uses a bulk-liquid decant, not upper-layer skimming. E126 adds water rather than making up to a target volume. E127 trapped atmospheric CO2 lowers the water-control pH, so the frozen 2-unit initial contrast is not met although the thermal contrast agrees. E140 limewater is destructive; E141 had explicit preseal environmental CO2 exchange. Their boundary/fixture qualifications are preserved, rather than retrospectively changing forecasts.

## Systematic coverage added and still needed

This round separates authoritative inventory from display resolution, checks native serialization at both small and large scales, couples temperature input precision to convergence requirements, tests transfer ownership on both sides, and makes incomplete observation data visible. Positive completed operations and atomically refused operations have separate assertions. A refusal is never evidence that a requested physical prediction was met. Checker mutation tests reject missing rows/events, wrong units, malformed numeric data, nonfinite/overflowing JSON and conflicting optical readings.

Remaining work should use the same contracts rather than adding isolated examples:

| Area | Required next evidence |
|---|---|
| Registry reachability | Curate anhydrous/hydrated identities and measured mandatory properties, then check lookup, dissolution, atom budgets and property coverage together. MgCl2 is the explicit example. |
| Neutral chromophores | Audit dissolved neutral species separately from native charged species; require solvent-specific spectral provenance for numerical readings. Existing iodine appearance is a bounded qualitative surrogate, not that provenance. |
| Native trace scale | Closed MIX now checks initial solvent H/O and cache replay preserves physical amounts. Expand this to reactive interfaces and complete mass with explicitly recorded atmosphere exchange. Do not confuse tiny absolute error with small relative error. |
| Oxidation-state representation | Distinguish native numerical noise from real unnameable trace valence fractions using relative conserved budgets before changing the remaining absolute tolerance. |
| Boundary energy | Define the work protocol for user-imposed headspace changes before claiming thermodynamic reversibility of resealing. |
| Missing extensive properties | Curate dissolved ionic partial volumes and reaction heats with conditions; solid density and default heat capacities cannot supply the missing solution data. |

## Resources and preservation

Two fresh agents were used for light design, review and edits when resource snapshots permitted it. Compilation and experiment replays used serial hosted jobs. Large executables and downloads remain on `/mnt/storage/kerotakis-maintenance-20261003`; the fast workspace contains source and small reviewable evidence. No local app/build execution was used in this round. Registered worktrees: 68; prunable: zero. Dirty or unmerged history was preserved. The prior inactive-build archive is separately hash-manifested in the thermal audit. `resources.json` records the local resource policy and final snapshot.

## Per-case baseline assessment

| Case | Experiment | Assessment |
|---|---|---|
| E101 | Unequal aliquots retain source ownership | physical agreement with observation limitation |
| E102 | Successive half decants form geometric leftovers | physical agreement with observation limitation |
| E103 | Filtration retains chalk but passes dissolved salt | physical agreement |
| E104 | Precipitate filtration versus homogeneous neutralization | physical agreement |
| E105 | Partial decant carries liquid while sediment remains | partial physical agreement; real solver robustness failure |
| E106 | Lower-phase drainage versus upper-phase decant | drain agrees; decant forecast unsupported by command semantics |
| E107 | Four fresh extraction portions versus one equal total portion | quantitative physical agreement; optical limit; organic dissolved-phase schema convention |
| E108 | Extraction receiver owns solute after reverse aqueous pour | ownership agrees; optical limit; same organic dissolved-phase schema convention |
| E109 | Distilling brine retains nonvolatile salt | physical agreement with observation limitation |
| E110 | Distillation stages enrich a volatile component | frozen comparative prediction agrees; additional numerical endpoint concern |
| E111 | Evaporation and captured distillation differ in bench ownership | physical agreement |
| E112 | Fixed-volume heating obeys pressure temperature proportionality | physical agreement including explicitly recorded ambient gas |
| E113 | Equal nitrogen amounts in unequal sealed headspaces | gas-law agreement; apparatus boundary energy ambiguity |
| E114 | Pressure regulation allows expansion during heating | physical agreement with ambient inventory adjustment |
| E115 | Opening and resealing cannot restore vented nitrogen | physical agreement with atmospheric-reservoir model limit |
| E116 | Sub-boiling sealed-water heat/cool cycle | pass |
| E117 | One heat pulse versus four equal pulses | pass |
| E118 | Water amount and heating energy doubled together | pass |
| E119 | Equal hot/cold water mixing versus direct midpoint preparation | pass |
| E120 | Added inert gas pressure increment halves with doubled headspace | pass |
| E121 | Gas amount and headspace scale together | pass |
| E122 | Sealed inert-gas temperature-pressure reversibility | pass |
| E123 | Isothermal headspace expansion and return | pass_with_rendering_defect |
| E124 | Repeated gas measurements must not consume or heat the sample | pass |
| E125 | Temperature units and instrument aliases give one physical state | partial_capability_limit |
| E126 | Direct versus staged water dilution of dilute salt | semantic_limit |
| E127 | Trace-acid aqueous heating versus pure-water thermal control | prediction_control_mismatch |
| E128 | Extreme strong-acid dilution approaches neutrality from acid side | real_defect |
| E129 | Intensive salt readings invariant under hundredfold sample scale | real_coverage_defect_with_physical_agreement |
| E130 | Dilution then partial evaporation returns salt concentration | partial_capability_limit |
| E131 | Acid/base order equivalence | physical agreement |
| E132 | Tenfold strong-acid dilution | physical agreement |
| E133 | Tenfold strong-base dilution | physical agreement |
| E134 | Pouring dilution versus direct water addition | physical agreement |
| E135 | Silver chloride precipitation order | physical agreement |
| E136 | Chloride limits silver chloride yield | physical agreement |
| E137 | Common chloride ion suppresses silver chloride dissolution | physical agreement |
| E138 | Repeated filtration cannot duplicate precipitate | physical agreement |
| E139 | Barium sulfate precipitation | physical agreement |
| E140 | Sealed carbonate requires two proton equivalents | qualified physical agreement; destructive test boundary |
| E141 | Bicarbonate versus carbonate at one proton equivalent | qualified physical agreement; preseal environmental exchange |
| E142 | Carbonate acid-dose ladder | physical agreement |
| E143 | Acid dissolves calcite with retained carbon | physical agreement |
| E144 | Too little acid cannot erase a calcite stock | physical agreement |
| E145 | Copper hydroxide consumes two hydroxide equivalents | physical agreement |
| E146 | Ferric hydroxide consumes three hydroxide equivalents | physical agreement |
| E147 | Magnesium hydroxide precipitation | unsupported ingredient |
| E148 | Magnet separates iron from silica | physical agreement |
| E149 | Magnet cannot convert ferric solution into iron metal | physical agreement |
| E150 | Multiple measurements leave a reactive sample unchanged | physical agreement with explicit coverage limits |
