# Two matched low-pressure thermal controls

The first seven replayed controls produced five passes and two frozen failures.
The original sealed beakers burst after spark heating exceeded the simulator's
405300 Pa glass threshold. Both reported the burst and danger warning, and every
owned gas molecule was accounted for in vent events. Fuel was preserved and
unsupported thermal chemistry was refused. Those original results remain intact.

These two new scripts add only `sweep v1 0.1bar` before sealing, establishing a
lower-pressure nitrogen boundary. The original oxygen doses and all later
commands remain. This keeps ignition below the modeled mechanical burst threshold,
so retained-inventory assertions can test the finite thermal refusal directly.
They are virtual CLI controls, not a procedure or pressure rating for real glass.

Read [the expectations](expectations.json) and [the frozen protocol](protocol.json).
Current state: **2/2 executed and passed** at source `10c963e1`; see the
[verified outcomes](../pr786-low-pressure-outcomes.json). The complete result
archive, report, receipt, binary, lock and raw-stream digests were verified;
reanalysis of saved streams reproduced the hosted report exactly. These results
are bound to that source, not the later PR revision. The already passing five controls need no rerun at
unchanged source. The original two remain failed against their original intact-seal
expectations; a separate saved-output diagnosis verifies the reported burst budget.
Own artifact/source verification and resource gates apply to this follow-up.
Closed chemical equilibrium remains unsupported by the narrow repair.
