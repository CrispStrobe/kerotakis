# GUI-098 GPU release evidence

Copy `gui098-gpu-release-evidence.template.json`, fill it from physical runs,
and validate it with:

```sh
node tools/gui098-release-audit.mjs evidence.json web/app/src/lib/ignitionFlameShader.ts
```

All five rows are release-blocking. `physical` must be `true`; simulator,
emulator, CI, or inferred values do not substitute for a device row. Record
raw artifacts outside the JSON and identify them with paths and SHA-256 sums.
Timing values are milliseconds. Each host records 10 cold starts and three
independent 600-frame runs (1,800 measured CPU/rAF frames after warm-up).
CPU-frame p95 must meet the BRD-072 9 ms governor in every run, and the
candidate-minus-baseline gzip delta must not exceed 64 KiB. Every fallback field means the SVG endpoint was observed throughout
that scenario. A template containing `null` is intentionally incomplete and
must fail validation.
