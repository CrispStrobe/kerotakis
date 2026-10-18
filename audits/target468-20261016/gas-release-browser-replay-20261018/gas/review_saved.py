"""Assess the four frozen deployed-source diagnostics using saved bytes only."""
import argparse,hashlib,json,math
from pathlib import Path

def sha(b): return hashlib.sha256(b).hexdigest()
def amount(v,key,phase=None):
    if isinstance(key,str): key={key}
    return sum(p['moles'] for p in v['contents'] if p['species'] in key and (phase is None or p['phase']==phase))
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--directory',type=Path,default=Path(__file__).parent);ap.add_argument('--out',type=Path,required=True);a=ap.parse_args();assert not a.out.exists()
    design=json.loads((a.directory/'design.json').read_text());props=json.loads((a.directory/'properties.json').read_text());species=props['species']
    def ledger(vessels):
        result={}
        for v in vessels:
            for p in v['contents']:
                assert math.isfinite(p['moles']) and p['moles']>=0
                data=species[p['species']]
                for e,n in {**data['elements'],'charge':data['charge']}.items():result[e]=result.get(e,0.0)+n*p['moles']
        return result
    def residual(after,before,delta=None):
        delta=delta or {};return {k:after.get(k,0)-before.get(k,0)-delta.get(k,0) for k in set(before)|set(after)|set(delta)}
    results=[]
    for c in design['cases']:
        raw=(a.directory/'responses'/(c['id'].replace(':','-')+'.json')).read_bytes();response=json.loads(raw);assert response['engine']==design['accepted_engine'] and response['script_sha256']==c['script_sha256'] and sha(c['script'].encode())==c['script_sha256'];assert response['exit_status']==0
        rows=response['observations'];state={};frames=[]
        for r in rows:
            before=state.copy();state.update({v['id']:v for v in r.get('bench',{}).get('vessels',[])});frames.append((r,before,state.copy()))
        inspections=[r['bench']['vessels'] for r in rows if r['operator']['op']=='inspect'];limits=c['changed_source_expectations'];events=[e for r in rows for e in r['events']];metrics={};checks={'no_solver_failure':not any(e['event']=='solver_failed' for e in events)}
        if c['id'].endswith('gs-010'):
            additions=[]
            for r,before,after in frames:
                op=r['operator']
                if op['op']=='add' and op['species']=='CO2':
                    vid=op['vessel'];old=before[vid];v=after[vid];delta={'C':op['moles'],'O':2*op['moles']};additions.append({'vessel':vid,'temperature_K':v['temperature'],'neutral_CO2_aq_mol':amount(v,'CO2(aq)','aqueous'),'bicarbonate_mol':amount(v,'HCO3-'),'carbonate_mol':amount(v,'CO3-2'),'gas_CO2_mol':amount(v,'CO2','gas'),'carbon_before_including_trapped_air_mol':ledger([old]).get('C',0),'closed_addition_residual':residual(ledger([v]),ledger([old]),delta)})
            assert len(additions)==2
            checks['closed_represented_material_and_charge']=all(abs(n)<=limits['represented_material_charge_resolution_mol_or_eq'] for v in additions for n in v['closed_addition_residual'].values());checks['neutral_CO2_salt_less_than_water']=additions[1]['neutral_CO2_aq_mol']<additions[0]['neutral_CO2_aq_mol'];metrics={'arms':additions,'temperature_difference_K':additions[1]['temperature_K']-additions[0]['temperature_K'],'qualification':'Qualitative salinity trend is not a scientific pass when closed-component conservation fails; temperatures differ and this is no empirical salinity calibration.'}
        elif c['id'].endswith('kn-007'):
            filt=next((r,b,f) for r,b,f in frames if r['operator']['op']=='filter');r,b,f=filt;donor=r['operator']['from'];tol=limits['represented_material_charge_resolution_mol_or_eq'];filter_delta=residual(ledger(f.values()),ledger(b.values()));oxygen=[amount(b[donor],'O2','gas'),amount(f[donor],'O2','gas')]
            expected={};catalyst_started=False;catalyst=[]
            for r,b,f in frames:
                op=r['operator']
                if op['op']=='add':
                    comp=species[op['species']]
                    for e,n in {**comp['elements'],'charge':comp['charge']}.items():expected[e]=expected.get(e,0)+n*op['moles']
                    catalyst_started=catalyst_started or op['species']=='MnO2'
                if catalyst_started:catalyst.append(sum(amount(v,'MnO2') for v in f.values()))
            waits=[(r,b,f) for r,b,f in frames if r['operator']['op']=='wait'];assert len(waits)==2;second_r,second_before,second_after=waits[1];consumed=amount(second_before[0],'H2O2')-amount(second_after[0],'H2O2');made=amount(second_after[0],'O2','gas')-amount(second_before[0],'O2','gas');whole=residual(ledger(state.values()),{},expected)
            checks.update(catalyst_retained=all(abs(n-limits['MnO2_retained_mol'])<=tol for n in catalyst),filter_whole_material_and_charge=all(abs(n)<=tol for n in filter_delta.values()),filter_donor_oxygen_retained=abs(oxygen[1]-oxygen[0])<=limits['donor_O2_before_after_filter_resolution_mol'],whole_script_material_and_charge=all(abs(n)<=tol for n in whole.values()),fresh_charge_decomposes=consumed>0,second_charge_oxygen_stoichiometry=abs(made-consumed/2)<=tol)
            metrics={'donor_O2_before_after_filter_mol':oxygen,'filter_whole_ledger_residual':filter_delta,'whole_script_ledger_residual':whole,'MnO2_all_retained_timepoints_mol':catalyst,'second_wait_seconds':second_r['operator']['seconds'],'second_H2O2_consumed_mol':consumed,'second_O2_formed_mol':made,'qualification':'Lumped teaching kinetics, not measured rate/temperature calibration. No rate equality is inferred between the two substrate charges.'}
        elif c['id'].endswith('or-006'):
            assert len(inspections)==2;v=inspections[-1][0];tol=limits['stoichiometric_resolution_mol'];metrics={'ester_mol':amount(v,'ethyl_acetate'),'ethanol_mol':amount(v,'ethanol'),'acetate_family_mol':amount(v,{'CH3COO-','CH3COOH','NaOAc'}),'sodium_mol':amount(v,{'Na+','NaOH','NaOAc'}),'before_after_ledger_residual':residual(ledger(inspections[-1]),ledger(inspections[0]))}
            for name,key in [('ester','post_request_ester_mol'),('ethanol','post_request_ethanol_mol'),('acetate_family','post_request_acetate_family_mol'),('sodium','post_request_sodium_mol')]:checks[name+'_matches_stoichiometry']=abs(metrics[name+'_mol']-limits[key])<=tol
        elif c['id'].endswith('th-016'):
            add=next(r for r in rows if r['operator']['op']=='add');heated=next(r for r in rows if r['operator']['op']=='heat');v=inspections[0][0];n=add['operator']['moles'];t0=add['bench']['vessels'][0]['temperature'];tm=props['Zn_melting_k'];interval=props['Zn_solid_cp']['intervals'][0];assert interval['t_min_k']<=t0<=tm<=interval['t_max_k'];co=interval['coefficients'];assert props['Zn_solid_cp']['form']=='nasa9'
            def primitive(t):
                q0,q1,q2,q3,q4,q5,q6=co
                return -q0/t+q1*math.log(t)+q2*t+q3*t*t/2+q4*t**3/3+q5*t**4/4+q6*t**5/5
            sensible=n*props['gas_constant_j_mol_k']*(primitive(tm)-primitive(t0));liquid=amount(v,'Zn','liquid');solid=amount(v,'Zn','solid');latent=props['Zn_fusion_j_mol'];expected=(limits['input_energy_J']-sensible)/latent;event_melt=sum(e.get('moles',0) for e in heated['events'] if e['event']=='state_changed' and e.get('species')=='Zn' and e.get('from')=='solid' and e.get('to')=='liquid');oxide=sum(amount(x,'ZnO') for x in state.values());tol=limits['zinc_inventory_resolution_mol'];checks.update(zinc_conserved=abs(solid+liquid-limits['represented_zinc_mol'])<=tol,retained_coexistence=solid>0 and liquid>0,event_matches_owned_liquid=abs(event_melt-liquid)<=limits['melt_event_owned_phase_difference_resolution_mol'],paid_latent_budget=abs(liquid-expected)<=tol,no_unaccounted_new_oxide=oxide==0)
            metrics={'temperature_K':v['temperature'],'solid_Zn_mol':solid,'liquid_Zn_mol':liquid,'melt_event_mol':event_melt,'source_sensible_to_transition_J':sensible,'source_latent_J_mol':latent,'expected_paid_liquid_mol':expected,'owned_phase_energy_excess_J':sensible+liquid*latent-limits['input_energy_J'],'ZnO_mol':oxide,'qualification':'Independent integration of declared source Cp plus declared fusion enthalpy, not experimental calibration.'}
        else:raise AssertionError(c['id'])
        results.append({'id':c['id'],'script_sha256':c['script_sha256'],'response_sha256':sha(raw),'original_forecast':c['original_forecast'],'original_criteria':c['original_criteria'],'criteria':checks,'metrics':metrics,'status':'PASS' if all(checks.values()) else 'FAIL'})
    result={'engine':design['accepted_engine'],'actual_source_property_sha256':props['property_source_sha256'],'saved_reader_sha256':sha(Path(__file__).read_bytes()),'cases':results,'new_requests':0,'catalog_additions_by_reader':0,'qualification':'Root must independently select only fully satisfied original criteria in their bounded domain; other failures remain excluded.'};a.out.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n');print(json.dumps({c['id']:c['status'] for c in results}))
if __name__=='__main__':main()
