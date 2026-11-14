# GUI003 carbonate model snapshot adjudication — 2026-11-12

[Browser job 131525505922](https://github.com/CrispStrobe/kerotakis/actions/runs/43316391517/job/131525505922)
on reviewed head `66d0d9704d5d443684acfefdadf0395f14c791c9` passes the repaired
foam checks, then fails the unchanged five-lesson semantic DOM comparison.
The full original golden and the complete actual first failed DOM readback are
preserved here, together with the exact eight-leaf delta. Nothing in the other
four lessons, the cabbage colour names, other three cabbage vessels, quantities,
commands, or DOM comparison predicate changes.

## Independent cause and readback

`0f992bd8b` corrects the instantaneous gas boundary in `Bench::step_with_solver`:
all zero-duration open operations, including `measure` and `look`, use the bubble
pressure boundary. Previously only additions and zero-time waits did so; observing
an open carbonate vessel could reopen the room-air equilibrium sink. The native
owner in `aqueous.rs` uses `zero_time_dose` and `co2_film_boundary` to distinguish
this from finite-time film transport. This lesson has no elapsed-time commands.
See [the carbonate boundary scope](../../docs/carbonate-boundaries-20261112.md).

One separately collected native replay uses the unchanged original lesson and
the authenticated CLI artifact from the same CI merge source
`c6f74c6ed07cb1734e7343835b44f6e70d28c5fc`. Its command is
`kero run cabbage-rainbow.lab --json`, exit 0, wall time 1.08s, with 30s CPU and
2GiB address-space bounds. It is a new bounded diagnostic, not a relabelled fleet
execution. `bindings.json` pins source, binary, genuine producer lock, artifact,
input, original browser log and raw native stdout. Compressed stdout expands to
the exact original bytes; stderr is retained. No build or network call was needed.

Every changed browser number agrees with the independent native scene at the
existing DOM rounding. `independent-readback.json` supplies the full numbers.

| Vessel / field | Original snapshot | Current browser and native snapshot | Owning calculation |
| --- | --- | --- | --- |
| v2 temperature K | 298.12 | 298.27 | Retained carbonate/gas stock and its modeled thermal pose |
| v2 boiling K | 373.15 | 373.17 | Current dissolved particle/solvent-activity transition owner |
| v2 liquid sRGB | 35,0,93 | 60,0,51 | Current pH-dependent pigment spectrum and Beer–Lambert calculation |
| v2 liquid L | 0.109726 | 0.109725 | Retained liquid-water stock, additive-volume approximation |
| v3 temperature K | 296.85 | 297.28 | Retained carbonate/gas stock and its modeled thermal pose |
| v3 boiling K | 373.28 | 373.34 | Current dissolved particle/solvent-activity transition owner |
| v3 liquid sRGB | 0,0,168 | 8,0,164 | Current pH-dependent pigment spectrum and Beer–Lambert calculation |
| v3 liquid L | 0.110147 | 0.110030 | Retained liquid-water stock, additive-volume approximation |

V2 now retains pH 3.930885 and dissolved CO2 partial pressure 0.911001atm after
its actual initial 0.010249006mol CO2 export. V3 retains pH 7.919497 and CO2 partial
pressure 0.111216atm with no gas export. Both are supersaturated relative to room
air, consistent with a zero-time observation that cannot perform film diffusion.
Their ionic strengths are 0.000118713 and 0.205273mol/kgw within the returned
WATEQ route's stated range. Both remain qualitatively the lesson's deep purple
and blue. The higher retained carbon changes pH and particle counts; the associated
stock/speciation and thermal pose explain the eight coordinated scene changes.

`review.py` independently sums known Na/C/H/O and charge from input lots and
returned stock plus actual gas. The largest residual is 1.30e-14mol/eq against
the existing 1e-10 bound. Pigment remains exactly 6.690753378830455e-6mol and
the 0.197g opaque juice portion remains separate; no opaque formula is invented.
The entire returned vessel is identical before and after each of its two
observations, with no gas or thermal event on either observation. The reader also
requires all eight native/DOM values to agree, permits exactly these eight leaves,
and verifies the saved file hashes. Run `python3 review.py` from this directory.

## Acceptance boundary

The eight-leaf update pins these reviewed model observations. It does not claim
empirical colour, beaker gas-transfer, finite-concentration heat or real cabbage
buffering calibration. The pigment ladder contains editorial optical parameters;
the liquid volume remains an additive approximation. This readback independently
checks material/charge closure and stable observations, not a newly derived
calorimetric energy budget. Existing native heat and carbonate controls remain
mandatory. No original experiment earns a new qualification here.

The saved reader and whitespace checks pass. The exact successor must still pass
the real-browser golden gate and all applicable source/platform/fleet acceptance.
The first failure remains preserved; the normal test's exact comparison and all
scientific assertions remain intact.
