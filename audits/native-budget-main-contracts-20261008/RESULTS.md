# Main-based native budget baseline

[Run 37731431537](https://github.com/CrispStrobe/kerotakis/actions/runs/37731431537) compiled and ran the contract-only main-based source: **62 library controls passed and 36 failed**. Of the 50 frozen boundary controls, **14 passed and 36 failed**. All 48 pre-existing library controls and all three live crystal controls passed. [The immutable receipt](baseline.json) binds every outcome and raw evidence hash.

The raw-column and closed-boundary fixture adaptations are explicit and original files remain preserved in [PR #760](https://github.com/CrispStrobe/kerotakis/pull/760). This demonstrates main-based failures independently of the donor branch. It does not accept the repaired port. The donor has additional library tests; its 113-test passing total cannot be used as this main-based test count. Required main PR gates and the later integrated-source audit remain separate.
