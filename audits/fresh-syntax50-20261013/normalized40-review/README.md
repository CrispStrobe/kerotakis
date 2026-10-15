# Saved-evidence assessment preparation

`prepare.py` reads the forty-case runner's saved `cases` directory. It never
executes a CLI, downloads evidence, edits frozen inputs, or assigns a scientific
verdict. It checks the original design/runner hashes, exact execution binding,
script-to-ordinal mapping, saved stream hashes/sizes and any completion marker.
The output is a deterministic review queue with forecasts and saved JSON rows.
It refuses to overwrite an existing output file.

After verifying the hosted run, producer source, artifact association and archive
or selected-member receipts, invoke:

```console
python3 -B audits/fresh-syntax50-20261013/normalized40-review/prepare.py \
  --design audits/fresh-syntax50-20261013/normalized40/design \
  --evidence EVIDENCE_CASES_DIRECTORY \
  --out NEW_REVIEW_QUEUE.json
```

The evidence directory must contain the unchanged runner's `binding.json` and
case files, with `completed.json` only if the producer completed. Missing cases
are explicit. Orphan outputs and inconsistent bindings/checksums stop preparation;
keep them for investigation rather than deleting or overwriting them. Malformed
stdout and timed-out processes remain recorded diagnostics with pending verdicts.
Valid stdout rows on either side of malformed lines retain their original line
numbers; any parse error prevents whole-stream sequence verification. A completion
marker with a timeout is rejected because the frozen runner cannot produce it.
Eight MiB per file is an ingestion bound; an oversized file requires separate
bounded review, not a scientific failure. The reader validates saved evidence
consistency; matching a copied binding alone does not authenticate execution.
Apply host resource gates before consuming a substantial archive or report.

## Physical assessment

- Keep `ordinal`, original experiment ID, CLI `output_sequence` and bench-log
  `step` separate. Filtered case ordinal 23 is original experiment 24. An inspect
  command can share a bench-log step with another response. Match operators and
  the script commands before choosing before/after observations; this reader does
  not infer that correspondence from numeric step equality.
- Compare every applicable identity, direction, conservation relation and bound
  against the unchanged forecast and its stated assumptions. Inspect actual units,
  volumes, temperatures and equilibration status first. Record evidence row
  references and reasons for every verdict in a separate assessment document.
- Process exit zero, complete output, an inventory snapshot or internal
  conservation alone is insufficient for a physical pass. Distinguish actual
  physical failures from explicit unsupported mechanisms, semantic ambiguity,
  incomplete execution and insufficient observations.
- Preserve first outcomes. Group demonstrated recurring causes by user impact
  and affected solver/writer routes before choosing repairs. Freeze any
  current-main replay separately; never relabel this old-source control set.

Validate the consumer without executing chemistry:

```console
python3 -B -m unittest discover \
  -s audits/fresh-syntax50-20261013/normalized40-review -p 'test_*.py' -v
```

The seventeen tests use tiny synthetic streams with a copy of the frozen design.
They are ingestion tests, not additional experiments or product acceptance.
