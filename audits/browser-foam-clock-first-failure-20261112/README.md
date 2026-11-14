# Browser foam observation: first failure

The GitHub Actions browser job for PR #881, source
`7bd82c8041fc9d67a8e5400bf054dc53b9c0a0cd`, failed on 2026-11-12
(job 131491254915, run 43306467715). Its original log SHA-256 is
`821f0b99781b674e0152130081cc5ae90783cba92c71b4e34067e4ccf8bfb67f`.
The original log is retained as evidence. The relevant assertions were:

```text
ok   the browser computed foam in both dose vessels
FAIL the high-dose vessel visibly overflowed
ok   the browser rendered both foam columns
FAIL the browser rendered the overflow outside the glass
ok   the higher KI dose produced more foam
ok   the foam browser scenario completed
```

Commit `f83088434fd0076c6bff5228818e8b652ca51436` replaced an editorial
peroxide–iodide demonstration rate with a measured, approximately 15-times
slower rate. The browser self-test still expected the spill after the shelf
interaction's at most ten-second interval. Existing native foam controls
observe the iodide experiment after 300 seconds. The candidate changes the
browser self-test to issue an explicit additional 300-second wait and retains
the original overflow and rendering assertions; it reports numeric foam values
to aid diagnosis. The first vessel also experiences the second dispense's
interval, so this browser scenario is not a matched-clock kinetic comparison.

This diagnosis and the changed observation interval are not evidence of a
passing browser run. A real browser run against the repaired source remains a
release requirement.
