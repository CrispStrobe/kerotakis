#!/usr/bin/env python3
"""Sixth fleet: original controls 195–218, no source-manual recipes."""
import run as recorder

c = recorder.case

def titrate(base=.0011, water=.42, acid=.04, step=7, predose=0):
    script = f"add v1 water {water}L\nadd v1 NaOH {base}mol"
    if predose:
        script += f"\nadd v1 HCl {predose}mol"
    return script + f"\ntitrate v1 HCl {acid}M {step}mL until ph 7"

def conduct(salt=None, amount=.00008, water=.4, sugar=0):
    script = f"add v1 water {water}L"
    if salt:
        script += f"\nadd v1 {salt} {amount}mol"
    if sugar:
        script += f"\nadd v1 glucose {sugar}mol"
    return script + "\nmeasure v1 conductivity"

def gas(n=.004, volume=.36, energies=(.2,)):
    return (f"seal v1 {volume}L\nadd v1 N2 {n}mol\ninspect v1" +
            "".join(f"\n{'heat' if e >= 0 else 'cool'} v1 {abs(e)}J" for e in energies) +
            "\nmeasure v1 pressure")

def ester(acid=.014, alcohol=.011, product=.003, water=.006, repeats=2):
    return (f"add v1 CH3COOH {acid}mol\nadd v1 ethanol {alcohol}mol\n"
            f"add v1 ethyl_acetate {product}mol\nadd v1 water {water}mol" +
            "\nreact v1 esterification" * repeats)

recorder.CASES = [
    c("195-descending-acid-titration", "Can acid titrant resolve a descending pH endpoint?", titrate()),
    c("196-descending-fine-trial", "Does a smaller trial increment preserve acid-equivalent demand?", titrate(step=1)),
    c("197-descending-half-concentration", "Does half concentration double endpoint volume?", titrate(acid=.02)),
    c("198-descending-neutralized-predose", "Does a defined acid predose reduce remaining base capacity?", titrate(predose=.0003)),
    c("199-descending-extra-solvent", "Does solvent dilution preserve strong-base equivalents?", titrate(water=.84)),
    c("200-descending-extensive-control", "Does a threefold extensive scale preserve endpoint concentration?", titrate(base=.0033, water=1.26, step=21)),
    c("201-conductivity-water-blank", "Is a water blank an explicit finite conductivity observation?", conduct()),
    c("202-conductivity-dilute-nitrate", "Does a dilute electrolyte exceed its water blank?", conduct("KNO3")),
    c("203-conductivity-double-nitrate", "Does doubling dilute charge carriers increase conductivity?", conduct("KNO3", amount=.00016)),
    c("204-conductivity-nitrate-extensive", "Does extensive scaling preserve an intensive electrical reading?", conduct("KNO3", amount=.00024, water=1.2)),
    c("205-conductivity-neutral-solute", "Does dilute neutral glucose fail to mimic a strong electrolyte?", conduct(sugar=.00008)),
    c("206-conductivity-chloride-control", "Does a second dilute registered electrolyte also exceed the blank?", conduct("KCl")),
    c("207-dry-gas-small-energy", "Does a sealed dry gas obey the pressure law after a small energy input?", gas()),
    c("208-dry-gas-wide-headspace", "Does more headspace retain atoms and pressure-law consistency?", gas(volume=.72)),
    c("209-dry-gas-partitioned-energy", "Does partitioning a small energy dose preserve its final state?", gas(energies=(.08,.12))),
    c("210-dry-gas-larger-energy", "Does more energy increase temperature in the same sealed inventory?", gas(energies=(.4,))),
    c("211-dry-gas-extensive-control", "Does scaling gas, trapped atmosphere, headspace and energy preserve temperature?", gas(n=.008, volume=.72, energies=(.4,))),
    c("212-dry-gas-energy-round-trip", "Does adding then removing the same energy recover the dry-gas state?", gas(energies=(.2,-.2))),
    c("213-ester-repeat-asymmetric", "Does a second equilibrium request leave an equilibrated asymmetric mixture unchanged?", ester()),
    c("214-ester-repeat-product-rich", "Is reverse equilibrium followed by an honest no-op?", ester(acid=.003, alcohol=.004, product=.016, water=.013)),
    c("215-ester-repeat-fourfold", "Does no-op behavior survive extensive scaling?", ester(acid=.056, alcohol=.044, product=.012, water=.024)),
    c("216-ester-repeat-low-water", "Does a low-water organic variant remain idempotent after equilibration?", ester(water=.0015)),
    c("217-ester-three-requests", "Do repeated equilibrium requests avoid repeated chemical conversion?", ester(repeats=3)),
    c("218-ester-balanced-quotient", "Does an initially balanced ideal molecular quotient avoid spurious conversion?", ester(acid=.009, alcohol=.012, product=.018, water=.024)),
]

if __name__ == "__main__":
    recorder.main()
