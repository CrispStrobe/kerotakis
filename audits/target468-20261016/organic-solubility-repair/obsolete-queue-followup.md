# Superseded workflow queue follow-up

The first organic-solubility source head was superseded by its required compile
and inventory correction. Its hosted build was already cancelled and no job was
running, but three aggregate jobs remained queued. The corrected reviewed-head
workflow therefore remained pending.

Normal cancellation did not clear the old run. After rechecking its exact head
and absence of running jobs, the documented force-cancel endpoint closed only
that obsolete run. The corrected workflow moved from pending to the runner queue.
The [receipt](obsolete-queue-followup.json) records before/final states and raw
receipt hashes. Valid acceptance runs, reviewed product source, first source
failure and scientific selections are unchanged. No new engine ran.

This clears an observed delivery obstruction; it does not establish that all
other queued jobs have the same cause. Reviewed-head CI and full fleet gates
remain mandatory. If this cancellation pattern recurs, review the three
`always()` aggregate job guards in `.github/workflows/chemistry-audit.yml` on a
separate narrow branch: cancellation should not allocate aggregate runners,
while ordinary failed fleet results must still be aggregated and fail acceptance.
Preserve missing-artifact and case-count assertions.
