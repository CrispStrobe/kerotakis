# Let cancelled fleet aggregates release their concurrency group

Normal cancellation of superseded fleet run39118933896 cancelled its build and
family jobs but left its three `always()` aggregates queued. The corrected-source
run stayed pending behind that concurrency group. Read the
[first observed state transitions](../audits/target468-20261016/fleet-cancellation-first-observation/observation.json).
No job was active at either cleanup request. After normal cancellation failed to
finish the obsolete run, a terminal cancellation released it; its conclusion is
cancelled, not successful chemistry acceptance.

The prepared workflow change uses `${{ !cancelled() }}` for the three aggregate
jobs. That status function still permits aggregation after family failures,
while respecting workflow cancellation. GitHub explicitly recommends this
alternative for cancellation-sensitive work rather than `always()`:
[expression guidance](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#always),
[cancellation behavior](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-cancellation).

All33 logical jobs, matrices, named assertions, raw-recording steps and artifact
uploads remain unchanged. Step-level `always()` uploads still preserve first
failed-family evidence. A cancelled workflow is never counted as an accepted
fleet; ordinary acceptance continues to require all33 original jobs satisfied.
Branch protection and required contexts remain unchanged.

## Acceptance and delivery

YAML semantic comparison verifies that only the three job conditions changed.
A separate bounded cancellation control must dispatch this exact source, inspect
that no physics recording has started, request normal cancellation once and
retain actual terminal status plus aggregate states. It is a scheduler control,
not another corpus execution or a scientific pass. Preserve any first failure;
never disguise it with a fabricated green check or a changed test condition.

Exact-head CI and the full ordinary fleet remain separately required. Keep the
PR draft while those or the cancellation control remain pending. Integrate only
through normal protected exact-head auto-merge after the earlier accepted
production-delivery checkpoint. This narrow workflow repair does not authorize
merging an isolated validation branch or broad infrastructure/audit history.
