#!/usr/bin/env python3
"""Retrospective assessment of frozen redox streams. No simulator/network calls."""
import collections, hashlib, json, math, subprocess, zipfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
BASE=HERE.parent/'common-engine-completion'
ROOT=HERE.parents[2]
SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83'
ENGINE={'source_head':SOURCE,'source_tree':'f17216653825f3149bb7a0e46946834ee5c7e3df','binary_sha256':'670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f','cargo_lock_sha256':'82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229'}
F=96485.33212
# Literal formula coordinates of current owned portions and original inputs.
FORMULAS={
'water':{'H':2,'O':1},'H+':{'H':1},'base_equivalents':{'H':1,'O':1},
'Ag':{'Ag':1},'Ag+':{'Ag':1},'Zn':{'Zn':1},'Zn+2':{'Zn':1},'Cu':{'Cu':1},'Cu+2':{'Cu':1},'Mg':{'Mg':1},'Mg+2':{'Mg':1},'Fe+3':{'Fe':1},'Fe+2':{'Fe':1},'Mn+2':{'Mn':1},'MnO2':{'Mn':1,'O':2},
'K+':{'K':1},'Na+':{'Na':1},'Cl-':{'Cl':1},'NO3-':{'N':1,'O':3},'SO4-2':{'S':1,'O':4},'S2O3-2':{'S':2,'O':3},
'CO2':{'C':1,'O':2},'CO2(aq)':{'C':1,'O':2},'HCO3-':{'H':1,'C':1,'O':3},'CO3-2':{'C':1,'O':3},'H2':{'H':2},'N2':{'N':2},'O2':{'O':2},
'KI':{'K':1,'I':1},'I2':{'I':2},'HI':{'H':1,'I':1},'I3-':{'I':3},
'ascorbic_acid':{'C':6,'H':8,'O':6},'dehydroascorbic_acid':{'C':6,'H':6,'O':6},'H2C2O4':{'H':2,'C':2,'O':4},'ethanol':{'C':2,'H':6,'O':1},'CH3COOH':{'C':2,'H':4,'O':2},'CH3COO-':{'C':2,'H':3,'O':2},
'antlerite':{'Cu':3,'S':1,'O':8,'H':4},
'AgNO3':{'Ag':1,'N':1,'O':3},'ZnSO4':{'Zn':1,'S':1,'O':4},'CuSO4':{'Cu':1,'S':1,'O':4},'Na2SO4':{'Na':2,'S':1,'O':4},'Na2S2O3':{'Na':2,'S':2,'O':3},'HCl':{'H':1,'Cl':1},'FeCl3':{'Fe':1,'Cl':3},'H2SO4':{'H':2,'S':1,'O':4},'KMnO4':{'K':1,'Mn':1,'O':4}}
CHARGES={'Ag+':1,'Zn+2':2,'Cu+2':2,'Mg+2':2,'Fe+3':3,'Fe+2':2,'Mn+2':2,'H+':1,'base_equivalents':-1,'K+':1,'Na+':1,'Cl-':-1,'NO3-':-1,'SO4-2':-2,'S2O3-2':-2,'HCO3-':-1,'CO3-2':-2,'CH3COO-':-1,'I3-':-1}
KEYS=['H','O','C','N','S','K','Na','Cl','I','Ag','Zn','Cu','Mg','Mn','Fe','charge']
def sha(b):return hashlib.sha256(b).hexdigest()
def strict_obj(pairs):
 out={}
 for k,v in pairs:
  if k in out:raise ValueError('Duplicate JSON key '+k)
  out[k]=v
 return out
def bad_constant(x):raise ValueError('Non-finite JSON constant '+x)
def loads(b):return json.loads(b,object_pairs_hook=strict_obj,parse_constant=bad_constant)
def zero():return dict.fromkeys(KEYS,0.0)
def book(out,species,n):
 assert math.isfinite(n)
 for k,c in FORMULAS[species].items():out[k]+=c*n
 out['charge']+=CHARGES.get(species,0)*n

