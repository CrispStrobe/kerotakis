# Stock refusal behavioral baseline

[Corrected run 37731328644](https://github.com/CrispStrobe/kerotakis/actions/runs/37731328644) compiled four controls: **one passed, three failed**. The refused species/material draws and invalid species request each reached the downstream solver when the expectation was zero calls. The accepted-draw control passed. [The source-bound receipt](baseline.json) preserves outcomes and hashes.

Each failing loop stops at its first mismatch; this baseline does not establish later mode/input outcomes. The original run failed in its lockfile-binding harness before executing tests and is not behavioral evidence. [PR #757](https://github.com/CrispStrobe/kerotakis/pull/757) contains the short-circuit repair and still needs exact-head required gates. No original forecast or assertion was changed.
