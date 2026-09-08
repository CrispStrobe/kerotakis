# Source-informed fleets: cases 219–458

Status: architecture frozen; manifests are frozen family by family before any
authoritative execution. Observed output must never be used to change an
expectation or tolerance.

The programme contains ten families of 24 original scripts: acid/base and
buffers, solubility and precipitation, gas boundaries, electrochemistry,
mixing and transfer, filtration and evaporation, distillation, thermal and
phase paths, organic equilibria, and observables with negative controls.
Learning progress, not learner age, determines later catalog placement.

Each expanded JSON manifest must contain exact scripts, original questions and
named relations. Every case participates in at least one conservation,
independent-law or metamorphic relation. Public files deliberately contain no
source-to-case mapping; that research record is retained privately.

## Execution contract

- One exact CLI binary is built on GitHub and its SHA-256 is bound to all runs.
- Ten matrix shards run one 24-case family each, sequentially and with
  `fail-fast: false`; the small deployment host does not run the fleet.
- Every script, stdout line, stderr line, timeout, diagnostic and final bench is
  retained even when analysis fails.
- Empty/malformed output, missing events, nonfinite or negative inventory,
  solver failure and violated relations fail explicitly.
- Exact expectations come only from conservation, stoichiometry, SI laws,
  independently solved declared equations or committed curated inputs.
- A datum used by both engine and check is labelled data consistency, never
  independent validation.
- Unsupported rate, nucleation, apparatus-efficiency, calibration or sensory
  claims require an explicit boundary; they are not silently promoted.
- Production crates and the GUI never read audit IDs or these manifests.

The aggregate gate requires exactly 240 unique IDs and scripts, a contiguous
range 219–458, ten family reports, and the frozen total check count. Only
distinct reviewed concepts enter Experiments/Codex/Missions; parameter variants
remain regression evidence.
