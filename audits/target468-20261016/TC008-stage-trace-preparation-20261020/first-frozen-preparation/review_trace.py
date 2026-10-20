#!/usr/bin/env python3
"""Independent saved-stage ledger for actual future TC008 traces. Never executes chemistry."""
import argparse,hashlib,json,pathlib,subprocess
from decimal import Decimal,localcontext
D=Decimal;Z=D(0);R=D('8.31446261815324');FUS=D(6010);T0=D('273.15');STD=D('298.15')
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=pathlib.Path,required=True);p.add_argument('--trace',type=pathlib.Path,required=True);p.add_argument('--source',required=True);p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args();assert not a.out.exists();get=lambda path:subprocess.check_output(['git','show',a.source+':'+path],cwd=a.repo);sha=lambda b:hashlib.sha256(b).hexdigest()
 goldraw=get('crates/kerotakis-core/tests/golden/registry.json');regraw=get('data/registry/registry-source-v1.json');gold={r['key']:r for r in json.loads(goldraw,parse_float=D)};reg=json.loads(regraw,parse_float=D);comps={x['species_id']:x for x in reg['compositions']};raw=a.trace.read_bytes();rows=[json.loads(x,parse_float=D) for x in raw.splitlines()];assert rows and [x['sequence'] for x in rows]==list(range(len(rows)));assert rows[0]['kind']=='trace_design' and rows[-1]['kind']=='final_bench';script=(pathlib.Path(__file__).parent/'fixtures/tc008-original.lab').read_text();assert rows[0]['script']==script;commands={i+1:s.strip() for i,s in enumerate(script.splitlines()) if s.strip() and not s.strip().startswith('#')};assert len(commands)==9
 assert gold['NaCl']['standard_phase']=='solid' and gold['NaCl']['dissolution_enthalpy_kj']==D('3.88');assert gold['Na+']['heat_capacity']==gold['Cl-']['heat_capacity']==0 and not gold['Na+']['heat_capacity_polys'] and not gold['Cl-']['heat_capacity_polys'];assert not any(r['kind']=='unexpected_non_equilibrate_call' for r in rows),'Unobserved alternate adapter entry must not silently pass'
 with localcontext() as ctx:
  ctx.prec=45
  def cp(row,phase,t):
   sets=[x for x in row['heat_capacity_polys'] if x['phase']==phase]
   if not sets:return row['heat_capacity']
   ints=sets[0]['intervals'];t=max(ints[0]['t_min'],min(t,ints[-1]['t_max']));q=next(q for q in ints if q['t_min']<=t<=q['t_max']);v=q['coefficients'];assert q['form']=='nasa9';return R*(v[0]/t**2+v[1]/t+sum((v[i]*t**(i-2) for i in range(2,7)),Z))
  def integral(row,phase,l,h):
   if l==h:return Z
   if l>h:return -integral(row,phase,h,l)
   sets=[x for x in row['heat_capacity_polys'] if x['phase']==phase]
   if not sets:return row['heat_capacity']*(h-l)
   ints=sets[0]['intervals'];cuts=sorted({l,h,*[q[k] for q in ints for k in ['t_min','t_max'] if l<q[k]<h]});total=Z
   for lo,hi in zip(cuts,cuts[1:]):
    mid=(lo+hi)/2
    if hi<=ints[0]['t_min'] or lo>=ints[-1]['t_max']:total+=cp(row,phase,mid)*(hi-lo);continue
    q=next(q for q in ints if q['t_min']<=mid<=q['t_max']);v=q['coefficients'];total+=R*(v[0]*(1/lo-1/hi)+v[1]*(hi/lo).ln()+sum((v[i]*(hi**(i-1)-lo**(i-1))/D(i-1) for i in range(2,7)),Z))
   return total
  def atom(v):
   result={}
   for q in v['contents']:
    c=comps['OH-' if q['species']=='base_equivalents' else q['species']];assert q['moles']>=0
    for k,n in [*[(x['element'],x['count']['value']) for x in c['elements']],('charge',c['net_charge']['value'])]:result[k]=result.get(k,Z)+q['moles']*n
   return result
  def energy(v):
   # One declared reference: water liquid273=0, ice273=-6010;
   # ionic master enthalpies298=0 and solidNaCl298=-3880.
   # Species Cp integrals retain their own phase and endpoint clamps.
   assert v['headspace']=={'boundary':'open'} and v['thermal_mode']=='adiabatic';assert not v['surfaces'] and v['pending_co2_transfer_mol']==0 and v['nuclides']=={'inventory':{}} and not any(v['foam'].values());assert all(not v.get(k) for k in ['unresolved_materials','material_objects','surface_colours','charred_materials','electrodes','exchanges','adsorbed','solid_solutions','soap_scum','lemon_paper_mark','surface_particles','emulsion'])
   terms=[]
   for q in v['contents']:
    key=q['species'];phase=q['phase'];n=q['moles'];T=v['temperature']
    if key=='water':assert phase in ['solid','liquid'];h=integral(gold[key],phase,T0,T)-(FUS if phase=='solid' else Z)
    elif key=='NaCl':assert phase=='solid';h=-D(3880)+integral(gold[key],phase,STD,T)
    elif key in ['Na+','Cl-']:assert phase=='aqueous';h=Z
    else:raise AssertionError(('No invented heat for unexpected canonical portion',key,phase))
    terms.append({'species':key,'phase':phase,'moles':n,'molar_declared_H_J_mol':h,'H_J':n*h})
   return {'terms':terms,'excess_J':v['excess_enthalpy_j'],'total_H_J':sum((x['H_J'] for x in terms),Z)+v['excess_enthalpy_j']}
  def delta(before,after):
   A=atom(before);B=atom(after);res={k:B.get(k,Z)-A.get(k,Z) for k in sorted(A.keys()|B.keys())};assert all(abs(v)<=D('1e-10') for v in res.values()),res;e0=energy(before);e1=energy(after);return {'atom_charge_delta':res,'before_energy':e0,'after_energy':e1,'closed_adiabatic_stage_residual_J':e1['total_H_J']-e0['total_H_J'],'within_frozen_0_001J_control_contract':abs(e1['total_H_J']-e0['total_H_J'])<=D('.001')}
  active=[];starts={};adapters=[];gaps=[];last_disp={};phase_calls={};enters={};exits={};whole=[]
  for row in rows:
   kind=row['kind'];command=row['command_index']
   if kind=='command_enter':assert commands[command]==row['text'];enters[command]=row
   if kind=='adapter_enter':
    call=row['call'];boundary=row['boundary'];assert call not in starts;starts[call]=row;parent=active[-1] if active else None;active.append(call)
    if boundary=='phase_coupled':phase_calls[call]={'entry':row,'displacements':[]}
    if boundary=='displacement':
     assert parent in phase_calls;phase_calls[parent]['displacements'].append(call)
     if parent in last_disp:gaps.append({'kind':'between_actual_displacement_calls','phase_call':parent,'command_index':command,'before_sequence':last_disp[parent]['sequence'],'after_sequence':row['sequence'],**delta(last_disp[parent]['state']['vessel'],row['state']['vessel']),'attribution':'Observed between-call change; source phase-coupled control flow is reviewed separately, not an invented private heat counter.'})
   if kind=='adapter_exit':
    call=row['call'];assert active.pop()==call;start=starts[call];assert row['boundary']==start['boundary'] and command==start['command_index'];assert row['result']=='ok',row;events=row['events'];assert not any(e['event'] in ['solver_failed','gas_evolved','gas_absorbed'] for e in events),'Boundary/domain changed; re-adjudicate rather than assume zero flux';v0=start['state']['vessel'];v1=row['state']['vessel'];adapters.append({'call':call,'boundary':row['boundary'],'command_index':command,'before_sequence':start['sequence'],'after_sequence':row['sequence'],**delta(v0,v1),'before_aqueous_heat_ownership':start['state']['aqueous_heat_ownership'],'after_aqueous_heat_ownership':row['state']['aqueous_heat_ownership'],'private_whole_step_vs_fallback_branch':'UNOBSERVABLE through public Equilibrator; public ownership flag is not a per-call heat-price receipt.'})
    if row['boundary']=='displacement':assert active[-1] in phase_calls;last_disp[active[-1]]=row
    if row['boundary']=='phase_coupled' and call in last_disp:gaps.append({'kind':'post_last_displacement_remainder','phase_call':call,'command_index':command,'before_sequence':last_disp[call]['sequence'],'after_sequence':row['sequence'],**delta(last_disp[call]['state']['vessel'],v1),'attribution':'Actual terminal remainder; may include StateEquilibrator and/or final projection. Do not label pure projection without source/event proof.'})
   if kind=='command_exit':
    assert not active and commands[command]==row['text'];exits[command]=row;v0=enters[command]['bench']['vessels'][0];v1=row['bench']['vessels'][0];e0=energy(v0);e1=energy(v1);op=row.get('operator');input_H=Z;external_Q=Z
    if op and op['op']=='add':
     key=op['species'];assert key in ['water','NaCl'];T=op['at'];input_H=op['moles']*(integral(gold['water'],'liquid',T0,T) if key=='water' else -D(3880)+integral(gold['NaCl'],'solid',STD,T))
    elif op and op['op']=='cool':external_Q=-op['energy']
    elif op:assert op['op']=='measure'
    A=atom(v0);B=atom(v1);expected_input={}
    if op and op['op']=='add':
     c=comps[op['species']]
     for k,n in [*[(x['element'],x['count']['value']) for x in c['elements']],('charge',c['net_charge']['value'])]:expected_input[k]=op['moles']*n
    material_residual={k:B.get(k,Z)-A.get(k,Z)-expected_input.get(k,Z) for k in sorted(A.keys()|B.keys()|expected_input.keys())};assert all(abs(v)<=D('1e-10') for v in material_residual.values()),material_residual
    res=e1['total_H_J']-e0['total_H_J']-input_H-external_Q;whole.append({'command_index':command,'text':row['text'],'owned_atom_charge_residual':material_residual,'before_energy':e0,'after_energy':e1,'incoming_declared_species_H_J':input_H,'external_heat_J':external_Q,'whole_command_residual_J':res,'within_frozen_0_001J_control_contract':abs(res)<=D('.001')})
  assert len(enters)==len(exits)==9 and not active and len(adapters)>0;assert rows[-1]['bench']==exits[max(exits)]['bench']
  result={'schema':'kerotakis.TC008.actual-stage-ledger.v1','trace_sha256':sha(raw),'original_script_sha256':sha(script.encode()),'source':a.source,'source_tree':subprocess.check_output(['git','rev-parse',a.source+'^{tree}'],cwd=a.repo,text=True).strip(),'registry_sha256':sha(regraw),'thermal_fixture_sha256':sha(goldraw),'source_binding_qualification':'Reader source argument must match separately authenticated actual producer source/tree/binary/lock receipt; no receipt is invented by this mathematical reader.','material_tolerance_mol':'1e-10','frozen_control_energy_tolerance_J':'.001','reference':'water-liquid273 zero; water-solid273 -6010J/mol; ionic-master298 zero; NaCl-solid298 -3880J/mol; independent phase-Cp integrals; explicit excess reservoir','adapter_boundaries':adapters,'intervening_phase_remainders':gaps,'whole_commands':whole,'observed_energy_contract_holds':all(x['within_frozen_0_001J_control_contract'] for x in whole),'new_engine_API_provider_build_calls':0,'scientific_catalog_selection_assigned':False,'scope':'Collection-test success only authenticates trace completion. This reader reports actual energy residuals; a baseline may truthfully fail. No chemical heat-price branch or independently calibrated brine enthalpy is guessed.'}
 a.out.write_text(json.dumps(result,indent=2,default=str)+'\n')
if __name__=='__main__':main()
