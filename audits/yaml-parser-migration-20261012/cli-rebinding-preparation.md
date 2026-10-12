# Prepare unchanged CLI expectations for a separately reviewed source

[The preparation helper](prepare-cli-rebinding.py) creates a new private preparation directory for either the immutable [fifteen CLI controls](cli-controls/manifest.json) or [twelve nested-rate controls](nested-rate-controls/manifest.json). It performs source/hash checks and writes files only. It never runs the CLI, builds code or claims a behavioral result.

Original freezes remain fixed at `6ae7b5bb7d0c6fedb5faffa62e16249e6c6b9263` and `c1fe0ef8f29f57de3b71365714cc1297ba0afe09`, with embedded manifest/collector SHA256 pins. The helper requires the current original files to match those immutable Git objects exactly. It copies `collect.py` byte-identically, saves `original-manifest.json` byte-identically, and writes a separately bound `manifest.json`. Only `source`, `source_tree`, existing `source_sha256` values and the nested matrix's existing `candidate_binary_sha256` may change. A new `rebinding_lineage` records original commit/hashes, reviewed own artifact association and preparation-only checks. Every case, ID, count, input, pack reference/expectation, timeout, refusal expectation, rationale, scope and limitation remains equal to the original. The fifteen-case collector receives no new strict candidate-binary field and is not rewritten.

First independently review the candidate's **own hosted CLI artifact receipt**. The helper accepts that existing receipt schema: `head`, `source_tree`, `binary_sha256`, `own_run_id`, `job_id`, `artifact_id`, `artifact_zip_sha256`. Supply its separately reviewed expected SHA256. The helper checks its bytes against that SHA, verifies association fields/digest forms, and hashes the actual supplied binary against the receipt. It does not independently download or review the archive, authenticate the reviewer, or require completed full CI/fleet/integration acceptance. The operator's independent artifact review and expected receipt SHA are the trust boundary; an arbitrary self-written receipt is not acceptable evidence.

The source checkout must have exact Git `HEAD` and tree equal to that receipt. Every tracked non-Gitlink blob is compared to its Git object, including tracked symlink contents; the manifest's parser/CLI path inventory stays fixed and receives new source-byte hashes. Original repository inputs must retain their frozen hashes. Gitlinks are recorded as pinned references without recursively validating module contents; use the existing source/module/artifact reviews for those separate guarantees. This helper does not prove build freshness, cache provenance or actual lock association beyond the independently reviewed artifact receipt.

Under a fresh resource gate, prepare each matrix independently:

```text
python3 audits/yaml-parser-migration-20261012/prepare-cli-rebinding.py --matrix cli15 --source REVIEWED_SOURCE_CHECKOUT --binary REVIEWED_OWN_CLI_BINARY --reviewed-artifact-receipt REVIEWED_OWN_ARTIFACT_RECEIPT --reviewed-artifact-receipt-sha256 REVIEWED_RECEIPT_SHA256 --out NEW_PREPARATION_DIRECTORY
```

Use `--matrix nested12` for the second directory. Preparation rejects an existing output, a symlink at output, a destination inside source or this audit/freeze directory, changed original freezes, wrong receipt/binary/source/input hashes, or missing artifact association. It copies the reviewed receipt into the new directory and prints the **new manifest SHA**, unchanged collector SHA, actual reviewed binary SHA and source/tree binding. The destination should be private because the reviewed receipt might contain environment details; publish only separately reviewed portable results.

Root must review the prepared lineage, original-versus-rebound manifest differences and artifact/source association before execution. A later outer runner must pass the actual independently reviewed binary hash and new manifest hash to the unchanged collector, especially for CLI15, whose checker has no strict candidate-binary field. The collector checks the supplied hash against actual binary bytes; the outer runner must also bind that hash to the reviewed artifact receipt, retain preparation lineage and capture new execution/lock/resource associations. Run into a **separate new evidence directory**, not the preparation directory or either first-outcome destination. Preserve original fifty, supplement twenty, first CLI15 and first nested12 outcomes without rewriting historical failures or declaring integration accepted.

[Eight synthetic tests](test_prepare_cli_rebinding.py) cover both byte-identical-collector/unchanged-expectation rebindings and refusals for changed freeze expectations, dirty tracked source, wrong source head, wrong reviewed receipt hash, wrong actual binary, existing/source/freeze destinations and changed source-path inventory. They create tiny temporary Git repositories and never execute the synthetic binary:

```text
python3 audits/yaml-parser-migration-20261012/test_prepare_cli_rebinding.py
```

This is preparation tooling for source-informed engineering controls. It does not establish empirical chemistry, migration correctness, runtime portability or integration acceptance.
