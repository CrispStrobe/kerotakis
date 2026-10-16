#!/usr/bin/env python3
"""Review saved protocol evidence only; never issue or repeat an API request."""
import argparse
import datetime
import hashlib
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
FORMULAS = {
    'CH3COOH': ({'C': 2, 'H': 4, 'O': 2}, 0),
    'ethanol': ({'C': 2, 'H': 6, 'O': 1}, 0),
    'ethyl_acetate': ({'C': 4, 'H': 8, 'O': 2}, 0),
    'water': ({'H': 2, 'O': 1}, 0), 'H2O2': ({'H': 2, 'O': 2}, 0),
    'MnO2': ({'Mn': 1, 'O': 2}, 0), 'O2': ({'O': 2}, 0),
}

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def pairs(items):
    result = {}
    for key, value in items:
        assert key not in result, 'duplicate JSON key'
        result[key] = value
    return result

def read(path):
    return json.loads(path.read_text(), object_pairs_hook=pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))

def finite(value):
    if type(value) in (float, int):
        assert math.isfinite(value)
    elif isinstance(value, dict):
        for nested in value.values(): finite(nested)
    elif isinstance(value, list):
        for nested in value: finite(nested)

def state(row):
    vessels = row['bench']['vessels']
    assert len(vessels) == 1 and type(vessels[0]['id']) is int and vessels[0]['id'] == 0
    for portion in vessels[0]['contents']:
        assert type(portion['moles']) in (float, int) and portion['moles'] >= 0
    return vessels[0]

def amount(vessel, key):
    return sum(p['moles'] for p in vessel['contents'] if p['species'] == key)

def ledger(vessel):
    elements = {}; charge = 0.0
    for portion in vessel['contents']:
        formula, z = FORMULAS[portion['species']]
        charge += z * portion['moles']
        for element, count in formula.items():
            elements[element] = elements.get(element, 0) + count * portion['moles']
    return elements, charge

