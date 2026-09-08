# Chemistry audit toolkit

Reproducible chemistry probes and independent analyzers from the EXP-53
computed experiment audit. Production chemistry must derive outcomes from
general models; it must never dispatch on an audit case or embed an expected
result. Nothing in this directory is part of the shipped engine — these are
evidence harnesses, and they are never imported by a crate or the web app.

## What is here

| File | Role |
| --- | --- |
| `run.py` | Process-isolated recorder and the first fleet, cases 1–50 |
| `next_batch.py`, `third_batch.py`, `fourth_batch.py`, `fifth_batch.py`, `sixth_batch.py` | Frozen case sets 51–86, 87–122, 123–158, 159–194, 195–218 |
| `replay_catalog.py` | Replays 13 shipped catalog entries through the real CLI |
| `analyse.py`, `analyse_next.py`, `analyse_third.py`, `analyse_fourth.py`, `analyse_fifth.py`, `analyse_sixth.py` | Predeclared law, conservation and matched-control checks per fleet |
| `analyse_catalog.py` | Law checks for changed catalog entries, beside `kero codex lint` |
| `check_native_inventory.py` | Native aqueous element projection against the analytical inventory |
| `hbr_conversion.py` | Reproduces the reviewed HBr standard-state conversion from shipped data |
| `basis_records.py`, `merge_codex_snapshot.py`, `revision.py`, `obsolete_tests.py` | One-shot migration, merge, source-revision and disk-hygiene helpers |
| `standalone_batch.rs`, `hbr_native_preflight.cpp` | Low-resource native probes for the thermo kernel and the HBr preflight |
| `HISTORY.md` | The narrative record of what the audit found |
| `SIXTH-BATCH.md` | Frozen contract for cases 195–218; its repaired-head 66/66 run is archived |
| `CONTINUATION.md`, `INTEGRATOR-HANDOFF.md` | Superseded operating state of PR #504, kept as record |
| `source_fleets.py`, `source-fleets/*.json`, `analyse_source_fleets.py`, `aggregate_source_fleets.py`, `SOURCE-FLEETS.md` | Validator, recorder, named-relation analyzer, aggregate gate and frozen contract for source-informed cases 219–458 |

Python standard library only; no new dependency, no external dataset.

## Running

The analyzers with a self-test verify themselves against nothing but this
checkout, and are the fastest proof the toolkit still works:

```sh
python3 tools/chemistry-audit/analyse_fifth.py --self-test
python3 tools/chemistry-audit/analyse_sixth.py --self-test
python3 tools/chemistry-audit/hbr_conversion.py     # needs vendor/iphreeqc checked out
```

Fleet runners need a `kero` binary and a fresh output directory; they refuse to
reuse one, so a replay can never overwrite an earlier run's raw evidence:

```sh
python3 tools/chemistry-audit/run.py --binary target/debug/kero --out /tmp/fleet-1
python3 tools/chemistry-audit/analyse.py /tmp/fleet-1 --out /tmp/fleet-1/law-checks.json
```

The repository's `Chemistry audit fleets` workflow performs authoritative
current-main runs for frozen fleet 195–218 and ten source-informed family
shards covering 219–458. It builds the CLI on GitHub's runner,
records each case in a separate process, binds the harness and frozen contract
by SHA-256, applies the independent analyzer and uploads the complete evidence
whether the checks pass or fail. This keeps expensive fleet work off the small
deployment host and makes a failed experiment an artifact rather than lost log
text.

Run 34200411053 established 66/66 for fleet 195–218. Its 78-file artifact is
archived at `/mnt/storage/kerotakis-archive/chemistry-audit-sixth-195-218-36eb425c.tar.gz`
(SHA-256 `5f19afcf35ffdf8047a43eaaf19429ec2c35fc24baaf315cbe37a6855f177459`).
The same run's first 219–458 execution, including all failures used by the next
repair, is an 821-file archive at
`/mnt/storage/kerotakis-archive/chemistry-audit-source-219-458-36eb425c.tar.gz`
(SHA-256 `98cbfe7e1c82c57edd3f5868840df40da1d6e77eecf53a9d27bcfc1912a73808`).

