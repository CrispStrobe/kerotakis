# Colour and mixture follow-up lanes

These lanes preserve the user's observations and distinguish presentation,
recipe choices and chemistry coverage. Start from integrated main; use narrow
PRs, retain first failures and bind executed checks to their actual source.
Do not change frozen experiments to agree with a candidate.

1. **Transient liquid overlay — integrated PR790.** Commit `d791d306` passed
   all 25 CI jobs. The actual-browser probe covers accepted fill/headspace and
   computed layer colour, persistent canvas, superseded fades, and restoration
   of full canvas dimensions after economy mode. Focused grid/pour controls
   passed 36/36. [Integration evidence](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/fresh-syntax50-20261013/fluid-rendering-integration.json)
   verifies parents, expected merge tree and unchanged reviewed runtime subtrees.
   Preserve this acceptance; repeat or extend controls when those paths change.
   Combined-head PR792 acceptance also passed; the recipe and rendering changes
   are integrated. A merge alone does not establish deployment.

2. **Recipe identity and historical replay — integrated PR792.** Merge
   `b22f3da5` contains the reviewed `3574752f` candidate: exact revision-1 archive,
   saved-history migration on browser/native transports, revision-specific finite
   bottles and version-2 defaults. All 25 CI and 33 own-fleet jobs passed, including
   current registry export, native recipe/protocol controls and actual-browser
   rendering checks. [Integration receipt](../audits/fresh-syntax50-20261013/watercolour-integration-20261015.json)
   verifies actual parents and expected merge tree. Earlier failed checks remain
   preserved. Extend replay controls whenever material schema or history changes;
   unavailable historical recipes must still refuse without mutation.

3. **Transparent teaching wash optics — versioned remedy accepted.** Seven native
   `watercolour_revision` controls passed: primary hues, visible blue/yellow green
   at the declared dose, order independence, separate finite bottles and old dense
   recipe replay. Revision-2 dyes are tenfold diluted; spectra and optical paths
   are unchanged. This closes the reported teaching-mixture regression at those
   modeled conditions. It does not calibrate retail paint or guarantee green for
   arbitrary dyes, concentration and path length. Future optical extensions need
   new frozen dose/path/geometry controls and independent spectral evidence.

4. **Duplicate diagnostics and equilibrium readback — establish event causes.**
   PR791 implements exact model-limit prose deduplication per step; raw events
   and chemistry are unchanged. PR791 is integrated as `f471b628` after all 25 CI
   and 33 fleet jobs passed, including four new native groups. The
   [integration receipt](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/fresh-syntax50-20261013/coverage-prose-integration.json)
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
