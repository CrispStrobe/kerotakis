# Amount foundation integration, 2026-10-07

This first AUD-00 implementation slice exports the existing bounded compensated
arithmetic primitive through `kerotakis_core::amount`. It does not migrate any
vessel, stock, still, solver or persistence owner on main.

Source: [accepted implementation](https://github.com/CrispStrobe/kerotakis/blob/8cdab1f6ecd0b1132c4008014a5a0c5476429d17/crates/kerotakis-core/src/amount.rs).
The production module is byte-identical to that source. The 15 original contracts
are preserved byte-for-byte in [the original snapshot](amount-original-contracts.rs.txt).
The compiled [integration test](../../crates/kerotakis-core/tests/amount_foundation.rs)
imports the public production API and is formatted for repository gates; this
adaptation preserves assertions, but its file hash differs from the original.
[The port receipt](amount-port.json) binds both forms.

Coverage includes cancellation, tiny debit recovery, overdraw, malformed
persistence, overflow, scaling and subnormal limitations. This is bounded
two-component arithmetic, not exact arbitrary-precision inventory. It does not
close NUM-01 through NUM-04.

Validation: hosted PR gates pending. Original audit evidence remains evidence for
its original source, not a fresh acceptance of this main-based port.
