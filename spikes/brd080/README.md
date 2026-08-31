# BRD-080 disposable viewer spike

This directory measures viewer candidates; it is not part of the application
dependency graph. Run `npm ci`, `npm test`, then
`node evidence.mjs > evidence.json`. The
evidence command is offline after `npm ci`, refuses an unlocked dependency or
missing fixture, and writes build products only beneath the operating-system
temporary directory.

The five tiny fixtures are project-authored format probes, not scientific
reference data. They test parser/rendering paths only and carry no physical or
computed-property claim.

`src/measure-*.ts` are deliberately separate from the interactive adapters so
the closure measurement cannot silently shrink to a lazy wrapper. Build and
exercise the disposable route with `npm run build` and `npm run test:browser`.

The route is implemented as an isolated Svelte component. After deploying the
contents of `dist/` to an HTTPS origin, run the keyboard/mobile-profile check
with `BRD080_ORIGIN=https://example.invalid npm run test:hosted`. This
Playwright profile is deployed-origin evidence only; it deliberately identifies
its emulation and cannot satisfy the physical-device gate documented in
`../../tools/brd080-device-evidence.md`.
