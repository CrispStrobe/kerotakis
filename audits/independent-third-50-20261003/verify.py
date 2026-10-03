"""Verify evidence integrity and quantitative original forecasts.

Unsupported inputs and qualified protocols are recorded separately in assessment.json;
passing integrity checks does not turn them into scientific agreement.
"""
import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path

SUITE = Path(__file__).resolve().parent


def strict(text):
    return json.loads(text, parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def vessel(row, index=0):
    return next(v for v in row['bench']['vessels'] if v['id'] == index)


def stock(v, name, phase=None):
    return sum(c['moles'] for c in v['contents'] if c['species'] == name and (phase is None or c['phase'] == phase))


def near(a, b, relative=1e-6, absolute=1e-12):
    return abs(a-b) <= max(absolute, relative*max(abs(a), abs(b)))


def inspections(rows):
    return [r for r in rows if r['operator']['op'] == 'inspect']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence', type=Path)
    parser.add_argument('--validation', type=Path, required=True)
    parser.add_argument('--baseline', action='store_true')
    args = parser.parse_args()
    checks = []

    def check(name, passed, **details):
        checks.append(dict(name=name, passed=None if passed is None else bool(passed), **details))

    receipt = strict((args.evidence/'execution.json').read_text())
    validation = strict(args.validation.read_text())
    forecast = strict((SUITE/'predictions.json').read_text())
    freeze = strict((SUITE/'freeze.json').read_text())
    check('immutable forecast hash', digest(SUITE/'predictions.json') == freeze['predictions_sha256'] == receipt['predictions_sha256'])
    required_stages = {'thermo','codex','cea','core','phreeqc','cli','wasm','replay'}
    stages = [s['name'] for s in validation['stages']]
    check('complete successful validation stages',required_stages.issubset(stages) and len(stages)==len(set(stages)))
    if not args.baseline:
        commands = {s['name']:s['command'] for s in validation['stages']}
        check('new core regressions included',all(target in commands.get('core',[]) for target in ['headspace','density_coverage_scaling']))
        check('new native regressions included',all(target in commands.get('phreeqc',[]) for target in ['--lib','settled_carbonate_transfer','trace_inventory','order_invariance','native_delta_h','engine_call_budget']))
    check('validated executable', validation['passed'] is True and all(s['exit_code'] == 0 for s in validation['stages']) and validation['commit'] == receipt['binary_source_commit'] and validation['binary_sha256'] == receipt['binary_sha256'])
    for name, expected in validation['source_hashes'].items():
        source = subprocess.check_output(['git','show',validation['commit']+':'+name],cwd=SUITE.parents[1])
        check('validation source digest: '+name,hashlib.sha256(source).hexdigest()==expected)
    for stage in validation['stages']:
        check('validation log digest: '+stage['name'], digest(args.validation.parent/stage['log']) == stage['log_sha256'])
    check('exact fifty cases in both modes', sorted((r['id'],r['mode']) for r in receipt['runs']) == sorted((c['id'],m) for c in forecast for m in ['text','json']))
    cases = {}
    qualifications = []
    for run in receipt['runs']:
        key = run['id']+' '+run['mode']
        for stream in ['stdout','stderr']:
            check(key+' '+stream+' digest', digest(args.evidence/run[stream]) == run[stream+'_sha256'])
        # MgCl2 was absent at the freeze; its explicit refusal remains a capability gap.
        expected_code = 1 if run['id'] == 'E147' else 0
        check(key+' process outcome', run['exit_code'] == expected_code, exit_code=run['exit_code'])
        if run['id'] == 'E147':
            check(key+' explicit ingredient refusal', "unknown species or material 'MgCl2'" in (args.evidence/run['stderr']).read_text())
        if run['mode'] == 'json':
            rows = [strict(line) for line in (args.evidence/run['stdout']).read_text().splitlines()]
            cases[run['id']] = rows
            prediction = next(c for c in forecast if c['id'] == run['id'])
            command_count = len([line for line in prediction['script'].splitlines() if line.strip() and not line.startswith(('register','#'))])
            expected_rows = 1 if run['id'] == 'E147' else command_count
            check(key+' complete operation output',len(rows)==expected_rows and [r['output_sequence'] for r in rows]==list(range(expected_rows)),expected_rows=expected_rows,actual_rows=len(rows))
            check(key+' finite thermal/boundary state',all(math.isfinite(v['temperature']) and math.isfinite(v['pressure']) for row in rows for v in row['bench']['vessels']))
            check(key+' finite available measurements',all(isinstance(e['value'],(int,float)) and math.isfinite(e['value']) for row in rows for e in row['events'] if e['event']=='measured'))
            failures = [e for row in rows for e in row.get('events',[]) if e['event'] == 'solver_failed']
            check(key+' no solver failure', not failures, failures=failures)
            check(key+' nonnegative finite inventory', all(math.isfinite(c['moles']) and c['moles'] >= 0 for row in rows for v in row['bench']['vessels'] for c in v['contents']))

    registry = strict((SUITE.parents[1]/'data/registry/registry-source-v1.json').read_text())
    formulas = {c['species_id']:{e['element']:e['count']['value'] for e in c['elements']} for c in registry['compositions']}

    def atom(row, element, index=None):
        return sum(c['moles']*formulas.get(c['species'],{}).get(element,0) for v in row['bench']['vessels'] if index is None or v['id'] == index for c in v['contents'])

    def budget(case, expected):
        for el,n in expected.items():
            actual = atom(cases[case][-1],el)
            check(case+' '+el+' atom budget',near(actual,n,relative=5e-8),actual=actual,expected=n)

    r=cases['E131'][-1]; a,b=vessel(r),vessel(r,1)
    check('E131 neutral in both orders', all(abs(v['solution']['ph']-7)<.5 for v in [a,b]) and abs(a['solution']['ph']-b['solution']['ph'])<.2)
    budget('E131',dict(Na=.002,Cl=.002))
    for key,sign,el in [('E132',1,'Cl'),('E133',-1,'Na')]:
        rs=inspections(cases[key]); delta=vessel(rs[-1])['solution']['ph']-vessel(rs[0])['solution']['ph']
        check(key+' tenfold dilution pH shift', abs(delta-sign)<.2,delta=delta)
        check(key+' retained solute',near(atom(rs[0],el),atom(rs[-1],el)))
        check(key+' tenfold water amount',near(stock(vessel(rs[-1]),'water'),10*stock(vessel(rs[0]),'water')))
    r=cases['E134'][-1]; a,b=vessel(r),vessel(r,1)
    check('E134 dilution by add or transfer',abs(a['solution']['ph']-b['solution']['ph'])<.1 and near(stock(a,'water'),stock(b,'water')))
    budget('E134',dict(Cl=.002))
    r=cases['E135'][-1]
    check('E135 both orders precipitate silver chloride',all(stock(vessel(r,i),'AgCl','solid')>=.0009 for i in [0,1]))
    check('E135 order equivalent silver chloride',near(stock(vessel(r),'AgCl'),stock(vessel(r,1),'AgCl')))
    budget('E135',dict(Ag=.002,Cl=.002,Na=.002,N=.002))
    r=cases['E136'][-1]; precip=stock(vessel(r),'AgCl','solid')
    check('E136 chloride limiting solid yield',.0009<=precip<=.00101,actual=precip)
    check('E136 excess silver retained',near(atom(r,'Ag')-precip,.001,relative=1e-3))
    budget('E136',dict(Ag=.002,Cl=.001,Na=.001,N=.002))
    r=cases['E137'][-1]; a,b=vessel(r),vessel(r,1)
    check('E137 common ion suppresses silver',stock(a,'Ag+','aqueous')>=10*stock(b,'Ag+','aqueous') and all(stock(v,'AgCl','solid')>0 for v in [a,b]),pure=stock(a,'Ag+'),salted=stock(b,'Ag+'))
    budget('E137',dict(Ag=.002,Cl=.003,Na=.001))
    rs=inspections(cases['E138'])
    for i,r in enumerate(rs):
        for el,n in dict(Ag=.001,Cl=.001).items():check(f'E138 filtration {i} {el} conserved',near(atom(r,el),n))
    check('E138 no duplicate precipitate',stock(vessel(rs[-1],2),'AgCl','solid')<1e-6 and near(stock(vessel(rs[0]),'AgCl','solid'),stock(vessel(rs[-1]),'AgCl','solid')))
    r=cases['E139'][-1]; check('E139 barium sulfate majority',stock(vessel(r),'BaSO4','solid')>=.00095)
    budget('E139',dict(Ba=.001,S=.001,Na=.002,Cl=.002))
    # Compare sealed inventories before destructive gas tests; test consumption is external transfer.
    rs=inspections(cases['E140']); check('E140 sealed carbon conserved before limewater test',near(atom(rs[0],'C'),atom(rs[-1],'C'),relative=5e-8))
    check('E140 carbon dioxide increases',stock(vessel(rs[-1]),'CO2')+stock(vessel(rs[-1]),'CO2(aq)')>stock(vessel(rs[0]),'CO2')+stock(vessel(rs[0]),'CO2(aq)'))
    check('E140 chloride dose retained',near(atom(rs[-1],'Cl'),.002))
    consumed=sum(e['moles'] for row in cases['E140'] for e in row.get('events',[]) if e['event']=='gas_consumed' and e.get('species')=='CO2')
    # Event shape is checked separately below rather than assuming a missing consumption ledger.
    final_loss=atom(rs[-1],'C')-atom(cases['E140'][-1],'C')
    check('E140 gas test removes positive carbon',final_loss>0,carbon_removed=final_loss,event_consumed=consumed)
    r=cases['E141'][-1]; co2=lambda v:stock(v,'CO2')+stock(v,'CO2(aq)')
    check('E141 bicarbonate makes more molecular CO2',co2(vessel(r))>co2(vessel(r,1)))
    vent=sum(e['moles'] for row in cases['E141'] for e in row.get('events',[]) if e['event']=='gas_evolved' and e.get('species')=='CO2')
    seals=[row for row in cases['E141'] if row['operator']['op']=='seal']
    trapped=sum(e['trapped_air']*.0004 for row in seals for e in row['events'] if e['event']=='vessel_sealed')
    check('E141 carbon includes preseal environmental loss',near(atom(r,'C')+vent,.002+trapped,relative=5e-8),vented=vent,trapped_carbon=trapped)
    budget('E141',dict(Na=.003,Cl=.002))
    rs=inspections(cases['E142'])
    for i,r in enumerate(rs):
        check(f'E142 carbon retained dose {i}',near(atom(r,'C'),atom(rs[0],'C'),relative=5e-8))
        check(f'E142 chloride incremental dose {i}',near(atom(r,'Cl'),i*.0005,relative=5e-8))
    check('E142 final CO2 exceeds initial',co2(vessel(rs[-1]))>co2(vessel(rs[0])))
    for key in ['E143','E144']:
        rs=inspections(cases[key]);
        for el in ['Ca','C']:check(key+' '+el+' conserved',near(atom(rs[0],el),atom(rs[-1],el),relative=5e-8))
    check('E143 enough acid dissolves calcite',stock(vessel(cases['E143'][-1]),'CaCO3','solid')<.0001)
    check('E144 limiting acid leaves calcite',stock(vessel(cases['E144'][-1]),'CaCO3','solid')>.0005)
    budget('E143',dict(Ca=.001,Cl=.002));budget('E144',dict(Ca=.001,Cl=.0002))
    check('E145 copper hydroxide majority',stock(vessel(cases['E145'][-1]),'Cu(OH)2','solid')>.0005)
    budget('E145',dict(Cu=.001,S=.001,Na=.002))
    check('E146 iron hydroxide majority',stock(vessel(cases['E146'][-1]),'Fe(OH)3','solid')>.0005)
    budget('E146',dict(Fe=.001,Na=.003,Cl=.003))
    r=cases['E148'][-1]
    check('E148 magnetic iron transferred, silica retained',stock(vessel(r,1),'Fe','solid')>.005 and atom(r,'Si',1)<1e-12 and near(stock(vessel(r),'water'),stock(vessel(inspections(cases['E148'])[0]),'water')))
    budget('E148',dict(Fe=.01,Si=.01))
    r=cases['E149'][-1]
    check('E149 aqueous iron not metallic magnetic solid',stock(vessel(r,1),'Fe','solid')==0 and near(atom(r,'Fe',0),.001) and not vessel(r,1)['contents'])
    budget('E149',dict(Fe=.001,Cl=.003))
    rs=inspections(cases['E150']);check('E150 all measurements leave full state unchanged',rs[0]['bench']==rs[-1]['bench'])
    reseals = [r for r in cases['E113'] if r['operator']['op']=='seal']
    event = next(e for e in reseals[-1]['events'] if e['event']=='vessel_sealed')
    check('E113 resealing records prior sealed boundary',event.get('previous_boundary',{}).get('boundary')=='sealed')
    text=(args.evidence/'E113.text.stdout').read_text()
    check('E113 resealing narration matches prior sealed boundary','boundary=sealed → sealed' in text)
    for module in ['check_a','check_b']:
        path=SUITE/(module+'.py')
        import importlib.util
        spec=importlib.util.spec_from_file_location(module,path); loaded=importlib.util.module_from_spec(spec);spec.loader.exec_module(loaded);qualifications.extend(loaded.run(cases,check) or [])
    follow = args.evidence/'followups'
    if follow.exists():
        fc = strict((SUITE/'followups/predictions.json').read_text())
        ff = strict((SUITE/'followups/freeze.json').read_text())
        fr = strict((follow/'execution.json').read_text())
        check('followups frozen identity',digest(SUITE/'followups/predictions.json')==ff['predictions_sha256']==fr['predictions_sha256'])
        check('followups exact case/mode coverage',sorted((r['id'],r['mode']) for r in fr['runs'])==sorted((c['id'],m) for c in fc for m in ['text','json']))
        check('followups same validated binary',fr['binary_sha256']==receipt['binary_sha256'] and fr['binary_source_commit']==receipt['binary_source_commit'])
        fcases={}
        for run in fr['runs']:
            check(run['id']+' '+run['mode']+' complete positive control',run['exit_code']==0)
            for stream in ['stdout','stderr']:
                check(run['id']+' '+run['mode']+' '+stream+' digest',digest(follow/run[stream])==run[stream+'_sha256'])
            if run['mode']=='json':
                fcases[run['id']]=[strict(line) for line in (follow/run['stdout']).read_text().splitlines()]
                check(run['id']+' no refusal/solver failure',not any(e['event'] in ['not_yet_modeled','solver_failed'] for row in fcases[run['id']] for e in row['events']))
        rows=inspections(fcases['F01']);before,after=rows[0],rows[-1]
        ratios=[]
        for i in [0,2]:
            source,receiver=vessel(after,i),vessel(after,i+1)
            for species in ['water','ethanol']:
                check(f'F01 pair {i} {species} conserved',near(stock(source,species)+stock(receiver,species),stock(vessel(before,i),species)))
                check(f'F01 pair {i} positive {species} residue',stock(source,species)>0)
            overhead=stock(receiver,'water')+stock(receiver,'ethanol')
            original=stock(vessel(before,i),'water')+stock(vessel(before,i),'ethanol')
            check(f'F01 pair {i} complete one-percent cut',near(overhead,.01*original))
            ratios.append(stock(receiver,'ethanol')/overhead)
        initial_ratio=stock(vessel(before),'ethanol')/(stock(vessel(before),'water')+stock(vessel(before),'ethanol'))
        check('F01 staged enrichment positive control',ratios[1]>ratios[0]>initial_ratio,receiver_ethanol_fractions=ratios)
        rows=inspections(fcases['F02']);before,after=rows[0],rows[-1]
        w0=stock(vessel(before),'water');w=stock(vessel(after),'water');wo=stock(vessel(after,1),'water')
        check('F02 pure water complete thirty-percent cut',near(wo,.3*w0) and near(w+wo,w0) and w>0)
        check('F02 pure-component cut is not azeotrope',not any(e.get('azeotrope_limited',False) for row in fcases['F02'] for e in row['events']))
    elif not args.baseline:
        check('required positive distillation controls captured',False)
    result=dict(checks=len(checks),passed=all(c['passed'] is not False for c in checks),failures=[c for c in checks if c['passed'] is False],qualified_checks=[c for c in checks if c['passed'] is None],qualifications=qualifications,details=checks)
    (args.evidence/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='details'},indent=2))
    if not result['passed'] and not args.baseline: raise SystemExit(1)


if __name__=='__main__':main()
