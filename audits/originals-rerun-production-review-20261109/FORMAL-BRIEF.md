# Formal criterion review (strict, read-only)
For each assigned original ID: dossier in review/dossier/<id with ':'->'_'>.md (forecast, criteria, limitations, script),
FULL raw production response in raw/<same>.json (field attempts[-1].body is JSON with observations: per step
operator, events, bench.vessels contents/solution/temperature). Use the RAW json (python allowed, read-only), not
only the digest. Do not call APIs, build, run engines or edit repo/other files.

Decide each ORIGINAL criterion (acceptance_checks + the forecast itself) as PASS / FAIL / UNSUPPORTED with numeric
evidence from raw output. Mandatory independent checks: element and charge inventory closure across all vessels and
transfers (incl. H/O, gas/headspace, adsorbed/solid pools, effluent) at relevant steps, relative error reported;
energy where the forecast concerns heat/temperature. A missing observable the forecast requires => UNSUPPORTED.
A clean exit, a warning-free run or plausibility is NOT a pass. Never alter thresholds or reinterpret the forecast.
QUALIFIED only if every criterion PASS (a coverage criterion may PASS if the limitation is explicitly reported and
does not touch the requested observable). Be adversarial: look for reasons it fails.

Write a JSON array to review/formal/<PART>.json (exclusive create):
{"id","qualified":bool,"criteria":[{"id","text","verdict","evidence"}],"closure":{"elements":{...rel err},"charge":...,
"energy":...},"observed_result":"1-2 sentences with numbers suitable for a catalog entry","limitations":"1-2 sentences",
"doubts":"anything that a second reviewer should check"}
Final message: qualified IDs, rejected IDs with one-line reasons.
