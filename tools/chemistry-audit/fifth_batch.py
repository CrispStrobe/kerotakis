#!/usr/bin/env python3
"""Fifth discovery fleet: 36 inputs, frozen before execution (159–194)."""
import run as recorder

c = recorder.case


def silver(ag=.0009, chloride=.0003, water=.3, reverse=False):
    feeds = [f"add v1 AgNO3 {ag}mol", f"add v1 NaCl {chloride}mol"]
    return f"add v1 water {water}L\n" + "\n".join(reversed(feeds) if reverse else feeds)


def zinc(base=.0006, scale=1, reverse=False):
    feeds = [f"add v1 ZnSO4 {.0008 * scale}mol", f"add v1 KOH {base * scale}mol"]
    return f"add v1 water {.4 * scale}L\n" + "\n".join(reversed(feeds) if reverse else feeds)


def titration(acid=.0007, water=.35, concentration=.05, step=3, preload=0):
    script = f"add v1 water {water}L\nadd v1 H2SO4 {acid}mol"
    if preload:
        script += f"\nadd v1 KOH {preload}mol"
    return script + f"\ntitrate v1 KOH {concentration}M {step}mL until ph 7"


def electrolysis(space=.12, current=.15, seconds=80, split=False, scale=1):
    script = f"add v1 water {.3 * scale}L\nadd v1 KNO3 {.0012 * scale}mol\nseal v1 {space}L"
    commands = [seconds / 2, seconds / 2] if split else [seconds]
    return script + "".join(f"\nelectrolyse v1 {current}A {t}s" for t in commands) + "\nmeasure v1 pressure"


def mix(scale=1, dilution=1, fraction=1, right=False, reverse=False):
    script = (f"add v1 water {.15 * scale * dilution}L\nadd v1 CaCl2 {.0006 * scale}mol\nnew\n"
              f"add v2 water {.25 * scale * dilution}L\nadd v2 MgSO4 {.0005 * scale}mol\nnew\n"
              f"add v3 water {.35 * scale * dilution}L\nadd v3 NaNO3 {.0007 * scale}mol\nnew\n")
    a, b, d = .4 * fraction, .6 * fraction, .8 * fraction
    if right:
        return script + f"mix v2 {b} v3 {d} into v4\nnew\nmix v1 {a} v4 1 into v5"
    operands = f"v2 {b} v1 {a}" if reverse else f"v1 {a} v2 {b}"
    return script + f"mix {operands} into v4\nnew\nmix v4 1 v3 {d} into v5"


def ester(scale=1, ethanol=.009, water=.004, reverse=False, repeat=False):
    feeds = [("CH3COOH", .017), ("ethanol", ethanol), ("ethyl_acetate", .002), ("water", water)]
    if reverse:
        feeds.reverse()
    script = "\n".join(f"add v1 {key} {n * scale}mol" for key, n in feeds)
    return script + "\nreact v1 esterification" * (2 if repeat else 1)


