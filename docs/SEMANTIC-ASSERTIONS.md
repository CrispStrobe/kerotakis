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

`kind` is `equal`, `increasing`, `decreasing`, `conserved`, or `unchanged`.
Every assertion needs at least two samples. `increasing` and `decreasing` are
strict after applying `tolerance`; the other three require every value to be
within `tolerance` of the first. The default tolerance is `1e-9`.

`step` is `initial`, `final`, or `after:N`, where N is the one-based count of
parsed operations (comments and blank lines do not count). A vessel is named
by its script id (`v1`, `v2`, …). Metrics are `mass_g`, `temperature_c`,
`pressure_kpa`, `elapsed_s`, `ph`, and `moles:SPECIES`. Omitting `vessel` sums `mass_g` or a
`moles:SPECIES` inventory over the whole bench, which is useful for
conservation claims; non-additive metrics require a vessel.

`kero codex lint` captures state after every operation from the actual engine
replay and fails on a false claim, an unknown metric or vessel, an unavailable
pH, or an out-of-range step. Assertions are exported with the rest of
`expect`. The script-backed Codex result view shows scene-projected values
without reinterpreting prose. Species-mole relationships remain visibly marked
unavailable there because Scene v1 does not expose complete species inventory;
the engine-side lint still verifies them.
