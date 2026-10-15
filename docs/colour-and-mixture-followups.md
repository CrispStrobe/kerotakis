# Colour and mixture follow-up lanes

These lanes preserve the user's observations and distinguish presentation,
recipe choices and chemistry coverage. Start from integrated main; use narrow
PRs, retain first failures and bind executed checks to their actual source.
Do not change frozen experiments to agree with a candidate.

1. **Transient liquid overlay — integrated PR790.** Commit `d791d306` passed
   all 25 CI jobs. The actual-browser probe covers accepted fill/headspace and
   computed layer colour, persistent canvas, superseded fades, and restoration
   of full canvas dimensions after economy mode. Focused grid/pour controls
   passed 36/36. [Integration evidence](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/fluid-rendering-integration.json)
   verifies parents, expected merge tree and unchanged reviewed runtime subtrees.
   Preserve this acceptance; repeat or extend controls when those paths change.
   Current recipe/replay PR792 includes this accepted main change and still needs
   combined-head acceptance. The separate black static watercolor mixture remains
   in lanes 2–3; no deployment claim follows from this merge.

2. **Recipe identity and historical replay — candidate PR792.** Current head
   `3196a8d6` contains archive/migration/revision-specific bottles and v2 defaults,
   plus accepted rendering changes from main. Source review passed after closing
   namespace and archive-selector gaps. Both exporter definition and constructor
   failures are preserved and corrected. The earlier combined app unit suite at `af788ba6` passed
   **1,666/1,666**; native export/protocol/optical, browser and fleet acceptance
   remain pending. Follow their actual current-head results before normal merge.

   Verify `crates/kerotakis-core/src/material.rs` revision lookup and the
   `Operator::AddMaterial` branch in `bench.rs`: exact revision, canonical identity,
   basis and finite bottle balances must survive replay. In
   `web/app/src/lib/session.svelte.ts`, verify whole-history preflight includes
   undone future entries, snapshot fallback, undo and export. Missing historical
   parsing or failed replay must preserve original saved bytes. Exercise both
   browser-worker and native-host transports. Keep archived recipes off the
   default shelf and refuse unavailable revisions without mutation. Prepared
   native controls and executed app mocks are separate evidence domains; report
   which actually ran. See [current native correction](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/watercolour-native-first-failure.json)
   and [combined app acceptance](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/watercolour-merged-app-acceptance.json).

3. **Transparent teaching wash optics — reproduction and versioned remedy.**
   The staged revision-2 blue/yellow recipes and seven native control groups are
   on PR792 at `3196a8d6`; native and full browser/CLI outcomes remain unexecuted.
   Review [the frozen candidate expectations](https://github.com/CrispStrobe/kerotakis/blob/b094f969/docs/watercolour-v2-controls.md)
   and retain their initial outcomes before any correction.
   Reproduce an empty vessel followed by `add v1 watercolour_blue 100g` and
   `add v1 watercolour_yellow 100g` using actual accepted recipe keys (confirm
   spelling in the registry). Inspect scene colour and browser pixels after
   motion stops. The current 0.02% mass-fraction dye surrogates and 4 cm beaker
   path predict near-black transmission in a source-only estimate; this has
   not yet been confirmed by an executed current-source CLI reproduction.
   Preserve that baseline. Freeze expected primary hues and a visible green
   teaching mixture at a declared dilution/path before running a candidate.
   Consider a separately versioned dilute teaching wash with honest surrogate
   provenance. Preserve the measured spectra and concentration/path dependence;
   do not clamp RGB or promise that arbitrary blue/yellow dyes always mix green.
   Verify amounts, addition order, dilution and optical path, old-version replay,
   and static/animated scene agreement. This is teaching-surrogate design, not
   empirical calibration or a claim about retail watercolor paint.

4. **Duplicate diagnostics and equilibrium readback — establish event causes.**
   PR791 implements exact model-limit prose deduplication per step; raw events
   and chemistry are unchanged. PR791 is integrated as `f471b628` after all 25 CI
   and 33 fleet jobs passed, including four new native groups. The
   [integration receipt](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/coverage-prose-integration.json)
   verifies the merged tree. This does not fix equilibrium readback churn.
   Preserve a minimal blue-color addition showing repeated indigo unsupported
   notes, then a bounded saline/alkaline case showing equal magnesium hydroxide
   dissolution/precipitation on repeated waits. Trace `aqueous.rs` through the
   public operator pipeline. Compare accepted inventories, lot metadata and
   aqueous totals before and after each operator. Distinguish multiple internal
   solver passes from real net changes. Group identical coverage messages at an
   appropriate presentation boundary without hiding genuine model limits or
   removing raw evidence. Verify that real dissolution and precipitation remain
   visible. Repeated settling percentages alone are not proof of a reset:
   `clock.rs` multiplies the remaining suspended fraction each step. Test the
   actual remaining inventory and metadata across successive waits.

5. **Material strength and unsupported reaction coverage.** Check why pure
   species `NaOCl` volume input is displayed as bleach, separately from a named
   dilute bleach recipe. Make the identity/strength distinction visible and
   preserve canonical stock identity; do not silently reinterpret old inputs.
   Inventory the hypochlorite/peroxide mechanism and its supported boundaries
   before adding chemistry. Require reviewed reference data, balanced amounts
   and supported conditions for any extension. Preserve explicit concentrated
   activity-model warnings and unsupported ionic-dye limits. An optical spectrum
   does not establish an aqueous speciation model; repeated pH output does not
   validate an out-of-domain concentrated prediction.

All five lanes require fresh resource headroom before local builds, browser
runs, CLI batches or additional agents. The original fifty experiment outcomes
and their separate spelling follow-ups remain unchanged by these new reports.
