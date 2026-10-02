"""Independent budgets and refusal checks on captured post-repair CLI evidence.

This checks specific repaired contracts, not full scientific agreement of the
original fifty experiments. Unsupported original forecasts remain unsupported.
"""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('evidence', type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parent
checks = []

def check(name, passed, **facts):
    checks.append(dict(name=name, passed=bool(passed), **facts))

def rows(folder, case):
    return [json.loads(line, parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value))) for line in (folder / f'{case}.json.stdout').read_text().splitlines()]

def amount(vessel, species):
    return sum(p['moles'] for p in vessel['contents'] if p['species'] == species)

def merge(previous, patch):
    # `inspect vN` serializes only that selected vessel. Reconstruct the
    # latest observed inventory rather than treating a partial view as deletion.
    merged = dict(previous or {})
    vessels = {v['id']: v for v in merged.get('vessels', [])}
    vessels.update({v['id']: v for v in patch.get('vessels', [])})
    merged.update(patch)
    merged['vessels'] = [vessels[k] for k in sorted(vessels)]
    return merged

def inventory(vessel):
    # Independent formulas for this frozen electrolysis family. The analytical
    # base-equivalent coordinate carries H1/O1/-1, not an extra free-ion dose.
    formulas = {
        'water': ({'H':2,'O':1},0), 'H2': ({'H':2},0), 'O2': ({'O':2},0),
        'N2': ({'N':2},0), 'Cl2': ({'Cl':2},0), 'Na+': ({'Na':1},1),
        'Cl-': ({'Cl':1},-1), 'SO4-2': ({'S':1,'O':4},-2),
        'CO2': ({'C':1,'O':2},0), 'CO2(aq)': ({'C':1,'O':2},0),
        'HCO3-': ({'H':1,'C':1,'O':3},-1), 'CO3-2': ({'C':1,'O':3},-2),
        'H+': ({'H':1},1), 'OH-': ({'H':1,'O':1},-1),
        'base_equivalents': ({'H':1,'O':1},-1),
    }
    result = {'charge':0.0}
    for portion in vessel['contents']:
        atoms, charge = formulas[portion['species']]
        result['charge'] += charge * portion['moles']
        for element, count in atoms.items():
            result[element] = result.get(element,0.0) + count*portion['moles']
    return result

def final(data):
    state = None
    for row in data:
        if 'bench' in row:
            state = merge(state, row['bench'])
    return state

def events(data, event):
    return [e for r in data for e in r.get('events', []) if e['event'] == event]

receipts = []
for suite, folder in [(root, args.evidence), (root/'followups', args.evidence/'followups'),
                      (root/'additional-controls', args.evidence/'additional-controls')]:
    receipt = json.loads((folder/'execution.json').read_text())
    receipts.append(receipt)
    digest = hashlib.sha256((suite/'predictions.json').read_bytes()).hexdigest()
    freeze = json.loads((suite/'freeze.json').read_text())
    check(f'{suite.name}: frozen forecasts', digest == receipt['predictions_sha256'] == freeze['predictions_sha256'])
    check(f'{suite.name}: every case in both modes',
          sorted((r['id'], r['mode']) for r in receipt['runs']) ==
          sorted((r['id'], mode) for r in json.loads((suite/'predictions.json').read_text()) for mode in ['text','json']))
    for run in receipt['runs']:
        for stream in ['stdout', 'stderr']:
            check(f"{run['id']} {run['mode']}: {stream} digest",
                  hashlib.sha256((folder/run[stream]).read_bytes()).hexdigest() == run[f'{stream}_sha256'])
        if run['mode'] == 'json':
            parsed = rows(folder, run['id'])
            check(f"{run['id']}: strict NDJSON", all(isinstance(r, dict) for r in parsed))
for label, receipt in zip(['followups','additional-controls'], receipts[1:]):
    check(f'{label}: every invocation succeeds', all(r['exit_code']==0 for r in receipt['runs']))
check('one validated executable for all suites', len({(r['binary_sha256'], r['binary_source_commit']) for r in receipts}) == 1)

