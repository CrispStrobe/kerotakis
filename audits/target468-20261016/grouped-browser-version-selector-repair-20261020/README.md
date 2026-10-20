# Actual provider version selector correction

The first collection of the issued runtime33 browser producer stopped after three HTTP404 status requests. Its source serialized numeric version1 as the label `1`. Actual read-only diagnosis found that the provider accepts explicit label `v1`; it rejected nonexistent `v999` with404. The same original run was still running and later reported complete. It was never reissued.

This collector successor changes only the version-label serialization for every status/output page and the matching log labels, plus an explicit parent-reviewed fresh collection-directory name. Seven local controls cover actual SDK request serialization, version rejection and strict fresh-directory guards; the parent independently reproduced all seven. The four helper dependencies remain byte-identical to the [previous host implementation](../grouped-browser-chain-preparation-20261020/README.md).

The first failed collection and its original authorization are immutable. A separate external authorization pins the successor and a different collection directory for the existing issue. Frozen producer/configuration/input bytes, original protocol assertions and the issue receipt remain unchanged. There are no producer calls in the collector and no automatic execution retry. Collection still streams bounded chunks through verified local storage and independent archival readback; it assigns no scientific or browser pass.

The previous offline serialization controls proved that `1` reached the API, but could not establish that the service accepted that label. This observed transport failure closes that compatibility gap without rerunning the browser or API protocols. Full raw-result and actual-reader invocation review remains mandatory after collection.
