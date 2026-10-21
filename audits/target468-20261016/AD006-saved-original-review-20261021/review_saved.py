"""Saved original AD006 review; no simulator, network or catalog writes."""
import argparse,collections,hashlib,json,math,os,pathlib,re,subprocess,zipfile
from decimal import Decimal as D,localcontext
SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83'
IDS=['ad-002','ad-003','ad-004','ad-006'];Z=D(0);TOL=D('1e-10')
def sha(raw):return hashlib.sha256(raw).hexdigest()
def digest(p):
 with p.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def plain(value):return json.loads(json.dumps(value,default=str))
def main():
 a=argparse.ArgumentParser();a.add_argument('--repo',type=pathlib.Path,required=True);a.add_argument('--out-dir',type=pathlib.Path,required=True);a=a.parse_args();assert not a.out_dir.exists()
 assert os.getloadavg()[0]<=3.5 and int(next(s.split()[1]for s in pathlib.Path('/proc/meminfo').read_text().splitlines()if s.startswith('MemAvailable:')))//1024>=3200
 B=a.repo/'audits/new-experiments-201-20261016';L=lambda p:json.loads(p.read_text(),parse_float=D);source=lambda p:subprocess.check_output(['git','show',SOURCE+':'+p],cwd=a.repo)
 source_paths=['data/registry/registry-source-v1.json','crates/kerotakis-core/tests/golden/registry.json','crates/kerotakis-core/src/adsorption.rs','crates/kerotakis-core/src/indicator.rs','crates/kerotakis-core/src/spectrum.rs','crates/kerotakis-core/src/solution_optics.rs','crates/kerotakis-core/src/instrument.rs','crates/kerotakis-core/src/relations.rs','crates/kerotakis-core/src/vessel.rs','crates/kerotakis-core/src/bench.rs']
 sources={p:source(p)for p in source_paths};reg=json.loads(sources[source_paths[0]],parse_float=D);gold={r['key']:r for r in json.loads(sources[source_paths[1]],parse_float=D)};comps={r['species_id']:r for r in reg['compositions']}
 archive=B/'native-screen-results/raw-artifact.zip';verification=L(B/'native-screen-results/verification.json');assert digest(archive)==verification['artifact_sha256'];binding=L(B/'native-screen-results/binding.json');assert binding['engine']['source_head']==SOURCE
 intake=L(B/'intake.json');cases={};raws={}
 with zipfile.ZipFile(archive)as zip:
  for k in IDS:
   oid='batch3-diverse:'+k;original=next(r for r in intake if r['original_id']==oid);prefix=oid.replace(':','_');summary=L(B/'native-screen-results/cases'/f'{prefix}.json');execution=json.loads(zip.read(prefix+'/execution.json'),parse_float=D);hashes={}
   for n,key in [('experiment.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
    raw=zip.read(prefix+'/'+n);assert sha(raw)==execution[key]==summary['execution'][key];hashes[n]=sha(raw)
   assert zip.read(prefix+'/experiment.lab')==(B/original['original_script']).read_bytes();assert hashes['experiment.lab']==original['script_sha256'];assert summary['engine']==binding['engine']and execution['exit_code']==0
   raws[k]=[json.loads(s,parse_float=D)for s in zip.read(prefix+'/stdout.jsonl').splitlines()];cases[k]={'original_id':oid,'original_forecast':original['original_expected_result'],'original_criteria':original['original_acceptance_checks'],'original_limits':original['original_limitations'],'script_sha256':original['script_sha256'],'raw_member_prefix':prefix,'raw_sha256':hashes,'summary_sha256':digest(B/'native-screen-results/cases'/f'{prefix}.json')}
 with localcontext()as ctx:
  ctx.prec=45;rows=[dict(r,raw_line=i+1)for i,r in enumerate(raws['ad-006'])if'bench'in r];expected=collections.defaultdict(lambda:collections.defaultdict(lambda:Z));previous={};trace=[];maxerr=Z
  def vector(key,n):
   c=comps[key];return {**{x['element']:n*D(x['count']['value'])for x in c['elements']},'charge':n*D(c['net_charge']['value'])}
  def add(dst,key,n):
   for el,amount in vector(key,n).items():dst[el]+=amount
  allowed={'vessel_created','added','not_yet_modeled','dissolved','solution_routed','solution_characterized','temperature_changed','adsorbed','filtered','measured','inert','observed'}
  for r in rows:
   op=r['operator'];vid=op.get('vessel')
   assert all(e['event']in allowed for e in r['events']),(r['raw_line'],r['events'])
   if op['op']=='add':add(expected[vid],op['species'],op['moles'])
   elif op['op']=='filter':
    before=previous[op['from']]
    for q in before['contents']:
     if q['phase']in ['liquid','aqueous']:add(expected[op['from']],q['species'],-q['moles']);add(expected[op['to']],q['species'],q['moles'])
   else:assert op['op']in ['new_vessel','measure','inspect']
   assert not r['bench'].get('spills')and not r['bench'].get('broken_vessels')
   snapshots=[]
   for v in r['bench']['vessels']:
    vi=v['id'];actual=collections.defaultdict(lambda:Z)
    assert v['headspace']=={'boundary':'open'}and v['elapsed_seconds']==0 and v['pending_co2_transfer_mol']==0 and not any(v['foam'].values())and v['nuclides']=={'inventory':{}}
    for field in ['surfaces','unresolved_materials','material_objects','surface_colours','charred_materials','electrodes','exchanges','solid_solutions','soap_scum','lemon_paper_mark','surface_particles','emulsion','step_start']:assert not v.get(field),(r['raw_line'],vi,field)
    for q in v['contents']:
     assert q['phase']in ['solid','liquid','aqueous']and q['moles']>=0;add(actual,q['species'],q['moles'])
    for q in v.get('adsorbed',[]):
     assert vi==1 and q['sorbent']=='activated_charcoal'and q['sorbate']=='methyl_orange'and q['moles']>=0;add(actual,q['sorbate'],q['moles'])
    residual={el:actual[el]-expected[vi][el]for el in set(actual)|set(expected[vi])};assert all(abs(n)<=TOL for n in residual.values()),(r['raw_line'],vi,residual);maxerr=max([maxerr,*map(abs,residual.values())])
    snapshots.append({'vessel':vi,'atoms_charge_residual':residual,'temperature_K':v['temperature'],'elapsed_s':v['elapsed_seconds'],'contents':v['contents'],'adsorbed':v.get('adsorbed',[]),'solution_is_derived_not_second_inventory':True});previous[vi]=v
   trace.append({'raw_line':r['raw_line'],'operator':op,'vessels':snapshots})
  final=rows[-1]['bench']['vessels'];solids,charcoal,receiver=final
  amount=lambda v,key,phase=None:sum((q['moles']for q in v['contents']if q['species']==key and(phase is None or q['phase']==phase)),Z)
  volume=lambda v:sum((q['moles']*gold[q['species']]['molar_mass']/gold[q['species']]['density']/1000 for q in v['contents']if q['phase']=='liquid'),Z)
  dye_dose=next(r['operator']['moles']for r in rows if r['operator'].get('species')=='methyl_orange');silica_dose=next(r['operator']['moles']for r in rows if r['operator'].get('species')=='SiO2');pet_dose=next(r['operator']['moles']for r in rows if r['operator'].get('species')=='PET')
  assert amount(solids,'PET','solid')==pet_dose and amount(solids,'SiO2','solid')>0 and not solids.get('adsorbed');assert all(q['phase']=='solid'for q in solids['contents'])
  assert amount(receiver,'SiO2','aqueous')>0 and abs(amount(receiver,'SiO2')+amount(solids,'SiO2')-silica_dose)<TOL
  dissolved_limit=gold['SiO2']['aqueous_solubility_g_per_100_ml']*volume(receiver)*10/gold['SiO2']['molar_mass'];assert abs(amount(receiver,'SiO2')-dissolved_limit)<D('1e-12')
  assert {q['species']for q in charcoal['contents']}=={'activated_charcoal'}and not receiver.get('adsorbed')and all(q['phase']!='solid'for q in receiver['contents'])
  held=charcoal['adsorbed'][0]['moles'];dye=amount(receiver,'methyl_orange');assert abs(held+dye-dye_dose)<TOL;carbon=amount(charcoal,'activated_charcoal')*gold['activated_charcoal']['molar_mass'];V=volume(receiver);totalmg=dye_dose*gold['methyl_orange']['molar_mass']*1000
  A=D('.05')*V;Bq=V+D('.05')*(D(200)*carbon-totalmg);C=2*totalmg/(Bq+(Bq*Bq+4*A*totalmg).sqrt());pred=(totalmg-C*V)/(gold['methyl_orange']['molar_mass']*1000);assert abs(pred-held)<D('1e-12')
  src=sources['crates/kerotakis-core/src/adsorption.rs'].decode();assert 'capacity_mg_per_g: 200.0'in src and'affinity_l_per_mg: 0.05'in src and'PENDING REVIEW'in src
  for key in ['Na+','Cl-']:assert abs(amount(receiver,key)-D('.003'))<TOL
  readings=[(r,e)for r in rows for e in r['events']if e['event']=='measured'];opt=[(r,e)for r,e in readings if e['instrument']=='spectrophotometer'];cond=[(r,e)for r,e in readings if e['instrument']=='conductivity_meter'];assert len(opt)==2 and len(cond)==1;assert opt[0][1]['vessel']==1 and opt[1][1]['vessel']==2 and cond[0][1]['vessel']==2 and cond[0][1]['value']>0 and cond[0][1]['unit']=='µS/cm'
  ind=sources['crates/kerotakis-core/src/indicator.rs'].decode();part=ind.split('key: "methyl_orange",',1)[1].split('    },',1)[0];assert re.search(r'pka:\s*3\.47\s*,',part)and'bands(&[(505.0, 24_000.0, 50.0)])'in part and'bands(&[(464.0, 26_000.0, 55.0)])'in part
  spec=sources['crates/kerotakis-core/src/spectrum.rs'].decode();bandtext=spec.split('pub const BAND_NM: [f64; BANDS] = [',1)[1].split('];',1)[0];assert [float(x)for x in re.findall(r'[0-9]+\.[0-9]+',bandtext)]==list(range(405,706,20))and'epsilon * (-0.5 * t * t).exp()'in spec
  ins=sources['crates/kerotakis-core/src/instrument.rs'].decode();assert'Self { path_cm: 1.0 }'in ins and'spectrum.iter().copied().fold(0.0f64, f64::max)'in ins and'(0.0..4.0).contains(&peak_abs)'in ins
  opticalsrc=sources['crates/kerotakis-core/src/solution_optics.rs'].decode();assert'spectrum_at_ph(key, s.ph)'in opticalsrc and'visible_moles(vessel, key, portion.moles.0) / litres'in opticalsrc
  assert'1.0 / (1.0 + 10f64.powf(pka - ph))'in sources['crates/kerotakis-core/src/relations.rs'].decode()
  predictions=[]
  for row,event in opt:
   v=next(v for v in row['bench']['vessels']if v['id']==event['vessel']);ph=float(v['solution']['ph']);base=1/(1+10**(3.47-ph));n=amount(v,'methyl_orange');vol=volume(v);bands=[]
   assert {q['species']for q in v['contents']}=={'water','SiO2','Na+','Cl-','methyl_orange'}
   assert {x['name']for x in v['solution']['species']}=={'H2O','Cl-','Na+','O2','H+','OH-'}
   for nm in range(405,706,20):bands.append(((1-base)*24000*math.exp(-.5*((nm-505)/50)**2)+base*26000*math.exp(-.5*((nm-464)/55)**2))*float(n/vol))
   predicted=max(bands);assert abs(predicted-float(event['value']))<1e-12 and event['unit']=='absorbance at 465 nm'and 405+20*bands.index(predicted)==465
   predictions.append({'raw_line':row['raw_line'],'actual_event':event,'dissolved_dye_mol':n,'actual_volume_L':vol,'actual_ph':v['solution']['ph'],'actual_temperature_K':v['temperature'],'path_cm':1,'independent16band_peak':predicted,'within_installed4AU_range':0<=predicted<4})
  assert opt[0][1]['value']>opt[1][1]['value']>0 and not predictions[0]['within_installed4AU_range']and predictions[1]['within_installed4AU_range']
  cases['ad-006'].update(status='BOUNDED_PROPOSAL_PENDING_PARENT_REVIEW',criterion_assessment={c['id']:'supported_with_limits'for c in cases['ad-006']['original_criteria']},metrics={'original_feed_SiO2_mol':silica_dose,'first_filter_retained_solid_SiO2_mol':amount(solids,'SiO2'),'final_aqueous_SiO2_mol':amount(receiver,'SiO2'),'PET_retained_mol':pet_dose,'dye_dose_mol':dye_dose,'charcoal_held_dye_mol':held,'final_dissolved_dye_mol':dye,'adsorbed_fraction':held/dye_dose,'allstep_atom_charge_max_residual_mol':maxerr,'allstep_vessel_ledger':trace,'optical_predictions':predictions,'final_conductivity_event':cond[0][1],'Na_Cl_final_mol':{k:amount(receiver,k)for k in ['Na+','Cl-']},'charcoal_g':carbon,'final_liquid_volume_L':V},exact_coverage_warnings=[{'raw_line':r['raw_line'],'operator':r['operator'],'event':e}for r in rows for e in r['events']if e['event']=='not_yet_modeled'],limits=['Before-treatment6.617AU exceeds installed4AU calibrated range; no calibrated optical ratio or quantitative decolourisation is claimed. Independent spectrum merely authenticates editorial model/readout identity.','InstalledLangmuir200mg/g,.05L/mg parameters pending provenance review; represented qualitative split only, no treatment-plant efficiency, competition/pH-dependent capacity/adsorption kinetics.','Dyeionic mapping and aqueousSiO2 nativechemistry are not represented by fullspeciation; canonical atoms/phase ownership is distinguished from complete mixturepH/conductivity/silicaequilibrium.','FilterPET solid retention is a modeled phase split, notpore-size/microplasticassay orpotabilitycertification. No fullthermal closure/calibration forecast.'])
 for k in ['ad-002','ad-003','ad-004']:
  rr=[r for r in raws[k]if'bench'in r];ms=[{'raw_line':i+1,'event':e}for i,r in enumerate(raws[k])for e in r.get('events',[])if e['event']=='measured'and e['instrument']=='spectrophotometer'];assert ms and all(x['event']['value']==0 and x['event']['unit']=='absorbance at 405 nm'for x in ms)
  cases[k].update(status='HOLD_NO_PROPOSAL',reason='All requested optical comparisons are405nmzero including controls/rescue/staging endpoints; no observed colourcontrast. Positive owned adsorption patterns remain useful source evidence, but no full ledger is certified here and no missingoptical comparison is relabeled scientificpass.',actual_optical_measurements=ms,actual_adsorption_events=[{'raw_line':i+1,'event':e}for i,r in enumerate(raws[k])for e in r.get('events',[])if e['event']=='adsorbed'])
 report={'source':SOURCE,'engine':binding['engine'],'lock_qualification':binding['lock_qualification'],'actual_binary_or_lock_bytes_rehashed':False,'archive_sha256':digest(archive),'source_blobs_sha256':{p:sha(b)for p,b in sources.items()},'cases':[cases[k]for k in IDS],'scientific_selection_performed':False,'product_engine_API_provider_build_calls':0}
 a.out_dir.mkdir(mode=0o700);(a.out_dir/'assessment.json').write_text(json.dumps(report,indent=2,ensure_ascii=False,default=str)+'\n');print(json.dumps({'proposal':['batch3-diverse:ad-006'],'holds':['ad-002','ad-003','ad-004'],'max_material':str(maxerr)}))
if __name__=='__main__':main()
