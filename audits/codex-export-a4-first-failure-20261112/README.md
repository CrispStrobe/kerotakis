# Codex export snapshot: first failure

[Linux CI job 131550351101](https://github.com/CrispStrobe/kerotakis/actions/runs/43323551555/job/131550351101)
on head `a4ca036f57232894ed008bd6f5f1f1574c105d6b` failed the unchanged
`codex_export_matches_golden_snapshot` test after the three carbonate lessons
were revised. Its complete log SHA-256 is
`cbb64fda765d4e448b2c1752ac6ed9ba9da103e04d4c303063e59d62d8832b83`.
The CI-produced artifact 13994986613 has ZIP SHA-256
`1acdeb589aaff515a6837432f7e81da02af7ba5c1e09cbb82f9993c0344c7c03`;
the original and actual export bytes are retained here losslessly compressed.

`review.py` verifies the stored byte hashes, unchanged models and concepts,
unchanged reaction ordering and IDs, and changes confined to `expect`,
`registers`, `provenance`, and the first entry's `equation` for the three
lessons independently adjudicated in [the codex carbonate review](../codex-carbonate-claims-adjudication-20261112/README.md).
The checked-in golden now equals the first actual CI export. The snapshot test
itself and its exact comparison are unchanged. A successor CI run is required;
this evidence is not a scientific validation or a new experiment qualification.

The macOS native job 131550351198 independently failed the same snapshot test.
Its complete log SHA-256 is
`55aa97affa16e80784e838173b37125655bfc08b4eff4b346efdd7f3001ee86c`.
Artifact 13995386814 has ZIP SHA-256
`d03ab90ea8da0a353c0b5e811b8becf89d00020f4586b0a6a99aa1703632fb2e`;
its exported JSON SHA-256 is the same as the retained Linux actual export,
`7845441d2193288071501e9746d82770ad1fc006790ac9811eb0cb512bea7bf9`.
