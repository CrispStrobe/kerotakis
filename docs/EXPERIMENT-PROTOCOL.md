# Experiment audit protocol

Status: protocol v1 (2026-11-24). Schema: [`schemas/experiment-audit.schema.json`](schemas/experiment-audit.schema.json),
example: [`schemas/experiment-audit.example.json`](schemas/experiment-audit.example.json),
validator: `python3 tools/experiment-audit.py <file.json>`.

## Purpose

Kerotakis is measured against real experiments — from school curricula,
teaching manuals and chemistry kits — not against prompts written to suit the
engine. An experiment audit asks one question per expected observation: *does
the engine, driven by an ordinary script, show what the experiment shows?*
Every negative answer becomes a tracked gap. The audit is a measurement, so it
must not be steered by the result it is measuring.

## Roles and clean-room separation

The pipeline uses separate agents (or people). Each sees only what its role
needs; separation is what keeps the expectations honest.

| role | sees | produces |
|---|---|---|
| **outliner** | the source manual | List A (components, devices, paraphrased procedure, the principle shown) and List B (expected observations). Writes `source`, `title`, `components`, `procedure`, `expectations`. |
| **scripter** | List A only — never List B | a `.lab` script per experiment, the run, and `kero` (support, library match, script, binary, run log, exit). |
| **grader** | List B + scripter output | one `verdicts` entry per expectation and a `gaps` entry for every non-`MATCH` that is not `NOT_APPLICABLE`. |
| **fixer(s)** | the gap list | engine/data/script fixes through the normal PR workflow, then a rerun of every affected experiment. |

The scripter must not see List B: a scripter who knows the expected answer
tends to write the script (or pick the reagent amounts) that produces it.
The grader must not edit scripts or expectations.

## The record

One JSON object per experiment; a file is a JSON array of records. The
schema is normative; summary:

- `id` — `<kit>-<edition>-<expno>`, e.g. `c1-6-E012`.
- `source` — `kit`, `edition`, optional `section`, `url`, `linked_files`
  (references only; never the text).
- `title` — an own short paraphrase.
- `components.reagents[]` — `name`, optional `formula`, `amount`, `unit`
  (`g|mL|mol|drops|pinch|piece`), `form` (`solid|solution|gas|liquid`),
  `concentration`; `components.devices[]` — plain names.
- `procedure[]` — paraphrased steps, one action each.
- `expectations[]` — `id` (`X1`, `X2`, …), `kind`
  (`observation|quantity|absence|equation`), `text`, optional `tolerance`.
- `kero` — `support` (`full|partial|none`), `library_match` (an existing
  lesson/example/curiosity prompt that already covers it, or `null`),
  `script`, `binary` (version + SHA), `run_log`, `exit`.
- `verdicts[]` — `expectation`, `verdict`, `evidence` (short quote of engine output).
- `gaps[]` — `class`, `detail`, `gap_list_id`.

### Verdict vocabulary

| verdict | meaning |
|---|---|
| `MATCH` | engine output shows the expectation |
| `ENGINE_DISCREPANCY` | script is faithful; engine output differs or is absent |
| `SCRIPT_DISCREPANCY` | script does not reproduce the procedure; fix the script and rerun |
| `NOT_APPLICABLE` | expectation is outside chemistry simulation (technique, handling, safety) — say why in `evidence` |
| `UNSUPPORTED` | the engine refuses or cannot express a needed step, species or device |

A `MATCH` that was not actually run (only inferred from `library_match`) is
not a `MATCH`; run the script or leave the verdict out.

### Gap classes

`missing-species`, `missing-device`, `missing-reaction`, `wrong-observation`,
`wrong-quantity`, `crash`, `syntax`. Aggregations may group gaps by a finer
topic (e.g. one missing indicator affecting twelve experiments) with an
acceptance check and the list of affected ids; rank by count × teaching importance.

## Rules

1. **Paraphrase only.** Manual or kit text, figures and wiki text never enter
   the repository. Raw sources stay in private storage; records hold own-words
   paraphrase and references.
2. **Never bend an expectation** to match the engine. A wrong expectation may be
   corrected only from the source, with the change and reason recorded.
3. **Every discrepancy becomes a gap.** `ENGINE_DISCREPANCY`, `SCRIPT_DISCREPANCY`
   and `UNSUPPORTED` each need a `gaps` entry (the validator enforces this),
   and each gap a line in the maintained gap list under a dated heading
   for the audit. Append; never rewrite other audits' lines.
4. **Unsupported is a coverage limit, not a pass.** Report counts per verdict.
5. **Rerun after fix.** A fix closes a gap only when the affected experiments
   are rerun on a binary containing it and regraded; keep the first verdict
   and append the new one with the new `binary`.
6. **Preserve evidence.** Script, binary identity and run log are kept for
   every verdict.

## Running scripts

Build the CLI once (`cargo build --release -p kerotakis-cli`) and run each
`.lab` script with it, capturing stdout/stderr and the exit code into the run
log; record the binary version and SHA-256. Use bounded per-script time and
memory limits. For large batches or when the local build host is busy, run on
a hosted CPU runner the same way the full gate does (see [PR-WORKFLOW](PR-WORKFLOW.md)).
Engine fixes arising from gaps follow the normal PR workflow.
