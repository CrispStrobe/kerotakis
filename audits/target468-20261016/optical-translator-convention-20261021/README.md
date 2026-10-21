# Optical translator convention repair

The actual [4d preflight job](https://github.com/CrispStrobe/kerotakis/actions/runs/39627136326/job/119315249276) completed workspace tests, then refused four joins whose helper name did not follow the translator naming convention documented in `tools/i18n-surface-lint.py`. Its function already translates registry names through `t()`.

Successor62f64727 changes precisely seven occurrences of `opticalSpeciesName` to `tOpticalSpeciesName`. The function body, translations, engine, golden data, original scientific criteria and checker rules are unchanged. `source-proof.json` binds both source bytes and that exact replacement. No allowlist or lint exception was added.

All nine actual lightweight tools in `actual-light-checks.json` returned zero; complete stdout/stderr are retained here. The surface check reports zero findings. Reporting-only unused electrolysis placeholders and untranslated console/slug echoes remain visible; this does not claim all i18n gaps closed. These are static checks, not new scientific or browser passes.

The prior4d Linux native job passed; macOS was cancelled, not accepted. The prior exact4d own-fleet run remains separately active and must not be cancelled or relabelled as successor evidence. Current full/platform acceptance and changed-release API/browser82 remain pending on PR854. Qualified coverage remains335/468.