`basis_records.py` is a completed one-time migration and now refuses by design;
`merge_codex_snapshot.py` requires an unmerged index; `obsolete_tests.py` is
tied to a 2026-09-06 build-cache cutoff and to a worktree with
`target/debug/deps`. All three exit with their reason rather than acting.

## Evidence archive

The 2,260 raw run records the audit produced are **not** in the repository —
27 MB of them would have been a third of the checkout. They are archived whole,
including copies of every file in this directory:

| | |
| --- | --- |
| Path | `/mnt/storage/kerotakis-archive/chemistry-audit-evidence-20260906.tar.gz` |
| SHA-256 | `9053802227ce2fc57ed74d65d6074a661f30affe8fdaf40e2d8e724be759bb28` |
| Size | 3,120,920 bytes compressed; 30,289,920 bytes as tar |
| Contents | 2,260 files (2,971 tar entries) under `tools/chemistry-audit/` |
| Source | commit `a96c4a74a0af55810040bad284b146f4e45316c9`, branch `audit/chemistry-experiments-20260906`, PR #504 (closed unmerged; its product half merged as #529) |

The records are the raw input and output of 231 process-isolated CLI runs:
per-case `.lab` scripts exactly as executed, full `stdout.ndjson` and
`stderr.txt`, per-fleet `summary.json` with exit state and timings, the law-check
reports the analyzers wrote, native element-inventory comparisons, `source.patch`
snapshots of the engine revision each fleet ran against, and SHA-256 manifests
binding each run to that revision. They are what makes EXP-53's result
checkable rather than asserted: 194 original cases accumulating 565 predeclared
conservation, model-law and matched-control passes, together with the failures —
a refused-operation mutation and a false zero-extent/missing-reactant diagnostic
— that were retained, repaired generically, and covered by regression tests.
This is evidence for declared model domains, not empirical certification.

### Verifying the archive

```sh
sha256sum /mnt/storage/kerotakis-archive/chemistry-audit-evidence-20260906.tar.gz
# 9053802227ce2fc57ed74d65d6074a661f30affe8fdaf40e2d8e724be759bb28

tar -tzf /mnt/storage/kerotakis-archive/chemistry-audit-evidence-20260906.tar.gz \
  | grep -vc '/$'
# 2260
```

`/mnt/storage` is the project's CIFS share on the build host, so this is
checkable from that host only; the SHA-256 above identifies the file wherever a
copy is kept. The archive was produced by `git archive` from the commit named
above, so it can be rebuilt byte-for-byte from Git as long as that branch
exists — it was not deleted:

```sh
git fetch origin audit/chemistry-experiments-20260906
git archive --format=tar a96c4a74 tools/chemistry-audit | gzip -9 | sha256sum
```

The uncompressed tar is `183a3310e5ab201fbf203fa273765a48e0b6b5e8ae213e06628f0e99581c6c7d`
(git 2.43.0). Compare that digest rather than the compressed one if it
disagrees: gzip framing varies between zlib versions, the tar stream does not.
The archive was verified at creation by extracting
it and diffing the whole tree against the worktree at that commit — zero
differences — and its SHA-256 was re-read at the destination after the copy.

## Scientific contract

- Audit IDs label evidence only; runtime code must not recognize them.
- Checks use independent equations, stoichiometry, conservation or paired
  controls. Simulator output is never an oracle for its own expected range.
- Tolerances are absolute plus relative, and are frozen before execution. Do not
  widen them after observing output.
- Open, sealed, pressure-controlled and reservoir boundaries have different
  ledgers. Count transferred or escaped matter at the correct interface.
- Analytical inventory, free species, activity, phase and instrument response
  are different observables. Passing one does not certify the others.
- Equilibrium does not establish rate, catalyst, nucleation, apparatus or
  detection behavior. Unsupported endpoints must say what extension is needed.
- Missing output, solver failure, unsupported behavior and implausible output
  remain visible; they are not passes. Process exit zero alone is not
  scientific validation.
- Preserve failed runs. Never overwrite a run directory; use a new one.
