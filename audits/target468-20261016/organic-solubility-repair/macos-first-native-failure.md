# First native acceptance failure on the organic repair

The [macOS job](https://github.com/CrispStrobe/kerotakis/actions/runs/38930003346/job/117071266322)
compiled the reviewed `c4b85d2d` head and passed all four new regression groups:
finite missing-row availability, pre-consumption warnings, liquid-only residues
with inert controls, and all66 pair/status/translation checks.

Three existing integration tests failed because they require no model-boundary
warning for the legacy organic-solubility entries. Their historical silence
contract contradicts the new explicit withdrawn-provenance boundary. The
[receipt](macos-first-native-failure.json) retains exact tests/source locations,
run/job identities and the raw-log hash. This is a failed acceptance attempt;
the complete CI and chemistry fleets remain unaccepted.

The required correction is specific: retain all existing dissolution amounts,
residual-solid, reaction and conservation checks; replace the zero-warning
requirements with exact checks of the provenance key, ModelBoundary cause,
vessel and affected solute/solvent pairs. Require the expected warnings and
reject unrelated diagnostics. Supported inert controls must remain supported.
Do not delete a numerical assertion or suppress a real limitation to obtain a
pass. Preserve this failed head and require new exact-head hosted acceptance.

This correction is scoped but not implemented at this checkpoint because local
source-work resource gates are closed. No local engine ran, no original corpus
result changed, and no scientific working case was added.
