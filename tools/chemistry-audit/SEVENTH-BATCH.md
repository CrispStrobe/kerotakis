# Seventh fleet contract: cases 459–488

Status: case inputs and intended physical laws were frozen before the first
execution. The first analyzer implementation was then corrected for unit and
species-accounting errors documented in `HISTORY.md`; observed output must not
be used to loosen a physical threshold or reverse an expected ordering.

The 30 original scripts exercise four repaired generic capabilities: dissolved
ammonia partitioning into finite headspace, reversible explicit esterification,
positive and negative classical gas-test thresholds, and pressure/venting
relations. Every script must complete with finite nonnegative inventory and no
solver failure.

Checks use conservation, the declared ester equilibrium quotient, matched
controls, monotonic relations, and the ideal gas law. A successful process is
not a scientific pass. Missing or malformed output fails closed. Production
code must never read case IDs or this audit module.

The fleet deliberately does not claim acid–metal rates. The comparative
curiosity scripts now contain matched vessels, but the engine still treats
acid–metal displacement as an instantaneous equilibrium. Surface-area and
time-dependent acid corrosion remain an explicit model gap pending reviewed
kinetic parameters.
