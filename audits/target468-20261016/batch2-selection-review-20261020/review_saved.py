#!/usr/bin/env python3
"""Independent bounded review of ten unchanged batch2 protocols; never executes Kero."""
import hashlib,json,math,subprocess,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'audits/new-experiments-201-20261016/native-screen-results'
IDS=('ec-011','tc-006','tc-009','se-004','se-005','se-006','ab-026','ab-033','ec-001','ec-002')
A={'water':{'H':2,'O':1},'H+':{'H':1},'base_equivalents':{'H':1,'O':1},'HCl':{'H':1,'Cl':1},'NaOH':{'Na':1,'O':1,'H':1},'NaCl':{'Na':1,'Cl':1},'Na+':{'Na':1},'Cl-':{'Cl':1},'Na2SO4':{'Na':2,'S':1,'O':4},'SO4-2':{'S':1,'O':4},'SiO2':{'Si':1,'O':2},'Cu':{'Cu':1},'Fe':{'Fe':1},'Zn':{'Zn':1},'Ag':{'Ag':1},'CuSO4':{'Cu':1,'S':1,'O':4},'ZnSO4':{'Zn':1,'S':1,'O':4},'AgNO3':{'Ag':1,'N':1,'O':3},'Cu+2':{'Cu':1},'Zn+2':{'Zn':1},'Ag+':{'Ag':1},'NO3-':{'N':1,'O':3},'antlerite':{'Cu':3,'S':1,'O':8,'H':4},'CH3COOH':{'C':2,'H':4,'O':2},'CH3COO-':{'C':2,'H':3,'O':2},'NaOAc':{'Na':1,'C':2,'H':3,'O':2},'N2':{'N':2},'O2':{'O':2},'H2':{'H':2},'CO2':{'C':1,'O':2},'CO2(aq)':{'C':1,'O':2},'HCO3-':{'H':1,'C':1,'O':3},'CO3-2':{'C':1,'O':3},'bromothymol_blue':{'C':27,'H':28,'Br':2,'O':5,'S':1}}
Q={'H+':1,'base_equivalents':-1,'Na+':1,'Cl-':-1,'SO4-2':-2,'Cu+2':2,'Zn+2':2,'Ag+':1,'NO3-':-1,'CH3COO-':-1,'HCO3-':-1,'CO3-2':-2}
KEYS=sorted({k for d in A.values() for k in d})+['charge']; TOL=1e-8
sha=lambda b:hashlib.sha256(b).hexdigest()
def zero():return dict.fromkeys(KEYS,0.)
def book(t,s,n):
 assert s in A,s
 for k,v in A[s].items():t[k]+=v*n
 t['charge']+=Q.get(s,0)*n

def amounts(v,s,phase=None):return sum(p['moles'] for p in v['contents'] if p['species']==s and (phase is None or p['phase']==phase))
def vectors(v):
 t=zero()
 for p in v['contents']: assert p['moles']>=0;book(t,p['species'],p['moles'])
 return t

def cp_model(row,phase,T):
 polys=[p for p in row['heat_capacity_polys'] if p['phase']==phase]
 if not polys:return row['heat_capacity']
 intervals=polys[0]['intervals'];T=max(intervals[0]['t_min'],min(T,intervals[-1]['t_max']));q=next(i for i in intervals if i['t_min']<=T<=i['t_max']);assert q['form']=='nasa9'
 a=q['coefficients'];return 8.31446261815324*(a[0]/T**2+a[1]/T+a[2]+a[3]*T+a[4]*T**2+a[5]*T**3+a[6]*T**4)
def sensible(row,phase,a,b):
 # Independent composite Simpson quadrature, no Kero code imported.
 N=400;h=(b-a)/N
 return h/3*sum((1 if i in (0,N) else 4 if i%2 else 2)*cp_model(row,phase,a+i*h) for i in range(N+1))
