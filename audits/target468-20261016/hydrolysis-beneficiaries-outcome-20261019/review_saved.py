"""Independent original-criteria review of two saved responses; no engine or HTTP."""
import argparse, hashlib, json, subprocess
from decimal import Decimal, localcontext
from pathlib import Path

D=Decimal
ZERO=D(0)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--directory',type=Path,default=Path(__file__).resolve().parent)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();assert not args.out.exists()
    folder=args.directory;sha=lambda raw:hashlib.sha256(raw).hexdigest()
    design=json.loads((folder/'design.json').read_bytes(),parse_float=D)
    source=design['accepted_engine']['source_head']
    registry_raw=subprocess.check_output(['git','show',source+':data/registry/registry-source-v1.json'])
    registry=json.loads(registry_raw,parse_float=D)
    compositions={r['species_id']:r for r in registry['compositions']}
    def composition(key):
        if key=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
        row=compositions[key]
        return {**{e['element']:e['count']['value'] for e in row['elements']},'charge':row['net_charge']['value']}
    water={r['property']:r['quantity'] for r in registry['phase_thermodynamics'] if r['species_id']=='water'}
    assert water['mass_density']['unit']['symbol']=='g/mL' and water['molar_mass']['unit']['symbol']=='g/mol'
    def amount(v,keys):return sum((p['moles'] for p in v['contents'] if p['species'] in keys),ZERO)
    def ledger(v):
        out={}
        for p in v['contents']:
            assert p['moles'].is_finite() and p['moles']>=0
            for key,n in composition(p['species']).items():out[key]=out.get(key,ZERO)+n*p['moles']
        return out
    def add_to(out, key, n):
        for element,coefficient in composition(key).items():out[element]=out.get(element,ZERO)+coefficient*n
    results=[]
    with localcontext() as context:
        context.prec=45
        carrier_mol_per_L=water['mass_density']['value']*D(1000)/water['molar_mass']['value']
        for case in design['cases']:
            stem=case['id'].replace(':','-');raw=(folder/'responses'/(stem+'.json')).read_bytes()
            response=json.loads(raw,parse_float=D)
            assert sha(case['script'].encode())==response['script_sha256']==case['script_sha256']
            assert response['engine']==design['accepted_engine'] and response['exit_status']==0
            state={};maximum=ZERO;per_step=[];frames=[]
            for row in response['observations']:
                before=state.copy();state={v['id']:v for v in row['bench']['vessels']};op=row['operator']
                assert not any(e['event'] in {'solver_failed','gas_evolved','gas_absorbed'} for e in row['events'])
                expected={key:ledger(value) for key,value in before.items()}
                for key in state:expected.setdefault(key,{})
                if op['op']=='add':add_to(expected[op['vessel']],op['species'],op['moles'])
                elif op['op']=='titrate':
                    events=[e for e in row['events'] if e['event']=='titrated'];assert len(events)==1
                    event=events[0];assert event['endpoint_reached'] and event['titrant']==op['titrant']=='HCl'
                    assert event['concentration']==op['concentration'] and event['steps']<=op['max_steps']
                    dose=event['total_volume']*op['concentration']
                    add_to(expected[op['vessel']],'HCl',dose)
                    add_to(expected[op['vessel']],'water',event['total_volume']*carrier_mol_per_L)
                else:assert op['op'] in {'inspect','react','measure','new_vessel'}
                residuals={}
                for key,v in state.items():
                    actual=ledger(v);residual={k:actual.get(k,ZERO)-expected[key].get(k,ZERO) for k in actual.keys()|expected[key].keys()}
                    worst=max(map(abs,residual.values()),default=ZERO);maximum=max(maximum,worst)
                    assert worst<=D('1e-8')
                    residuals[str(key)]=residual
                per_step.append({'operator':op,'residuals':residuals});frames.append((row,before,state.copy()))
            checks={'every_step_atoms_charge':True,'no_solver_failure':True};metrics={}
            if case['id']=='batch2:mo-006':
                titrations=[f for f in frames if f[0]['operator']['op']=='titrate'];assert len(titrations)==2
                delivered=[];ends=[]
                for row,before,after in titrations:
                    vid=row['operator']['vessel'];event=next(e for e in row['events'] if e['event']=='titrated')
                    dose=event['total_volume']*event['concentration'];chloride=amount(after[vid],{'Cl-'})-amount(before[vid],{'Cl-'})
                    assert abs(dose-chloride)<=D('1e-12')
                    assert abs(event['final_ph']-row['operator']['target_ph'])<=D('0.0001')
                    delivered.append(dose);ends.append(after[vid])
                reactions=[f for f in frames if f[0]['operator']['op']=='react'];assert len(reactions)==1
                _,before,after=reactions[0]
                ester_loss=amount(before[0],{'ethyl_acetate'})-amount(after[0],{'ethyl_acetate'})
                assert abs(ester_loss-D('0.0005'))<=D('1e-8') and amount(after[0],{'ethyl_acetate'})<=D('1e-8')
                assert abs(amount(after[0],{'ethanol'})-ester_loss)<=D('1e-8')
                difference=delivered[1]-delivered[0]
                protonated=amount(ends[0],{'CH3COOH'})
                basis_difference=amount(ends[1],{'base_equivalents'})-amount(ends[0],{'base_equivalents'})
                corrected=difference+protonated+basis_difference
                increment=titrations[0][0]['operator']['concentration']*titrations[0][0]['operator']['step']
                checks.update({'MO-006-C1':abs(difference-ester_loss)<=increment,'MO-006-C2':abs(corrected-ester_loss)<=D('1e-8') and protonated<increment})
                metrics={'actual_sample_acid_mol':delivered[0],'actual_blank_acid_mol':delivered[1],'acid_difference_mol':difference,'ester_loss_mol':ester_loss,'declared_increment_mol':increment,'sample_protonated_acetate_mol':protonated,'blank_minus_sample_residual_base_coordinate_mol':basis_difference,'corrected_acid_difference_mol':corrected,'uncorrected_bias_fraction':(ester_loss-difference)/ester_loss,'endpoint_temperatures_K':[v['temperature'] for v in ends],'titrant_water_moles_per_litre':carrier_mol_per_L}
                status='PASS' if all(checks.values()) else 'FAIL'
            elif case['id']=='batch2:mo-011':
                measures=[f for f in frames if f[0]['operator']['op']=='measure'];assert len(measures)==3
                stages=[]
                for row,_,after in measures:
                    event=next(e for e in row['events'] if e['event']=='chromatographed');v=after[0]
                    peaks=[e['species'] for e in event['peaks']]
                    assert all(amount(v,{key})>ZERO for key in peaks)
                    stages.append({'ester_mol':amount(v,{'ethyl_acetate'}),'ethanol_mol':amount(v,{'ethanol'}),'acetate_family_mol':amount(v,{'CH3COO-','CH3COOH'}),'modeled_peak_species':peaks,'outside_method':event['outside_method'],'raw_missing_parameter_events':[e for e in row['events'] if e['event']=='not_yet_modeled']})
                for index in [1,2]:
                    assert abs(stages[index-1]['ester_mol']-stages[index]['ester_mol']-D('0.0005'))<=D('1e-8')
                    assert abs(stages[index]['ethanol_mol']-stages[index-1]['ethanol_mol']-D('0.0005'))<=D('1e-8')
                    assert abs(stages[index]['acetate_family_mol']-stages[index-1]['acetate_family_mol']-D('0.0005'))<=D('1e-8')
                assert 'ethyl_acetate' not in stages[0]['modeled_peak_species']
                assert any(e.get('reason',{}).get('key')=='not-modeled.no-group-decomposition' and 'ethyl_acetate' in e.get('what','') for e in stages[0]['raw_missing_parameter_events'])
                assert all('ethanol' in stage['modeled_peak_species'] and 'CH3COO-' in stage['outside_method'] for stage in stages[1:])
                checks.update({'owned_two_stage_conversions':True,'MO-011-C1':False,'MO-011-C2':True})
                metrics={'stages':stages,'reason':'Ester retention/group parameters are absent: no initial ester peak exists to track. Correct molecular conversion and ethanol peaks do not satisfy the original ester-feature criterion. Raw relative peak areas are not mole ratios.'}
                status='UNSUPPORTED'
            else:raise AssertionError(case['id'])
            results.append({'id':case['id'],'status':status,'script_sha256':case['script_sha256'],'response_sha256':sha(raw),'original_criteria':case['original_criteria'],'criteria':checks,'metrics':metrics,'maximum_per_step_atom_charge_residual':maximum,'per_step':per_step})
    out={'source':source,'engine':design['accepted_engine'],'registry_sha256':sha(registry_raw),'cases':results,'new_engine_or_API_requests':0,'catalog_additions_by_reader':0,'qualification':'Bounded declared analytical/template and ideal detector domains, with actual temperatures retained; not rate/heat/retention or empirical apparatus calibration.'}
    args.out.write_text(json.dumps(out,indent=2,default=str)+'\n')
    print(json.dumps({c['id']:c['status'] for c in results}))

if __name__=='__main__':main()
