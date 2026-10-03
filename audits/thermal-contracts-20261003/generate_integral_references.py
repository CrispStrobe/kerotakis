"""Small independent 80-digit oracle for unchanged compiled NASA-9 records.

Decimal.from_float preserves the exact binary inputs used by Rust; converting
their decimal labels instead would test transcription rounding as integration.
This evaluates ordinary antiderivatives at high precision, independently of
the production factored, compensated algorithm. It executes no application.
"""
from decimal import Decimal, localcontext
import hashlib
import json
import math
from pathlib import Path

root = Path(__file__).resolve().parent
registry_path = root.parents[1] / "data/registry/registry-source-v1.json"
registry = json.loads(registry_path.read_text())
records = registry["heat_capacity_polynomials"]
cases = [
    ("water", "liquid", 273.15, 308.0889802715948),
    ("water", "liquid", 273.15, 298.15),
    ("water", "liquid", 273.15, 373.15),
    ("water", "liquid", 298.15, 298.150000001),
    ("water", "liquid", 298.15, math.nextafter(298.15, math.inf)),
    ("water", "liquid", math.nextafter(373.15, -math.inf), math.nextafter(373.15, math.inf)),
    ("water", "liquid", 373.15, 500.0),
    ("water", "liquid", 500.0, 600.0),
    ("water", "solid", 200.0, 273.15),
    ("water", "solid", math.nextafter(273.15, -math.inf), 273.15),
    ("N2", "gas", 200.0, 6000.0),
    ("CaCO3", "solid", 300.0, 1603.0),
]
references = []
with localcontext() as ctx:
    ctx.prec = 80
    gas_constant = Decimal.from_float(8.31446261815324)
    for species, phase, t0, t1 in cases:
        record = next(r for r in records if r["species_id"] == species and r["phase"] == phase)
        assert record["form"] == "nasa9"
        assert record["intervals"][0]["t_min_k"] <= t0 <= t1 <= record["intervals"][-1]["t_max_k"]
        total = Decimal(0)
        for interval in record["intervals"]:
            lower, upper = max(t0, interval["t_min_k"]), min(t1, interval["t_max_k"])
            if upper <= lower:
                continue
            c = [Decimal.from_float(float(value)) for value in interval["coefficients"]]

            def primitive(temperature):
                t = Decimal.from_float(float(temperature))
                return -c[0]/t + c[1]*t.ln() + sum(c[i]*t**(i-1)/Decimal(i-1) for i in range(2, 7))

            total += gas_constant * (primitive(upper)-primitive(lower))
        references.append(dict(species=species, phase=phase, t0=t0, t1=t1,
                               joules_per_mol=float(total), high_precision_joules_per_mol=str(total)))
(root / "integral-references.json").write_text(json.dumps(dict(
    precision_digits=80, registry_sha256=hashlib.sha256(registry_path.read_bytes()).hexdigest(),
    input_basis="exact binary64 coefficients, temperatures and gas constant", references=references), indent=2)+"\n")
print(f"Generated {len(references)} independent integral references")
