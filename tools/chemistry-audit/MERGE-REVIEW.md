# Safe integration review, 2026-09-07

The user requested checking CI, safely merging PR #504, then continuing.
This review integrates main `3860e36f4fc0787946bc017518607d197fe5e7ba`
into audit head `f33faa96ff1f5dcf4aa5d5ca5680e6dff2a934ad` before any
merge of the PR into main. No failing or incomplete gate is a merge approval.

## Conflicts and semantic integration

- The three textual conflicts were `codex/rates.toml`, `codex/i18n/de.toml`,
  and the codex export snapshot. Field-level three-way comparison found no
  overlapping value changes: preserve our finite-acid corrections and two
  lessons alongside main's new entries, translations, and progress metadata.
- Main requires authored `progress` on every entry. Endpoint refinement is
  `intermediate`; reverse mass action is `advanced`. All 110 entries have
  valid metadata. The snapshot is merged by ID/field and ordered by source
  TOML, not regenerated from guessed chemistry outputs.
- Registry/source review retains all eight aqueous-basis additions and
  main's liquid-nitrogen provenance/latent-heat update. The two missing KSCN
  and SCN- registry snapshot records are transcribed from the curated source
  using the actual serializer's field mapping; all 176 identity keys agree.
- Read-only review found no overlapping calculation lost between our solved
  phase readback and main's CEA atmospheric-energy changes, nor between our
  finite-acid/solvent code and main's scene/event updates. This is a bounded
  integration review, not independent validation of every upstream model.

## CI findings repaired before retesting

- Native macOS Clippy: replace an indexed loop with an iterator and move the
  redox-ledger test module after production items. No lint suppression.
- Web catalog labels/template and exact count updated for the merged 110
  entries. About notice and license HTML regenerated from complete locked
  dependency sources; no package-install scripts or application build ran.
- All 20 focused locale/catalog/About tests pass locally after integration.
- The live CLI provenance gate, unlike the shell gate, lacked the two reviewed
  federal public-domain identifiers. Added those exact **data-only** references
  with tests retaining GPL, NC, code-lane and unreviewed-reference refusals.

## Still required

The earlier WebAssembly gate exposed an independent phase-physics regression:
an unavailable aqueous engine prevented pure water from freezing. The fallback
now retains `SolverFailed`, clears stale speciation, and computes water phase
changes only for a verified pure-water inventory. Unknown and ionic mixtures
remain withheld. New tests cover freezing, melting, boiling, conserved mass and
latent energy at three scales, and mixture refusal; execution remains a CI gate.
Workflow concurrency supersedes only revisions of the same PR/ref, never other
worktrees' branches or their runs.

A local full web sweep found one further stale corpus-count assertion; updated
it to 110 codex plus 60 guided entries. Its focused suite passes all 26 tests.
The sweep itself was not clean (1,121 passed, one stale-count failure and one
worker timeout), so full web validation remains required. Pre-execution review
also strengthened third-fleet atomic-refusal physical-state checks and added
electrolysis atom ledgers including external gas exchange without changing
the predeclared numeric tolerances.

Two new lessons lack core frozen-output snapshot entries. Their entries must
come from actual test execution and be reviewed; they have not been invented.
CI now uploads generated `*.actual.json` files even on failure for that review.
The native matrix no longer cancels Linux when macOS fails, allowing both
platforms' independent results and the Linux fleet evidence to be collected.

The combined source requires fresh CI, including workspace tests, catalog
lint, provenance, and the experiment checks. Cases 87–122 remain designed but
not executed at this checkpoint. Recheck main/head identities and the final
CI conclusions immediately before any PR merge; never force an unresolved PR.
