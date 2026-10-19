"""Independently review the unchanged OR006 response; never execute chemistry."""
import argparse, hashlib, json, subprocess
from decimal import Decimal, localcontext
from pathlib import Path

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--directory', type=Path, default=Path(__file__).resolve().parent)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    assert not args.out.exists()
    folder = args.directory
    sha = lambda data: hashlib.sha256(data).hexdigest()
    load = lambda path: json.loads(path.read_bytes(), parse_float=Decimal)
    design = load(folder/'design.json')
    props = load(folder/'properties.json')
    raw = (folder/'responses/new120-or-006.json').read_bytes()
    response = json.loads(raw, parse_float=Decimal)
    assert len(design['cases']) == 1
    case = design['cases'][0]
    assert case['id'] == 'new120:or-006'
    assert response['engine'] == design['accepted_engine']
    assert sha(case['script'].encode()) == response['script_sha256'] == case['script_sha256']
    assert response['exit_status'] == 0
    registry_raw = subprocess.check_output(['git', 'show', props['source']+':data/registry/registry-source-v1.json'])
    assert sha(registry_raw) == props['registry_source_sha256']
    registry = json.loads(registry_raw, parse_float=Decimal)
    compositions = {r['species_id']:r for r in registry['compositions']}
    needed = {p['species'] for r in response['observations'] for v in r['bench']['vessels'] for p in v['contents']}
    assert needed == {'water', 'ethyl_acetate', 'Na+', 'base_equivalents', 'ethanol', 'CH3COOH', 'CH3COO-'}
    added = {r['operator']['species'] for r in response['observations'] if r['operator']['op']=='add'}
    for key in (needed | added) - {'base_equivalents'}:
        row = compositions[key]
        actual = props['species'][key]
        assert actual['elements'] == {e['element']:e['count']['value'] for e in row['elements']}
        assert actual['charge'] == row['net_charge']['value']
    basis = props['species']['base_equivalents']
    assert basis['elements'] == {'H':1,'O':1} and basis['charge'] == -1
    def amount(v, keys):
        return sum((p['moles'] for p in v['contents'] if p['species'] in keys), Decimal(0))
    def ledger(v):
        out = {}
        for p in v['contents']:
            n = p['moles']
            assert n.is_finite() and n >= 0
            data = props['species'][p['species']]
            for key, coefficient in {**data['elements'], 'charge':data['charge']}.items():
                out[key] = out.get(key, Decimal(0)) + n * Decimal(coefficient)
        # solute_charge/free-ion meters are derived coordinates, not extra ions.
        return out
    with localcontext() as ctx:
        ctx.prec = 45
        previous = {}; rows = []; maximum = Decimal(0)
        inspections = []
        for row in response['observations']:
            assert len(row['bench']['vessels']) == 1
            v = row['bench']['vessels'][0]
            assert v['id'] == 0
            assert not any(e['event'] in {'solver_failed', 'gas_evolved', 'gas_absorbed'} for e in row['events'])
            op = row['operator']; expected = previous.copy()
            if op['op'] == 'add':
                data = props['species'][op['species']]
                for key, coefficient in {**data['elements'], 'charge':data['charge']}.items():
                    expected[key] = expected.get(key, Decimal(0)) + Decimal(coefficient) * op['moles']
            else:
                assert op['op'] in {'inspect', 'react'}
            actual = ledger(v)
            residual = {k:actual.get(k,Decimal(0))-expected.get(k,Decimal(0)) for k in actual.keys()|expected.keys()}
            worst = max(map(abs,residual.values()),default=Decimal(0)); maximum = max(maximum,worst)
            assert worst <= case['changed_source_expectations']['represented_material_charge_resolution_mol_or_eq']
            rows.append({'operator':op,'residual':residual})
            previous = actual
            if op['op'] == 'inspect': inspections.append(v)
        assert len(inspections) == 2
        before, after = inspections
        ester_before = amount(before,{'ethyl_acetate'}); ester_after = amount(after,{'ethyl_acetate'})
        consumed = ester_before-ester_after
        ethanol = amount(after,{'ethanol'})-amount(before,{'ethanol'})
        acetate = amount(after,{'CH3COO-','CH3COOH'})-amount(before,{'CH3COO-','CH3COOH'})
        tol = case['changed_source_expectations']['stoichiometric_resolution_mol']
        reserve_before = amount(before,{'Na+'})-amount(before,{'CH3COO-','CH3COOH'})
        reserve_after = amount(after,{'Na+'})-amount(after,{'CH3COO-','CH3COOH'})
        assert abs(reserve_before-Decimal('0.005')) <= tol and abs(reserve_after) <= tol
        assert ester_after > tol and abs(consumed-reserve_before) <= tol
        assert consumed > tol and abs(consumed-ethanol) <= tol and abs(consumed-acetate) <= tol
        events = [e for r in response['observations'] for e in r['events'] if e['event']=='org_reacted']
        assert len(events)==1 and events[0]['name']=='saponification'
        assert abs(events[0]['extent']-consumed) <= tol
        result={'source':props['source'],'engine':response['engine'],'response_sha256':sha(raw),
          'original_criteria_passed':['OR-006-C1','OR-006-C2'],'ester_remaining_mol':ester_after,
          'ester_consumed_mol':consumed,'ethanol_formed_mol':ethanol,'acetate_family_formed_mol':acetate,
          'restricted_stoichiometric_reserve_before_mol':reserve_before,'restricted_stoichiometric_reserve_after_mol':reserve_after,
          'maximum_per_step_atom_charge_residual':maximum,'per_step_residuals':rows,
          'temperatures_K':[before['temperature'],after['temperature']], 'new_engine_or_API_requests':0,
          'qualification':'Explicit overall template and this closed analytical inventory only; not a rate, heat, activity-model or empirical calibration claim. Free-ion/base-equivalent meters alone are not available-base capacity.'}
    args.out.write_text(json.dumps(result,indent=2,default=str)+'\n')
    print(json.dumps({'original_criteria_passed':result['original_criteria_passed'],'max_residual':str(maximum),'new_requests':0}))

if __name__=='__main__': main()
