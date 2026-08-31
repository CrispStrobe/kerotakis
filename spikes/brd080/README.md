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
