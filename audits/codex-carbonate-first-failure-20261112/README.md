# Carbonate codex claims: first failure

The macOS native job in [CI run 43316391517](https://github.com/CrispStrobe/kerotakis/actions/runs/43316391517/job/131525505849)
ran the workspace tests, then failed `kero codex lint` on source
`66d0d9704d5d443684acfefdadf0395f14c791c9`. The original complete log has
SHA-256 `dbe5757512a1785d90f57c5c250ef294ed4085aa627d940e39e124e8c4098bd6`.
Its reported disagreements were:

```text
baking-soda-vents-carbon-dioxide: claims pH 9.5–9.9, computed 8.04
bicarbonate-plays-both-sides: claims pH 5.4–5.8, computed 4.54
the-antacid-runs-on-arithmetic: claims pH 9–9.4, computed 8.00
```

These are model/lesson disagreements, not evidence that either the original
claims or the new values are scientifically validated. Investigate the
reaction and gas-boundary ownership, conservation and the wording in every
supported language before revising expectations. Preserve the original
forecasts and do not widen checks merely to make them pass. Exact successor
source must pass the unweakened codex and release gates.
