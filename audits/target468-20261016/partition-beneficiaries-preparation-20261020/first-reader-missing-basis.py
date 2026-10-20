#!/usr/bin/env python3
"""Independent saved six-original production review. Never invokes an engine or HTTP."""
import argparse,hashlib,json,math,subprocess
from decimal import Decimal,localcontext
from pathlib import Path
D=Decimal;Z=D(0);TOL=D('1e-8');sha=lambda b:hashlib.sha256(b).hexdigest()
IDS={'batch2:gs-014','batch2:gs-018','batch2:ki-002','batch2:se-009','batch2:tc-002','batch2:tc-009'}
def load(p):return json.loads(p.read_bytes(),parse_float=D)
def amount(v,sp,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==sp and (phase is None or p['phase']==phase)),Z)
def cp(row,phase,T):
 sets=[r for r in row['heat_capacity_polys'] if r['phase']==phase]
 if not sets:return row['heat_capacity']
 rows=sets[0]['intervals'];T=max(rows[0]['t_min'],min(T,rows[-1]['t_max']));q=next(r for r in rows if r['t_min']<=T<=r['t_max']);assert q['form']=='nasa9';a=q['coefficients'];return 8.31446261815324*(a[0]/T**2+a[1]/T+a[2]+a[3]*T+a[4]*T**2+a[5]*T**3+a[6]*T**4)
