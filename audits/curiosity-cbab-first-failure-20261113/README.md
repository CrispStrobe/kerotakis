# Curiosity coverage: first failure on `cbab23b6`

[Linux native job 131593300068](https://github.com/CrispStrobe/kerotakis/actions/runs/43335619428/job/131593300068)
passed the codex lint but failed `kero coverage curiosity --check` on
`cbab23b6f36532a1628bcd8984b48514ae08d4d0`. Its original complete log
SHA-256 is `30957ba42c4db42a13e7045d0aa26516b737bd735de656e7d919d0e8469dbaa3`.
The gate reported seven solver/runtime failures: `mat-049`, `mat-054`, and
`mat-092` lacked thermodynamic data for `SiO2(L)` at 1773.2 K; `bio-086`
through `bio-089` refused a clock CO2 parcel without a certified native
component projection. The existing baseline had classified the four biology
cases as computed. The gate also reported three curated-to-computed changes
(`aq-059`, `aq-122`, `bio-114`); these require independent review rather than
an automatic baseline refresh.

The biology scripts add an explicit finite CO2 dose before illumination and
an hour's wait. `native_bio_curiosity_clock.rs` deliberately tests atomic
refusal of an uncertified finite gas dose; simply removing that safeguard or
reclassifying failures would not establish correct carbon accounting. Scope a
source-bound reproduction, review gas and aqueous ownership and conservation,
and add positive and negative controls before changing this boundary. Review
silica phase data, route and temperature coverage independently; do not invent
thermochemical values. Preserve raw outcomes and original baseline records.

The previous browser, source-bound fleet, and full preflight passes do not
cover this later curiosity gate. The PR and release remain pending a reviewed
fix and exact-head acceptance.

A bounded native replay of the unchanged `bio-086` script, using the
previously authenticated fleet CLI binary SHA-256
`04aefaa6bccf91bfd08584e9991c69c19dad88c29ae7d31ab44034c9a3b61b2a`,
exited zero but emitted `solver_failed` during `wait 1h`. Its explicit
0.01 mol CO2 addition first emitted 0.003420321395285 mol absorbed and
0.006579678604715 mol evolved; a subsequent observation emitted a much smaller
CO2 evolution event. The refusal appears during the later clock step. This
replay narrows the investigation but does not prove which inventory field
prevents certification or justify removing the atomic-refusal guard. Preserve
the raw replay, producer identity and the full initial failure when repairing
and validating this lane.
The saved diagnostic input SHA-256 is
`d93de0ebe65fd235be5d34f793a7c28b8e15dca8bf014aad569acb144df574b8`;
its complete stdout SHA-256 is
`90bddb7dfdec0d63145c73982e98fb2fe5658623bff0c96ff739fa026901ab1a`.
The CLI and its genuine producer lock were authenticated in the preceding
source-bound fleet. This replay is an additional diagnostic, not a new
qualification or validation of the whole successor.

A separate bounded replay of the unchanged `mat-049` script with the same
fleet binary exited zero but emitted the same `cea-thermal` solver failure on
its heat step. The retained input SHA-256 is
`5a09a64098b27e4ca2111a736331fb5c424377e6037f112c1d77a0995ea1741c`
and stdout SHA-256 is
`1907f185da3665a96ae9b48a12201e84597a14b3f1b4fb1732dc04a7eb9277a9`.
A zero CLI process exit does not make a solver-failed experiment successful.

For the finite-dose lane, start at `project_gas_aware_components` in
`crates/kerotakis-phreeqc/src/aqueous.rs`: its certification path explicitly
returns false for `ExternalGasKind::Dose`, while a clock parcel requires a
certified projection. Trace the dose, any remaining gas, the clock parcel,
and aqueous carbon through the first dose and later wait; distinguish a
certifiable exhausted dose from gas still requiring its own owner. Add a
positive source-bound biology control and negative unrepresented-ion, opaque
stock, remaining-gas and no-double-count controls. Preserve the existing atomic
rollback test and compare complete inventories, charge, and reported external
gas events. These are acceptance requirements for a proposed implementation,
not a claim that the guard can already be relaxed safely.

For the silica lane, reproduce all three original firing scripts and trace the
selected phase and temperature interval through the CEA data lookup. Establish
whether the `SiO2(L)` request is physically appropriate at 1773.2 K, whether a
supported solid phase should retain ownership, and which reviewed source covers
any genuinely required transition. Check energy and mass closure and adjacent
temperatures before adding data or changing route selection. Preserve the first
failures and keep unsupported phases as explicit coverage limits.

The retained `mat-054` replay independently reproduces the same CEA failure
on heating silica glass; its input SHA-256 is
`a2e208339930da023a02234d5603f1342f765604457581774e39de6d2ec575be`
and stdout SHA-256 is
`a20bc4aee97eb3e889a02ab4df6e8728aae38fa7af95ab35fc20eb5ca98660a4`.
The `mat-049` and `mat-092` corpus scripts are identical, so their two gate
records are not two independent physical protocols.

The macOS native job 131593300149 independently reported the same seven
solver failures and seven baseline changes. Its complete original log SHA-256
is `801128706868ace4053109c6c8a85b1107c549c813e4779ca31ac78c1539ed47`.

A preliminary source check strengthens the phase-selection question without
settling this mixture's equilibrium: a [National Bureau of Standards report](https://nvlpubs.nist.gov/nistpubs/jres/17/jresv17n5p727_A1b.pdf)
reports cristobalite melting at 1713 °C (1986.15 K), above the failing 1773.2 K.
An [NBS thermal-conductivity volume](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nbsspecialpublication302.pdf)
distinguishes vitreous silica, which has no sharp melting point, from the
crystalline form. These sources do not prove a clay mixture's phase or supply
an implementation-ready latent-heat model; verify the specific CEA species,
mixture and transition ownership before changing the solver.
The repository's reviewed transition table also records 1883 K for metastable
quartz fusion and 2001 K for cristobalite fusion from NBS Circular 500; both
exceed the failing 1773.2 K. This corroborates investigating a request for a
liquid-phase record below its valid interval. It does not establish whether
that temperature is the final physical state or an intermediate solver trial.

The retained `bio-086` replay shows no free CO2 gas portion after the explicit
dose: approximately 0.003407817 mol is in aqueous CO2 and 0.000012504 mol in
bicarbonate. After the failed wait, the pending clock transfer is about
−0.001867128 mol and the prior aqueous carbon inventory remains in place while
the solution is invalidated. This narrows the question further, but the
internal `ExternalGasKind` inputs and their ownership still need tracing;
absence of a visible gas portion alone is not proof of a safe projection.
