# Would the suite notice if the engine were wrong?

*An instrument, its first measurement, and — at equal length — what the
measurement cannot say.*

Three times in the last month a check here read stronger than it was, and
each was found by accident: a solver grid that stopped exactly where the
solver stopped converging; a prose lint whose headline rule reached one
quest file of thirty-six; and a buoyancy assertion that a deliberately
broken engine passed, because the assertion read the scene's position
while the comparison is made twice. The third is this file's seed, and the
agent that found it drew the distinction everything below is built on:

> **Mutating a test shows an assertion is live. Mutating the engine shows
> whether the assertion is aimed at code that could break.**

Only the second found the gap. It did it by hand, for one case. Nothing in
the tree does it systematically, so nobody knows which of the thousands of
assertions here would notice if the code underneath them changed.

## The bounds, declared before the first run

This is the most build-hungry instrument anyone could add to this
repository, and the bounds are part of the design rather than something
discovered at ninety per cent:

| bound | value | why |
|---|---|---|
| surface | one module family in `kerotakis-core`, not the workspace | a full workspace pass is days |
| wall clock | a stated ceiling per run, enforced by the harness | the box is shared |
| disk | one shared `target/`, deleted before the branch closes | the volume is at 89 % |

## Status

Draft. Surface selection, harness and first score land in this branch.
