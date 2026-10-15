# Pinning named-material recipes in scripts

`add v1 watercolour_blue@1 100g` requests revision 1 of that named recipe.
An unavailable or malformed revision is refused; it never falls back to the
current recipe. Pure species do not use this syntax. The recipe name, id,
revision and basis must agree before an addition changes inventory or stock.

New canonical material-addition commands include their accepted revision. The
app already saves the engine's canonical command, so subsequent text replay,
undo and export can retain the same pin that the typed operator carries.
Aliases resolve to the canonical material name; finite bottle accounting uses
that same identity. `@` is reserved in names of newly loaded material recipes.

This prerequisite does not change any recipe concentration or default. Current
registry versions remain the available versions; an archive and its admission
checks must precede a future default revision. Existing unversioned saved scripts
still select a default. Their migration, session snapshot fallback and bottle
contents across recipe revisions need explicit controls before watercolor
defaults change. Pinning alone does not establish historical optical calibration.
