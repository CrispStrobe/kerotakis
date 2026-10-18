# First repaired finite-solid replay: failed invariance

One unchanged wait-cycle invocation on PR822 head `aa08be0f`, tested merge
`3fdc1e01`, completed with exit0 and nine observations. The [assessment](assessment.json)
records **FAIL**: retained Fe(OH)2 drifted by0.000198107150974mol and ferric
inventory appeared. The represented iron residual was at most3.9e-15mol;
conservation of that element alone does not validate the phase/oxidation state.
Original forecasts and the earlier c446 first outcomes remain unchanged.

The [frozen design](frozen-design.json), [raw output](stdout.jsonl), actual
producer lock hash, binary hash and [execution envelope](execution.json) bind
this first result. No threshold is weakened. The subsequent phase-state pinning
correction requires its own binary/lock/source binding and replay. Do not rerun
this unchanged failed binary or increment the working catalog.

The first SDK setup failed before VM creation or experiment issue because
project/team context was absent. That raw host-specific failure remains in the
private operating evidence; the corrected setup performed exactly one native
invocation. The VM had deny-all networking, an empty account environment and
bounded CPU/wall/memory. SDK destruction returned; independent deletion is not
attested. This is CLI replay, not REST/deployment acceptance.

The raw manifest inherits `build_image`, `rustc` and `target` metadata from the
base snapshot; those are not attested producer build metadata. The actual binary
came from an authenticated Ubuntu Actions artifact. The frozen design binds the
replacement Ubuntu loader and four runtime libraries. Source merge/tree,
binary/actual producer lock bytes and runtime-library identity are separately
verified; do not describe the binary as built in the inherited image.