def review():
 artifact=BASE/'raw-artifact.zip'; digest=hashlib.file_digest(artifact.open('rb'),'sha256').hexdigest(); assert digest==json.loads((BASE/'artifact-metadata.json').read_text())['digest'].removeprefix('sha256:')
 intake={r['original_id']:r for r in json.loads((BASE.parent/'intake.json').read_text())}
 binding=json.loads((BASE/'binding.json').read_text()); out=[]
 source=binding['engine']['source_head'];assert subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip()==binding['engine']['source_tree']
 registry_raw=subprocess.check_output(['git','show',source+':crates/kerotakis-core/tests/golden/registry.json'],cwd=ROOT);registry={r['key']:r for r in json.loads(registry_raw)};assert registry['water']['molar_mass']==18.015 and registry['water']['density']==.997
 
 membership=json.loads((ROOT/'audits/target468-20261016/hydrolysis-beneficiaries-outcome-20261019/catalog-membership.json').read_text()); selected={m['original_target_id'] for m in membership['mapping']}
 with zipfile.ZipFile(artifact) as z:
  for suffix in IDS:
   key='batch2_'+suffix;case=json.loads((BASE/'cases'/f'{key}.json').read_text()); assert case['id'] not in selected
   assert case['engine']==binding['engine'];original=intake[case['id']];assert case['original_physical_forecast']==original['original_expected_result'];assert case['original_acceptance_checks']==original['original_acceptance_checks'];script=z.read(key+'/experiment.lab'); raw=z.read(key+'/stdout.jsonl');err=z.read(key+'/stderr.txt')
   assert script==(BASE.parent/original['original_script']).read_bytes();assert sha(script)==case['original_script_sha256']==case['selected_script_sha256']==case['execution']['script_sha256']; assert sha(raw)==case['execution']['stdout_sha256'];assert sha(err)==case['execution']['stderr_sha256'];assert case['execution']['exit_code']==0
   rows=[json.loads(x) for x in raw.splitlines() if x.strip()]; expected={}; prior={};maxima=zero();partial=0; seals=[]
   for ix,row in enumerate(rows):
    op=row['operator'];typ=op['op'];assert typ in ('add','new_vessel','seal','electrolyse','inspect','heat','measure','decant','dilute','grind','centrifuge','cell'),typ
    if typ=='add':book(expected.setdefault(op['vessel'],zero()),op['species'],op['moles'])
    elif typ=='seal':
     n=101325*op['headspace_volume']/(8314.462618*prior[op['vessel']]['temperature']);target=expected[op['vessel']]
     for sp,f in [('N2',.7901),('O2',.2095),('CO2',.0004)]:book(target,sp,n*f)
     seals.append({'row':ix,'vessel':op['vessel'],'independent_air_mol':n})
    elif typ=='dilute':book(expected[op['vessel']],'water',op['volume']*.997*1000/18.015)
    elif typ=='decant':
     frm,to=op['from'],op['to'];expected.setdefault(to,zero())
     for p in prior[frm]['contents']:
      if p['phase'] in ('liquid','aqueous'):
       n=op['fraction']*p['moles'];book(expected[frm],p['species'],-n);book(expected[to],p['species'],n)
    assert not any(e['event'] in ('solver_failed','gas_evolved','gas_absorbed') for e in row['events'])
    current={v['id']:v for v in row['bench']['vessels']}
    if set(prior)-set(current):assert typ=='inspect' and not row['events'];assert all(v==prior[k] for k,v in current.items());partial+=1
    prior.update(current)
    for vid,v in prior.items():
     assert not v.get('unresolved_materials') and not v.get('surfaces');actual=vectors(v);exp=expected.setdefault(vid,zero())
     for k in KEYS:
      d=abs(actual[k]-exp[k]);maxima[k]=max(maxima[k],d);assert d<=TOL,(suffix,ix,typ,vid,k,d)
   # Numerical observations below are derived from inventory, not labels alone.
   evidence={}; met=[True,True]; limitations=[]
   events=[e for r in rows for e in r['events']]
   if suffix=='ec-011':
    increments=[]
    for i,r in enumerate(rows):
     if r['operator']['op']!='electrolyse':continue
     op=r['operator'];vid=op['vessel'];before={v['id']:v for v in rows[i-1]['bench']['vessels']}[vid];after={v['id']:v for v in r['bench']['vessels']}[vid];q=op['amps']*op['seconds'];h=amounts(after,'H2','gas')-amounts(before,'H2','gas');o=amounts(after,'O2','gas')-amounts(before,'O2','gas');assert abs(q-5)<1e-14;assert abs(h-q/(2*96485.33212))<1e-12;assert abs(o-q/(4*96485.33212))<1e-12
     increments.append({'vessel':vid,'charge_C':q,'H2_increment_mol':h,'O2_increment_mol':o,'inventory_Faradaic_efficiency_H2':h*2*96485.33212/q})
    assert len(increments)==2 and abs(increments[0]['H2_increment_mol']-increments[1]['H2_increment_mol'])<1e-12;evidence={'increments':increments,'air':seals};limitations=['Modelled water electrolysis uses unit current efficiency; no empirical electrode/overpotential validation.']
   elif suffix=='tc-006':
    vals=[]
    for vid,v in prior.items():
     n=v['contents'][0]['moles'];dt=v['temperature']-298.15;assert dt>0;vals.append({'vessel':vid,'species':v['contents'][0]['species'],'moles':n,'delta_K':dt,'inferred_molar_average_Cp_J_mol_K':10/(n*dt),'independent_curve_energy_J':n*sensible(registry[v['contents'][0]['species']],'solid',298.15,v['temperature'])})
    assert all(abs(x['independent_curve_energy_J']-10)<1e-5 for x in vals)
    assert abs(vals[0]['delta_K']-vals[1]['delta_K'])/vals[1]['delta_K']>abs(vals[2]['delta_K']-vals[3]['delta_K'])/vals[3]['delta_K'];assert all(20<x['inferred_molar_average_Cp_J_mol_K']<30 for x in vals);evidence={'metal_capacity_inference':vals};limitations=['Interval-average heat capacities inferred from known10J and actual inventory, not an empirical calibration or assertion of identical molar curves.']
   elif suffix=='tc-009':
    vals=[]
    for i,r in enumerate(rows):
     if r['operator']['op']!='decant':continue
     op=r['operator'];pre={v['id']:v for v in rows[i-1]['bench']['vessels']};v0,v1=pre[op['to']],pre[op['from']];n0,n1=amounts(v0,'water'),amounts(v1,'water');baseline=(n0*v0['temperature']+n1*v1['temperature'])/(n0+n1);after={v['id']:v for v in r['bench']['vessels']}[op['to']];vals.append({'vessel':op['to'],'water_weighted_baseline_K':baseline,'final_K':after['temperature'],'corrected_rise_K':after['temperature']-baseline,'independent_mixing_energy_J':n0*sensible(registry['water'],'liquid',v0['temperature'],after['temperature'])+n1*sensible(registry['water'],'liquid',v1['temperature'],after['temperature'])})
    met=[False,False];evidence={'mixing':vals,'calorimeter_existing_energy_J':[e['value']*1000 for e in events if e['event']=='measured' and e['instrument']=='calorimeter']};limitations=['Both mixing-corrected rises are essentially zero. Existing calorimeter energy includes earlier NaOH dissolution; original neutralisation-energy forecast fails.']
   elif suffix=='se-004':
    ns=[amounts(prior[v],'Na+') for v in range(4)];cls=[amounts(prior[v],'Cl-') for v in range(4)];si=[amounts(prior[v],'SiO2') for v in range(4)];assert abs(ns[0]/.001-.2**3)<1e-12 and all(abs(a-b)<1e-12 for a,b in zip(ns,cls));assert abs(sum(ns)-.001)<TOL;assert amounts(prior[0],'SiO2','solid')>0 and all(amounts(prior[v],'SiO2','solid')==0 for v in (1,2,3));evidence={'Na_mol_by_vessel':ns,'Cl_mol_by_vessel':cls,'SiO2_mol_by_vessel':si,'remaining_salt_fraction':ns[0]/.001};limitations=['Silica has finite dissolved loss in wash fractions; retention is of the solid phase, not an assertion of exactly insoluble silica.']
   elif suffix in ('se-005','se-006'):
    ce=[e for e in events if e['event']=='centrifuged'];vals=[]
    for e in ce:
     s,=e['separations'];rcf=(2*math.pi*e['rpm']/60)**2*e['rotor_radius_m']/9.80665;velocity=(s['particle_density_kg_m3']-e['fluid_density_kg_m3'])*(s['particle_diameter_um']*1e-6)**2*rcf*9.80665/(18*e['dynamic_viscosity_pa_s']);assert abs(rcf/e['rcf']-1)<1e-12;assert abs(velocity/s['terminal_speed_m_s']-1)<1e-12;assert not s['particle_size_assumed'];assert abs(s['distance_m']-velocity*e['seconds'])<1e-12;v=prior[e['vessel']];lots=[l for l in v['lots'] if l['species']=='SiO2' and l['phase']=='solid'];assert lots and all(abs(l['particle_size_um']-s['particle_diameter_um'])<1e-12 for l in lots);assert all(abs(l['suspended_fraction']-(1-s['separated_fraction']))<1e-12 for l in lots);vals.append({'vessel':e['vessel'],'independent_rcf':rcf,'independent_speed_m_s':velocity,'diameter_um':s['particle_diameter_um'],'fraction':s['separated_fraction']})
    assert vals[1]['independent_speed_m_s']>vals[0]['independent_speed_m_s'];assert vals[1]['fraction']>=vals[0]['fraction'];evidence={'centrifuge':vals};limitations=['Ideal Stokes dilute-suspension model; capped settling fractions are not proportional pellet yields. BothSE005 arms reach the full-settling cap.']
   elif suffix=='ab-026':
    vals=[]
    for vid in (0,1):
     v=prior[vid];acet=amounts(v,'CH3COOH')+amounts(v,'CH3COO-');assert abs(acet-.002)<TOL;assert abs(amounts(v,'Na+')-.002)<TOL;assert abs(amounts(v,'Cl-')-.001)<TOL;vals.append({'vessel':vid,'acetate_mol':acet,'Na_mol':amounts(v,'Na+'),'Cl_mol':amounts(v,'Cl-'),'water_mol':amounts(v,'water'),'T_K':v['temperature'],'pH':v['solution']['ph']})
    assert abs(vals[0]['pH']-vals[1]['pH'])<.001;evidence={'arms':vals};limitations=['Near-identical pH is assessed with actual temperatures and reaction-water amounts; not bottle-order equivalence at arbitrary concentration.']
   elif suffix=='ab-033':
    observed=[e['appearance'] for e in events if e['event']=='observed'];ph=[e['value'] for e in events if e['event']=='measured' and e['instrument']=='ph_meter'];assert ph[0]<4 and ph[1]>10 and ph[2]<4;assert observed[0]['liquid']==observed[2]['liquid'] and observed[0]['liquid']!=observed[1]['liquid'];assert all(abs(amounts(v,'bromothymol_blue')-1e-7)<1e-15 for r in rows[1:] for v in r['bench']['vessels']);evidence={'pH':ph,'appearance':observed};limitations=['Model indicator environment response and conserved trace inventory, not measured spectral calibration.']
   elif suffix in ('ec-001','ec-002'):
    cell=[e for e in events if e['event']=='cell_voltage'];measure_rows=[(i,r) for i,r in enumerate(rows) if r['operator']['op']=='cell'];assert all(r['bench']==rows[i-1]['bench'] for i,r in measure_rows)
    if suffix=='ec-002':assert abs(cell[0]['volts']+cell[1]['volts']-cell[2]['volts'])<1e-12
    else:
     assert abs(cell[0]['volts']-cell[1]['volts'])<1e-6;assert abs(amounts(prior[0],'Zn','solid')-.001)<1e-12;assert abs(amounts(prior[1],'Zn','solid')-.01)<1e-12
    independent=[]
    couples={0:('Zn+2',-.7618,2),1:('Zn+2',-.7618,2)} if suffix=='ec-001' else {0:('Zn+2',-.7618,2),1:('Cu+2',.3419,2),2:('Ag+',.7996,1)}
    if suffix=='ec-001':couples[2]=('Cu+2',.3419,2)
    potentials={}
    for vid,(ion,e0,n) in couples.items():
     v=prior[vid];activity=next(a['activity'] for a in v['solution']['species'] if a['name']==ion);potentials[vid]=e0+8.31446261815324*v['temperature']/(n*96485.33212)*math.log(activity)
    for ce in cell:
     value=potentials[ce['cathode']]-potentials[ce['anode']];assert abs(value-ce['volts'])<1e-12;independent.append(value)
    evidence={'independent_Nernst_volts':independent,'signed_cells':cell,'half_cells':[{'vessel':vid,'T_K':v['temperature'],'Zn2_activity':next((s['activity'] for s in v.get('solution',{}).get('species',[]) if s['name']=='Zn+2'),None)} for vid,v in prior.items()]};limitations=['Named slow metal-water corrosion limit does not invalidate instantaneous nonconsuming equilibrium potential comparison; no corrosion-timescale prediction.']
   out.append({'id':case['id'],'proposed_candidate':all(met),'original_forecast':case['original_physical_forecast'],'criteria':[{**c,'result':'met' if ok else 'not_met'} for c,ok in zip(case['original_acceptance_checks'],met,strict=True)],'observed':evidence,'max_per_vessel_per_step_atom_charge_residual_mol':maxima,'snapshots':len(rows),'partial_inspects':partial,'named_model_limits':case['named_model_limits'],'limitations':limitations,'source':case['engine'],'script_sha256':sha(script),'stdout_sha256':sha(raw),'stderr_sha256':sha(err),'raw_members':[key+'/experiment.lab',key+'/stdout.jsonl',key+'/stderr.txt']})
 return {'schema':'kerotakis.batch2.saved-independent-review.v1','cases':out,'raw_artifact_sha256':digest,'new_engine_executions':0,'selection_performed':False,'tolerance_mol':TOL,'source_blobs_sha256':{path:sha(subprocess.check_output(['git','show',source+':'+path],cwd=ROOT)) for path in ['crates/kerotakis-core/src/bench.rs','crates/kerotakis-core/src/displacement.rs','crates/kerotakis-core/src/relations.rs','crates/kerotakis-core/src/heat_capacity.rs','crates/kerotakis-core/src/constants.rs','crates/kerotakis-core/tests/golden/registry.json']},'registry_source_blob_sha256':sha(registry_raw),'producer_lock_qualification':binding['lock_qualification'],'reviewed_head':binding['engine']['source_head']}
if __name__=='__main__':print(json.dumps(review(),indent=2,allow_nan=False))
