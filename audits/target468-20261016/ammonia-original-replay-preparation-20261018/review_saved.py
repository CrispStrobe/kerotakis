"""Read a once-issued GS005 response; no network or simulator invocation."""
import argparse
import decimal
import hashlib
import json
import pathlib
import subprocess

p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--repository', type=pathlib.Path, default=pathlib.Path.cwd())
p.add_argument('--out', type=pathlib.Path, required=True)
a = p.parse_args()
D = decimal.Decimal
decimal.getcontext().prec = 45
sha = lambda b: hashlib.sha256(b).hexdigest()
design_raw = (a.directory / 'bound-shared-diagnostics.json').read_bytes()
design = json.loads(design_raw)
accepted = json.loads((a.directory / 'production-acceptance.json').read_bytes())
assert len(design['cases']) == 1 and design['cases'][0]['id'] == 'new120:gs-005'
c = design['cases'][0]
engine = design['accepted_engine']
assert accepted['source'] == engine['source_head'] and accepted['engine'] == engine
assert accepted['browser_checks'] == 37 and accepted['eleven_frozen_script_controls_pass']
assert sha(c['script'].encode()) == c['script_sha256']
original = json.loads((pathlib.Path(__file__).resolve().parent / 'original-protocol-before-binding.json').read_bytes())['cases'][0]
assert all(c[k] == original[k] for k in ['id', 'script', 'script_sha256', 'original_forecast', 'original_criteria', 'changed_source_expectations'])
raw = (a.directory / 'shared-diagnostics/new120-gs-005/response.body').read_bytes()
response = json.loads(raw, parse_float=D)
assert response['engine'] == engine and response['script_sha256'] == c['script_sha256']
assert response['execution_status'] == 'completed' and response['exit_status'] == 0
get = lambda path: subprocess.check_output(['git', 'show', engine['source_head'] + ':' + path], cwd=a.repository)
registry_raw = get('data/registry/registry-source-v1.json')
assert sha(registry_raw) == design['actual_release_property_source_sha256']['data/registry/registry-source-v1.json']
registry = json.loads(registry_raw, parse_float=D)
formula = {r['species_id']: {**{e['element']: D(e['count']['value']) for e in r['elements']}, 'charge': D(r['net_charge']['value'])} for r in registry['compositions']}
formula['base_equivalents'] = formula['OH-'].copy()
bench = get('crates/kerotakis-core/src/bench.rs')
for key, value in [('N2', '0.7901'), ('O2', '0.2095'), ('CO2', '0.0004')]:
    assert f'const AIR_{key}: f64 = {value};'.encode() in bench
incoming, outgoing, state, seals, routes, residuals, limits = {}, {}, {}, {}, [], [], []

def book(destination, vessel, species, amount):
    d = destination.setdefault(vessel, {})
    for key, n in formula[species].items():
        d[key] = d.get(key, D(0)) + D(amount) * n

def ledger(v):
    result = {}
    assert not any(v.get(key, []) for key in ['unresolved_materials', 'material_objects', 'surfaces', 'electrodes', 'exchanges', 'adsorbed', 'solid_solutions'])
    for part in v['contents']:
        n = D(part['moles'])
        assert n.is_finite() and n >= 0
        for key, count in formula[part['species']].items():
            result[key] = result.get(key, D(0)) + n * count
    return result

def amount(v, species, phase=None):
    return sum((D(q['moles']) for q in v['contents'] if q['species'] == species and (phase is None or q['phase'] == phase)), D(0))

