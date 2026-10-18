# Named hydrolysis and conserved strong-base capacity

The named `react ... saponification` operator previously required literal `NaOH`.
After aqueous settlement the same finite dose is owned as sodium plus analytical
H/O basis coordinates, so the command could leave all ester unchanged. Replacing
the literal lookup with `base_equivalents` or measured OH would incorrectly turn
buffer charge or a weak-base measurement into a stoichiometric caustic reserve.

The new core capacity rule is deliberately bounded. In a closed admitted pool of
Na/K, strong halide/nitrate counterions, water, acetate and the named ester/alcohol,
let M be total alkali equivalents, X total strong-acid anion equivalents and A
total acetate in every admitted protonation/salt form. The strong-base excess is
`max(0, M - X - A)`: the whole acetate acid reservoir is reserved before cleavage.
For a charge-closed inventory this also equals owned hydroxide carriers minus
admitted proton reservoirs. Admission checks both charge closure and the separate
owned hydroxide-carrier bound. Measured hydroxide is never added to capacity.

An acetate buffer can own positive basis coordinates and still have zero strong
base excess. A bare sodium ion has no backed hydroxide carrier. Water's own tiny
proton/hydroxide pair does not supply alkali excess. Carbonate, ammonia, other
buffers, unresolved or bound material pools and pending CO2 transfer explicitly
refuse this capacity certificate. This is a supported closed subset, not a general
weak-acid titration or buffer-equilibrium implementation. Ionic and potassium
routes require the existing water-majority medium gate; the already supported
literal-sodium molecular template retains its explicit ideal-request semantics.

The applier proposes on a clone and retires actual finite owned carriers. Spending
an already owned OH/basis coordinate forms acetate and ethanol while retaining
its spectator cation. Spending a literal NaOH molecule forms the existing NaOAc
product, with its sodium supplied exactly once by that withdrawn molecule.
Literal KOH forms potassium plus acetate, introducing no sodium. Formal charge,
elemental conservation, positivity and transactional commit are checked before
state replacement. The dissolved-charge cache is refreshed only after the full
change; persisted hydroxide is debited consistently with the shared family path
before aqueous remeasurement, so a retained excess is not erased or hydrolysis
billed as acid/base neutralization. No reaction rate or curated reaction heat is
added.

Controls cover base/ester limits, potassium, literal and solved mixtures, acetate
reserve, bare cations, unknown reservoirs, atomic refusal, repeated requests and
serialization. Native controls exercise actual PHREEQC settlement, both alkalis,
prior neutralization in either order and real weak-base/buffer negative arms.
The test-only baseline contains unchanged production runtime plus those controls;
its first actual failures must remain separate from changed-source acceptance.
Source preparation is not a pass. Native/core acceptance, applicable original
fleet/platform checks, protected integration and a separately bound original
OR006 replay remain required before any scientific selection.
