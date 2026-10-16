# Finite-headspace carbonate results

Both separately frozen controls completed once in
[run38875794689](https://github.com/CrispStrobe/kerotakis/actions/runs/38875794689).
The authenticated [raw artifact](raw-artifact.zip), [binding](binding.json),
[verification](verification.json) and [saved reader](review_saved.py) preserve
actual scripts, first outputs and the c446/670 engine association.

| Control | First HCl endpoint | Cumulative second endpoint | Frozen forecast |
| --- | --- | --- | --- |
| QA-004 finite headspace | 0.297168 mmol | 0.993164 mmol | 0.3 / 1.0 mmol |
| QA-005 finite headspace | 0.698660 mmol | 0.999226 mmol | 0.7 / 1.0 mmol |

Both meet the unchanged prospective0.02 mmol endpoint tolerance. Both endpoints
are reached without solver failures, named coverage limits, gas export or rupture
after sealing. H/O/C/Na/Cl/N and charge residuals close within the frozen1e-8 mol
resolution, including trapped atmosphere and actual titrant carrier water. Final
headspaces remain1 mL; reported liquid volumes are109.944 and110.005 mL and
pressures119853.09 and109274.55 Pa. Existing-model nRT/V agrees with the owned gas
inventory. See [case-level arithmetic](assessment.json).

The original open controls remain failed endpoint forecasts with accounted CO2
export. These finite-boundary controls isolate that boundary choice: they do not
validate open-system rate assumptions, new vessel geometry, empirical activity
or heat calibration. No chemical or numerical implementation changed. The working
catalog remains unchanged and the reader runs no simulator process.
