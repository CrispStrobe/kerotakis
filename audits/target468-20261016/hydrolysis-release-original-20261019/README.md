# Delivered hydrolysis and original OR006 acceptance

Delivered source `8c584e20`, binary `13a4d134`, actual producer lock `66886813`, passes11 unchanged API controls and37 unchanged real-browser controls. [Independent delivery review](ROOT-DELIVERY-REVIEW.json) rehashes the actual native/runtime/frontend archive bytes,45 browser evidence files and all11 API response bodies; browser check names match the preceding accepted suite.

The single unchanged original OR006 request now passes both original criteria. It retains0.005mol ester and forms matched0.005mol ethanol and total acetate/acetic acid. [Independent45-digit per-step review](ROOT-INDEPENDENT-REVIEW.json) closes C/H/O/Na and charge with maximum residual2.938e-17mol/eq. Remaining post-speciation base coordinates do not constitute another strong-base dose. Actual temperatures300.764379K/300.764205K are retained; no rate, heat or empirical activity calibration is claimed.

The first saved reader stopped with missing ethanol property metadata; acetate/acetic acid were also absent from that older snapshot. [First failure](first-reader-failure.json), original property bytes and their hashes remain preserved. [Completed properties](property-completion.json) add only actual compositions from the exact tested registry. The original reader, commands, response, criteria and tolerances remain unchanged. No extra API request was issued by the reviewers.

Reproduce saved review and catalog membership from the repository root, without executing chemistry or making requests:

```console
python3 audits/target468-20261016/hydrolysis-release-original-20261019/review_decimal.py --out hydrolysis-decimal-review.json
python3 audits/target468-20261016/hydrolysis-release-original-20261019/verify_catalog.py
```

[Selection](selection.json) adds exactly one original protocol, preserving all246 prior entries byte-for-value:247 total /243 unique fixed468, leaving158 more to401. Earlier failed hydrolysis observations remain historical. The next executable beneficiaries are unchanged MO006 and MO011, separately bound to this accepted release; sealed-carbonate MO003 remains outside the proved reserve domain.
