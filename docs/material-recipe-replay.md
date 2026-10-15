# Pinning named-material recipes in scripts

`add v1 watercolour_blue@1 100g` requests revision 1 of that named recipe.
An unavailable or malformed revision is refused; it never falls back to the
current recipe. Material stock commands use the same selector, for example
`stock watercolour_blue@1 100g`. Pure species do not use this syntax. The recipe name, id,
revision and basis must agree before an addition changes inventory or stock.

New canonical material-addition commands include their accepted revision. The
app already saves the engine's canonical command, so subsequent text replay,
undo and export can retain the same pin that the typed operator carries.
Aliases resolve to a canonical material name and revision; finite bottles retain
that revision in their stock key. Old stored unversioned bottle keys migrate to
the reviewed historical default. Balances for different revisions remain separate.
`@` is reserved in newly loaded species and material names; pinned ingredient
tokens can never select a pure species.

The reviewed `data/registry/material-recipe-history-v1.json` archive retains the
original blue/yellow revision-1 recipes and their legacy default selectors.
`kerotakis-data` validates archive identities, versions, amounts, provenance and
alias ownership at build time. Historical records remain outside the default shelf.

App restoration preflights every saved command, including undone future commands,
through the engine's separate `parse_legacy` operation before snapshot restoration
or replay. Both browser and native transports support this operation. Newly typed
commands continue to select current defaults. Unknown historical commands or an
older engine without this operation preserve the original save; automatic writes
remain blocked until the user explicitly clears the bench. A valid history that
fails during execution is also preserved, with its partial replay cleared.

This staged prerequisite does not change recipe concentrations or defaults.
Native/protocol and actual app acceptance remain required before promotion.
Unversioned standalone script imports still select current defaults unless the
caller explicitly requests historical parsing; only a saved session is known to
be historical. Pinning and archival replay are not optical calibration.
