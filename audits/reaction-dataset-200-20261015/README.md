# CLI probes for all 200 supplied reaction records

Both supplied JSON files were read. The 200-record dataset validates against
its supplied JSON Schema; the schema itself is not a chemical reaction.
[Input hashes](input-inventory.json), [the frozen manifest](design/manifest.json)
and [all 200 scripts](design/scripts) retain exact record accounting.

Execution is dispatched in [run 38763662200](https://github.com/CrispStrobe/kerotakis/actions/runs/38763662200).
Follow this existing run; do not duplicate the batch. Its isolated branch
`audit/reaction-dataset-200-20261015` must never merge into main. Only the explicit
200-record lane is eligible; all nine ordinary fleet jobs were verified skipped.
No new binary build is performed.

[Execution binding](execution-binding.json) identifies preserved main source
`d791d306`, its executable, dependency lock and authenticated producer artifact.
This source was main at the earlier build; later merges do not silently change
the binding. The hosted producer authenticates the API association, ZIP checksum,
safe member set, source identity, binary and lock before starting the runner.

Every record states that no procedure is provided. These are declared
contact/activation probes, with one millimole per numeric stoichiometric unit,
source ordering, a ten-second wait and per-case inspections. Hydronium is
represented explicitly as H+ plus its associated water. Charged notation and
name mappings require matching elemental composition and charge; molecular
formula alone is not used to select an isomer. Unknown identities are submitted
unchanged, not omitted. Symbolic coefficients use one solely for a finite probe.

Water is added for aqueous phase labels, and gas cases use a sealed one-litre
headspace with an actual initial-air baseline. Activation doses for ignition,
heat, electrolysis or light are declared screening assumptions. Dataset phase
labels cannot override registry phases. Missing catalysts and multistage
procedures remain missing. Read the independent [protocol review](protocol-review.json):
some organic water setups are incompatible or incomplete, and starch amounts
use a repeat-unit surrogate. Negative results there do not establish a chemistry
defect.

The runner gives every record its own CLI process and preserves stdout, stderr,
exit status, timeout, timing and hashes. Each child has a 30-second timeout and
a one-GiB address-space ceiling. It continues after individual refusals and
timeouts; a resource stop preserves partial results and stops the batch. A CLI
script itself stops at its first refusal, so attempting 200 scripts does not
mean every reactant or activation was reached.

After completion, authenticate the run/artifact and then use
`assess_execution.py EXECUTIONS_DIRECTORY NEW_REPORT_DIRECTORY` to verify all
200 records and streams. The report distinguishes execution coverage and
commands reached. It assigns no physical passes. Preserve first outputs and
freeze any corrected or condition-specific follow-ups separately.
