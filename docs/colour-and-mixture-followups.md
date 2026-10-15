# Colour and mixture follow-up lanes

These lanes preserve the user's observations and distinguish presentation,
recipe choices and chemistry coverage. Start from integrated main; use narrow
PRs, retain first failures and bind executed checks to their actual source.
Do not change frozen experiments to agree with a candidate.

1. **Transient liquid overlay — candidate PR790, acceptance pending.** The
   whole-vessel tint disappears after a few seconds. Source defects fill the
   animation grid for any positive volume, discard computed `solution` colour,
   replace the canvas node and leave a superseded fade timer alive. Candidate
   `c4513357` addresses these paths in `web/app/src/lib/fluidScene.ts`, `pour.ts`
   and `components/FluidOverlay.svelte`. Review the whole diff, run focused app
   controls and the actual-browser `tools/test-fluid-rendering.mjs` probe through
   CI. Verify 100 mL fill/headspace, all glassware, layer colours, conserved pour
   handoff, node identity and replacement activity during fading. Also inspect
   economy-grid transitions: a fresh full-resolution run must restore canvas
   dimensions after a previous run reduced them. Preserve any first failure;
   require accepted checks before normal exact-head auto-merge. Do not claim
   the black static watercolor mixture is fixed by this renderer change.

2. **Recipe identity and historical replay — prerequisite for new defaults.**
   PR792 is the first candidate: explicit revision selectors and canonical script
   pins, with no default recipe changes. Native/protocol acceptance and independent
   review are pending; migration/archive/bottle semantics remain unfinished.
   Inspect `crates/kerotakis-core/src/material.rs` version lookup and the
   `Operator::AddMaterial` branch in `bench.rs`. Current admission compares the
   requested version against the current recipe. A version archive alone is
   therefore insufficient. Inspect `web/app/src/lib/session.svelte.ts` save,
   snapshot fallback, undo and script export: text replay reparses current
   recipe names. Define how existing version-1 experiments retain their exact
   expansion, including sessions without a usable snapshot. Keep historical
   versions out of default shelf enumeration, validate id/version/basis, and
   reject unknown versions without mutation. Deliver native and app controls
   for old saves, new defaults, undo, export/replay and snapshot fallback before
   promoting a changed default. State explicitly which unversioned historical
   script formats can be interpreted reproducibly.

3. **Transparent teaching wash optics — reproduction and versioned remedy.**
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
   and chemistry are unchanged. Source review passed, actual native/fleet checks
   are pending. This does not fix equilibrium readback churn.
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
