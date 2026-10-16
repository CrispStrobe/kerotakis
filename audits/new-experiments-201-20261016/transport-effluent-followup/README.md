# Prospective transport effluent temperature controls

These two separately frozen controls target the actual-main bench adapter's
loss of parcel temperature when collecting transport effluent. Each moves one
20mL water cell into an empty receiver while the representative inlet is at a
different temperature. The receiver should take the original cell temperature;
the cell should take the inlet temperature, within the prospective0.01K bound.

The [design](design.json) preserves exact commands, forecasts and accepted
c446/670 runtime identity. The runner reuses that authenticated artifact once
per case, with unchanged1GiB/15CPU/30s/1MiB limits; it never rebuilds the app.
This is a targeted follow-up, not a blind experiment or a native execution claim.
Actual execution and physical assessment remain pending at this freeze.

The existing inlet is a representative external composition. Count injected
water explicitly; do not infer missing matter from the unchanged inlet vessel.
The original TR003/TR007 uniform-volume refusals remain separate. No hydraulic
guard, chemistry, geometry, heat parameter or earlier forecast changes.
