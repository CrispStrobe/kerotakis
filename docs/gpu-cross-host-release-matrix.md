# GPU cross-host release matrix

This matrix is a collection template, not evidence of a pass. Run it on each
physical release host; do not substitute CI, a VM, a simulator, or inferred
results. Keep the raw JSON artifacts with the release evidence.

## Exact commands

From a clean checkout of the candidate commit:

```sh
npm --prefix web/app ci
npm --prefix web/app run build
node tools/frontend-asset-budget.mjs --dir web/app/dist --json > frontend-assets.json
node tools/gpu5-probe.mjs web/app/dist --host-label "HOST-OS-GPU" > gpu-probe.json
git rev-parse HEAD
node --version
```

If the release has a checked asset budget, insert
`--budget path/to/budget.json` before `--json`. A budget file has this form:

```json
{
  "version": 1,
  "limits": {
    "javascript": { "rawBytes": 0, "gzipBytes": 0 },
    "css": { "rawBytes": 0, "gzipBytes": 0 },
    "all": { "rawBytes": 0, "gzipBytes": 0 }
  }
}
```

Replace zeroes with approved byte limits. Exceeding any supplied limit exits 1;
invalid input exits 2. With no budget, the command measures without gating.

## Physical-host matrix

| Host | Status | Commit | OS + version | Browser + version | GPU / driver | Adapter backend | Probe artifact + SHA-256 | Asset artifact + SHA-256 | Tester + UTC time | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Web (physical desktop) | PENDING | — | — | — | — | — | — | — | — | — |
| Android physical device | PENDING | — | — | — | — | — | — | — | — | — |
| iOS physical device | PENDING | — | — | — | — | — | — | — | — | — |
| macOS physical host | PENDING | — | — | — | — | — | — | — | — | — |
| Windows physical host | PENDING | — | — | — | — | — | — | — | — | — |

For each artifact, record `sha256sum FILE` (Linux) or `shasum -a 256 FILE`
(macOS). On Windows PowerShell use `Get-FileHash FILE -Algorithm SHA256`.
Change a row to PASS only after the release gate accepts the saved probe and
the asset command exits 0 against the approved budget. Record failures as FAIL;
never delete or relabel a failing artifact.