def integral(row,phase,a,b):
 n=400;h=(b-a)/n;return h/3*sum((1 if i in (0,n) else 4 if i%2 else 2)*cp(row,phase,a+i*h) for i in range(n+1))
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--design',type=Path,required=True);p.add_argument('--results-dir',type=Path,required=True);p.add_argument('--artifact-receipt',type=Path,required=True);p.add_argument('--production-receipt',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();assert not a.out.exists();design=load(a.design);artifact=load(a.artifact_receipt);production=load(a.production_receipt);engine=design['accepted_engine'];assert set(c['id'] for c in design['cases'])==IDS and len(design['cases'])==6
 assert design['origin']=='https://kerotakis-web-payload.vercel.app' and design['bound_to_authenticated_accepted_artifact'];assert sha(a.artifact_receipt.read_bytes())==design['authenticated_artifact_receipt_sha256'];assert artifact['actual_binary_bytes_rehashed'] and artifact['actual_producer_lock_bytes_rehashed'];assert all(artifact['engine'][k]==v for k,v in engine.items());assert production['engine']==engine and production['source']==engine['source_head'] and production['actual_five_asset_hashes_verified'];assert production['unchanged_API_controls']==11 and production['unchanged_browser_controls']==37 and production['fresh_targeted_browser_controls']==7
 assert a.design.read_bytes()==(a.results_dir/'design.json').read_bytes();completion=load(a.results_dir/'completion.json');assert completion['no_retries'] and len(completion['cases'])==6
 source=engine['source_head'];assert source=='dacf76513328c2226e2f5133a79fb9af84343431';show=lambda path:subprocess.check_output(['git','show',source+':'+path],cwd=a.repo);assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=a.repo,text=True).strip()==artifact['engine']['source_tree']
 regraw=show('data/registry/registry-source-v1.json');reg=json.loads(regraw,parse_float=D);comps={r['species_id']:r for r in reg['compositions']};goldraw=show('crates/kerotakis-core/tests/golden/registry.json');gold={r['key']:r for r in json.loads(goldraw)};intake={r['original_id']:r for r in load(a.repo/'audits/new-experiments-201-20261016/intake.json')}
 def composition(s):
  if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
  r=comps[s];return {**{x['element']:x['count']['value'] for x in r['elements']},'charge':r['net_charge']['value']}
 def book(out,s,n):
  for k,c in composition(s).items():out[k]=out.get(k,Z)+c*n
 def ledger(v):
  out={}
  for p in v['contents']:assert p['moles'].is_finite() and p['moles']>=0;book(out,p['species'],p['moles'])
  return out
 def carbon(v):return sum((p['moles']*composition(p['species']).get('C',Z) for p in v['contents']),Z)
 results=[]
 with localcontext() as ctx:
  ctx.prec=45
  for case in design['cases']:
   cid=case['id'];stem=cid.replace(':','-');folder=a.results_dir/stem;raw=(folder/'response.body').read_bytes();http=load(folder/'http.json');request=load(folder/'request.json');response=json.loads(raw,parse_float=D);original=intake[cid]
   assert http['id']==cid and http['http_status']==200 and http['attempts']==1 and http['response_sha256']==sha(raw);assert request=={'script':case['script']};assert sha(case['script'].encode())==case['script_sha256']==response['script_sha256']==original['script_sha256'];assert case['original_forecast']==original['original_expected_result'] and case['original_acceptance_checks']==original['original_acceptance_checks'];assert response['engine']==engine
   assert response['execution_status'] in ('completed','refused');rows=response.get('observations');assert isinstance(rows,list) and rows,'Unsupported/missing observation encoding: do not invent decoded evidence'
   expected={};state={};frames=[];step_ledgers=[];maxima={};limits=[];exports=[];phase_checks=[]
   for i,row in enumerate(rows):
    before=state.copy();op=row['operator'];typ=op['op'];events=row['events'];published={v['id']:v for v in row['bench']['vessels']};expected.setdefault(op.get('vessel',0),{})
    if typ=='add':book(expected[op['vessel']],op['species'],op['moles'])
    elif typ=='seal':
     v=before[op['vessel']];n=Z if v['headspace']['boundary'] in ('sealed','pressure_controlled') else D(101325)*op['headspace_volume']/(D('8314.462618')*v['temperature'])
     for sp,f in [('N2',D('.7901')),('O2',D('.2095')),('CO2',D('.0004'))]:book(expected[op['vessel']],sp,n*f)
    elif typ in ('filter','drain','decant'):
     success=any(e['event']=={'filter':'filtered','drain':'drained','decant':'transferred'}[typ] for e in events)
     if success:
      frm,to=op['from'],op['to'];expected.setdefault(to,{})
      for part in before[frm]['contents']:
       move=part['phase'] in ('liquid','aqueous') if typ!='drain' else part['species']=='water' or part['phase']=='aqueous'
       if move:
        n=part['moles']*op.get('fraction',D(1));book(expected[frm],part['species'],-n);book(expected[to],part['species'],n)
      if typ=='filter':
       gases=[part for part in before[frm]['contents'] if part['phase']=='gas'];owned=all(abs(amount(published[frm],g['species'],'gas')-g['moles'])<=D('1e-12') for g in gases);phase_checks.append({'row':i,'filter_donor_gases_kept':owned,'owned_gases':gases})
    else:assert typ in ('new_vessel','wait','inspect','measure'),typ
    for e in events:
     if e['event'] in ('gas_evolved','gas_absorbed'):
      direction=D(-1) if e['event']=='gas_evolved' else D(1);book(expected.setdefault(e['vessel'],{}),e['species'],direction*e['moles']);exports.append({'row':i,**e})
     if e['event'] in ('solver_failed','not_yet_modeled','safety_veto'):limits.append({'row':i,**e})
    if set(before)-set(published):assert typ in ('inspect','measure');assert all(v==before[k] for k,v in published.items())
    if typ=='wait':assert all(abs(v['elapsed_seconds']-before[k]['elapsed_seconds']-op['seconds'])<=D('1e-12') for k,v in published.items())
    state.update(published)
    for vid,v in state.items():
     assert not v.get('unresolved_materials') and not v.get('surfaces');actual=ledger(v);exp=expected.setdefault(vid,{});res={k:actual.get(k,Z)-exp.get(k,Z) for k in actual.keys()|exp.keys()}
     for k,d in res.items():maxima[k]=max(maxima.get(k,Z),abs(d))
     step_ledgers.append({'row':i,'vessel':vid,'residuals':res})
    frames.append((row,before,state.copy()))
   checks={'complete_execution':response['execution_status']=='completed' and response['exit_status']==0,'every_step_atoms_charge':all(v<=TOL for v in maxima.values()),'filter_phase_ownership':all(p['filter_donor_gases_kept'] for p in phase_checks),'no_relevant_unmodeled_or_solver_failure':not limits};metrics={};criteria={}
   if cid=='batch2:gs-014':
    stages=[after[0] for row,_,after in frames if row['operator']['op']=='inspect'];base,acid=stages[0],stages[-1];gasbase=amount(base,'NH3','gas');gasacid=amount(acid,'NH3','gas');reduced=lambda v:amount(v,'NH3')+amount(v,'NH4+');criteria={'GS-014-C1':gasbase>0 and gasacid<gasbase and amount(acid,'NH4+')>amount(base,'NH4+'),'GS-014-C2':abs(reduced(base)-D('.001'))<=TOL and abs(reduced(acid)-D('.001'))<=TOL};metrics={'base_NH3_gas_mol':gasbase,'acid_NH3_gas_mol':gasacid,'base_NH3_aq_mol':amount(base,'NH3','aqueous'),'acid_NH3_aq_mol':amount(acid,'NH3','aqueous'),'base_NH4_mol':amount(base,'NH4+'),'acid_NH4_mol':amount(acid,'NH4+'),'actual_temperature_K':[base['temperature'],acid['temperature']]}
   elif cid=='batch2:gs-018':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');captured=amount(before[0],'CaCO3','solid');finalcarbon=carbon(state[0]);criteria={'GS-018-C1':captured>0 and abs(captured-amount(after[0],'CaCO3','solid'))<=TOL and amount(state[0],'CaCO3','solid')<=TOL and abs(finalcarbon-carbon(after[0]))<=TOL,'GS-018-C2':checks['every_step_atoms_charge'] and checks['filter_phase_ownership']};metrics={'captured_carbonate_mol':captured,'postfilter_source_total_C_mol':carbon(after[0]),'final_source_C_mol':finalcarbon,'final_receiver_C_mol':carbon(state[1]),'filter_phase_checks':phase_checks}
   elif cid=='batch2:ki-002':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');f0=amount(after[2],'H2O2');c0=amount(after[1],'H2O2');floss=f0-amount(state[2],'H2O2');closs=c0-amount(state[1],'H2O2');criteria={'KI-002-C1':f0>0 and abs(f0-amount(before[0],'H2O2'))<=TOL and amount(after[2],'MnO2')==0,'KI-002-C2':0<floss/f0<closs/c0 and checks['filter_phase_ownership']};metrics={'filtrate_start_mol':f0,'control_start_mol':c0,'filtrate_additional_loss_mol':floss,'control_additional_loss_mol':closs,'actual_elapsed_s':[state[v]['elapsed_seconds'] for v in (0,1,2)],'filter_phase_checks':phase_checks}
   elif cid=='batch2:se-009':
    dry=[{'row':i,'vessel':v['id'],'aqueous_silica_mol':amount(v,'SiO2','aqueous')} for i,(row,_,after) in enumerate(frames) for v in after.values() if amount(v,'water')==0 and amount(v,'hexane')>0 and amount(v,'SiO2','aqueous')>D('1e-12')];fractions=[{'vessel':v['id'],'water':amount(v,'water'),'hexane':amount(v,'hexane'),'solid_silica':amount(v,'SiO2','solid'),'aqueous_silica':amount(v,'SiO2','aqueous'),'Na':amount(v,'Na+'),'Cl':amount(v,'Cl-')} for v in state.values()];criteria={'SE-009-C1':not dry and amount(state[0],'SiO2','solid')>0 and amount(state[1],'water')>0 and amount(state[2],'hexane')>0,'SE-009-C2':checks['every_step_atoms_charge']};metrics={'fractions':fractions,'dry_organic_aqueous_silica':dry}
   else:
    mixes=[]
    for row,before,after in frames:
     if row['operator']['op']!='decant':continue
     op=row['operator'];dst,src=op['to'],op['from'];final=after[dst];Tf=float(final['temperature']);q=sum(float(part['moles'])*integral(gold[part['species']],part['phase'],float(v['temperature']),Tf) for v in (before[dst],before[src]) for part in v['contents']);nw=[amount(before[v],'water') for v in (dst,src)];baseline=sum(n*before[v]['temperature'] for n,v in zip(nw,(dst,src)))/sum(nw);mixes.append({'vessel':dst,'premix_T_K':[before[v]['temperature'] for v in (dst,src)],'actual_final_T_K':final['temperature'],'water_weighted_baseline_K':baseline,'water_weighted_corrected_rise_K':final['temperature']-baseline,'independent_curve_mixing_sensible_J':q,'premix_excess_enthalpy_J':sum(before[v]['excess_enthalpy_j'] for v in (dst,src)),'final_excess_enthalpy_J':final['excess_enthalpy_j'],'acid_basis_mol':amount(final,'H+'),'hydroxide_basis_mol':amount(final,'base_equivalents')})
    assert len(mixes)==2;energies=[m['independent_curve_mixing_sensible_J'] for m in mixes];positive=all(q>1 for q in energies);similar=positive and abs(energies[0]-energies[1])/max(energies)<.1
    if cid=='batch2:tc-002':criteria={'TC-002-C1':positive and similar,'TC-002-C2':all(abs(m['hydroxide_basis_mol'])<=TOL for m in mixes) and abs(mixes[0]['acid_basis_mol']-D('.0005'))<=TOL and abs(mixes[1]['acid_basis_mol']-D('.0015'))<=TOL}
    else:criteria={'TC-009-C1':positive and mixes[0]['water_weighted_corrected_rise_K']>mixes[1]['water_weighted_corrected_rise_K']>0,'TC-009-C2':similar}
    metrics={'mixes':mixes,'thermal_interpretation':'Integrate actual premixing inventory Cp to actual finalT; subtract sensible mixing and retain earlier excess/dissolution ledgers separately. Existing calorimeter energy is not newly generated neutralisation heat. No exactfourfold or empiricalenthalpyclaim.'}
   passed=all(checks.values()) and all(criteria.values());results.append({'id':cid,'status':'PASS' if passed else 'HOLD','source':engine,'script_sha256':case['script_sha256'],'response_sha256':sha(raw),'original_forecast':case['original_forecast'],'original_checks':case['original_acceptance_checks'],'original_limitations':case['original_limitations'],'checks':checks,'criteria':criteria,'metrics':metrics,'named_limits_and_failures':limits,'exports':exports,'per_step_material_charge':step_ledgers,'maximum_residuals':maxima})
 out={'schema':'kerotakis.partition-beneficiaries.saved-review.v1','engine':engine,'design_sha256':sha(a.design.read_bytes()),'artifact_receipt_sha256':sha(a.artifact_receipt.read_bytes()),'production_receipt_sha256':sha(a.production_receipt.read_bytes()),'registry_sha256':sha(regraw),'heat_capacity_fixture_sha256':sha(goldraw),'cases':results,'new_engine_API_executions':0,'catalog_selection_assigned':False,'qualification':'Bounded original criteria, complete owned material/phase accounting and explicit diagnostic limits; no experimental calibration.'};a.out.write_text(json.dumps(out,indent=2,default=str)+'\n');print(json.dumps({c['id']:c['status'] for c in results}))
if __name__=='__main__':main()
