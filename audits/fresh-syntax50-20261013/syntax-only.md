# Kero English script syntax reference

One command per line. Blank lines and lines starting with # are ignored.
Arguments are whitespace-separated. Quantities attach their unit with no space.
Vessel references use v followed by an integer. A fresh CLI process is a fresh bench.
A fresh bench already has v1. Each new creates the next vessel (v2, then v3).
Use inspect to request state; no vessel argument requests all vessels.
Ingredient identifiers are single tokens: chemical formula keys or named material keys.
This reference deliberately does not enumerate the registry or assert supported chemistry.
You may choose plausible ingredient identifiers from your own knowledge; availability is a question for execution.

```
new
remove <vessel>
discard <vessel>
add <vessel> <ingredient> <number>mol|g|mL [@ <number>C]
stock <ingredient> <number>mol|g|mL
heat <vessel> <number>J|kJ [on burner|hotplate|candle]
cool <vessel> <number>J|kJ
wait <number>s|min|h
stir <vessel> [<number>rpm] [<number>s|min]
seal <vessel> <number>mL|L
regulate <vessel> <number>Pa|kPa|bar|atm <number>mL|L
sweep <vessel> <number>Pa|kPa|bar|atm
open <vessel>
ignite <vessel>
filter <from-vessel> <to-vessel>
magnet <from-vessel> <to-vessel>
evaporate <vessel> <fraction>
decant <from-vessel> <to-vessel> <fraction>
drain <from-vessel> <to-vessel>
extract <from-vessel> <to-vessel> <solvent-key> <number>mol|g|mL [stages <integer>]
distil <from-vessel> <to-vessel> <fraction> [stages <integer>]
distil <from-vessel> <to-vessel> <number>J|kJ [stages <integer>]
dilute <vessel> <number>mL|L
titrate <vessel> <titrant-key> [<number>M] <number>mL|L until ph <number> [max <integer>]
titrate <vessel> <titrant-key> [<number>M] <number>mL|L until pe >|>=|<|<= <number> [max <integer>]
titrate <vessel> <titrant-key> [<number>M] <number>mL|L until colour persists [max <integer>]
measure <vessel> <instrument>
look <vessel>
particles <vessel>
smell <vessel>
test <vessel> pop|splint|limewater|litmus
chromatograph <vessel>
cell <vessel> <vessel>
electrolyse <vessel> <number>A <number>s|min|h
grind <vessel> <ingredient-key> <number>um
centrifuge <vessel> <number>rpm <number>s <number>cm [<number>g]
irradiate <vessel> <number>nm <number>W/m2
transport <vessels...> from <inlet-vessel> to <receiver-vessel> steps <integer> [courant <number>]
react <vessel> <reaction-name>
inspect [<vessel>]
register lv1|lv2|lv3
```

Instrument tokens: thermometer, balance, ph, eyes, pressure, volume,
conductivity, density, spectrophotometer, calorimeter, chromatograph,
geiger, melting_point, boiling_point.

Fraction syntax is a bare decimal. Scientific notation is usable for numbers.
CLI execution (operator only, not the experiment designer): kero run FILE.lab --json.
No experiments, outcomes, solver behavior, or corpus examples are included here.
