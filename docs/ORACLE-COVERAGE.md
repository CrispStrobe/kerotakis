# Build-time oracle coverage

*Tier C of the three-tier plan in [`EXPERIMENTS.md`](../EXPERIMENTS.md)
("How the expectations get checked: three tiers"). Tier A is perturbation,
tier B is authored expectations; this file is about the middle one —
external oracles, widened to whatever they can legitimately reach.*

This file answers one question per row: **what is this oracle independent
of?** The rule it is written against is
[`ROADMAP-Webapp.md`](../ROADMAP-Webapp.md)'s, in that document's own words:

> agreement between two paths using the same database is useful but is not
> an independent validation of that database.

An oracle sharing a database with the path it checks proves the code reads
the database. That is worth having, and it must not be written down as more
than that. Two recent findings are why the sentence is repeated here rather
than assumed: three of the colligative accuracy family's six rows turned out
not to be independent of the path they test, because the coefficients being
checked had been fitted to the measurements used as the reference; and a
proposed corroboration was rejected because at infinite dilution a tracer
diffusion coefficient *is* a conductivity measurement in different units, so
the agreement was a round trip.

**The number that matters is quantities covered with a stated independence,
not checks passing.** A count of passing checks can be raised by adding easy
ones.

## Status before this document existed

(Filled in by the survey; see the sections below.)
