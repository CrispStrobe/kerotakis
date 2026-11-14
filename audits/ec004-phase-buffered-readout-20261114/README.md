# EC004: phase-buffered silver activity readout

Both unchanged original criteria pass with **two distinct execution stages**. The historical native chemical prefix at `4bd93add656c936b0724673c7ea8332cf21e906f` supplies the actual precipitated AgCl and native activities. Its executable checksum was never recorded and remains null. The pure readout continuation at `4151791fafb05dbc88ce39baec83b33ca4dd7b05` reads the exact saved pair through the actual compiled core. This is not a new native chemistry replay or a uniform current-CLI execution.

The source-bound pair SHA256 is `2bdc16d1848513c8b7eb8d921e6f0b9e19aecfcbcd0bd796b2f3d4b2d46e8fff`. The independently hashed readout executable is `7618519809785d5673b6e6689edb6eeaa9906aa6b121482fe6db062539ccfe3c`. The historical producer lock is `a43bbe9a6ce7be1b5ecbbfbf091e25124c60122b08bd2ba003c8e250d3262bdd`; the changed-core producer lock is `c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7`. Neither lock is claimed source-committed. [Verification receipt](VERIFIED.json) and [independent review](first-review.json) keep these identities separate.

## Actual original criteria

C1: treated solid AgCl is 0.00009994411608353 mol, each silver electrode retains 0.001 mol, and native free Ag+ activity drops from 0.0009674 to 0.0000001865. All59 known H/O/Ag/N/Na/Cl/charge budget checkpoints close within1.78e-15 mol/eq, below the unchanged1e-10 guard. No solver failure, opaque stock or pending gas crossing is accepted.

C2: the original equal-pair cell readout reports zero; the treated pair reports **0.21978722436811088 V**, anode v2/cathode v1, with zero standard-couple difference. Independent `(R/F)*(T1*ln(a1)-T2*ln(a2))` gives0.2197872243681109 V using each captured temperature, not an imposed common25 C. Explicit positive/negative leads yield equal opposite signs; the legacy cell magnitude stays positive. Every readout leaves the complete vessel arrays exactly unchanged.

Historical C1 was already true, but historical C2 remained false because the trace-ion admission refused the readout. The forecast is assessed against actual changed-core readout events, not the theoretical calculation alone. The original script and two criteria are unchanged.

## Controls and limits

The single bounded command actually passes10 matched tests (5new,3signed-lead,2acid-cell), including27 incomplete/nonfinite/unowned/ambiguous negative cases. Ten independent reader mutations reject. `electrode`, `electrolyse`, and `pin_electrode` bodies remain byte-exact relative to the prerequisite source; their original observable-amount thresholds are retained. The extension requires owned aqueous Ag+/Cl-, visible retained Ag and AgCl solids, complete finite native activities and finite stock/solvent ownership. It admits no molality fallback, new standard potential, kinetic/passivation or liquid-junction claim. This qualifies the supported Ag+/Ag activity readout only, not a dedicated calibrated Ag/AgCl electrode model.

[Raw historical observations](ec004-historical.stdout), [separate actual readout events](readouts/after_chloride.json), [signed leads](readouts/signed-leads.json), [negative cases](readouts/negative-cases.json), [raw command result](readout-controls.stdout), [independent reader](review.py), and [frozen forecast](frozen-design.json) are preserved. No combined synthetic CLI transcript is created. The compiled binary and genuine locks remain in authenticated producer artifacts; their measured hashes are preserved here. Full source/platform/fleet acceptance and PR882/PR883 prerequisite integration remain separate delivery requirements.
