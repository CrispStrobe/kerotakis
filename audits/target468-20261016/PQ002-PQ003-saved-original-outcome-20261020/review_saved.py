#!/usr/bin/env python3
"""Two frozen original native-complex reviews; saved data only, no engine/network."""
import argparse,hashlib,json,os,pathlib,re,subprocess,zipfile
from decimal import Decimal,localcontext
D=Decimal; Z=D(0); TOL=D('1e-10')
def main():
 p=argparse.ArgumentParser();p.add_argument('--repo',type=pathlib.Path,required=True);p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args();assert not a.out.exists()
 assert os.getloadavg()[0]<=3.5 and int(next(x.split()[1] for x in pathlib.Path('/proc/meminfo').read_text().splitlines() if x.startswith('MemAvailable:')))>=3200*1024
 R=a.repo;E=R/'audits/new-experiments-201-20261016/native-screen-results';load=lambda p:json.loads(p.read_bytes(),parse_float=D);sha=lambda b:hashlib.sha256(b).hexdigest();binding=load(E/'binding.json');engine=binding['engine'];source=engine['source_head'];assert source=='c44692053a94c59dcf62e7fb67549f84c7d18f83';assert engine['binary_sha256']=='670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f'
 assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=R,text=True).strip()==engine['source_tree']
 get=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=R)
 paths=['data/registry/registry-source-v1.json','crates/kerotakis-core/src/vessel.rs','crates/kerotakis-core/src/species.rs','crates/kerotakis-core/src/bench.rs','crates/kerotakis-phreeqc/src/aqueous.rs','crates/kerotakis-phreeqc/src/complexation.rs','data/thermo/usbm-ic9429-complexes.dat'];blobs={p:get(p) for p in paths};registry=json.loads(blobs[paths[0]],parse_float=D);comps={x['species_id']:x for x in registry['compositions']}
 vendor='47b459c4a5e3e20b9990fa304e207eca0dcc6ff1';assert vendor in subprocess.check_output(['git','ls-tree',source,'vendor/iphreeqc'],cwd=R,text=True)
 printer=subprocess.check_output(['git','show',vendor+':src/phreeqcpp/print.cpp'],cwd=R/'vendor/iphreeqc');assert b'%-13s%12.3e%12.3e%10.3f%10.3f%10.3f' in printer
 aq=blobs[paths[4]].decode();assert 'output.rfind("Distribution of species")' in aq and 'if molality > 1e-9' in aq
 assert b'Fe+3 + Thiocyanate- = Fe(Thiocyanate)+2\n        log_k 3.0' in blobs[paths[-1]] and b'Fe+3 + 2 Thiocyanate- = Fe(Thiocyanate)2+\n        log_k 3.6' in blobs[paths[-1]]
 database=subprocess.check_output(['git','show',vendor+':database/wateq4f.dat'],cwd=R/'vendor/iphreeqc');assert b'FeHPO4+' in database and b'FeH2PO4+2' in database and b'Goethite' in database
 vectors=['unresolved_materials','material_objects','surface_colours','charred_materials','electrodes','exchanges','adsorbed','solid_solutions'];objects=['soap_scum','lemon_paper_mark','surface_particles','emulsion','step_start'];vs=blobs[paths[1]].decode()
 for n in vectors:assert re.search(r'#\[serde\(default, skip_serializing_if = "Vec::is_empty"\)\]\s+pub '+n+r':',vs),n
 for n in objects:assert re.search(r'#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]\s+pub '+n+r':',vs),n
 assert 'Material lots tracking provenance of additions' in vs and 'Solver-derived state' in vs
 with(E/'raw-artifact.zip').open('rb') as f:archive=hashlib.file_digest(f,'sha256').hexdigest()
 assert archive==load(E/'artifact-metadata.json')['digest'].removeprefix('sha256:')
 with zipfile.ZipFile(E/'raw-artifact.zip') as z,localcontext() as ctx:
  ctx.prec=45
  assert json.loads(z.read('binding.json'),parse_float=D)==binding;producer=json.loads(z.read('producer-manifest.json'),parse_float=D);assert all(producer[k]==v for k,v in engine.items());execution=json.loads(z.read('execution.json'),parse_float=D);assert execution['engine']==engine;assert not [n for n in z.namelist() if n.endswith(('/kero','/Cargo.lock','.exe'))]
  def composition(s):
   c=comps['OH-' if s=='base_equivalents' else s];return {**{x['element']:x['count']['value'] for x in c['elements']},'charge':c['net_charge']['value']}
  def book(d,s,n):
   for k,v in composition(s).items():d[k]=d.get(k,Z)+v*n
  def atoms(v):
   d={}
   for q in v['contents']:assert q['moles']>=0;book(d,q['species'],q['moles'])
   return d
  def amount(v,s):return sum((x['moles'] for x in v['contents'] if x['species']==s),Z)
  def interval(v):
   assert v>0
   # %12.3e: half of one final printed decimal place. At a decade boundary
   # this symmetric interval is conservative on the lower side, never narrower.
   h=D(5)*D(10)**(v.adjusted()-4);return (v-h,v+h)
  def weight(name,pool):
   if pool=='Fe':return int(re.match(r'Fe(\d*)',name)[1] or 1) if name.startswith('Fe') else 0
   if pool=='SCN':return 2 if '(SCN)2' in name else 1 if 'SCN' in name else 0
   if pool=='P':return 1 if 'PO4' in name else 0
  cases=[]
  for cid in ['pq-002','pq-003']:
   case=load(E/('cases/batch2_'+cid+'.json'));intake=next(x for x in load(E.parent/'intake.json') if x['original_id']=='batch2:'+cid);prefix='batch2_'+cid+'/';script=z.read(prefix+'experiment.lab');raw=z.read(prefix+'stdout.jsonl');stderr=z.read(prefix+'stderr.txt');assert script==(E.parent/intake['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256']==intake['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(stderr)==case['execution']['stderr_sha256'] and not stderr;assert case['engine']==engine and case['execution']['exit_code']==0 and next(x for x in execution['results'] if x['id']==case['id'])==case['execution'];assert case['original_acceptance_checks']==intake['original_acceptance_checks'] and case['original_physical_forecast']==intake['original_expected_result']
   rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x];assert len(rows)==(18 if cid=='pq-002' else 13);state={};expected={};ledgers=[];maxima={};frames=[];decants=[];limits=[];instruments=[]
   for i,row in enumerate(rows):
    before=state.copy();op=row['operator'];kind=op['op'];published={v['id']:v for v in row['bench']['vessels']};assert set(row['bench'])=={'vessels'}
    if kind=='add':book(expected.setdefault(op['vessel'],{}),op['species'],op['moles'])
    elif kind=='decant':
     assert cid=='pq-003' and i==5 and op['fraction']==D('.5');src=op['from'];dst=op['to'];transfer={};expected.setdefault(dst,{})
     for q in before[src]['contents']:
      assert q['phase'] in ['liquid','aqueous'];n=q['moles']*op['fraction'];book(expected[src],q['species'],-n);book(expected[dst],q['species'],n);transfer[q['species']]=n
     # Neutralisation can repartition H/O basis, so atom check is authoritative;
     # every Fe/SCN/counterion/carrier has exact half transfer independently.
     for key in ['Fe+3','SCN-','K+','Cl-','water']:assert abs(amount(published[src],key)-amount(before[src],key)/2)<=TOL and abs(amount(published[dst],key)-amount(before[src],key)/2)<=TOL
     decants.append({'row':i,'original_fraction':op['fraction'],'transferred_all_eligible_parts_mol':transfer})
    else:assert kind in ['new_vessel','measure','inspect'],kind
    assert not any(e['event'] in ['solver_failed','gas_evolved','gas_absorbed','gas_produced','gas_contained','vessel_sealed','vessel_swept','filtered','drained'] for e in row['events'])
    state.update(published)
    for vid,v in sorted(state.items()):
     expected.setdefault(vid,{});assert v['headspace']=={'boundary':'open'} and v['pressure']==101325 and v['elapsed_seconds']==0 and v['thermal_mode']=='adiabatic';assert not v['surfaces'] and v['pending_co2_transfer_mol']==0 and all(not v.get(n) for n in vectors) and all(v.get(n) is None for n in objects);assert v['nuclides']=={'inventory':{}} and all(x==0 for x in v['foam'].values()) and v['resolved']=={'solution':None,'valid':False}
     actual=atoms(v);res={k:actual.get(k,Z)-expected[vid].get(k,Z) for k in sorted(actual.keys()|expected[vid].keys())};assert all(abs(x)<=TOL for x in res.values()),(cid,i,vid,res)
     for k,x in res.items():maxima[k]=max(maxima.get(k,Z),abs(x))
     ledgers.append({'row':i,'vessel':vid,'operator':kind,'actual_parts':v['contents'],'expected_cumulative_atoms_charge':expected[vid].copy(),'owned_atoms_charge':actual,'residuals':res,'elapsed_seconds':v['elapsed_seconds'],'temperature_K':v['temperature'],'pH':v['solution']['ph'] if v.get('solution') else None,'excess_enthalpy_J':v['excess_enthalpy_j'],'all_other_owned_reservoirs_empty':True})
    for e in row['events']:
     if e['event']=='not_yet_modeled':limits.append({'row':i,**e})
     if e['event']=='measured':instruments.append({'row':i,**e})
    frames.append(state.copy())
   final=frames[-1];alloc=[]
   for vid,v in sorted(final.items()):
    sol=v['solution'];kg=sol['solvent_kg'];assert kg==(D('.0997') if cid=='pq-002' else D('.04985'));s={x['name']:x for x in sol['species']};details=[];totals={}
    for pool,owned in [('Fe',amount(v,'Fe+3')),('SCN',amount(v,'SCN-')),('P',sum((q['moles']*composition(q['species']).get('P',Z) for q in v['contents']),Z))]:
     terms=[(n,x,weight(n,pool)) for n,x in s.items() if weight(n,pool)];center=sum((x['molality']*w*kg for n,x,w in terms),Z);lo=sum((interval(x['molality'])[0]*w*kg for n,x,w in terms),Z);hi=sum((interval(x['molality'])[1]*w*kg for n,x,w in terms),Z)
     assert lo<=owned<=hi,(cid,vid,pool,lo,owned,hi)
     totals[pool]={'analytical_owned_mol':owned,'reported_native_pool_center_mol':center,'four_significant_digit_sum_interval_mol':[lo,hi],'normalized_center_fraction':center/owned if owned else None,'omitted_species_not_proved_zero':True}
    for n,x in s.items():
     if any(weight(n,pool) for pool in ['Fe','P','SCN']):details.append({'name':n,**x,'moles_from_reported_molality':x['molality']*kg,'rounding_interval_mol':[y*kg for y in interval(x['molality'])],'Fe_atoms':weight(n,'Fe'),'SCN_ligands':weight(n,'SCN'),'P_atoms':weight(n,'P')})
    one=s['Fe(SCN)+2'];two=s['Fe(SCN)2+'];fe=s['Fe+3'];scn=s['SCN-'];massaction={}
    for name,q,power,k in [('one_ligand',one,1,D(1000)),('two_ligands',two,2,D(10)**D('3.6'))]:
     xl,xh=interval(q['activity']);fl,fh=interval(fe['activity']);sl,sh=interval(scn['activity']);bounds=[xl/(fh*sh**power),xh/(fl*sl**power)];assert bounds[0]<=k<=bounds[1],(cid,vid,name,bounds,k);massaction[name]={'derived_activity_ratio_interval':bounds,'source_cumulative_K':k}
    alloc.append({'vessel':vid,'solvent_kg':kg,'pH':sol['ph'],'temperature_K':v['temperature'],'all_reported_Fe_P_SCN_species':details,'analytical_normalization':totals,'activity_mass_action':massaction,'FeSCN_one_ligand_mol':one['molality']*kg,'FeSCN_two_ligand_mol':two['molality']*kg,'Fe_in_phosphate_complexes_mol':sum((x['molality']*kg for n,x in s.items() if n.startswith('Fe') and 'PO4' in n),Z)})
   assert not any(x.get('instrument')=='spectrophotometer' for x in instruments)
   optical_rows=[i for i,r in enumerate(rows) if r['operator']['op']=='measure' and r['operator']['instrument']=='spectrophotometer'];assert len(optical_rows)==len(final)
   for i in optical_rows:assert any(e.get('reason',{}).get('key')=='not-modeled.incomplete-absorbance' for e in rows[i]['events'])
   if cid=='pq-002':
    assert all(x['analytical_normalization']['Fe']['analytical_owned_mol']+x['analytical_normalization']['SCN']['analytical_owned_mol']==D('.0001') for x in alloc)
    peak=[next(x['rounding_interval_mol'] for x in v['all_reported_Fe_P_SCN_species'] if x['name']=='Fe(SCN)+2') for v in alloc];assert peak[1][0]>max(peak[0][1],peak[2][1]);assert all(v['FeSCN_one_ligand_mol']>100*v['FeSCN_two_ligand_mol'] for v in alloc)
   else:
    assert alloc[0]['FeSCN_one_ligand_mol']<alloc[1]['FeSCN_one_ligand_mol'] and alloc[0]['Fe_in_phosphate_complexes_mol']/D('.00005')>D('.96');assert alloc[0]['pH']!=alloc[1]['pH'];assert len(instruments)==2
    for reading in instruments:assert reading['instrument']=='ph_meter' and reading['unit']=='pH' and reading['value']==final[reading['vessel']]['solution']['ph']
   cases.append({'original_id':case['id'],'status':'BOUNDED_NATIVE_CRITERIA_PROPOSAL_PENDING_PARENT_DOMAIN_REVIEW','original_forecast':intake['original_expected_result'],'original_criteria':intake['original_acceptance_checks'],'original_limitations':intake['original_limitations'],'original_script':script.decode(),'raw_binding':{'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(stderr),'archive_members':[prefix+x for x in ['experiment.lab','stdout.jsonl','stderr.txt']]},'criteria':{x['id']:True for x in intake['original_acceptance_checks']},'per_command_per_vessel':ledgers,'maximum_owned_atom_charge_residual_mol':maxima,'half_decant':decants,'final_native_allocations':alloc,'actual_instruments':instruments,'all_named_limits':limits,'domain':{'native_concentrations_explicitly_permitted':True,'UVvis_success':False,'unregistered_or_unoffered_oxide_precipitation_not_modelled':True,'metastable_soluble_model_only_no_oxide_suppression_assumption':True,'reference_25C_complex_constants_no_empirical_validation':True,'unequal_pH_not_isolated_ligand_causality':cid=='pq-003','native_rounded_diagnostics_are_not_additional_owned_atoms':True,'no_temperature_energy_forecast_no_calorimetry_claim':True},'scope':'Conditional native analytical comparison only. All original oxide/spectral/speciation warnings remain. No complete physical phase equilibrium, empirical Job stoichiometry, optical response, pH-matched causal comparison, calibrated temperature dependence or rate is inferred.'})
 result={'schema':'kerotakis.PQ002.PQ003.saved-native-review.v1','source_engine':engine,'lock_qualification':binding['lock_qualification'],'binary_qualification':'670 binary identified by authenticated archived producer manifest only; executable/Cargo.lock absent, neither independently rehashed. Historical c446 source, not current-source performance.','archive_sha256':archive,'source_blobs_sha256':{p:sha(b) for p,b in blobs.items()},'native_printer_binding':{'vendor_gitlink':vendor,'path':'src/phreeqcpp/print.cpp','sha256':sha(printer),'database_path':'database/wateq4f.dat','database_sha256':sha(database),'format':'%12.3e molality and activity, four significant digits','interval_rule':'x +/- 0.5*10^(adjusted(x)-3), conservative at decade boundaries; native censor >1e-9, omitted species not proved zero.'},'owned_atom_charge_tolerance_mol':TOL,'cases':cases,'catalog_selection_assigned':False,'new_engine_API_provider_build_calls':0}
 a.out.write_text(json.dumps(result,indent=2,default=str)+'\n');print(json.dumps({'cases':[{'id':c['original_id'],'status':c['status'],'ledgers':len(c['per_command_per_vessel']),'maxima':c['maximum_owned_atom_charge_residual_mol']} for c in cases]},default=str))
if __name__=='__main__':main()