def states(rows):
 current={}; result=[]
 for row in rows:
  published={v['id']:v for v in row['bench']['vessels']};assert len(published)==len(row['bench']['vessels'])
  current.update(published);result.append(current.copy())
 return result

def validate_partial(case,rows):
 commands=[line.strip() for line in case['script'].splitlines() if line.strip() and not line.lstrip().startswith('#') and not line.strip().startswith('register ')]
 assert len(commands)==len(rows)
 current={}
 for command,row in zip(commands,rows):
  published={v['id']:v for v in row['bench']['vessels']};assert len(published)==len(row['bench']['vessels'])
  if current.keys()-published.keys():
   words=command.split();assert len(words)==2 and words[0]=='inspect'
   target=int(words[1][1:])-1
   assert set(published)=={target} and row['operator']['op']=='inspect' and not row['events'] and published[target]==current[target]
  current.update(published)

def inventory(vessels):
 out=zero()
 for v in vessels:
  assert not v['surfaces'] and not v.get('unresolved_materials')
  for p in v['contents']:
   assert math.isfinite(p['moles']) and p['moles']>=0
   book(out,p['species'],p['moles'])
 return out

def ledger(rows):
 incoming=zero();outgoing=zero();expected={};air=[];exports=[];ss=states(rows)
 def own(v,s,n):
  expected.setdefault(v,zero());book(expected[v],s,n)
 for i,row in enumerate(rows):
  op=row['operator']
  if op['op']=='add':book(incoming,op['species'],op['moles']);own(op['vessel'],op['species'],op['moles'])
  if op['op']=='filter':
   for p in ss[i-1][op['from']]['contents']:
    if p['phase'] in ['aqueous','liquid']:own(op['from'],p['species'],-p['moles']);own(op['to'],p['species'],p['moles'])
  for e in row['events']:
   if e['event']=='vessel_sealed':
    air.append({'row':i,**e})
    for s,x in [('N2',.7901),('O2',.2095),('CO2',.0004)]:book(incoming,s,e['trapped_air']*x);own(e['vessel'],s,e['trapped_air']*x)
   if e['event']=='gas_evolved':book(outgoing,e['species'],e['moles']);own(e['vessel'],e['species'],-e['moles']);exports.append({'row':i,**e})
   assert e['event'] not in ['gas_absorbed','diluted','gas_vented'], 'Unimplemented boundary in this fixed review'
 retained=inventory(ss[-1].values());residual={k:retained[k]+outgoing[k]-incoming[k] for k in KEYS}
 per=[]
 for vid,v in ss[-1].items():
  actual=inventory([v]);exp=expected.get(vid,zero());delta={k:actual[k]-exp[k] for k in KEYS}
  per.append({'vessel':vid,'retained':actual,'expected_after_inputs_boundaries_and_internal_transfers':exp,'residual':delta})
 maximum=max([abs(x) for x in residual.values()]+[abs(x) for v in per for x in v['residual'].values()])
 return {'scope':'Final whole-script and per-vessel represented material/charge, with explicit input amounts, trapped air and gas exports. Filter transfers only the previously owned liquid/aqueous portions. No solver species, free H/OH diagnostics, historical lots or solute_charge double counting; no energy/per-step closure claim.','incoming':incoming,'retained':retained,'exported':outgoing,'residual':residual,'per_vessel':per,'initial_air':air,'export_events':exports,'max_abs_residual_mol_atoms_or_charge':maximum,'existing_inventory_audit_resolution_mol':1e-8,'within_existing_inventory_audit_resolution':maximum<=1e-8}

def amount(v,keys,phase=None):
 if isinstance(keys,str):keys={keys}
 return sum(x['moles'] for x in v['contents'] if x['species'] in keys and (phase is None or x['phase']==phase))
