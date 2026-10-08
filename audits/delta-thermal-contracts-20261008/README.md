# Thermal-input diagnostic — reviewed

[Run 37786091864](https://github.com/CrispStrobe/kerotakis/actions/runs/37786091864), artifact `delta-thermal-37786091864`, ran six functions against baseline `5b305000` and repair `c3415e70` under the same generated lock. Baseline passes three and fails three; repair passes all six. Both pass the inherited 16 delta unit controls. [The offline review](review.json) verifies source/tree, clean status, frozen fixture, harness, lock and raw log outcomes.

The fixture predates the repair and covers invalid absolute temperatures, nonfinite energy with and without heat capacity, atomic refusal, supported positive temperatures, signed finite energy and zero energy. These are source-informed regression controls. They certify input validity in this bounded path, not physical thermal-domain or energy closure. PR #767's final head also removes an unused helper; its exact-head required CI is separate.