recorder.CASES = [
    c("159-chloride-limited-silver", "Does unequal Ag/Cl feed respect the chloride capacity?", silver()),
    c("160-chloride-limited-reordered", "Is precipitated silver independent of feed order?", silver(reverse=True)),
    c("161-silver-limited-chloride", "Does exchanging the excess reagent exchange the limiting inventory?", silver(ag=.0003, chloride=.0009)),
    c("162-silver-extensive-scale", "Does fourfold extensive scaling scale the silver solid?", silver(ag=.0036, chloride=.0012, water=1.2)),
    c("163-silver-solvent-dilution", "Does fourfold dilution preserve atoms and not increase precipitated silver?", silver(water=1.2)),
    c("164-silver-equal-low-feed", "Does equal low feed precipitate within its stoichiometric capacity?", silver(ag=.0003)),
    c("165-zinc-substoichiometric-base", "Does zinc solid stay below half the supplied hydroxide equivalents?", zinc()),
    c("166-zinc-base-first", "Does reversing a base-limited zinc feed preserve precipitation?", zinc(reverse=True)),
    c("167-zinc-double-base", "Does doubling a still-limiting hydroxide dose increase zinc precipitation?", zinc(base=.0012)),
    c("168-zinc-quarter-base", "Does a quarter base dose reduce precipitation within the tighter capacity?", zinc(base=.00015)),
    c("169-zinc-base-limited-scale", "Does threefold solvent/feed scaling scale the zinc precipitate?", zinc(scale=3)),
    c("170-zinc-acid-removes-limited-solid", "Does excess strong acid dissolve the previously limited zinc solid?", zinc() + "\nadd v1 HCl 0.0015mol"),
    c("171-diprotic-titration-capacity", "Does sulfate require two KOH equivalents at the neutral endpoint?", titration()),
    c("172-diprotic-coarse-burette", "Does coarse dosing refine to the same diprotic capacity?", titration(step=11)),
    c("173-diprotic-dilute-burette", "Does halving burette concentration double endpoint volume?", titration(concentration=.025)),
    c("174-diprotic-extensive-scale", "Does doubling all extensive quantities double titrant demand?", titration(acid=.0014, water=.7, step=6)),
    c("175-diprotic-solvent-control", "Does doubling solvent leave the acid-equivalent capacity unchanged?", titration(water=.7)),
    c("176-diprotic-partly-spent", "Does a known KOH predose reduce subsequent titrant demand by its own equivalents?", titration(preload=.0004)),
    c("177-sealed-electrolysis-charge", "Do both Faraday products remain accounted for in finite headspace?", electrolysis()),
    c("178-sealed-electrolysis-wide-space", "Does changing headspace preserve electrode production and the ideal gas law?", electrolysis(space=.48)),
    c("179-sealed-electrolysis-equal-charge", "Does an alternative current/time pair conserve charge yield in a sealed vessel?", electrolysis(current=.3, seconds=40)),
    c("180-sealed-electrolysis-split-charge", "Does a divided electrical dose preserve cumulative electrode production?", electrolysis(split=True)),
    c("181-sealed-electrolysis-double-charge", "Does doubling charge double both generated products?", electrolysis(seconds=160)),
    c("182-sealed-electrolysis-extensive-scale", "Does scaling solution, headspace and charge preserve gas pressure?", electrolysis(space=.24, seconds=160, scale=2)),
    c("183-three-salt-fractional-mix", "Does serial fractional MIX conserve divalent and spectator feeds?", mix()),
    c("184-three-salt-regrouped", "Does regrouping three salt feeds preserve the receiver?", mix(right=True)),
    c("185-three-salt-operands-reversed", "Does reversing first MIX operands preserve the receiver?", mix(reverse=True)),
    c("186-three-salt-extensive-scale", "Does fourfold extensive scaling scale the selected receiver inventories?", mix(scale=4)),
    c("187-three-salt-double-solvent", "Does doubling only solvent preserve receiver salt atoms?", mix(dilution=2)),
    c("188-three-salt-half-fractions", "Does halving all withdrawal fractions halve receiver atoms?", mix(fraction=.5)),
    c("189-ester-alcohol-limited", "Does asymmetric alcohol-limited equilibrium solve the current mass-action constraint?", ester()),
    c("190-ester-feed-order", "Does reversed organic feed order preserve signed equilibrium extent?", ester(reverse=True)),
    c("191-ester-threefold-scale", "Does threefold organic scale preserve intensive equilibrium?", ester(scale=3)),
    c("192-ester-repeat-equilibrium", "Is a second equilibrium request idempotent rather than a second yield recipe?", ester(repeat=True)),
    c("193-ester-more-alcohol", "Does increasing alcohol increase forward extent at otherwise fixed feed?", ester(ethanol=.018)),
    c("194-ester-more-water", "Does adding initial water suppress forward extent?", ester(water=.008)),
]

if __name__ == "__main__":
    recorder.main()
