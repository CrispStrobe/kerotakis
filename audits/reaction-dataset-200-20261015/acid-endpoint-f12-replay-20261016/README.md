# Unchanged original F12: isolated hosted replay preparation

This directory is prepared on `validation/acid-stock-replay-20261016` only.
**Never merge this validation branch.** It replaces the existing `vercel.yml`
workflow on this isolated ref with a bounded raw replay; it contains no deploy,
Vercel credentials, engine rebuild or production configuration change. PR805's
source branch remains unchanged.

Status: prepared before execution; **not dispatched**. Dispatch requires source
review and the successful producer build/artifact from fleet run38863384507.
The rest of that fleet can still be running: artifact reuse does not accept it.

`original-design.json` is a byte-identical copy of the original physical-stock
freeze. `input.lab`, its SHA256 and its qualitative forecast are unchanged:
acidic permanganate/peroxide consumes permanganate and yields MnII/O2 if the
route is supported; background peroxide decomposition remains distinct.
`design.json` binds the producer source `a03113a2`, tree `81a79c3c`, original
freeze, script, runner and isolated workflow. This is a source-informed targeted
replay, not another blind experiment or an edited original outcome.

The controller authenticates the GitHub producer run, successful build job and
single CLI artifact. It verifies GitHub's archive digest, actual binary record,
raw generated Cargo.lock bytes, source head/tree, and the artifact-named tested
commit/tree through the authenticated Git API before any CLI process. It also
checks the actual producer workflow bytes. A pull-request tested merge is
accepted only with the exact frozen tree and source-head parent binding.

Exactly one process runs the original script in JSON mode. CPU time is limited
to60seconds, address space to1GiB and wall time to90seconds. It has a fresh0700
working directory and a minimal environment with one worker and no inherited
token, GitHub context or deployment credentials. There are no retries. A second
workflow dispatch or rerun on this isolated branch refuses before execution.

The always-uploaded artifact retains authentication responses, ZIP and binary
records, actual generated lock bytes, pre-execution binding, raw stdout/stderr,
exit/timeout, limits and collector receipt. Prerequisite failures are preserved
before any process; execution failures preserve their first streams. The
controller assigns `physical_verdict: not_assigned`. A zero exit or green
collection workflow is not a chemical agreement or calibration claim.

Root review must authenticate the resulting workflow/artifact independently,
then assess final MnII, depletion ratios, target oxygen versus background,
complete material/gas accounting, final readings and explicit model limits.
Keep that assessment separate from both the original F12 failure and product
integration acceptance. No dispatch or successful chemistry is established by
these prepared files.
