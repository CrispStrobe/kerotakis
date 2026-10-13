# Corrected PR785 comparison protocol, version2

Current source is `d06ffe3a` (tree `25d1dd49`), after the test-only locale-reader
justification repair. Own CI38387729203 and fleet38387729183 must supply new
source-bound outcomes and artifact review. The earlier `55359032` artifact and
failed gates do not certify this head.

Use the entry points [controls.py](controls.py) and [paired.py](paired.py).
They select this definition namespace and reuse the reviewed shared consumers;
the freeze hashes both wrappers and shared implementations. Original version1
files and first failures are preserved. [The receipt and execution protocol](../PAIRED.md)
otherwise applies, with this version's head, definitions and freeze.

All seven control scripts and expectations are unchanged; their saved baseline
streams are rebound explicitly to this source revision without rerunning any
baseline admission. The four original performance pilots and the unadmitted
safety outcome remain unchanged. Actual candidate execution, paired numbers and
new own CI acceptance are pending; PR785 remains a draft.

The existing baseline binary is still required by this version. If compiler or
native-flag witnesses are missing, use a controlled rebuild of both sources and
a further version that binds those actual rebuilt bytes before measurement.
The supplied runtime binaries and source snapshots must both reside on the same
fast filesystem; confirm this in the operating review before timed execution.