for i, row in enumerate(response['observations']):
    op = row['operator']
    assert op['op'] in ['add', 'seal', 'new_vessel', 'inspect'], op
    if op['op'] == 'add':
        book(incoming, op['vessel'], op['species'], op['moles'])
    for e in row['events']:
        assert e['event'] != 'solver_failed'
        if e['event'] == 'vessel_sealed':
            vid = e['vessel']
            assert vid not in seals
            seals[vid] = e['trapped_air']
            for species, fraction in [('N2', '.7901'), ('O2', '.2095'), ('CO2', '.0004')]:
                book(incoming, vid, species, D(e['trapped_air']) * D(fraction))
        if e['event'] in ['gas_absorbed', 'gas_evolved', 'evaporated', 'diluted']:
            dest = incoming if e['event'] in ['gas_absorbed', 'diluted'] else outgoing
            species = 'water' if e['event'] in ['evaporated', 'diluted'] else e['species']
            book(dest, e['vessel'], species, e['moles'])
        if e['event'] == 'solution_routed':
            routes.append(e)
        if e['event'] == 'not_yet_modeled':
            limits.append(e)
    published = {v['id']: v for v in row['bench']['vessels']}
    if set(state) - set(published):
        assert op['op'] == 'inspect' and not row['events']
        assert all(state[k] == v for k, v in published.items())
    state.update(published)
    for vid, v in state.items():
        actual, given, exported = ledger(v), incoming.get(vid, {}), outgoing.get(vid, {})
        r = {k: actual.get(k, D(0)) + exported.get(k, D(0)) - given.get(k, D(0)) for k in actual.keys() | given.keys() | exported.keys()}
        assert all(abs(x) < D('1e-8') for x in r.values()), (i, vid, r)
        residuals.append({'row': i, 'vessel': vid, 'material_charge_residual_mol_or_eq': r})
assert set(state) == set(seals) == {0, 1}
assert not limits, limits
arms = []
for vid in [0, 1]:
    v = state[vid]
    reduced = amount(v, 'NH3') + amount(v, 'NH4+')
    assert abs(reduced - D('.001')) < D('1e-8')
    assert D(v['temperature']).is_finite() and v['temperature'] > 0
    assert v['headspace']['boundary'] == 'sealed'
    arms.append({'vessel': vid, 'temperature_K': v['temperature'], 'pH': v['solution']['ph'], 'NH3_gas_mol': amount(v, 'NH3', 'gas'), 'NH3_nongas_mol': amount(v, 'NH3') - amount(v, 'NH3', 'gas'), 'NH4_mol': amount(v, 'NH4+'), 'reduced_nitrogen_mol': reduced, 'ammonium_fraction': amount(v, 'NH4+') / reduced, 'gas_NH3_fraction': amount(v, 'NH3', 'gas') / reduced, 'trapped_air_mol': seals[vid], 'total_owned_nitrogen_atoms_mol': ledger(v)['N']})
neutral, acid = arms
assert neutral['NH3_gas_mol'] > 0
ratio = acid['NH3_gas_mol'] / neutral['NH3_gas_mol']
assert ratio < D(str(c['changed_source_expectations']['acid_gas_NH3_relative_to_neutral_maximum']))
assert acid['ammonium_fraction'] > neutral['ammonium_fraction']
assert acid['gas_NH3_fraction'] < neutral['gas_NH3_fraction']
criteria = [dict(x, result='PASS') for x in c['original_criteria']]
criteria[0]['reason'] = 'Actual acidified inventory has higher ammonium fraction and robustly lower finite headspace NH3; both phase fractions, temperatures and pH are retained.'
criteria[1]['reason'] = 'Each original1mmol reduced-nitrogen addition closes separately; trapped-air N2 is counted in total owned nitrogen. All operation snapshots independently close represented atoms/charge, with explicit boundary accounting.'
result = {'source': engine['source_head'], 'engine': engine, 'case': c['id'], 'script_sha256': c['script_sha256'], 'raw_response_sha256': sha(raw), 'design_sha256': sha(design_raw), 'registry_sha256': sha(registry_raw), 'air_source_sha256': sha(bench), 'decimal_precision': 45, 'arms': arms, 'acid_to_neutral_gas_NH3_ratio': ratio, 'per_step_per_vessel_residuals': residuals, 'maximum_atom_charge_residual_mol_or_eq': max(abs(n) for r in residuals for n in r['material_charge_residual_mol_or_eq'].values()), 'actual_route_events': routes, 'original_criteria_review': criteria, 'new_engine_or_api_requests': 0, 'catalog_additions': 0, 'qualification': 'Saved actual changed-source original-protocol review. No empirical Henry/activity/rate calibration, general ligand ownership or isothermal matched-arm claim. Root raw/source/criterion review and an explicit separate selection remain required.'}
with a.out.open('x') as f:
    json.dump(result, f, indent=2, default=str)
    f.write('\n')
print(json.dumps({'case': c['id'], 'original_criteria_pass': 2, 'acid_gas_ratio': str(ratio), 'max_material_charge_residual': str(result['maximum_atom_charge_residual_mol_or_eq']), 'new_requests': 0}))
