# Source candidate acceptance — 2026-11-12

Candidate `7bd82c8041fc9d67a8e5400bf054dc53b9c0a0cd`, tree `98c9aec404a70432f3ba97b66beb4a8e96095df0`, passed the hosted source checks: 4,410 Rust tests across 491 suites (4 ignored), formatting, workspace Clippy, 1,727 frontend tests, frontend type checking, locale checks and independent saved-evidence corruption controls.

The authenticated native binary SHA256 is `4c3ac9945c990ae405d9cfa0d0b0fc78060cfcac5621adab32beae9f3fc95fe4`; the genuine producer lock SHA256 is `a43bbe9a6ce7be1b5ecbbfbf091e25124c60122b08bd2ba003c8e250d3262bdd`. The lock is retained verbatim as a producer artifact; it is not represented as a source-committed lock.

The full own-fleet run used that exact binary and lock. All 33 jobs and 142 authored shell blocks passed; 29 actual case summaries contain 702 records, and 29 law reports contain 991 checks with zero unmet. The complete 2,694-file raw inventory was hash-verified after a collection timeout; only the 101 missing files were recovered, without rerunning experiments. The retained summaries and law reports are actual producer output. They do not establish empirical calibration or add original experiment qualifications.

`producer/COMMAND-OUTCOMES.json` is a derived view that omits private runtime paths; its recorded raw hash binds the unmodified producer receipt. Earlier candidate failures and carbonate first-failure evidence remain separately preserved in the linked audit scopes. This source result does not replace required protected CI, normal integration, deployment checks, or Apple build and TestFlight readback.