for case, expected_count in [('074',1), ('075',2), ('077',2)]:
    data = rows(args.evidence, case)
    runs=[r for r in receipts[0]['runs'] if r['id']==case]
    check(f'{case}: both modes execute successfully', len(runs)==2 and all(r['exit_code']==0 for r in runs))
    check(f'{case}: every predicted electrolysis occurs',
          len([r for r in data if r.get('operator',{}).get('op')=='electrolyse']) == expected_count
          and len(events(data,'electrolysed')) == expected_count)

    previous = None
    for r in data:
        if 'bench' not in r:
            continue
        if r.get('operator', {}).get('op') == 'electrolyse':
            op = r['operator']; vid = op['vessel']
            before = next(v for v in previous['vessels'] if v['id'] == vid)
            after = next(v for v in r['bench']['vessels'] if v['id'] == vid)
            e = next(e for e in r['events'] if e['event'] == 'electrolysed')
            electrons = op['amps'] * op['seconds'] / 96485.33212
            hydrogen = amount(after, 'H2') - amount(before, 'H2')
            check(f'{case} v{vid}: cathodic Faraday budget', abs(2*hydrogen-electrons) < 1e-12,
                  hydrogen_mol=hydrogen, expected_electrons_mol=electrons)
            check(f'{case} v{vid}: anodic Faraday budget',
                  abs(e['anode_moles']*(2 if e['anode_species']=='Cl2' else 4)-electrons)<1e-12)
            anode_gain=amount(after,e['anode_species'])-amount(before,e['anode_species'])
            check(f'{case} v{vid}: anodic product enters physical stock',
                  abs(anode_gain-e['anode_moles'])<1e-12,
                  gas_inventory_gain_mol=anode_gain, reported_mol=e['anode_moles'])
            water_loss = amount(before, 'water')-amount(after, 'water')
            expected_loss = electrons if e['anode_species']=='Cl2' else electrons/2
            before_budget, after_budget = inventory(before), inventory(after)
            for coordinate in sorted(before_budget.keys() | after_budget.keys()):
                change=after_budget.get(coordinate,0.0)-before_budget.get(coordinate,0.0)
                check(f'{case} v{vid}: conserved {coordinate} inventory', abs(change)<1e-8,
                      inventory_delta_mol=change)
            if e['anode_species']=='O2':
                check(f'{case} v{vid}: water consumption', abs(water_loss-expected_loss)<2e-8,
                      water_loss_mol=water_loss, expected_mol=expected_loss)
            # In brine, subsequent CO2/alkali neutralisation also changes
            # solvent water. Its complete atom/charge budgets above, rather
            # than the gross pre-settling water debit, must close.
            if e['anode_species'] == 'O2':
                acid_delta = amount(after, 'H+')-amount(before, 'H+')
                check(f'{case} v{vid}: no spurious electron-equivalent acid', abs(acid_delta)<electrons*.05,
                      proton_inventory_delta_mol=acid_delta)
            check(f'{case} v{vid}: electrode assumptions disclosed',
                  any(e['event']=='not_yet_modeled' and 'inert electrodes' in e.get('what','') for e in r['events']))
        previous = merge(previous, r['bench'])

original = rows(args.evidence, '097')
state = final(original)
check('097: no superheated observable ice',
      all(v['temperature']<=273.2 or not any(p['species']=='water' and p['phase']=='solid' and p['moles']>1e-9 for p in v['contents']) for v in state['vessels']))
check('097: unsupported heating refused explicitly',
      'heating was not committed' in (args.evidence/'097.json.stderr').read_text())

previous = None
for r in rows(args.evidence, '098'):
    if 'bench' not in r:
        continue
    if r.get('operator',{}).get('op')=='distil':
        check(f"098 stages {r['operator']['stages']}: complete refusal preserves physical state",
              merge(previous, r['bench'])==previous and not any(e['event']=='distilled' for e in r.get('events',[])),
              diagnostic=[e.get('what') for e in r.get('events',[]) if e['event']=='not_yet_modeled'])
    previous = merge(previous, r['bench'])

chrom = events(rows(args.evidence/'followups', 'F082'), 'chromatographed')
check('F082: missing neutrals visible alongside ethanol', bool(chrom) and
      any('ethyl_acetate' in e.get('unparameterised',[]) and e['peaks'] for e in chrom))
check('F082: text discloses incomplete coverage', 'Incomplete sample coverage' in (args.evidence/'followups/F082.text.stdout').read_text())

control = rows(args.evidence/'additional-controls','C097')
v = final(control)['vessels'][0]
check('C097: independent pure-water heat cycle closes', abs(v['temperature']-298.15)<=.2,
      final_temperature_k=v['temperature'])
initial = next(r['bench']['vessels'][0] for r in control if 'bench' in r and amount(r['bench']['vessels'][0], 'water')>0)
check('C097: water conserved', abs(amount(v,'water')-amount(initial,'water'))<1e-8)

vessels=final(rows(args.evidence/'additional-controls','C098'))['vessels']
ratios=[]
for donor, receiver in [(0,2),(1,3)]:
    d=next(v for v in vessels if v['id']==donor); v=next(v for v in vessels if v['id']==receiver)
    cut=amount(v,'water')+amount(v,'ethanol'); ratios.append(amount(v,'ethanol')/cut if cut else 0)
    check(f'C098 receiver {receiver}: complete 1% cut', abs(cut-.01)<1e-8, cut_mol=cut)
    for species in ['water','ethanol']:
        check(f'C098 pair {donor}/{receiver}: {species} conserved', abs(amount(d,species)+amount(v,species)-.5)<1e-8)
check('C098: second stage enriches ethanol', ratios[1]>ratios[0], ethanol_fractions=ratios)

canonical_runs=[r for r in receipts[2]['runs'] if r['id']=='C084']
check('C084: canonical propanone fixture executes in both modes',
      len(canonical_runs)==2 and all(r['exit_code']==0 for r in canonical_runs))
silver_control=final(rows(args.evidence/'additional-controls','C084'))
silver_observation=[dict(vessel=v['id'], silver_metal_mol=amount(v,'Ag'),
                         silver_inventory=[p for p in v['contents'] if 'Ag' in p['species']])
                    for v in silver_control['vessels']]

out=dict(binary_source_commit=receipts[0]['binary_source_commit'], binary_sha256=receipts[0]['binary_sha256'],
         invocation_count=sum(len(r['runs']) for r in receipts),
         canonical_tollens_control=silver_observation,
         checks=checks, passed=all(c['passed'] for c in checks),
         limitations=['Original sealed freeze-thaw forecast remains unsupported.',
                      'These contract checks do not certify every reaction or optical/retention parameter.'])
(args.evidence/'repair-verification.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(dict(passed=out['passed'], checks=len(checks), failures=[c for c in checks if not c['passed']]),indent=2))
raise SystemExit(0 if out['passed'] else 1)
