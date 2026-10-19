# First peroxide/locale candidate outcome

Candidate `80a81d6a` reproduced the original baseline failures and passed Clippy/all616 native tests. Full core reported2139 passes and one failure in the expanded locale scanner: a production constant was misread as English text. The provider correctly stopped before all33 product jobs. [Actual outcome and authenticated raw hashes](outcome.json) preserve that failure. No full-source acceptance, deployed-source certification or scientific selection is claimed.

The corrected constant-aware scanner and freshness repair are combined in PR839 candidate `20ea572c`; a separately frozen corrected-gate baseline and new source-bound validation retain this first result unchanged.
