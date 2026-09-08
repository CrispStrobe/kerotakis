#!/usr/bin/env python3
"""Seventh fleet: 30 original controls for the repaired coverage tail."""
import run as recorder

c = recorder.case


def ammonia(amount=.01, water=0.0, temperature=None):
    lines = []
    if water:
        suffix = f" @ {temperature}C" if temperature is not None else ""
        lines.append(f"add v1 water {water}L{suffix}")
    lines += ["seal v1 0.5L", f"add v1 NH3 {amount}mol", "test v1 litmus"]
    return "\n".join(lines)


def ester(acid=.01, alcohol=.01, product=0.0, water=0.0, reverse=False, repeats=1):
    feeds = [("CH3COOH", acid), ("ethanol", alcohol)]
    if reverse:
        feeds.reverse()
    feeds += [("ethyl_acetate", product), ("water", water)]
    lines = [f"add v1 {name} {amount}mol" for name, amount in feeds if amount]
    lines += ["react v1 esterification"] * repeats
    return "\n".join(lines)


def gas_test(species, amount, test):
    return f"seal v1 0.5L\nadd v1 {species} {amount}mol\ntest v1 {test}"


def pressure(amount=.05, volume=1.0, heat=0.0, opened=False, species="N2"):
    lines = [f"add v1 {species} {amount}mol", f"seal v1 {volume}L", "measure v1 pressure"]
    if heat:
        lines += [f"heat v1 {heat}J", "measure v1 pressure"]
    if opened:
        lines += ["open v1", "measure v1 pressure"]
    return "\n".join(lines)


recorder.CASES = [
    c("459-ammonia-half-dose", "Does a half-dose of ammonia reach a finite sealed headspace?", ammonia(.005)),
    c("460-ammonia-full-dose", "Does more ammonia put more ammonia in the same headspace?", ammonia(.01)),
    c("461-ammonia-water", "Does water retain part of ammonia against the headspace?", ammonia(.01, .1)),
    c("462-ammonia-more-water", "Does more water retain a larger share of ammonia?", ammonia(.01, .5)),
    c("463-ammonia-cold-water", "Does cold water retain more ammonia than warm water?", ammonia(.01, .1, 10)),
    c("464-ammonia-warm-water", "Does warming shift ammonia toward the headspace?", ammonia(.01, .1, 40)),
    c("465-ester-balanced-feed", "Does an equal acid/alcohol feed approach the declared ester equilibrium?", ester()),
    c("466-ester-water-rich", "Does added water reduce the forward esterification extent?", ester(water=.04)),
    c("467-ester-product-rich", "Can a product-rich feed approach the same equilibrium from reverse?", ester(.003, .004, .016, .013)),
    c("468-ester-extensive-scale", "Does extensive scaling preserve the ester equilibrium quotient?", ester(.02, .02)),
    c("469-ester-feed-order", "Does reversing acid and alcohol feed order preserve equilibrium?", ester(reverse=True)),
    c("470-ester-repeat", "Is a repeated ester equilibrium request an explicit material no-op?", ester(repeats=2)),
    c("471-ester-low-water", "Does a low-water feed lie between dry and water-rich conversion?", ester(water=.002)),
    c("472-ester-products-only", "Can ester and water regenerate acid and alcohol?", ester(0, 0, .01, .01)),
    c("473-pop-positive", "Does hydrogen above its ignition floor give the curated pop verdict?", gas_test("H2", .01, "pop")),
    c("474-pop-negative", "Does trace hydrogen stay below the pop-test floor?", gas_test("H2", .00001, "pop")),
    c("475-splint-positive", "Does oxygen enrichment above the threshold relight a splint?", gas_test("O2", .02, "splint")),
    c("476-splint-negative", "Does trace added oxygen stay below the enrichment threshold?", gas_test("O2", .00001, "splint")),
    c("477-limewater-positive", "Does carbon dioxide above the trace floor turn limewater cloudy?", gas_test("CO2", .01, "limewater")),
    c("478-limewater-negative", "Does trace carbon dioxide remain below the limewater floor?", gas_test("CO2", .000001, "limewater")),
    c("479-litmus-positive", "Does partitioned ammonia above the trace floor turn damp litmus blue?", gas_test("NH3", .008, "litmus")),
    c("480-litmus-negative", "Does trace partitioned ammonia remain below the litmus floor?", gas_test("NH3", .000001, "litmus")),
    c("481-pressure-reference", "Does a sealed nitrogen charge obey the ideal-gas pressure relation?", pressure()),
    c("482-pressure-double-moles", "Does doubling added nitrogen increase pressure at fixed volume?", pressure(.10)),
    c("483-pressure-double-volume", "Does doubling headspace reduce the added-gas pressure contribution?", pressure(.05, 2.0)),
    c("484-pressure-heated", "Does heating a fixed sealed gas increase its pressure consistently?", pressure(heat=20)),
    c("485-pressure-extensive", "Does doubling gas and headspace preserve the added-gas pressure contribution?", pressure(.10, 2.0)),
    c("486-vent-nitrogen", "Does opening a pressurised nitrogen vessel return it to ambient pressure?", pressure(opened=True)),
    c("487-vent-carbon-dioxide", "Does opening a pressurised carbon-dioxide vessel return it to ambient pressure?", pressure(opened=True, species="CO2")),
    c("488-pressure-small-charge", "Does a smaller nitrogen charge produce a smaller pressure rise?", pressure(.01)),
]


if __name__ == "__main__":
    recorder.main()
