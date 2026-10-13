# Controlled-build comparison protocol, version3

This namespace binds rebuilt baseline `b1867d84` and candidate `1e94804f`
from controlled build run38396270148 at the unchanged source heads `2bf9c771`
and `d06ffe3a`. Both use captured lock `5135e2cc`. The original seven baseline
transcripts and four admitted canonical performance outputs remain unchanged;
the original failed safety forecast remains unadmitted.

Use [controls.py](controls.py) and [paired.py](paired.py). Both wrappers reuse
unchanged, hash-bound consumers. Version1/version2 and their first outcomes are
preserved. The seven passing version2 controls used ordinary binary `b290b436`;
replicate all seven on this rebuilt candidate before paired timing.

Definitions may be reviewed before execution. Execution requires accepted
source-bound independent artifact/build receipts and actual operating resource
gates. Bind their actual bytes through a separate runtime bindings file. Both
source snapshots and binaries must reside on the same fast filesystem, verified
by the operating review. The structural collection receipt is evidence of
collection only; it cannot satisfy independent artifact acceptance.

After all seven rebuilt-candidate controls pass and their raw evidence is
accepted, collect 64 serial paired samples across four admitted workloads using
the original alternating order, output equality, clocks and variance policy in
[PAIRED.md](../PAIRED.md). Review raw samples with [review-paired.py](../review-paired.py).
No execution, compiler acceptance, speedup or merge is implied by this freeze.
