"""Independent Decimal inventory and reaction review of actual native states."""
import decimal
import hashlib
import json
import pathlib
import subprocess

B = pathlib.Path(__file__).resolve().parent
D = decimal.Decimal
decimal.getcontext().prec = 45
proof = json.loads((B / 'ROOT-CONTROLS-AUTHENTICATED.json').read_bytes())
assert proof['accepted']
source = proof['source']
repo = B.parents[2]
get = lambda path: subprocess.check_output(['git', 'show', source + ':' + path], cwd=repo)
registry_raw = get('data/registry/registry-source-v1.json')
registry = json.loads(registry_raw, parse_float=D)
compositions = {x['species_id']: {**{a['element']: D(a['count']['value']) for a in x['elements']}, 'charge': D(x['net_charge']['value'])} for x in registry['compositions']}
# The basis carrier is a deliberately distinct key with OH's conserved formula.
assert 'pub const BASE_EQUIVALENTS: &str = "base_equivalents";' in get('crates/kerotakis-core/src/species.rs').decode()
compositions['base_equivalents'] = dict(compositions['OH-'])
roots = list((B / 'output').rglob('fix-1.stderr'))
assert len(roots) == 1
groups = {key: [] for key in ['NAMED_ALKALINE_STATE', 'NAMED_ALKALINE_ACID', 'NAMED_ALKALINE_NEGATIVE']}
for line in roots[0].read_text().splitlines():
    for prefix in groups:
        if line.startswith(prefix + ' '):
            groups[prefix].append(json.loads(line[len(prefix) + 1:], parse_float=D))
assert [len(rows) for rows in groups.values()] == [4, 2, 3]

def amount(v, key):
    return sum((D(p['moles']) for p in v['contents'] if p['species'] == key), D(0))

def ledger(v):
    result = {}
    assert not any(v.get(k, []) for k in ['unresolved_materials', 'material_objects', 'surfaces', 'electrodes', 'exchanges', 'adsorbed', 'solid_solutions'])
    for p in v['contents']:
        n = D(p['moles'])
        assert n.is_finite() and n >= 0
        for key, count in compositions[p['species']].items():
            result[key] = result.get(key, D(0)) + n * count
    return result

reviews = []
for prefix, rows in groups.items():
    for row in rows:
        before, after = row['before'], row['after']
        a, b = ledger(before), ledger(after)
        residual = {key: b.get(key, D(0)) - a.get(key, D(0)) for key in set(a) | set(b)}
        assert all(abs(value) < D('1e-9') for value in residual.values()), residual
        expected = min(D(row['base']), D('.01')) if prefix == 'NAMED_ALKALINE_STATE' else (D('.002') if prefix == 'NAMED_ALKALINE_ACID' else D(0))
        alcohol = amount(after, 'ethanol') - amount(before, 'ethanol')
        ester_spent = amount(before, 'ethyl_acetate') - amount(after, 'ethyl_acetate')
        acetate = sum((amount(after, key) - amount(before, key) for key in ['CH3COOH', 'CH3COO-', 'NaOAc']), D(0))
        assert all(abs(value - expected) < D('1e-9') for value in [alcohol, ester_spent, acetate]), (prefix, expected, alcohol, ester_spent, acetate)
        assert all(abs(residual.get(key, D(0))) < D('1e-9') for key in ['Na', 'K', 'charge'])
        if prefix != 'NAMED_ALKALINE_NEGATIVE':
            # Independent certificate reconstruction, distinct from the Rust helper.
            m = sum((amount(before, key) for key in ['Na+', 'K+', 'NaOH', 'KOH', 'NaOAc', 'NaCl', 'KCl', 'NaBr', 'KBr', 'NaNO3', 'KNO3']), D(0))
            x = sum((amount(before, key) for key in ['Cl-', 'Br-', 'NO3-', 'HCl', 'HBr', 'HNO3', 'NaCl', 'KCl', 'NaBr', 'KBr', 'NaNO3', 'KNO3']), D(0))
            ac = sum((amount(before, key) for key in ['CH3COOH', 'CH3COO-', 'NaOAc']), D(0))
            assert abs(min(max(D(0), m - x - ac), amount(before, 'ethyl_acetate')) - expected) < D('1e-9')
        reviews.append({'group': prefix, 'arm': {key: value for key, value in row.items() if key not in ['before', 'after']}, 'expected_extent': expected, 'alcohol_gain': alcohol, 'ester_spent': ester_spent, 'acetate_gain': acetate, 'whole_material_charge_residual_mol_or_eq': residual})
result = {'source': source, 'registry_source_sha256': hashlib.sha256(registry_raw).hexdigest(), 'decimal_precision': 45, 'actual_native_arms': len(reviews), 'reviews': reviews, 'maximum_material_charge_residual_mol_or_eq': max(abs(n) for review in reviews for n in review['whole_material_charge_residual_mol_or_eq'].values()), 'scientific_catalog_additions': 0, 'qualification': 'Bounded named to-completion controls, not rate/heat calibration or general weak-buffer chemistry. Carbonate/ammonia negative arms demonstrate refusal of unproved capacity, not chemical inactivity. Remaining full-fleet/platform/protected and delivery acceptance are separate.'}
out = B / 'ROOT-INDEPENDENT-CHEMICAL-REVIEW.json'
encoded = json.dumps(result, indent=2, default=str) + '\n'
if out.exists():
    assert json.loads(out.read_bytes()) == json.loads(encoded)
else:
    out.write_text(encoded)
print(json.dumps({'actual_native_arms': len(reviews), 'max_residual': str(result['maximum_material_charge_residual_mol_or_eq']), 'catalog_additions': 0}))