def balance(before, after, exports):
    first, q_first = ledger(before); last, q_last = ledger(after)
    for species, moles in exports:
        formula, z = FORMULAS[species]; q_last += z*moles
        for element, count in formula.items():
            last[element] = last.get(element, 0) + count*moles
    residual = {el: last.get(el, 0)-first.get(el, 0) for el in sorted(first.keys() | last.keys())}
    return {'element_residual_mol': residual, 'charge_residual_mol': q_last-q_first,
            'accepted': all(abs(n) <= 1e-8 for n in residual.values()) and abs(q_last-q_first) <= 1e-9}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, default=HERE/'assessment.json')
    args = parser.parse_args()
    design = read(HERE/'design.json'); execution = read(HERE/'execution.json')
    assert design['engine'] == execution['engine']
    assert digest(HERE/'api-binding.json') == design['api_binding_sha256']
    assert digest(HERE/'originals.json') == design['originals_sha256']
    rows = []; measurements = {}
    for case in design['cases']:
        path = HERE/(case['id']+'.response.json'); doc = read(path); finite(doc)
        receipt = next(r for r in execution['results'] if r['id'] == case['id'])
        assert digest(path) == receipt['response_sha256']
        assert doc['engine'] == design['engine'] and doc['script_sha256'] == case['script_sha256']
        assert digest(HERE/(case['id']+'.lab')) == case['script_sha256']
        observations = doc['observations']; sequence = [r['operator']['op'] for r in observations]
        events = [e for r in observations for e in r['events']]
        checks = {'bound_engine_script_raw_stream': True,
                  'http200_completed_zero_exit': receipt['http_status'] == 200 and doc['execution_status'] == 'completed' and doc['exit_status'] == 0,
                  'exact_operator_sequence': sequence == case['operator_sequence'],
                  'finite_nonnegative_inventory': all(all(p['moles'] >= 0 for p in state(r)['contents']) for r in observations),
                  'no_solver_failure_or_veto': not any(e['event'] in ('solver_failed','safety_veto') for e in events)}
        before = state(observations[-3]); after = state(observations[-2]); final = state(observations[-1])
        if case['rank'] == 17:
            org = [e for e in events if e['event'] == 'org_reacted' and e.get('name') == 'esterification']
            x = org[0]['extent'] if len(org) == 1 else float('nan')
            closure = balance(before, final, [])
            checks.update(one_explicit_org_reacted=len(org)==1,
                neat_initial_reactants=abs(amount(before,'CH3COOH')-.001)<=1e-12 and abs(amount(before,'ethanol')-.001)<=1e-12,
                ideal_k4_extent=math.isfinite(x) and abs(x-case['quantitative_expectations']['ideal_extent_mol'])<=1e-8,
                matched_acid_debit=abs(amount(before,'CH3COOH')-amount(final,'CH3COOH')-x)<=1e-8,
                matched_ethanol_debit=abs(amount(before,'ethanol')-amount(final,'ethanol')-x)<=1e-8,
                matched_ester_product=abs(amount(final,'ethyl_acetate')-amount(before,'ethyl_acetate')-x)<=1e-8,
                matched_water_product=abs(amount(final,'water')-amount(before,'water')-x)<=1e-8,
                no_applied_heat=final['temperature']==before['temperature'],
                explicit_model_limits=len(org)==1 and all(text in org[0]['boundary'] for text in ('Ideal-mixture equilibrium','not a reaction rate','reaction heat are not modelled')),
                material_charge_closure=closure['accepted'])
            model_limits = [e.get('reason',{}).get('key') for e in events if e['event']=='not_yet_modeled']
            observed = {'extent_mol':x,'acid_remaining_mol':amount(final,'CH3COOH'),'ethanol_remaining_mol':amount(final,'ethanol'),
                        'ethyl_acetate_mol':amount(final,'ethyl_acetate'),'water_mol':amount(final,'water'),
                        'temperature_before_kelvin':before['temperature'],'temperature_after_kelvin':final['temperature'],
                        'closure':closure,'org_reacted':org,'named_model_limits':model_limits}
            classification = 'explicit_ideal_equilibrium_exercise_demonstrated_with_scope_limits'
        else:
            wait = observations[-2]
            exports = [(e['species'],e['moles']) for e in wait['events'] if e['event']=='gas_evolved']
            produced = [e for e in wait['events'] if e['event']=='gas_produced' and e.get('reaction')=='peroxide-decomposition']
            reacted = [e for e in wait['events'] if e['event']=='reacted' and e.get('reaction')=='peroxide-decomposition']
            loss = amount(before,'H2O2')-amount(final,'H2O2')
            water = amount(final,'water')-amount(before,'water')
            retained = amount(final,'O2')-amount(before,'O2')
            exported = sum(n for species,n in exports if species=='O2')
            oxygen = retained+exported
            closure = balance(before, final, exports)
            checks.update(wait_10s=wait['operator']['seconds']==10,
                matched_water_gain=abs(water-loss)<=1e-8,
                target_oxygen_ratio=abs(2*oxygen-loss)<=1e-8,
                catalyst_inventory_unchanged=abs(amount(final,'MnO2')-case['quantitative_expectations']['mno2_dose_mol'])<=1e-10,
                material_charge_closure_including_retained_gas=closure['accepted'],
                declared_open_boundary=final['headspace']['boundary']=='open',
                frozen_open_export_expectation=abs(exported-oxygen)<=1e-8,
                no_reported_contained_gas_in_open_vessel=not any(e['event']=='gas_contained' for e in wait['events']))
            if case['id'].endswith('without-catalyst'):
                checks['no_silent_catalyst'] = amount(final,'MnO2')==0 and all(e.get('catalyst') is None for e in reacted)
            else:
                checks['reported_positive_catalyst_response'] = bool(reacted) and any(e.get('catalyst')=='manganese dioxide' for e in reacted)
            observed = {'peroxide_loss_mol':loss,'water_gain_mol':water,'target_oxygen_total_mol':oxygen,
                        'oxygen_retained_in_bulk_gas_mol':retained,'oxygen_export_event_mol':exported,
                        'mno2_retained_mol':amount(final,'MnO2'),'final_headspace':final['headspace'],
                        'gas_produced_events':produced,'reacted_events':reacted,'gas_evolved_events':exports,
                        'gas_contained_events':[e for e in wait['events'] if e['event']=='gas_contained'],
                        'closure':closure}
            measurements[case['id']] = observed
            classification = 'reaction_response_demonstrated_but_frozen_boundary_forecast_not_met'
        rows.append({'id':case['id'],'rank':case['rank'],'classification':classification,
                     'all_frozen_engineering_expectations_met':all(checks.values()),'checks':checks,'observed':observed,
                     'response_sha256':digest(path),'catalog_case_closed':False,'physical_verdict':'not_assigned'})
    cat = measurements['032-peroxide-with-mno2']; plain = measurements['032-peroxide-without-catalyst']
    pair = {'more_peroxide_loss_with_mno2':cat['peroxide_loss_mol']>plain['peroxide_loss_mol']+1e-12,
            'more_target_oxygen_with_mno2':cat['target_oxygen_total_mol']>plain['target_oxygen_total_mol']+1e-12,
            'virtual_oxygen_ratio_with_to_without':cat['target_oxygen_total_mol']/plain['target_oxygen_total_mol'],
            'empirical_rate_agreement':'not_assigned'}
    report = {'schema':1,'reviewed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'kind':'Independent saved-state/formula/event review of prospectively frozen protocol-only followups',
              'source':design['source'],'engine':design['engine'],'design_sha256':digest(HERE/'design.json'),
              'execution_sha256':digest(HERE/'execution.json'),'health_validation_sha256':digest(HERE/'health-validation.json'),
              'cases':rows,'matched_catalyst_pair':pair,'model_issue':'MODEL-005',
              'model_issue_scope':'Open-vessel retained bulk gas versus external-transfer/bubble-residence contract; closure with retained gas passes, so missing mass is not established.',
              'original_outcomes_changed':False,'catalog_cases_closed':0,'local_cli_executed':False,
              'production_changed':False,'physical_verdict':'not_assigned'}
    with args.report.open('x') as stream: json.dump(report,stream,indent=2,allow_nan=False,ensure_ascii=False);stream.write('\n')
    print(json.dumps({'cases':[(r['id'],r['all_frozen_engineering_expectations_met']) for r in rows], 'pair':pair},indent=2))

if __name__=='__main__': main()