def spec(v,name):return next((x for x in (v.get('solution')or{}).get('species',[]) if x['name']==name),None)
def authenticate():
 data=(BASE/'originals/new-experiments-20261016.json').read_bytes();assert sha(data)=='8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd'
 archive=(BASE/'results/raw-artifact.zip').read_bytes();verified=loads((BASE/'results/verified-results.json').read_bytes());metadata=loads((BASE/'results/artifact-metadata.json').read_bytes())
 assert sha(archive)==verified['raw_artifact_sha256']==metadata['digest'].removeprefix('sha256:')=='9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f'
 assert verified['engine']==ENGINE
 original=loads(data);original_cases={x['id']:x for x in original['experiments']};receipts={x['id']:x for x in verified['cases']};cases=[]
 with zipfile.ZipFile(BASE/'results/raw-artifact.zip') as z:
  assert len(z.namelist())==len(set(z.namelist()))
  binding=loads(z.read('binding.json'));assert binding==verified['binding'] and binding['engine']==ENGINE
  design=(BASE/'native-screen-design.json').read_bytes();assert sha(design)==binding['design_sha256'];assert loads(design)==loads(z.read('design.json'))
  design_check={'original_design_sha256':sha(design),'archived_design_sha256':sha(z.read('design.json')),'complete_parsed_value_equality':True,'qualification':'Archived design is ASCII-escaped reserialization, not original raw bytes.'}
  for n in range(1,15):
   cid=f'new120:rx-{n:03}';c=original_cases[cid];r=receipts[cid];prefix=cid.replace(':','_');eraw=z.read(prefix+'/execution.json');actual=loads(eraw);hashes={}
   for name,key in [('experiment.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
    raw=z.read(prefix+'/'+name);assert sha(raw)==r[key]==actual[key];hashes[name]=sha(raw)
    if name=='experiment.lab':assert raw==c['script'].encode() and sha(raw)==c['script_sha256']
   assert all(actual[k]==r[k] for k in actual) and r['engine']==ENGINE
   rows=[loads(l) for l in z.read(prefix+'/stdout.jsonl').splitlines()];assert len(rows)==r['parsed_observations'] and actual['stdout_parse_error'] is None and actual['exit_code']==0
   validate_partial(c,rows);cases.append((c,rows,{'archive_prefix':prefix,'execution_receipt_sha256':sha(eraw),'member_sha256':hashes,'receipt':actual}))
 return cases,binding,design_check

def source_binding():
 paths=['crates/kerotakis-core/src/displacement.rs','crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/curated.rs','crates/kerotakis-core/src/relations.rs','crates/kerotakis-cli/src/main.rs']
 result=[]
 for p in paths:
  raw=subprocess.check_output(['git','show',SOURCE+':'+p],cwd=ROOT)
  result.append({'repository_path':p,'git_blob':subprocess.check_output(['git','rev-parse',SOURCE+':'+p],cwd=ROOT,text=True).strip(),'sha256':sha(raw)})
 return result

def assess(c,rows,authentication):
 n=int(c['id'][-3:]);ss=states(rows);final=ss[-1];ev=[{'row':i,**e} for i,r in enumerate(rows) for e in r['events']];limits=[e for e in ev if e['event'] in ['not_yet_modeled','solver_failed']];cells=[e for e in ev if e['event']=='cell_voltage'];checks=[];obs={};qual=[]
 def check(status,evidence):checks.append({'status':status,'observations':evidence})
 if n==1:
  silver=sum(amount(v,'Ag','solid') for v in final.values());zinc=sum(amount(v,'Zn+2') for v in final.values());obs.update(metallic_silver_mol=silver,oxidized_zinc_mol=zinc,electrons_given_mol=2*zinc,electrons_taken_mol=silver)
  check('met' if silver>0 else 'failed','Metallic Ag solid, not a silver salt: '+repr(silver)+' mol, retained in source after filtering.')
  check('met' if silver==.002 and zinc==.001 else 'failed',f'Zn oxidation {zinc} mol gives {2*zinc} mol electrons; Ag deposition {silver} mol takes the same amount, at the 0.002 mol ceiling.')
 elif n==2:
  v=final[0];obs.update(copper_solid_mol=amount(v,'Cu','solid'),cupric_mol=amount(v,'Cu+2'),ferric_mol=amount(v,'Fe+3'),ferrous_mol=amount(v,'Fe+2'))
  check('failed','No coupled reaction in the saved result: '+repr(obs))
  check('inconclusive','The input ferric budget is 0.002 mol electrons and hence 0.001 mol Cu ceiling, but zero observed Cu oxidation supplies no positive transfer evidence. Do not count staying below a ceiling as a chemistry pass.')
  qual.append('The metallic activity-series route carries metal/ion couples, not the requested FeIII/FeII acceptor pathway. Iron phase exclusions and incomplete absorbance are also explicitly named. Underlying ferric-mediated metal oxidation needs a separate model lane; this assessment does not execute or credit later code.')
 elif n==3:
  obs.update(magnesium_H2_mol=amount(final[0],'H2'),copper_H2_mol=amount(final[1],'H2'),magnesium_aqueous_mol=amount(final[0],'Mg+2'),magnesium_acid_remaining_mol=amount(final[0],'H+'),copper_solid_mol=amount(final[1],'Cu','solid'))
  check('met' if obs['magnesium_H2_mol']>0 and obs['magnesium_aqueous_mol']>0 else 'failed',repr(obs))
  check('met' if obs['copper_H2_mol']==0 and obs['copper_solid_mol']==.001 else 'failed','Cu retains its 0.001 mol solid and has no H2; its own sealed-air baseline is retained. Oxygen-driven copper corrosion is outside the stated displacement route, not disproved.')
 elif n==4:
  v=final[0];obs.update(ferric_mol=amount(v,'Fe+3'),ferrous_mol=amount(v,'Fe+2'),iodine_I2_mol=amount(v,'I2'),iodine_I3_mol=amount(v,'I3-'),unmapped_KI_mol=amount(v,'KI'))
  check('unsupported','No iodine product or ferrous inventory appears: '+repr(obs))
  check('unsupported','KI has no aqueous component mapping; no I2/I3 partition is resolved and ferric spectra are explicitly incomplete. Colour cannot establish this electron transfer.')
 elif n==5:
  stages=[ss[i][0] for i in [2,6,9]];obs['stages']=[{'row':i,'I2_mol':amount(v,'I2'),'I3_mol':amount(v,'I3-'),'thiosulfate_mol':amount(v,'S2O3-2'),'tetrathionate_mol':amount(v,{'S4O6-2','Na2S4O6'})} for i,v in zip([2,6,9],stages)]
  check('met','The represented oxidised pool is explicitly tracked: all three stages retain 0.0001 mol I2 and no I3 representation appears. This literal tracking criterion does not establish reduction or full physical speciation; KI mapping is unavailable: '+repr(obs['stages']))
  check('failed','Final thiosulfate is 0.0002 mol (all added), iodine 0.0001 mol (all original), and tetrathionate absent. No 2:1 consumption is demonstrated. The named thiosulfate-acid rate refusal is a separate competing pathway and is not an iodometric reaction implementation.')
  qual.append('Both initial and final UV/visible readings are 0.0 at 405 nm despite retained I2. No chemical inference may be made from this wavelength/coverage result; iodine/triiodide spectra and input mapping need review.')
 elif n in [6,7]:
  v=final[0];obs.update(final_ascorbic_acid_mol=amount(v,'ascorbic_acid'),final_oxidized_vitaminC_mol=amount(v,'dehydroascorbic_acid'),final_HI_mol=amount(v,'HI'),final_I2_mol=amount(v,'I2'),final_I3_mol=amount(v,'I3-'))
  if n==6:
   check('met' if obs['final_oxidized_vitaminC_mol']==.0001 and obs['final_I2_mol']==0 else 'failed','Curated named-product inventory: '+repr(obs))
   check('unsupported','Represented molecular conversion is 0.0001 mol vitamin C to 0.0001 mol oxidized product with 0.0002 mol HI, but KI is unmapped and iodine complexation is not supplied. A complete iodine-complex budget requested by the original criterion is not established.')
  else:
   obs['early_stages']=[{'row':i,'ascorbic_acid_mol':amount(ss[i][0],'ascorbic_acid'),'I2_mol':amount(ss[i][0],'I2'),'product_mol':amount(ss[i][0],'dehydroascorbic_acid')} for i in [4,6,8]]
   check('met' if amount(ss[4][0],'ascorbic_acid')==.00005 and amount(ss[4][0],'I2')==0 and amount(ss[6][0],'ascorbic_acid')==0 else 'failed',repr(obs['early_stages']))
   check('met' if obs['final_I2_mol']>0 and obs['final_ascorbic_acid_mol']==0 else 'failed','Final I2 excess is 0.00005 mol without regenerated vitamin C: '+repr(obs))
  qual.append('Hold working selection: positive literal curated stoichiometry is narrower than the unmodelled KI ion/iodine-complex mixture. HI is an undissociated product carrier, not evidence of a resolved iodide activity; no spectral or rate calibration is supplied. RX007 also begins with an explicit ascorbic dissolution limitation. Full mixture claims remain unsupported even when literal redox portions transform.')
 elif n==8:
  co2=sum(e['moles'] for e in ev if e['event']=='gas_evolved' and e['species']=='CO2');v=final[0];obs.update(exported_CO2_mol=co2,manganeseII_mol=amount(v,'Mn+2'),oxalic_acid_remaining_mol=amount(v,'H2C2O4'),final_ph=v['solution']['ph'],final_T_K=v['temperature'],nominal_oxalic_to_permanganate=.0005/.0002)
  check('met' if co2==.001 and obs['manganeseII_mol']==.0002 else 'failed',repr(obs))
  check('met' if obs['oxalic_acid_remaining_mol']==0 and obs['nominal_oxalic_to_permanganate']==2.5 else 'failed','0.0005 mol oxalic acid and 0.0002 mol permanganate consumed, producing 0.001 mol CO2 and 0.0002 mol MnII: 5:2 with ten electrons per 5 oxalic acids.')
  qual.append('Only this warm acidic protocol is proposed. At c446 the curated oxalate row explicitly has no acidity or temperature gate; it cannot validate failure outside those conditions or actual autocatalytic kinetics. UV/visible absorbance is explicitly unavailable for MnII species. No heat/rate calibration or later repair credit.')
 elif n==9:
  v=final[0];pool=amount(v,{'CH3COOH','CH3COO-'});obs.update(manganese_dioxide_mol=amount(v,'MnO2','solid'),acid_mol=amount(v,'CH3COOH'),acetate_mol=amount(v,'CH3COO-'),organic_product_family_mol=pool,final_ph=v['solution']['ph'],base_equivalents_mol=amount(v,'base_equivalents'))
  check('met' if obs['manganese_dioxide_mol']>0 and amount(v,{'KMnO4','MnO4-'})==0 else 'failed','MnO2 solid 0.0004 mol; permanganate input is exhausted: '+repr(obs))
  check('met' if abs(pool-.0003)<1e-15 else 'failed','Final acid/base partition is measured in the canonical pool; total Mn is 0.0004 mol and product-family 0.0003 mol. Gross 4:3 reaction is followed by equilibration, giving predominantly acetate at pH '+str(obs['final_ph']))
  qual.append('Curated stoichiometry only: not a measured product selectivity, rate law, general acidic/neutral routing comparison or heat calibration. Registered coefficients/properties have not been independently scientifically calibrated by these saved observations.')
 elif n in [10,11,12]:
  obs['cell_events']=cells;obs['cell_calls_preserve_contents']=all(ss[e['row']]==ss[e['row']-1] for e in cells)
  obs['reported_free_ion_activities']=[{'row':e['row'],'vessels':[{'id':v['id'],'temperature_K':v['temperature'],'solvent_kg':(v.get('solution')or{}).get('solvent_kg'),'ions':{k:spec(v,k) for k in ['Zn+2','Cu+2','Ag+'] if spec(v,k)}} for v in ss[e['row']].values()]} for e in cells]
  if n==10:
   check('met' if cells[1]['volts']<cells[0]['volts'] and cells[0]['anode']==cells[1]['anode']==0 else 'failed','Same physical anode/cathode and order: voltage '+str([e['volts'] for e in cells])+' V; zinc activity rises 0.003758 -> 0.01604 while copper activity remains 0.003879. Actual zinc temperature changes, so no isothermal textbook slope is imposed.')
  elif n==11:
   check('met' if cells[0]['anode']==0 and cells[0]['cathode']==1 and cells[0]['volts']>0 else 'failed','Runtime identifies v1 Cu as anode and v2 Ag as positive cathode; voltage '+str(cells[0]['volts'])+' V. Positive value is cathode minus anode, not a signed input-lead convention.')
  else:
   check('failed','Both reversed calls report +0.04003851213353726 V, anode v1 and cathode v2. Original opposite-sign expectation is not met. Source cell() explicitly sorts electrode potentials and reports a positive spontaneous EMF magnitude; not an established engine sign bug.')
   obs['rounded_activity_nernst_volts']=8.31446261815324/(2*F)*(ss[7][1]['temperature']*math.log(spec(ss[7][1],'Zn+2')['activity'])-ss[7][0]['temperature']*math.log(spec(ss[7][0],'Zn+2')['activity']))
   check('met','Nernst comparison uses reported Zn activities 0.0006922 and 0.01604 at actual temperatures 298.1692633 and 300.0761572 K, not nominal 100-fold ZnSO4 molarity. Activities have report rounding; no exact equality or new numerical pass band is asserted. Source uses resolved free-ion activity in electrode().')
  if n in [10,11]:check('met' if obs['cell_calls_preserve_contents'] else 'failed','Every cell call leaves all published vessel state unchanged relative to its preceding snapshot. No discharge or metal consumption is assigned to voltage measurement.')
  qual.append('Ideal open-circuit Nernst calculation, ideal salt bridge and no junction potential; E0 is used at vessel temperature without dE0/dT. Zinc is below the hydrogen line and mixed-potential/passivation rates are outside this model. No empirical electrode calibration; full paired snapshots and named limits are retained.')
 elif n==13:
  before=ss[3][0];after=ss[5][0];dh=amount(after,'H2')-amount(before,'H2');do=amount(after,'O2')-amount(before,'O2');obs.update(H2_baseline_mol=amount(before,'H2'),O2_baseline_mol=amount(before,'O2'),H2_increment_mol=dh,O2_increment_mol=do,gas_ratio=dh/do,ideal_H2_mol=10/(2*F),ideal_O2_mol=10/(4*F),H2_residual_mol=dh-10/(2*F),O2_residual_mol=do-10/(4*F))
  check('met','Canonical owned dissolved-plus-gas portions after subtracting the actual sealed baseline: '+repr(obs)+'. Neither snapshot has owned aqueous H2 or O2; both products are in retained headspace. Diagnostic solver gas species are not additional material.')
  check('met' if abs(obs['H2_residual_mol'])<1e-15 and abs(obs['O2_residual_mol'])<1e-15 else 'failed','Q=10 C and F=96485.33212 C/mol; identified H2/O2 production agrees with Faraday amounts and 2:1 ratio within the shown floating-point residuals. Inert-electrode spectator-sulfate route is admitted, not merely an unsupported request.')
  qual.append('Ideal imposed-current undivided water splitting. No voltage/current-geometry prediction, overpotentials, empirical gas collection efficiency, kinetic gas-transfer or measured Faradaic efficiency. Floating residual comparisons are extraction arithmetic, not a new physical tolerance.')
 elif n==14:
  before=ss[2][0];after=final[0];cu=amount(after,'Cu','solid');dc=amount(before,'Cu+2')-amount(after,'Cu+2');ant=amount(before,'antlerite','solid');oxygen=sum(e['moles'] for e in ev if e['event']=='gas_evolved' and e['species']=='O2');obs.update(deposited_Cu_mol=cu,aqueous_Cu_decrease_mol=dc,initial_antlerite_formula_mol=ant,initial_antlerite_Cu_mol=3*ant,exported_O2_mol=oxygen,H2_generated_mol=sum(e.get('moles',0) for e in ev if e['event'] in ['gas_evolved','gas_contained'] and e.get('species')=='H2'),ideal_Cu_mol=10/(2*F),final_H_plus_mol=amount(after,'H+'),final_ph=after['solution']['ph'])
  check('met' if cu>0 and abs(cu-10/(2*F))<1e-15 else 'failed','Explicit metallic Cu deposit and full phase budget: '+repr(obs)+'. Aqueous-only decrease is smaller than deposit because antlerite dissolves; include three Cu per antlerite formula.')
  check('met','Source solvent-electrolysis route selects Cu plating by a positive standard-potential heuristic with sufficient ion supply; inert anode oxidises water to O2 and produces net acidity. No metal electrode was initially present. Supply-exhausted residual charge would generate H2, but none is generated at this small charge. O2 export is 2.5910674141544323e-5 mol, and the represented H/O/charge ledger is checked separately.')
  qual.append('Ideal imposed-current model with exhaustion-based charge split, not simultaneous kinetic H2/Cu competition or measured efficiency. A soluble copper anode is not assumed. Standard-potential heuristic, electrode geometry/overpotentials and quantitative gas collection remain unvalidated.')
 assert len(checks)==len(c['acceptance_checks'])==2
 for original,check_result in zip(c['acceptance_checks'],checks):check_result['original']=original
 mat=ledger(rows)
 held={2,4,5,6,7,12}
 proposed=n not in held and all(x['status']=='met' for x in checks) and mat['within_existing_inventory_audit_resolution']
 snapshots=[{'row':i,'step':r['step'],'operator':r['operator'],'vessels':[{'id':v['id'],'contents':v['contents'],'headspace':v['headspace'],'temperature_K':v['temperature'],'ph':(v.get('solution')or{}).get('ph'),'solvent_kg':(v.get('solution')or{}).get('solvent_kg'),'species_distribution':(v.get('solution')or{}).get('species',[]),'solution_provenance':(v.get('solution')or{}).get('provenance')} for v in r['bench']['vessels']]} for i,r in enumerate(rows)]
 return {'id':c['id'],'rank':c['rank'],'title':c['title'],'original_expected_result':c['expected_result'],'original_expected_processes':c['expected_processes'],'script':c['script'],'script_sha256':c['script_sha256'],'authentication':authentication,'criteria':checks,'quantitative_observations':obs,'ledger':mat,'qualifications':qual,'named_limits':limits,'events':[e for e in ev if e['event']!='solution_routed'],'saved_snapshots':snapshots,'proposed_selection':'proposed_bounded_selection' if proposed else 'hold_for_named_followup','catalog_case_added':False}

def main():
 inputs,binding,design_check=authenticate();cases=[assess(*x) for x in inputs]
 result={'schema':1,'scope':'Saved original first120 redox review: fourteen protocols, twenty-eight unchanged criteria. Proposed selections only; no catalog mutation, engine/API execution, forecast rewriting, empirical calibration or full heat certification.','original_dataset_sha256':'8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd','raw_artifact_sha256':'9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f','binding':binding,'engine':ENGINE,'design_authentication':design_check,'source_files':source_binding(),'binary_lock_qualification':'Producer-bound source/tree/binary/lock and library manifest; raw executable and Cargo.lock bytes absent from this case-output archive. No reconstructed binary-byte verification is claimed.','assessment_policy':'Criteria retain literal wording. Honesty guards and absence of response/overshoot alone are not positive chemistry evidence. Original approximate words have no new numerical pass band. Current contents are analytical owners; solution distributions and historical lots are not extra matter. E0 coefficients come from the source-declared CRC commercial compilation, enthalpies from source-declared NBS tables; neither is independently rederived or empirically calibrated by this review. No withdrawn-source organic solubility values determine these aqueous protocols. v1/v2 correspond to JSON ids0/1; row indices are zero-based.','criterion_counts':dict(collections.Counter(x['status'] for c in cases for x in c['criteria'])),'proposed_bounded_selections':[c['id'] for c in cases if c['proposed_selection']=='proposed_bounded_selection'],'held_cases':[c['id'] for c in cases if c['proposed_selection']!='proposed_bounded_selection'],'new_engine_executions':0,'new_api_requests':0,'catalog_additions':0,'cases':cases}
 print(json.dumps(result,indent=2,ensure_ascii=False,allow_nan=False))
if __name__=='__main__':main()
