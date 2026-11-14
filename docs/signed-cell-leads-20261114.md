# Ordered voltmeter leads — 2026-11-14

`voltmeter positive v1 negative v2` reads `E(v1) − E(v2)` in volts.
Swapping the named leads reverses the sign without drawing current or changing
vessel contents. `cell v1 v2` and its existing `voltmeter v1 v2` alias retain
their automatic anode/cathode selection and positive offered-cell potential.
The named lead words are fixed grammar, like `from` and `into`.

The new `voltmeter_read` event carries `positive`, `negative`, and signed `volts`.
For a nonzero admitted cell, the existing cell model event follows, preserving
its activity-based Nernst explanation or bounded zinc/acid/copper estimate.
The signed event has its own CLI semantic scalar `event:voltmeter_read.volts`.
EN/DE/FR rendering labels the leads and retains the numerical sign.
Two admitted equal-potential half-cells read zero; legacy `cell` retains its
existing identical-half-cell refusal. Missing electrodes and invalid vessel
pairs remain refused. The observable-ion admission threshold is unchanged.
No liquid-junction, load, internal resistance, or kinetic model is added.

RX-012's original script uses `cell v1 v2` followed by `cell v2 v1`; its forecast
expects opposite signs. That original cannot meet the forecast unchanged under
the intentionally preserved legacy interface. The new explicit syntax supplies
the requested observable as a separate interface follow-up, not an original
qualification. The original script and its forecast must remain unchanged.
EC-004's trace silver-ion admission issue remains held: lowering the observable
threshold would also affect the documented equivalence/addition-order guard.

Focused tests cover parser/serialization compatibility, exact lead reversal,
activity-based magnitudes in zinc and copper concentration cells, Daniell and
copper/silver cells, equal-potential zero, no vessel mutation, missing electrodes,
invalid vessel pairs, bounded acid estimates, locale signs and REST admission.
Rust runtime and PHREEQC tests require hosted validation; no local native build
or scientific acceptance is claimed by this implementation note.
