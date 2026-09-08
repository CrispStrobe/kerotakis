# Semantic catalog assertions

An event check proves that something happened. A semantic assertion proves
the relationship an experiment advertises. Authors may add zero or more
assertions below a reaction's `[expect]` table; old entries remain valid.

```toml
[[reaction.expect.assertions]]
kind = "increasing"
tolerance = 0.01

[[reaction.expect.assertions.samples]]
step = "after:2"
vessel = "v1"
metric = "temperature_c"

[[reaction.expect.assertions.samples]]
step = "final"
vessel = "v1"
metric = "temperature_c"
```

`kind` is `equal`, `increasing`, `decreasing`, `conserved`, `unchanged`, or
`ratio`. Every assertion needs at least two samples. `increasing` and
`decreasing` are strict after applying `tolerance`; equality-style assertions
require every value to agree with the first. The default tolerance is `1e-9`.

`ratio` requires exactly two samples and a finite `ratio`; the second value is
checked against `ratio × first`. `relative_tolerance` is optional and defaults
to zero. Ratio and equality-style checks use
`tolerance + relative_tolerance × max(|expected|, |observed|)`. Ordered checks
continue to use the absolute `tolerance` as their minimum step.

`step` is `initial`, `final`, or `after:N`, where N is the one-based count of
parsed bench operations. Comments, blank lines, and shell-only reads such as
`inspect` do not count. A vessel is named by its script id (`v1`, `v2`, …).
Metrics are `mass_g`, `temperature_c`,
`pressure_kpa`, `elapsed_s`, `ph`, `moles:SPECIES`, and
`event:EVENT.FIELD`. Event metrics read one finite numeric field from the named
event emitted by an exact `after:N` operation. They do not accept `initial` or
`final`, a vessel selector, a missing event/field, or multiple matching events
at the same step. Omitting `vessel` sums `mass_g` or a
`moles:SPECIES` inventory over the whole bench, which is useful for
conservation claims; non-additive metrics require a vessel.

`kero codex lint` captures state after every operation from the actual engine
replay and fails on a false claim, an unknown metric or vessel, an unavailable
pH, or an out-of-range step. Assertions are exported with the rest of
`expect`. The script-backed Codex result view shows scene-projected values
without reinterpreting prose. Species-mole and event-field relationships remain
visibly marked unavailable there because Scene v1 exposes neither complete
species inventory nor replay event payloads; engine-side lint still verifies
them. The GUI must not substitute a badge, caption, or other scene proxy.
