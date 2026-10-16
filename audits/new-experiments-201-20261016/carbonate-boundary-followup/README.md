# Carbonate endpoint boundary follow-ups

Original QA-004/005 both reach their requested endpoints, but the first acid
amount is about0.855mmol instead of the original0.3/0.7mmol forecasts.
[Saved open-control diagnostics](original-open-diagnostics.json) show explicit
CO2 export and retained-carbon closure. Those forecasts remain unmet; no
missing mass or numerical-tolerance defect is established.

The [prospective design](design.json) adds only a fixed1mL headspace before the
first titration in each protocol, retaining all original inputs, doses, endpoints,
step sizes, maximum steps and forecasts. Trapped atmosphere must be included;
sealing is not evacuation. Acceptance includes delivered acid, endpoint status,
carbon/boundary accounting and absence of solver failure. The amount tolerance
is two original burette increments. If finite-headspace support fails, preserve
the refusal/failure instead of removing air or relaxing the expectation.

The bounded [native runner](hosted_screen.py) is derived from the earlier201
runner for exactly two cases, with the same process limits and accepted producer
binary. It does not modify the earlier frozen runner or repeat the open controls.
Static artifact/script verification passed without engine execution. Actual
execution and chemical assessment are pending a separately recorded dispatch.
