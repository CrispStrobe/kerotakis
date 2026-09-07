# Rebuilt refusal-fix replay — incomplete validation

This fresh replay preserves the output of the CLI rebuilt after the atomic
distillation-refusal repair. The recorder summary contains the binary hash and
source metadata. No executable was replaced while these cases were running.

All 36 cases were attempted. Cases 91–94 timed out at the unchanged 90-second
limit before emitting output; the other 32 exited successfully. Analysis found
155 passing checks and ten unmet checks tied to those missing outputs. Severe
shared-host contention was observed (load average approximately 89), but the
timeouts are not treated as passing chemistry or definitively attributed to
that observation alone.

The chained native-inventory command did not execute because the scientific
analysis exited nonzero. No third-native-inventory report is claimed here.
The earlier discovery run remains separately preserved. A clean fresh replay
and full CI are still required; focused atomic-refusal tests passing does not
substitute for either.
