# Chemical input names

A small reviewed set of names resolves to existing species keys:

| Input spelling | Stored species key |
| --- | --- |
| `H2O` | `water` |
| `oxygen` | `O2` |
| `nitrogen` | `N2` |
| `carbon dioxide` | `CO2` |
| `sodium chloride` | `NaCl` |

These exact spellings work in ingredient arguments, including multiword names:

```text
stock sodium chloride 1mol
add v1 sodium chloride 0.25mol
```

The same names resolve through direct/JSON species identity inputs. Operators,
inventory and saved state retain canonical keys. Mixing names and keys draws
from one finite bottle; restoring independently saved alias and canonical stock
entries together is refused rather than summing them silently. Existing German
and French name parsing remains available.

Formula case and identity distinctions remain significant. Names do not imply
new reaction support, and a formula is not a general alias: carbon dioxide gas,
dry ice and aqueous carbon dioxide keep distinct keys, as do nitrogen and liquid
nitrogen. Other names and synonyms are not automatically accepted. Optional packs
cannot install a species or material under a reserved input alias.

Story kit supplies come from the engine's parsed lesson ingredients. A lesson
using `carbon dioxide` supplies `CO2`; it does not loan a separate ingredient
called `carbon`. Availability, finite usage, safety refusals and positive-dose
accounting apply to the canonical ingredient.
