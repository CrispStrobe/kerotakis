# Deployed ammonia: actual browser and original GS005 acceptance

Production `c19b6e94`, native binary `26a1e8b8`, producer lock `66886813`, passes authenticated artifact/11 API/37 unchanged real-browser checks. Root authenticated45 browser evidence files and compared every check name with the previous accepted suite. Actual served assets and uncached Wasm/native reference checks pass.

Only the unchanged original GS005 acid-trap protocol was replayed once. Neutral inventory retains3.311907926e-6mol gas NH3; the acidified arm retains0.999999936 of reduced nitrogen as ammonium, versus0.049068109 neutral. Acid gas NH3 is below the source persistence threshold1e-12mol, not physically proven zero. Its conservative acid/neutral gas ratio is below3.019408e-7. Both original criteria pass, with each1mmol added reduced-nitrogen dose and trapped-air N2 accounted separately. Independent per-vessel/per-step atom/charge residual is at most7.272e-13mol/eq. Actual arm temperatures298.243K and300.150K are retained; no isothermal Henry/activity/rate calibration is claimed.

[Root review](ROOT-INDEPENDENT-REVIEW.json), [saved assessment](assessment.json), full original response, forecasts and source-qualified reader remain available. Reproduce without an engine/network call from the repository root:

```console
python3 audits/target468-20261016/ammonia-release-browser-replay-20261019/review_saved.py --directory audits/target468-20261016/ammonia-release-browser-replay-20261019 --out ammonia-saved-reproduction.json
```

This earns one separately bound original selection. Earlier failed gas partitioning remains historical. No OR006/GS010/KN007/TH016 was repeated. The browser’s actual Pages probe now observes source `d388375` and USER_PUNCH2+3=5; its previously disabled interpreter is superseded. A separate Pages uncached chemistry/native parity check remains required.
