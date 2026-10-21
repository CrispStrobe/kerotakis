"""Saved original AD005 review; no simulator, network or catalog writes."""
import argparse,collections,hashlib,json,math,os,pathlib,re,subprocess,zipfile
from decimal import Decimal as D,localcontext
SOURCE='c44692053a94c59dcf62e7fb67549f84c7d18f83'
IDS=['ad-001','ad-005','en-001','en-002'];Z=D(0);TOL=D('1e-10')
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
  ctx.prec=45;rows=[dict(r,raw_line=i+1)for i,r in enumerate(raws['ad-005'])if'bench'in r];expected=collections.defaultdict(lambda:collections.defaultdict(lambda:Z));previous={};trace=[];maxerr=Z
  def vector(key,n):
   c=comps[key];return {**{x['element']:n*D(x['count']['value'])for x in c['elements']},'charge':n*D(c['net_charge']['value'])}
  def add(dst,key,n):
   for el,amount in vector(key,n).items():dst[el]+=amount
  allowed={'vessel_created','added','not_yet_modeled','dissolved','solution_routed','solution_characterized','temperature_changed','adsorbed','filtered','measured'}
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
     assert vi==0 and q['sorbent']=='activated_charcoal'and q['sorbate']=='methyl_orange'and q['moles']>=0;add(actual,q['sorbate'],q['moles'])
    residual={el:actual[el]-expected[vi][el]for el in set(actual)|set(expected[vi])};assert all(abs(n)<=TOL for n in residual.values()),(r['raw_line'],vi,residual);maxerr=max([maxerr,*map(abs,residual.values())])
    snapshots.append({'vessel':vi,'atoms_charge_residual':residual,'temperature_K':v['temperature'],'elapsed_s':v['elapsed_seconds'],'contents':v['contents'],'adsorbed':v.get('adsorbed',[]),'solution_is_derived_not_second_inventory':True});previous[vi]=v
   trace.append({'raw_line':r['raw_line'],'operator':op,'vessels':snapshots})
  final=rows[-1]['bench']['vessels'];donor,control,receiver=final;dose=next(r['operator']['moles']for r in rows if r['operator'].get('species')=='methyl_orange');held=donor['adsorbed'][0]['moles'];dye=next(q['moles']for q in receiver['contents']if q['species']=='methyl_orange');assert abs(held+dye-dose)<=TOL
  assert donor['contents']==[q for q in donor['contents']if q['species']=='activated_charcoal'and q['phase']=='solid'];assert not receiver.get('adsorbed')and all(q['phase']!='solid'for q in receiver['contents'])
  volume=lambda v:sum((q['moles']*gold[q['species']]['molar_mass']/gold[q['species']]['density']/1000 for q in v['contents']if q['phase']=='liquid'),Z)
  V=volume(receiver);adsrow=next(r for r in rows if r['operator'].get('species')=='activated_charcoal');carbon=adsrow['operator']['moles']*gold['activated_charcoal']['molar_mass'];totalmg=dose*gold['methyl_orange']['molar_mass']*1000
  # Independent analytic quadratic, not the implementation's bisection.
  A=D('.05')*V;Bq=V+D('.05')*(D(200)*carbon-totalmg);C=2*totalmg/(Bq+(Bq*Bq+4*A*totalmg).sqrt());pred=(totalmg-C*V)/(gold['methyl_orange']['molar_mass']*1000);assert abs(pred-held)<D('1e-12')
  src=sources['crates/kerotakis-core/src/adsorption.rs'].decode();assert 'capacity_mg_per_g: 200.0' in src and 'affinity_l_per_mg: 0.05'in src and 'PENDING REVIEW'in src
  readings=[(r,e)for r in rows for e in r['events']if e['event']=='measured'];cond=[(r,e)for r,e in readings if e['instrument']=='conductivity_meter'];opt=[(r,e)for r,e in readings if e['instrument']=='spectrophotometer'];assert len(cond)==2 and cond[0][1]['vessel']==1 and cond[1][1]['vessel']==2 and cond[0][1]['value']==cond[1][1]['value']>0 and all(e['unit']=='µS/cm'for _,e in cond)
  for key in ['Na+','Cl-']:
   vals=[sum((q['moles']for q in v['contents']if q['species']==key),Z)for v in [control,receiver]];assert all(abs(n-D('.005'))<TOL for n in vals)
  assert volume(control)==volume(receiver)and control['solution']==receiver['solution']
  assert len(opt)==1;orow,oe=opt[0];assert oe['vessel']==2 and oe['unit']=='absorbance at 465 nm'and oe['value']>0;ph=float(receiver['solution']['ph']);base=1/(1+10**(3.47-ph));bands=[]
  for nm in range(405,706,20):
   eps=(1-base)*24000*math.exp(-.5*((nm-505)/50)**2)+base*26000*math.exp(-.5*((nm-464)/55)**2);bands.append(eps*float(dye/V))
  # Independently bind every optical constant/convention to actual historical source.
  ind=sources['crates/kerotakis-core/src/indicator.rs'].decode();part=ind.split('key: "methyl_orange",',1)[1].split('    },',1)[0]
  assert re.search(r'pka:\s*3\.47\s*,',part)and 'bands(&[(505.0, 24_000.0, 50.0)])'in part and 'bands(&[(464.0, 26_000.0, 55.0)])'in part
  spec=sources['crates/kerotakis-core/src/spectrum.rs'].decode();bandtext=spec.split('pub const BAND_NM: [f64; BANDS] = [',1)[1].split('];',1)[0];assert [float(x)for x in re.findall(r'[0-9]+\.[0-9]+',bandtext)]==list(range(405,706,20));assert 'epsilon * (-0.5 * t * t).exp()'in spec
  ins=sources['crates/kerotakis-core/src/instrument.rs'].decode();assert 'Self { path_cm: 1.0 }'in ins and 'spectrum.iter().copied().fold(0.0f64, f64::max)'in ins
  volsrc=sources['crates/kerotakis-core/src/vessel.rs'].decode();assert '.filter(|p| p.phase == Phase::Liquid)'in volsrc and 'd.liters_from_moles(p.moles).0'in volsrc
  opticalsrc=sources['crates/kerotakis-core/src/solution_optics.rs'].decode();assert 'spectrum_at_ph(key, s.ph)'in opticalsrc and 'visible_moles(vessel, key, portion.moles.0) / litres'in opticalsrc and '*out += epsilon * concentration * path_cm'in opticalsrc
  relation=sources['crates/kerotakis-core/src/relations.rs'].decode();assert 'fn henderson_hasselbalch_fraction'in relation and '1.0 / (1.0 + 10f64.powf(pka - ph))'in relation
  assert {q['species']for q in receiver['contents']}=={'water','Na+','Cl-','methyl_orange'}
  assert {native['name']for native in receiver['solution']['species']}=={'H2O','Cl-','Na+','O2','H+','OH-'}, 'Dye optical native identity would need separate evaluation'
  predicted=max(bands);assert abs(predicted-float(oe['value']))<1e-12 and 405+20*bands.index(predicted)==465
  assert oe['value']<dose/dye*oe['value'];metrics={'dye_input_mol':dose,'held_dye_mol':held,'filtrate_dye_mol':dye,'held_fraction':held/dose,'charcoal_g':carbon,'liquid_volume_L':V,'actual_readings':[e for _,e in readings],'control_temperature_K':control['temperature'],'filtrate_temperature_K':receiver['temperature'],'actual_ph_for_editorial_spectrum':receiver['solution']['ph'],'independent_peak_absorbance':predicted,'path_cm':1,'peak_nm':465,'same_state_unadsorbed_counterfactual_absorbance':float(dose/dye)*predicted,'counterfactual_is_NOT_an_additional_observation':True,'maximum_allstep_atom_charge_residual_mol':maxerr,'allstep_vessel_ledger':trace}
 cases['ad-005']['exact_coverage_warnings']=[{'raw_line':r['raw_line'],'operator':r['operator'],'event':e}for r in rows for e in r['events']if e['event']=='not_yet_modeled']
 cases['ad-005']['optical_source_authorities']={'pKa':'indicator.rs methyl_orange pka3.47','acid_band_nm_epsilon_sigma':[505,24000,50],'base_band_nm_epsilon_sigma':[464,26000,55],'band_centres_nm':list(range(405,706,20)),'Beer_Lambert':'solution_optics.rs epsilon*visible_moles/liquid_volume*path_cm','path_cm':1,'instrument':'instrument.rs max over all16bands','volume':'vessel.rs sum Liquid portions n*M/rho/1000; only water here','native_visibility':'No native C/S/methyl species can represent the azo dye; analytical aqueous dye fallback retained; no complex guessed'}
 cases['ad-005'].update(status='BOUNDED_PROPOSAL_PENDING_PARENT_REVIEW',metrics=metrics,criterion_assessment={c['id']:'supported_with_limits'for c in cases['ad-005']['original_criteria']},limits=['Installed200mg/g and.05L/mg Langmuir parameters explicitly pending provenance review; no empirical quantitative adsorption calibration/pH-dependence/rate/competition claim.','465nm actual positive optical signal is not complete decolourisation or transparency; same-state restored-dose optical contrast is a derived counterfactual, not a measured control; its7.94AU is beyond installed4AU calibrated range and is not a quantitative spectrophotometric validation.','Native pH/conductivity cover represented NaCl components; unsupported dye ionic mapping means no full-mixture speciation/activity/conductivity claim.','Actual filtrate vs salt-only control matches represented Na/Cl, water, native solution and conductivity; actual temperatures differ by0.0000298908344K and are retained, not asserted equal. No desalination from adsorption/filtering.','No calorimetric forecast; actual source temperatures/clocks retained; no full thermal closure claim.'])
 # Preserve precise unqualified failures/holds; no incomplete ledger is a pass.
 for k in ['ad-001','en-001','en-002']:
  rr=[r for r in raws[k]if'bench'in r];meas=[e for r in rr for e in r['events']if e['event']=='measured'];cases[k].update(status='HOLD_NO_PROPOSAL',actual_measurements=meas)
 cases['ad-001']['reason']='Actual adsorption and filter-only inventories differ, but all4 optical observations are zero405nm; no measured colour-removal contrast. No full transfer/source-domain ledger yet, no selection.'
 cases['en-001']['reason']='All3 original optical endpoints are zero405nm including intact-starch positive control; actual starch hydrolysis does not establish original requested colour discrimination. Unknown enzyme formula cannot repair absent optical contrast.'
 cases['en-002']['reason']='Actual lactase aggregate state differs from milk blank/glucose-only controls; full milk mineral/proton/charge and unresolved-carrier ledger not completed here. No molecular glucose/galactose prediction or earned inventory criterion.'
 report={'source':SOURCE,'engine':binding['engine'],'lock_qualification':binding['lock_qualification'],'actual_binary_or_lock_bytes_rehashed':False,'archive_sha256':digest(archive),'source_blobs_sha256':{p:sha(b)for p,b in sources.items()},'cases':[cases[k]for k in IDS],'scientific_selection_performed':False,'product_engine_API_provider_build_calls':0,'first_early_reading_correction':'Early message called AD005 positive value405nm; exact raw event actually465nm and full source spectrum reconstruction above confirms465. No raw/forecast altered.'}
 a.out_dir.mkdir(mode=0o700);(a.out_dir/'assessment.json').write_text(json.dumps(report,indent=2,ensure_ascii=False,default=str)+'\n');print(json.dumps({'proposal':['batch3-diverse:ad-005'],'holds':[cases[k]['original_id']for k in IDS if k!='ad-005'],'max_material':str(maxerr)}))
if __name__=='__main__':main()
