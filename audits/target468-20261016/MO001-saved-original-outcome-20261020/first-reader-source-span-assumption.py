#!/usr/bin/env python3
"""Original MO001 saved-only mathematical review. Never executes an engine or network."""
import argparse,hashlib,json,os,pathlib,re,subprocess,zipfile
from decimal import Decimal,localcontext
D=Decimal;Z=D(0);TOL=D('1e-10');PRECISE=D('1e-12')
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=pathlib.Path,required=True);p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args();assert not a.out.exists()
 available=int(next(s.split()[1] for s in pathlib.Path('/proc/meminfo').read_text().splitlines() if s.startswith('MemAvailable:')))//1024;assert available>=3200 and os.getloadavg()[0]<=3.5,'Saved-review gate closed'
 R=a.repo;E=R/'audits/new-experiments-201-20261016/native-screen-results';load=lambda p:json.loads(p.read_bytes(),parse_float=D);sha=lambda b:hashlib.sha256(b).hexdigest();case=load(E/'cases/batch2_mo-001.json');intake=next(x for x in load(E.parent/'intake.json') if x['original_id']=='batch2:mo-001');binding=load(E/'binding.json');engine=binding['engine'];source=engine['source_head'];assert source=='c44692053a94c59dcf62e7fb67549f84c7d18f83' and engine['binary_sha256']=='670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f';assert case['engine']==engine
 assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=R,text=True).strip()==engine['source_tree'];get=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=R)
 paths=['data/registry/registry-source-v1.json','crates/kerotakis-core/src/vessel.rs','crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/curated.rs','crates/kerotakis-core/src/family.rs','crates/kerotakis-core/src/species.rs','crates/kerotakis-phreeqc/src/aqueous.rs'];blobs={p:get(p) for p in paths};registry=json.loads(blobs[paths[0]],parse_float=D);comps={x['species_id']:x for x in registry['compositions']};vsource=blobs[paths[1]].decode();bench=blobs[paths[2]].decode();curated=blobs[paths[3]].decode()
 vectors=['unresolved_materials','material_objects','surface_colours','charred_materials','electrodes','exchanges','adsorbed','solid_solutions'];objects=['soap_scum','lemon_paper_mark','surface_particles','emulsion','step_start']
 for name in vectors:assert re.search(r'#\[serde\(default, skip_serializing_if = "Vec::is_empty"\)\]\s+pub '+name+r':',vsource),name
 for name in objects:assert re.search(r'#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]\s+pub '+name+r':',vsource),name
 assert 'Material lots tracking provenance of additions' in vsource and 'Solver-derived state' in vsource
 assert 'name: "esterification"' in curated and 'equilibrium_log_k: Some(0.6020599913279624)' in curated
 assert 'this explicit request assumes equilibration, not a reaction rate' in curated and 'temperature dependence and reaction heat are not modelled' in curated
 titrate_source=bench.split('    fn titrate_loop(',1)[1].split('\n    fn ',1)[0]
 assert 'let mut v = start.clone();' in titrate_source and 'fraction * moles_per_step.0' in titrate_source and 'fraction * water_per_step.0' in titrate_source and 'solver.equilibrate(&mut v)' in titrate_source
 assert 'clock::advance' not in titrate_source and 'Operator::Wait' not in titrate_source and 'elapsed_seconds' not in titrate_source
 with(E/'raw-artifact.zip').open('rb') as f:archive_digest=hashlib.file_digest(f,'sha256').hexdigest()
 assert archive_digest==load(E/'artifact-metadata.json')['digest'].removeprefix('sha256:')
 with zipfile.ZipFile(E/'raw-artifact.zip') as z:
  script=z.read('batch2_mo-001/experiment.lab');raw=z.read('batch2_mo-001/stdout.jsonl');stderr=z.read('batch2_mo-001/stderr.txt')
 assert script==(E.parent/intake['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256']==intake['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(stderr)==case['execution']['stderr_sha256'] and not stderr;assert case['execution']['exit_code']==0
 assert case['original_acceptance_checks']==intake['original_acceptance_checks'] and case['original_physical_forecast']==intake['original_expected_result'];assert len(intake['original_acceptance_checks'])==2
 rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x];assert len(rows)==13
 with localcontext() as ctx:
  ctx.prec=45
  def composition(s):
   if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
   c=comps[s];return {**{e['element']:e['count']['value'] for e in c['elements']},'charge':c['net_charge']['value']}
  def book(d,s,n):
   for k,v in composition(s).items():d[k]=d.get(k,Z)+v*n
  def atoms(v):
   d={}
   for q in v['contents']:
    assert q['moles'].is_finite() and q['moles']>=0;book(d,q['species'],q['moles'])
   return d
  def amount(v,*keys):return sum((p['moles'] for p in v['contents'] if p['species'] in keys),Z)
  water={x['property']:x['quantity'] for x in registry['phase_thermodynamics'] if x['species_id']=='water'};assert water['mass_density']['unit']['symbol']=='g/mL' and water['molar_mass']['unit']['symbol']=='g/mol';carrier=water['mass_density']['value']*D(1000)/water['molar_mass']['value']
  state={};introduced={};frames=[];ledgers=[];maxima={};noncontents_checks=[];clocks=[];phase_states=[];titrations=[]
  for i,row in enumerate(rows):
   assert set(row['bench'])=={'vessels'};before=state.copy();published={v['id']:v for v in row['bench']['vessels']};op=row['operator'];kind=op['op'];events=row['events'];delta_inputs={}
   assert not any(e['event'] in ['solver_failed','gas_evolved','gas_absorbed','gas_produced','gas_contained','vessel_sealed','vessel_swept','transferred','drained','filtered','decanted'] for e in events)
   if kind=='add':
    book(introduced.setdefault(op['vessel'],{}),op['species'],op['moles']);delta_inputs={'vessel':op['vessel'],'species':op['species'],'moles':op['moles']}
   elif kind=='titrate':
    es=[e for e in events if e['event']=='titrated'];assert len(es)==1;e=es[0];assert e['titrant']==op['titrant']=='NaOH' and e['concentration']==op['concentration']==D('.1') and e['endpoint_reached'] and 0<e['steps']<=op['max_steps']==150;assert op['step']==D('.0001') and op['target_ph']==D('8.5')
    dose=e['total_volume']*op['concentration'];waterdose=e['total_volume']*carrier;book(introduced[op['vessel']],'NaOH',dose);book(introduced[op['vessel']],'water',waterdose);delta_inputs={'vessel':op['vessel'],'NaOH_mol':dose,'water_mol':waterdose,'water_per_L':carrier,'volume_L':e['total_volume']}
    curve=e['curve'];assert curve[0][0]==0 and len(curve)==e['steps']+1;assert all(x[0]<y[0] and x[1]<=y[1] for x,y in zip(curve,curve[1:]));assert curve[-2][1]<op['target_ph']<=curve[-1][1] and abs(curve[-1][0]/D(1000)-e['total_volume'])<=PRECISE;assert curve[-1][1]==e['final_ph']==published[op['vessel']]['solution']['ph'];assert abs(e['final_ph']-op['target_ph'])<=D('.0001');titrations.append((i,op,e,dose,waterdose))
   else:assert kind in ['react','inspect','new_vessel'],kind
   if set(before)-set(published):assert kind=='inspect' and all(v==before[k] for k,v in published.items())
   state.update(published)
   for vid,v in sorted(state.items()):
    introduced.setdefault(vid,{})
    assert v['headspace']=={'boundary':'open'} and v['pressure']==D(101325) and v['elapsed_seconds']==0 and v['thermal_mode']=='adiabatic';assert v['pending_co2_transfer_mol']==0 and not v['surfaces'];assert all(not v.get(n) for n in vectors) and all(v.get(n) is None for n in objects);assert v['nuclides']=={'inventory':{}} and all(x==0 for x in v['foam'].values());assert v['resolved']=={'solution':None,'valid':False}
    actual=atoms(v);res={k:actual.get(k,Z)-introduced[vid].get(k,Z) for k in sorted(actual.keys()|introduced[vid].keys())};assert all(abs(x)<=TOL for x in res.values()),(i,vid,res)
    for k,value in res.items():maxima[k]=max(maxima.get(k,Z),abs(value))
    ledgers.append({'row':i,'vessel':vid,'operator':kind,'cumulative_external_atom_charge_budget':dict(sorted(introduced[vid].items())),'owned_atoms_charge':dict(sorted(actual.items())),'residuals':res,'actual_parts':v['contents'],'external_input_this_command':delta_inputs if delta_inputs.get('vessel')==vid else {}})
    noncontents_checks.append({'row':i,'vessel':vid,'all_optional_material_reservoirs_empty':True,'nuclides_empty':True,'foam_zero':True,'pending_external_CO2_credit_mol':v['pending_co2_transfer_mol'],'source_derived_solution_and_lots_not_double_counted':True})
    clocks.append({'row':i,'vessel':vid,'seconds':v['elapsed_seconds'],'temperature_K':v['temperature'],'pH':v['solution']['ph'] if v.get('solution') else None})
    phase_states.append({'row':i,'vessel':vid,'water_by_phase':{phase:sum((p['moles'] for p in v['contents'] if p['species']=='water' and p['phase']==phase),Z) for phase in ['liquid','aqueous','solid','gas']},'ester_by_phase':{phase:sum((p['moles'] for p in v['contents'] if p['species']=='ethyl_acetate' and p['phase']==phase),Z) for phase in ['liquid','aqueous','solid','gas']}})
   frames.append((row,before,state.copy()))
  assert len(titrations)==2 and len(ledgers)==19
  reaction_rows=[(i,f) for i,f in enumerate(frames) if f[0]['operator']['op']=='react'];assert len(reaction_rows)==1 and reaction_rows[0][0]==2
  row,before,after=frames[2];reactions=[e for e in row['events'] if e['event']=='org_reacted'];assert len(reactions)==1 and reactions[0]['name']=='esterification';xi=reactions[0]['extent'];pre=after[0];assert before[0]['solution'] is None and pre['solution'] is None
  independent_xi=D('.001')*D(2)/D(3);assert abs(xi-independent_xi)<=PRECISE
  assert abs(amount(before[0],'CH3COOH')-amount(pre,'CH3COOH')-xi)<=PRECISE and abs(amount(before[0],'ethanol')-amount(pre,'ethanol')-xi)<=PRECISE and abs(amount(pre,'ethyl_acetate')-xi)<=PRECISE and abs(amount(pre,'water')-xi)<=PRECISE
  quotient=amount(pre,'ethyl_acetate')*amount(pre,'water')/(amount(pre,'CH3COOH')*amount(pre,'ethanol'));assert abs(quotient-D(4))<=D('1e-12')
  ester_trace=[]
  for i,(_,_,after) in enumerate(frames):
   if i>=2:
    v=after[0];assert amount(v,'ethyl_acetate')==xi;assert amount(v,'ethanol')==amount(pre,'ethanol');ester_trace.append({'row':i,'ester_mol':amount(v,'ethyl_acetate'),'ethanol_mol':amount(v,'ethanol'),'acetate_family_mol':amount(v,'CH3COOH','CH3COO-','NaOAc'),'elapsed_s':v['elapsed_seconds']})
  assert all(amount(after[1],'ethyl_acetate')==0 for _,_,after in frames if 1 in after)
  endpoints=[];waters=[]
  for i,op,e,dose,waterdose in titrations:
   _,before,after=frames[i];vid=op['vessel'];v=after[vid];v0=before[vid]
   assert all(q['species'] in ['Na+','CH3COOH','CH3COO-','base_equivalents','H+','OH-','water','ethyl_acetate','ethanol'] for q in v['contents'])
   delta_Na=amount(v,'Na+')-amount(v0,'Na+');assert abs(delta_Na-dose)<=PRECISE
   B=amount(v,'base_equivalents','OH-');H=amount(v,'H+');HA=amount(v,'CH3COOH');A=amount(v,'CH3COOH','CH3COO-');acid_from_endpoint=dose+HA-B+H;assert abs(acid_from_endpoint-A)<=PRECISE
   water_gain=amount(v,'water')-amount(v0,'water');independent_neutralisation_water=dose-(B-amount(v0,'base_equivalents','OH-'));water_res=water_gain-waterdose-independent_neutralisation_water;assert abs(water_res)<=PRECISE
   endpoints.append({'row':i,'vessel':vid,'delivered_OH_mol':dose,'owned_sodium_increment_mol':delta_Na,'protonated_acid_mol':HA,'residual_owned_base_coordinate_mol':B,'owned_proton_coordinate_mol':H,'acetate_family_mol':A,'independent_acid_from_endpoint_mol':acid_from_endpoint,'pH':e['final_ph'],'temperature_K':v['temperature'],'curve_points':len(e['curve']),'coarse_step_count':e['steps'],'actual_total_volume_L':e['total_volume'],'last_two_curve_points':e['curve'][-2:]})
   waters.append({'row':i,'vessel':vid,'actual_water_gain_mol':water_gain,'independent_carrier_water_mol':waterdose,'independent_NaOH_neutralisation_water_mol':independent_neutralisation_water,'residual_mol':water_res})
  sample,blank=endpoints;difference=blank['delivered_OH_mol']-sample['delivered_OH_mol'];acid_correction=blank['protonated_acid_mol']-sample['protonated_acid_mol'];base_correction=blank['residual_owned_base_coordinate_mol']-sample['residual_owned_base_coordinate_mol'];proton_correction=blank['owned_proton_coordinate_mol']-sample['owned_proton_coordinate_mol'];corrected=difference+acid_correction-base_correction+proton_correction;increment=D('.1')*D('.0001');assert abs(difference-xi)<=increment and abs(corrected-xi)<=PRECISE
  named=[{'row':i,'vessel':e['vessel'],'cause':e['cause'],'what':e['what'],'reason_key':e.get('reason',{}).get('key')} for i,r in enumerate(rows) for e in r['events'] if e['event']=='not_yet_modeled'];assert len(named)==1 and named[0]['row']==2 and named[0]['reason_key']=='not-modeled.mostly-organic-solvent'
  result={'schema':'kerotakis.MO001.original.saved-review.v1','status':'PASS_PROPOSAL_PENDING_PARENT_REVIEW','original_id':case['id'],'original_forecast':intake['original_expected_result'],'original_criteria':intake['original_acceptance_checks'],'original_limitations':intake['original_limitations'],'criteria':{'MO-001-C1':True,'MO-001-C2':True,'every_captured_command_per_vessel_atoms_charge':True,'all_owned_auxiliary_reservoirs_accounted':True,'carrier_water_independently_reconciled':True,'actual_phase_ownership_retained':True,'no_hidden_elapsed_time':True,'only_explicit_requested_ester_transformation':True,'declared_model_domain':True},'raw_binding':{'source':engine,'lock_qualification':binding['lock_qualification'],'archive_sha256':archive_digest,'original_script_path':str((E.parent/intake['original_script']).relative_to(R)),'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(stderr),'members':['batch2_mo-001/experiment.lab','batch2_mo-001/stdout.jsonl','batch2_mo-001/stderr.txt'],'captured_rows':len(rows)},'source_blobs_sha256':{p:sha(v) for p,v in blobs.items()},'metrics':{'independent_ideal_K4_extent_mol':independent_xi,'actual_requested_extent_mol':xi,'pre_workup_reaction_quotient':quotient,'endpoint_sample_blank':endpoints,'raw_blank_minus_sample_base_mol':difference,'nominal_dosing_resolution_mol':increment,'acid_protonation_correction_mol':acid_correction,'subtract_residual_base_difference_mol':base_correction,'add_owned_proton_difference_mol':proton_correction,'corrected_extent_mol':corrected,'corrected_minus_original_ester_mol':corrected-xi,'carrier_mol_per_L':carrier,'titrant_carrier_and_neutralisation_water':waters,'maximum_cumulative_atom_charge_residuals':maxima},'per_command_per_vessel':ledgers,'noncontents_reservoir_checks':noncontents_checks,'actual_clocks_temperatures_pH':clocks,'phase_water_ester':phase_states,'ester_workup_trace':ester_trace,'named_model_boundaries':named,'scope':['Original conditional requested ideal-mixture teaching model only; K4 and no reaction heat/temperature/activity dependence remain explicit. No claim about actual esterification rate, catalyst sufficiency, true thermodynamic equilibrium or empirical assay calibration.','Pure-organic initial request yields uncharacterised ionic state honestly; original assay occurs only after100mLwater addition with actual solved pH. The one mostly-organic warning does not establish failed original chemistry or authority for physical mixed-solvent acid/base values.','No extra ester hydrolysis is observed across saved dilution/titration commands, and source dose closure advances no time. This is a requested-only model observation, not proof of chemical inertness during a real time-consuming titration.','Native solution speciation/free-ion caches and MaterialLots are source-declared mirrors/provenance, not extra owned atoms. Every actual captured vessel is balanced from cumulative external original additions plus actual titrant/carrier.','Titration curve contains every committed coarse reading plus refined endpoint, but not every internal trial state; source clones each pre-increment trial and commits one dose. This review does not claim saved intermediate trial inventories exist.','No optical/calorimetry/mass-transfer or continuously coupled kinetic qualification is inferred. Actual assay temperatures are retained and may differ.'],'scientific_catalog_selection_assigned':False,'new_engine_API_provider_build_calls':0,'material_tolerance_mol':TOL,'endpoint_reconstruction_tolerance_mol':PRECISE}
 a.out.write_text(json.dumps(result,indent=2,default=str)+'\n');print(json.dumps({'status':result['status'],'rows':len(rows),'vessel_ledgers':len(ledgers),'maxima':maxima,'corrected_extent_mol':corrected},default=str))
if __name__=='__main__':main()
