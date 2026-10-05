This batch contains 50 new, independently imagined CLI experiments, numbered 051–100. The three authors used command syntax/help and their own physical expectations, without using an existing experiment corpus or chemistry implementation to construct the forecasts.

`predictions.json` and the `.lab` scripts are frozen before execution; `freeze.json` records the digest and date. A syntax correction or missing-species substitution belongs in a separate followup, never in the original forecast. Agreement, missing capability, wrong physical behavior, and ambiguous evidence are assessed separately. An explicit refusal may correctly enforce a model boundary while still revealing a missing capability.

`run.py` runs the actual CLI serially, in text and JSON modes. Before each local invocation it requires load1 ≤3.5, available memory ≥3200 MiB, swap free ≥600 MiB, and workspace disk free ≥2048 MiB and system disk free ≥512 MiB. CLI temporary files are directed into the selected results directory. Each CLI invocation has a 60-second timeout. Large raw outputs are initially stored with the corresponding archived CI evidence; the execution receipt includes prediction and executable hashes, invocation commands, resources, exit codes, and output hashes. No local compilation is needed when replaying the previously validated executable.

Assessment and any resulting repairs are documented separately after execution. Forecasts remain unchanged.

[Results and repairs](RESULTS.md) distinguish the original assessment from separate controls and post-repair evidence. Hosted runs reuse an executable only after verifying its successful validation receipt and binary digest.
