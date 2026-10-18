# Native ammonia diagnosis preserves failed assertions

Exact `4d8e1705` admission control fails because supplied NH3 maps to `NH3(aq)(g)` instead of native `NH3(g)`. The registry display formula carries `(aq)` and is not a database phase identifier. That invalid component is silently filtered out; acid trapping, supplied-gas absorption and extended-route refusal fail. The corrected dry-pressure fixture passes. All raw input/readback, command outputs and hashes are preserved.

Follow-up `6dee9254` uses explicit native capability identities for supplied gas, including existing CO2/HBr mappings. Original chemical/conservation/refusal assertions remain unchanged; Clippy and the full native suite are running. This evidence does not establish acceptance or any catalog selection.
