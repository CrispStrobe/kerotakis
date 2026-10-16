# First transport temperature baseline — preserved failure

[Run38884179479](https://github.com/CrispStrobe/kerotakis/actions/runs/38884179479)
executes both frozen controls once on the authenticated c446/670 native engine.
Both commands exit zero with no parse errors or retries. The
[assessment](assessment.json) finds receiver temperature errors of−40K and+40K:
the receiver follows the inlet rather than the outgoing cell. Cell temperature,
represented H/O/charge and unchanged representative inlet checks pass.

Raw first streams, scripts, artifact digest, source/binary/lock binding and
completion records are preserved here. These are two demonstrated failures,
not working-catalog additions or empirical heat calibration. A repaired-source
replay must have a new binding and preserve these results.
