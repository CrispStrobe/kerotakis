# QA-003: liquid-volume supplement

The first saved reader preserved the two gas-meter refusals but did not examine
the scene payload. Its assessment, reader and independent-review hashes remain
unchanged. This supplement uses the existing raw output and changes no protocol,
original forecast or instrument semantics.

At the two pre-titration measurement steps, the scene already reports liquid
volumes of0.15 L and0.05 L. Dividing the respective endpoint base doses by those
volumes gives0.0100000037511 and0.0100000037193 mol/L. Their difference is
3.1789e-11 mol/L, within the original dosing resolution. The physical
concentration-agreement forecast is therefore met using existing reported
liquid volumes.

The requested gas-meter readings remain unsupported. Scene volumes are computed
model observables, not independent instrument readings. There is no new
liquid-volume instrument, inferred empirical measurement or calibration claim.
The [reader](review_scene_volume.py) authenticates the exact saved stdout and
checks the measurement-step scene values; its [result](scene-volume-assessment.json)
is separate from the first assessment. No experiment ran again and the working
catalog remains unchanged.
