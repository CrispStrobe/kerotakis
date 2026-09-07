# PR #504 integration handoff

## Ownership

The audit owns only branch `audit/chemistry-experiments-20260906` and PR #504.
The parallel integrator owns GUI003, KIDS procedures, adsorption/partition/
osmosis readouts, live Catalog/StoryMap discovery and main synchronization.
Do not edit, rebase, push or merge the integrator's branches or reserved files.

## Merge gate

Do not merge PR #504. A green rebuild is necessary but does not grant merge
authority. CI must rebuild the current audit branch, including functional repair
`5338e3e6` as rebased onto main `139a18d6`, and all required checks must pass.
Run 34133207021 failed on fixture/translation drift and GUI003 behavior; the
narrow repair is recorded in `HISTORY.md`. Local narrow checks do not replace
rebuilt CI. No successful run is yet attached to the repaired head.

When CI settles, send the integrator:

- run URL and tested head/merge SHA;
- exact failing or passing jobs;
- whether current main requires another delegated rebase;
- confirmation that 113 entries and audit evidence remain preserved;
- confirmation that no merge was performed.

Keep GitHub status polls at least 300 seconds apart. Use the screen protocol in
`/mnt/volume1/code/screen-usage.md` when relaying to integration session
`3762447.kerobw`; send the message and carriage return as separate commands,
then verify delivery. Recheck the session name before every relay.

## Shared-file boundary

Catalog and documentation changes in this branch may overlap the integrator's
final discovery tranche. Do not add more catalog entries until repaired CI and
the pending sixth-fleet review complete. Coordinate before editing shared catalog,
StoryMap, README or planning files. Private source research never enters the
public branch.
