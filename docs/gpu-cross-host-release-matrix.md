# GPU cross-host release matrix

This is a collection template, not evidence of a pass. Run it on each physical
release host; CI, VMs, simulators and inferred results do not substitute.

## Exact commands

Build and measure both the approved lightweight baseline and candidate in clean
checkouts. Preserve each JSON file and its SHA-256 digest.

```sh
npm --prefix web/app ci
npm --prefix web/app run build
node tools/frontend-asset-budget.mjs --dir web/app/dist --json > frontend-assets.json
node tools/gpu5-probe.mjs web/app/dist --mode lightweight --host-label "HOST-OS-GPU" > gpu-lightweight.json
node tools/gpu5-probe.mjs web/app/dist --mode webgpu --host-label "HOST-OS-GPU" > gpu-webgpu.json
node tools/gpu5-release-evaluate.mjs --host PLATFORM \
  --baseline-probe gpu-lightweight.json --candidate-probe gpu-webgpu.json \
  --baseline-assets frontend-assets-baseline.json \
  --candidate-assets frontend-assets.json > gpu-release-row.json
git rev-parse HEAD
node --version
```

`PLATFORM` is exactly one of `web`, `android`, `ios`, `macos`, or `windows`.
The single-host evaluator deliberately reports `releasePassed: null`. After all
five physical rows exist, create a version-1 matrix manifest referencing each
row's four raw artifacts and run:

```sh
node tools/gpu5-release-evaluate.mjs --matrix MATRIX.json
```

Only that command's complete five-host result may claim release gate passage.

## Physical-host matrix

| Host | Status | Commit | OS + runtime | GPU / driver | Probe + asset artifact SHA-256 | Tester + UTC time | Notes |
|---|---|---|---|---|---|---|---|
| Web physical desktop | PENDING | — | — | — | — | — | — |
| Android physical device | PENDING | — | — | — | — | — | — |
| iOS physical device | PENDING | — | — | — | — | — | — |
| macOS physical host | PENDING | — | — | — | — | — | — |
| Windows physical host | PENDING | — | — | — | — | — | — |

Use `sha256sum`, `shasum -a 256`, or PowerShell `Get-FileHash -Algorithm
SHA256`. Record failures as FAIL; never delete or relabel a failing artifact.
