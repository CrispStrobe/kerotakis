#!/usr/bin/env python3
"""Eight disjoint originals: authenticate historical raw bytes and independently review saved physics."""
import argparse,hashlib,json,re,subprocess,zipfile
from decimal import Decimal,localcontext
from pathlib import Path
D=Decimal;Z=D(0);TOL=D('1e-8');IDS=('gs-011','se-010','mo-012','qa-007','qa-008','ki-005','ki-009','ki-010');sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_bytes(),parse_float=D)
def amount(v,s,phase=None):return sum((p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase)),Z)
def review(root):
 base=root/'audits/new-experiments-201-20261016/native-screen-results';ar=base/'raw-artifact.zip';digest=hashlib.file_digest(ar.open('rb'),'sha256').hexdigest();assert digest==load(base/'artifact-metadata.json')['digest'].removeprefix('sha256:');binding=load(base/'binding.json');engine=binding['engine'];source=engine['source_head'];assert source=='c44692053a94c59dcf62e7fb67549f84c7d18f83';assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=root,text=True).strip()==engine['source_tree'];show=lambda p:subprocess.check_output(['git','show',source+':'+p],cwd=root)
 paths=['crates/kerotakis-core/src/'+p for p in ['bench.rs','vessel.rs','solve.rs','kinetics.rs','curated.rs','spectrum.rs','starch_iodine.rs']];blobs={p:show(p) for p in paths};bench=blobs[paths[0]].decode();vsource=blobs[paths[1]].decode();kinetics=blobs[paths[3]].decode();curated=blobs[paths[4]].decode();regraw=show('data/registry/registry-source-v1.json');reg=json.loads(regraw,parse_float=D);comps={r['species_id']:r for r in reg['compositions']};goldraw=show('crates/kerotakis-core/tests/golden/registry.json');gold={r['key']:r for r in json.loads(goldraw,parse_float=D)};intake={r['original_id']:r for r in load(base.parent/'intake.json')};selected={r['original_target_id'] for r in load(root/'audits/target468-20261016/selection-respiration-beneficiary-20261020/catalog-membership.json')['mapping']}
 vectors=['unresolved_materials','material_objects','electrodes','exchanges','adsorbed','solid_solutions','surface_colours','charred_materials'];objects=['soap_scum','lemon_paper_mark','surface_particles','emulsion']
 for f in vectors:assert re.search(r'#\[serde\(default, skip_serializing_if = "Vec::is_empty"\)\]\s+pub '+f+r':',vsource)
 for f in objects:assert re.search(r'#\[serde\(default, skip_serializing_if = "Option::is_none"\)\]\s+pub '+f+r':',vsource)
 def comp(s):
  if s=='catalase':return {} # Mass/amount tracer only; registry C placeholder is NOT a protein formula.
  if s=='base_equivalents':return {'H':D(1),'O':D(1),'charge':D(-1)}
  r=comps[s];return {**{e['element']:e['count']['value'] for e in r['elements']},'charge':r['net_charge']['value']}
 def book(d,s,n):
  for k,v in comp(s).items():d[k]=d.get(k,Z)+v*n
 def atoms(v,phase=None):
  d={}
  for p in v['contents']:
   assert p['moles']>=0 and p['moles'].is_finite()
   if phase is None or p['phase']==phase:book(d,p['species'],p['moles'])
  return d
 def elem(v,k,phase=None):return atoms(v,phase).get(k,Z)
 def oxid(v):return amount(v,'I2')+amount(v,'I3-') # Each I3- contains one I2 oxidant equivalent and one iodide.
 def netacid(v):return amount(v,'H+')-amount(v,'base_equivalents')
 output=[]
 with localcontext() as ctx,zipfile.ZipFile(ar) as archive:
  ctx.prec=45
  for suffix in IDS:
   key='batch2_'+suffix;case=load(base/'cases'/f'{key}.json');o=intake[case['id']];assert case['id'] not in selected;assert case['engine']==engine;assert case['original_acceptance_checks']==o['original_acceptance_checks'] and case['original_physical_forecast']==o['original_expected_result'];script=archive.read(key+'/experiment.lab');raw=archive.read(key+'/stdout.jsonl');err=archive.read(key+'/stderr.txt');assert script==(base.parent/o['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256'];assert sha(raw)==case['execution']['stdout_sha256'] and sha(err)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0;rows=[json.loads(x,parse_float=D) for x in raw.splitlines() if x.strip()];state={};expected={};frames=[];ledger=[];maximum={};air=[];exports=[];transfers=[];clocks=[];reservoirs=0
   for i,row in enumerate(rows):
    assert set(row['bench'])=={'vessels'};before=state.copy();op=row['operator'];typ=op['op'];published={v['id']:v for v in row['bench']['vessels']};events=row['events']
    if typ=='add':book(expected.setdefault(op['vessel'],{}),op['species'],op['moles'])
    elif typ=='dilute':book(expected[op['vessel']],'water',op['volume']*D(997)/D('18.015'))
    elif typ=='seal':
     v=before[op['vessel']];n=Z if v['headspace']['boundary']!='open' else D(101325)*op['headspace_volume']/(D('8314.462618')*v['temperature']);rec={'row':i,'vessel':op['vessel'],'N2_mol':n*D('.7901'),'O2_mol':n*D('.2095'),'CO2_mol':n*D('.0004')};air.append(rec)
     for s,f in [('N2',D('.7901')),('O2',D('.2095')),('CO2',D('.0004'))]:book(expected[op['vessel']],s,n*f)
    elif typ in ('filter','decant','magnet'):
     frm,to=op['from'],op['to'];expected.setdefault(to,{});moved=[]
     for p in before[frm]['contents']:
      eligible=(p['phase'] in ('liquid','aqueous')) if typ!='magnet' else (p['phase']=='solid' and gold[p['species']]['magnetic'])
      if eligible:
       n=p['moles']*op.get('fraction',D(1));book(expected[frm],p['species'],-n);book(expected[to],p['species'],n);moved.append({'species':p['species'],'moles':n,'phase':p['phase']})
     transfers.append({'row':i,'operation':typ,'from':frm,'to':to,'independently_expected_portions':moved,'original_donor_gases':[p for p in before[frm]['contents'] if p['phase']=='gas'],'actual_donor_gases':[p for p in published[frm]['contents'] if p['phase']=='gas']})
    else:assert typ in ('new_vessel','wait','inspect','measure'),typ
    for e in events:
     assert e['event']!='solver_failed'
     if e['event'] in ('gas_evolved','gas_absorbed'):
      # A finite open add's gas_absorbed event describes the already-booked dose, not an additional reservoir.
      dose_event=typ=='add' and e['event']=='gas_absorbed' and e['species']==op['species'] and e['vessel']==op['vessel']
      if not dose_event:book(expected[e['vessel']],e['species'],e['moles']*(D(-1) if e['event']=='gas_evolved' else D(1)))
      exports.append({'row':i,**e,'already_owned_finite_addition':dose_event})
    if set(before)-set(published):assert typ in ('inspect','measure') and all(v==before[k] for k,v in published.items())
    state.update(published)
    for vid,v in state.items():
     if vid in before:
      elapsed=v['elapsed_seconds']-before[vid]['elapsed_seconds'];assert elapsed==(op['seconds'] if typ=='wait' else Z)
     else:elapsed=Z;assert v['elapsed_seconds']==0
     clocks.append({'row':i,'vessel':vid,'op':typ,'elapsed_seconds':v['elapsed_seconds'],'delta_seconds':elapsed,'temperature_K':v['temperature'],'pH':v['solution']['ph'] if v.get('solution') else None});assert not v['surfaces'];assert all(not v.get(f) for f in vectors);assert all(v.get(f) is None for f in objects);assert v['nuclides']=={'inventory':{}} and not any(v['foam'].values()) and v['pending_co2_transfer_mol']==0 and v['resolved']=={'solution':None,'valid':False};reservoirs+=1;actual=atoms(v);ext=expected.setdefault(vid,{});res={k:actual.get(k,Z)-ext.get(k,Z) for k in actual.keys()|ext.keys()}
     for k,d in res.items():maximum[k]=max(maximum.get(k,Z),abs(d))
     ledger.append({'row':i,'vessel':vid,'residuals':res})
    frames.append((row,before,state.copy()))
   material=all(v<=TOL for v in maximum.values());full=material and suffix!='ki-005';criteria=[True,True];metrics={};notes=[];hold=None
   if suffix=='gs-011':
    firstseal=next(f for f in frames if f[0]['operator']['op']=='seal');b=firstseal[1][0];v=firstseal[2][0];escaped=sum((e['moles'] for e in exports if e['event']=='gas_evolved' and e['species']=='CO2' and e['row']<3),Z);air0=air[0]['CO2_mol'];air1=air[1]['CO2_mol'];assert escaped>D('.0009');assert abs(elem(b,'C')+escaped-D('.001'))<D('1e-10');assert abs(elem(v,'C')-elem(b,'C')-air0)<D('1e-10');finalC={k:elem(v,'C') for k,v in state.items()};assert abs(finalC[0]+escaped-air0-D('.001'))<D('1e-10') and abs(finalC[1]-air1-D('.001'))<D('1e-10');readings=[e for r in rows for e in r['events'] if e['event']=='measured'];metrics={'preseal_exported_CO2_mol':escaped,'preseal_retained_C_mol':elem(b,'C'),'postseal_retained_C_mol':elem(v,'C'),'final_C_mol':finalC,'independently_added_air':air,'actual_measurements':readings};notes=['Boundary-budget demonstration only: open-dose equilibrium and subsequent explicit reservoir update, not calibrated60s mass-transfer or degassing kinetics. Actual clocks differ120s/60s; no equal-aged rate comparison. No lost carbon is restored by sealing, apart from explicitly added baseline air.']
   elif suffix=='se-010':
    assert gold['Fe']['magnetic'] and not gold['Fe+3']['magnetic'];assert 'p.phase == Phase::Solid' in bench and 'species::lookup(&p.species).is_some_and(|d| d.magnetic)' in bench;metal=amount(state[2],'Fe','solid');ion=elem(state[1],'Fe','aqueous');assert metal==D('.001') and elem(state[0],'Fe')==0 and ion==D('.001') and elem(state[3],'Fe')==0;metrics={'collected_metallic_Fe_mol':metal,'uncollected_aqueous_Fe_mol':ion,'dissolved_arm_receiver_Fe_mol':elem(state[3],'Fe'),'final_canonical_contents':{k:v['contents'] for k,v in state.items()},'magnetic_registry':{'Fe':gold['Fe']['magnetic'],'Fe+3':gold['Fe+3']['magnetic']},'magnet_transfers':transfers,'actual_solved_iron_species':[s for s in state[1]['solution']['species'] if 'Fe' in s['name']]};notes=['Selectivity of explicit magnetic-solid operation on the actual metallic/dissolved inputs only. Ordinary bulk extraction is distinguished from nonzero ionic susceptibility. Slow metal corrosion and unoffered metastable iron-oxide/hydroxide formation remain unpredicted; no stable-phase precipitation or long-term corrosion claim.']
   elif suffix in ('mo-012','qa-007','qa-008'):
    spectra=[{'row':i,'vessel':r['operator']['vessel'],'value':e['value'],'instrument':e['instrument']} for i,r in enumerate(rows) for e in r['events'] if e['event']=='measured' and e['instrument']=='spectrophotometer'];assert spectra and all(e['value']==0 for e in spectra);assert 'S4O6' not in curated and 'S4O6' not in kinetics;metrics={'actual_spectra':spectra,'final_oxidised_I2_equivalents_mol':{k:oxid(v) for k,v in state.items()},'final_total_I_atoms_mol':{k:elem(v,'I') for k,v in state.items()},'final_contents':{k:v['contents'] for k,v in state.items()},'I3_canonical_mol':{k:amount(v,'I3-') for k,v in state.items()}}
    if suffix=='mo-012':
     assert amount(state[0],'I2')==D('.00001') and oxid(state[1])==0;assert amount(state[1],'dehydroascorbic_acid')==D('.00001') and amount(state[1],'HI')==D('.00002');assert abs(elem(state[0],'I')-D('.00102'))<TOL and abs(elem(state[1],'I')-D('.00102'))<TOL;criteria=[False,True];hold='iodine-speciation-optical-authority';notes=['VitaminC stoichiometric reduction and conserved iodine/carbon are real positive evidence. Both baseline and final UVvis are zero, and KI/I3 speciation is outside represented chemistry; the original colour/dilution contrast cannot be qualified by inventory alone.']
    elif suffix=='qa-007':
     stages=[];dose=Z
     for row,before,after in frames:
      if row['operator']['op']=='add' and row['operator'].get('species')=='Na2S2O3':
       dose+=row['operator']['moles'];v=after[0];stages.append({'cumulative_thiosulfate_dose_mol':dose,'oxidised_I2_equivalents_mol':oxid(v),'remaining_S2O3_mol':amount(v,'S2O3-2')})
     assert all(abs(x['oxidised_I2_equivalents_mol']-D('.0001'))<TOL and abs(x['remaining_S2O3_mol']-x['cumulative_thiosulfate_dose_mol'])<TOL for x in stages);criteria=[False,False];metrics['dose_stages']=stages;hold='iodine-thiosulfate-reaction-unrepresented';notes=['No iodine is consumed at any dose, including nominal0.2mmol endpoint. All0.22mmol thiosulfate remains, not merely a0.02mmol endpoint excess. Missing reaction/speciation is a coverage failure, not an optical endpoint.']
    else:
     at=next(after for row,_,after in frames if row['operator']['op']=='inspect');assert abs(oxid(at[0])-D('.0003'))<TOL and abs(oxid(at[1])-D('.0002'))<TOL;assert amount(at[1],'dehydroascorbic_acid')==D('.0001');criteria=[False,True];metrics['after_equal_0_4mmol_thiosulfate_I2_equivalents']={k:oxid(v) for k,v in at.items()};hold='iodine-thiosulfate-reaction-unrepresented';notes=['VitaminC consumes0.1mmol iodine with balancedcarbon/iodine, but subsequent thiosulfate does not assay either arm. Blank remains0.3mmol rather than0.1mmol; sample0.2mmol rather than nearzero. No inferred0.2mmol thiosulfate-demand difference is substituted for the failed actual reaction.']
   elif suffix=='ki-005':
    row,before,after=next(f for f in frames if f[0]['operator']['op']=='decant');start=after;end=state;portion=[]
    for vid,fraction in [(0,D('.75')),(1,D('.25'))]:
     for s in ['catalase','H2O2','water']:assert abs(amount(start[vid],s)-fraction*amount(before[0],s))<D('1e-14')
     loss=amount(start[vid],'H2O2')-amount(end[vid],'H2O2');portion.append({'vessel':vid,'postsplit_H2O2_mol':amount(start[vid],'H2O2'),'postsplit_catalase_tracer_mol':amount(start[vid],'catalase'),'postsplit_water_mol':amount(start[vid],'water'),'further_H2O2_loss_mol':loss,'fractional_loss':loss/amount(start[vid],'H2O2'),'actual_wait_seconds':end[vid]['elapsed_seconds']-start[vid]['elapsed_seconds']})
    assert all(p['fractional_loss']==0 and p['actual_wait_seconds']==20 for p in portion);criteria=[True,False];metrics={'before_split_residual_H2O2_mol':amount(before[0],'H2O2'),'initial_H2O2_mol':D('.001'),'postsplit_portions':portion,'enzyme_registry_formula_placeholder':gold['catalase']['formula'],'enzyme_registry_molar_mass_g_mol':gold['catalase']['molar_mass']};hold='precomparison-substrate-exhaustion-and-enzyme-formula-domain';notes=['Proportional enzyme/substrate transfer is proven, but99.999999% substrate was consumed before the split. Equalzero subsequent losses cannot establish matching nonzero fractional kinetics. Registry C is a surrogate amount tracer, not a240kDa protein formula; no invented full molecular atom closure is claimed. Other declared molecules close separately.']
   elif suffix in ('ki-009','ki-010'):
    waits=[]
    for row,before,after in frames:
     if row['operator']['op']!='wait':continue
     for vid,b in before.items():
      v=after[vid];loss=amount(b,'S2O3-2')-amount(v,'S2O3-2');s_gain=amount(v,'S')-amount(b,'S');so_gain=amount(v,'SO2')-amount(b,'SO2');h_spent=netacid(b)-netacid(v);water=amount(v,'water')-amount(b,'water');assert all(abs(x)<=D('1e-10') for x in [s_gain-loss,so_gain-loss,h_spent-2*loss,water-loss]);waits.append({'vessel':vid,'seconds':row['operator']['seconds'],'initial_temperature_K':b['temperature'],'final_temperature_K':v['temperature'],'initial_pH':b['solution']['ph'] if b.get('solution') else None,'final_pH':v['solution']['ph'] if v.get('solution') else None,'thiosulfate_loss_mol':loss,'all_phase_S_gain_mol':s_gain,'solid_S_change_mol':amount(v,'S','solid')-amount(b,'S','solid'),'SO2_gain_mol':so_gain,'acid_equivalents_consumed_mol':h_spent,'water_produced_mol':water,'initial_analytical_acid_mol':netacid(b),'final_analytical_acid_mol':netacid(v)})
    assert 'activation_energy: 51_000.0' in kinetics and 'pre_exponential: 3.0e8' in kinetics;metrics={'actual_wait_intervals':waits,'exact_rate_authority':'source kinetics.rs thiosulfate-acid: firstorder thiosulfate/proton activity; A3e8,Ea51000; initialactivitycoefficient locallyfrozen, analyticalacidconsumed, equilibrium resolvedbetweenintervals'};notes=['Bounded qualitative stop/restart or separation ownership demonstration. No continuously coupled equilibrium, buffered-acid reserve, isothermal/calibrated-rate or universal acid-kinetics claim. Sulfur is all-phase inventory; tiny aqueous dissolution is distinguished from new elemental production. SO2 remains a separately owned canonical gas unless an explicit boundary event exports it.']
    if suffix=='ki-009':
     assert len(waits)==3 and waits[0]['all_phase_S_gain_mol']>D('1e-5') and abs(waits[1]['all_phase_S_gain_mol'])<D('1e-12') and waits[2]['all_phase_S_gain_mol']>D('1e-4');neutral=[]
     for row,before,after in frames:
      op=row['operator']
      if op['op']=='add' and op.get('species') in ('NaOH','HCl') and op['vessel'] in before:
       b=before[0];v=after[0];dose=op['moles']*(D(-1) if op['species']=='NaOH' else D(1));change=netacid(v)-netacid(b);assert abs(change-dose)<D('1e-10');neutral.append({'species':op['species'],'added_signed_acid_mol':dose,'before_net_acid_mol':netacid(b),'after_net_acid_mol':netacid(v),'net_acid_change_mol':change,'neutralised_events':[e for e in row['events'] if e['event']=='neutralised']})
     metrics['independent_acid_base_steps']=neutral
    else:
     row,before,after=next(f for f in frames if f[0]['operator']['op']=='filter');assert amount(after[1],'S','solid')==0 and amount(after[1],'H+')>0 and amount(after[1],'S2O3-2')>0 and amount(state[1],'S','solid')>D('.0002');assert not material;criteria=[True,True];hold='historical-filter-gas-ownership';metrics['immediate_filtrate']={'acid_mol':amount(after[1],'H+'),'thiosulfate_mol':amount(after[1],'S2O3-2'),'old_S_solid_mol':amount(after[1],'S','solid'),'old_S_aqueous_mol':amount(after[1],'S','aqueous')};metrics['new_receiver_S_solid_mol']=amount(state[1],'S','solid');metrics['lost_unexported_donor_SO2_mol']=amount(before[0],'SO2','gas')-amount(after[0],'SO2','gas');notes+=['Both literal sulfur-contrast criteria have positive evidence, but the complete original SO2/S/O budget fails at historical filtration. Later gas-preserving source may repair it; this saved failure is not a newly reproduced current-source defect.']
   else:raise AssertionError(suffix)
   assert len(criteria)==len(case['original_acceptance_checks']);output.append({'id':case['id'],'proposed_candidate':all(criteria) and full and hold is None,'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],criteria,strict=True)],'metrics':metrics,'limitations':notes,'hold_or_gap':hold,'named_model_limits':case['named_model_limits'],'source':engine,'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(err),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt'],'material_budget_pass':full,'declared_nonprotein_molecular_budget_pass':material,'unresolved_molecular_formula':['catalase'] if suffix=='ki-005' else [],'material_maxima':maximum,'per_step_vessel_material_charge':ledger,'actual_clocks_temperatures_pH':clocks,'boundary_exports':exports,'initial_air':air,'transfers':transfers,'noncontents_reservoirs':{'checked_snapshots':reservoirs,'all_optional_owned_vectors_empty':vectors,'all_optional_owned_objects_none':objects,'surfaces_empty':True,'nuclides_empty':True,'foam_zero':True,'pending_CO2_zero':True,'resolved_mirror_empty':True}})
 return {'schema':'kerotakis.batch2.sixth-saved-review.v1','cases':output,'raw_archive_sha256':digest,'registry_sha256':sha(regraw),'thermal_registry_fixture_sha256':sha(goldraw),'source_blobs_sha256':{p:sha(b) for p,b in blobs.items()},'original_lock_qualification':binding['lock_qualification'],'new_engine_or_HTTP_requests':0,'catalog_selection_assigned':False,'material_tolerance_mol':TOL}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[3]);a=p.parse_args();print(json.dumps(review(a.repo),indent=2,default=str))
