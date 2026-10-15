# Liquid animation contract

The static vessel and its transient fluid animation must agree on the accepted
scene. `glassware.fillHeight` determines the drawn liquid top; the glassware
bottom and the shared 100 × 140 SVG viewBox determine its grid rows. Air, glass
walls and the region below the bottom are outside the liquid field domain.
Droplets may fly through the headspace before handing off their accepted visual
mass to a valid liquid cell.

Each layer's scene `srgb` is authoritative. A computed mixture called `solution`
is not a pure shelf species; the overlay must not replace its colour with a
catalogue fallback. Shelf lookups may still provide transport properties. Local
motion and crossfades are presentation, not a new colour or reaction model.

Keep one canvas mounted across animation activation and fading. A replacement
activity cancels both the previous animation frame and its pending fade timer.
Otherwise callbacks can paint a detached node or hide a newer run.

Focused checks cover fill levels across all glassware, masked walls, a transparent
headspace and bottom, engine-owned mixture/layer colours, and conserved pour
handoff. The real-browser check exercises the command path, samples actual canvas
pixels against the static scene and checks node identity and replacement activity.

The reported blue/yellow watercolor becoming nearly black also involves the
current dye concentrations/spectra and optical path. That model/recipe issue,
duplicate aqueous diagnostics and repeated equilibrium readback events are
separate follow-ups; this renderer change does not claim to fix them.
