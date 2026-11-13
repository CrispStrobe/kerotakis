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
