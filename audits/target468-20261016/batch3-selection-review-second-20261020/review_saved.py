#!/usr/bin/env python3
"""Second disjoint saved-byte review; no engine execution."""
import collections, hashlib, json, math, pathlib, runpy, subprocess, zipfile
OUT=pathlib.Path(__file__).resolve().parent
ROOT=OUT.parents[2]
FIRST=ROOT/'audits/target468-20261016/batch3-selection-review-20261020/review_saved.py'
a=runpy.run_path(str(FIRST));sha=a['sha'];source=a['source'];REG=a['REGISTRY'];RECIPES=a['RECIPES'];MASSES=a['MASSES'];close=a['close'];dump=a['dump'];SOURCE=a['SOURCE'];BASE=a['BASE']
COMPOSITION={x['species_id']:{e['element']:e['count']['value'] for e in x['elements']} for x in REG['compositions']}
CHARGE={x['species_id']:x['net_charge']['value'] for x in REG['compositions']}
R=8.31446261815324
SELECT={'en-004','en-009','li-002','li-008','co-003','mi-006','mi-001','th-005'}
ALLOWED={'added','material_added','vessel_created','solution_routed','solution_characterized','observed','measured','neutralised','dissolved','temperature_changed','energy_transferred','enzyme_hydrolysed','stirred','material_layers_formed','magnet_separated','vessel_sealed'}

def atoms(sp,n):
    # Missing enzyme molecular formula is retained as its own exact carrier;
    # no invented atom formula for pepsin or other macromolecules.
    return {e:n*c for e,c in COMPOSITION.get(sp,{'owned_species:'+sp:1}).items()}
def vecadd(target,sp,n):
    for e,c in atoms(sp,n).items():target[e]+=c
    target['charge']+=n*CHARGE.get(sp,0)
def ledger(rows):
    expected=collections.defaultdict(lambda:collections.defaultdict(float));unresolved=collections.defaultdict(lambda:collections.defaultdict(float));clock=collections.defaultdict(float);trace=[]
    for i,row in enumerate(rows):
        op=row['operator'];vid=op.get('vessel')
        if op['op']=='add':vecadd(expected[vid],op['species'],op['moles'])
        if op['op']=='seal':
            prior=rows[i-1]['bench']['vessels'][vid]
            n=101325*op['headspace_volume']/1000/(R*prior['temperature'])
            for sp,share in [('N2',.7901),('O2',.2095),('CO2',.0004)]:vecadd(expected[vid],sp,n*share)
            event=next(e for e in row['events'] if e['event']=='vessel_sealed');assert close(n,event['trapped_air'])
        if op['op']=='magnet':
            assert op['from']==0 and op['to']==1
            prior=rows[i-1]['bench']['vessels'][0]
            ferrometal=sum(p['moles'] for p in prior['contents'] if p['species']=='Fe')
            vecadd(expected[0],'Fe',-ferrometal);vecadd(expected[1],'Fe',ferrometal)
        if op['op']=='wait':
            for v in row['bench']['vessels']:clock[v['id']]+=op['seconds']
        if op['op']=='stir':clock[vid]+=op['seconds']
        for e in row['events']:
            assert e['event'] in ALLOWED,(i,e)
            if e['event']=='material_added':
                assert op['op']=='add_material' and op['basis']=='mass_fraction'
                recipe=RECIPES[(op['recipe_id'],op['recipe_version'])];dose=op['total_amount']
                assert len(recipe['components'])==len(e['components'])
                for c,observed in zip(recipe['components'],e['components']):
                    assert c['fraction']['lower']==c['fraction']['upper'];n=dose*c['fraction']['lower']/MASSES[c['species_id']]
                    assert close(n,observed['moles']) and observed['species']==c['species_id'];vecadd(expected[vid],c['species_id'],n)
                uf=recipe['unresolved_fraction'];u=dose*uf['lower'] if uf else 0
                assert uf is None or uf['lower']==uf['upper'];assert close(u,e['unresolved_amount'])
                unresolved[vid][op['recipe_id']]+=u
        snapshots=[];maxr=0;maxu=0
        for v in row['bench']['vessels']:
            vi=v['id'];actual=collections.defaultdict(float)
            for p in v['contents']:vecadd(actual,p['species'],p['moles'])
            for e in set(actual)|set(expected[vi]):
                err=abs(actual[e]-expected[vi][e]);maxr=max(maxr,err)
                assert close(actual[e],expected[vi][e]),(i,vi,e,actual[e],expected[vi][e])
            uv=collections.defaultdict(float)
            for p in v.get('unresolved_materials',[]):uv[p['recipe_id']]+=p['amount']
            for k in set(uv)|set(unresolved[vi]):
                maxu=max(maxu,abs(uv[k]-unresolved[vi][k]));assert close(uv[k],unresolved[vi][k])
            assert close(v['elapsed_seconds'],clock[vi]),(i,vi,'clock')
            snapshots.append({'id':vi,'temperature_k':v['temperature'],'elapsed_s':v['elapsed_seconds'],'actual_elemental_and_macromolecule_carrier_totals':dict(actual),'expected_explicit_input_totals':dict(expected[vi]),'unresolved_g':dict(uv),'contents':v['contents']})
        trace.append({'raw_jsonl_line':row['raw_jsonl_line'],'step':row['step'],'operator':op,'max_element_or_charge_residual':maxr,'max_unresolved_g_residual':maxu,'vessels':snapshots})
    return {'domain':'Explicit additions, pinned recipe expansion, elemental/charge ownership, exact formula-free enzyme carriers, sealed dry air and Fe-only magnetic transfer. No gas export or unknown event admitted. Free proton/hydroxide readback is not counted twice over analytical basis contents.','trace':trace,'max_element_or_charge_residual':max(x['max_element_or_charge_residual'] for x in trace),'max_unresolved_g_residual':max(x['max_unresolved_g_residual'] for x in trace)}

def events(rows,n):return [(r['raw_jsonl_line'],e) for r in rows for e in r['events'] if e['event']==n]
def measures(rows,n):return [(i,e) for i,e in events(rows,'measured') if e['instrument']==n]
def conv(v):return sum(p['amount']*p.get('enzyme_hydrolysis',{}).get('converted_fraction',0) for p in v.get('unresolved_materials',[]))
def gas_enthalpy(sp,t0,t1):
    polys=[p for p in REG['heat_capacity_polynomials'] if p['species_id']==sp and p['phase']=='gas']
    if not polys:
        cp=next(p['quantity']['value'] for p in REG['phase_thermodynamics'] if p['species_id']==sp and p['property']=='molar_heat_capacity');return cp*(t1-t0)
    p=polys[0];assert p['form']=='nasa9';inter=next(x for x in p['intervals'] if x['t_min_k']<=min(t0,t1) and x['t_max_k']>=max(t0,t1));c=inter['coefficients']
    return R*(c[0]*(1/t0-1/t1)+c[1]*math.log(t1/t0)+c[2]*(t1-t0)+sum(c[n]*(t1**(n-1)-t0**(n-1))/(n-1) for n in range(3,7)))

def palette(reflectance):
    def lobe(x,mu,lo,hi):return math.exp(-.5*((x-mu)/(lo if x<mu else hi))**2)
    xyz=[0.,0.,0.];white=[0.,0.,0.]
    for nm,frac in zip(range(405,706,20),reflectance):
        cm=[1.056*lobe(nm,599.8,37.9,31)+.362*lobe(nm,442,16,26.7)-.065*lobe(nm,501.1,20.4,26.2),.821*lobe(nm,568.8,46.9,40.5)+.286*lobe(nm,530.9,16.3,31.1),1.217*lobe(nm,437,11.8,36)+.681*lobe(nm,459,26,13.8)]
        for c in range(3):xyz[c]+=frac*cm[c];white[c]+=cm[c]
    xyz=[xyz[i]/white[i]*d for i,d in enumerate([.95047,1,1.08883])];x,y,z=xyz
    linear=[3.2406*x-1.5372*y-.4986*z,-.9689*x+1.8758*y+.0415*z,.0557*x-.2040*y+1.057*z]
    def encode(c):
        c=max(0,min(c,1));v=12.92*c if c<=.0031308 else 1.055*c**(1/2.4)-.055
        return math.floor(v*255+.5)
    return [encode(c) for c in linear]
def paint(parts):
    roles=[(n,next(r for r in RECIPES[(key,1)]['roles'] if r['kind']=='opaque_pigment')) for key,n in parts]
    reflect=[]
    for b in range(16):
        k=sum(n*r['absorption'][b] for n,r in roles);s=sum(n*r['scattering'][b] for n,r in roles);q=1+k/s;reflect.append(q-math.sqrt(q*q-1))
    return palette(reflect)

def assess(k,rows,scenes):
    final=rows[-1]['bench']['vessels'];evidence=[];independent={}
    if k in {'en-004','en-009'}:
        e=events(rows,'enzyme_hydrolysed');evidence=e+measures(rows,'ph_meter')
        for line,eve in e:
            assert eve['family']=='pepsin'
            row=next(r for r in rows if r['raw_jsonl_line']==line);v=row['bench']['vessels'][eve['vessel']];assert any(close(p.get('enzyme_hydrolysis',{}).get('converted_fraction',-1),eve['converted_fraction']) for p in v['unresolved_materials'])
        if k=='en-004':
            assert len(e)==2 and e[0][1]['hydrolysed_mass_g']>1000*e[1][1]['hydrolysed_mass_g'];assert conv(final[2])==0
            assert final[2]['solution']['ph']<3
            assert all(298<v['temperature']<334 for v in final)
            assert close(conv(final[0]),e[0][1]['hydrolysed_mass_g']) and close(conv(final[1]),e[1][1]['hydrolysed_mass_g'])
            ph=measures(rows,'ph_meter');assert ph[0][1]['value']<3 and ph[1][1]['value']>6
            assert all(close(v['elapsed_seconds'],1800) for v in final)
            note='Acid pepsin vessel converts far more retained albumin than neutral pepsin; acid-only blank remains unconverted. Measured pH2.039/6.810 and actual temperature histories replace nominal acidity. Protein buffering, peptide products and physiological or empirical-rate claims remain excluded.'
        else:
            assert len(e)==3;a0,an,a1=[x[1]['hydrolysed_mass_g'] for x in e];assert min(a0,a1)>1000*an
            incremental=[conv(rows[i]['bench']['vessels'][0])-conv(rows[i-1]['bench']['vessels'][0]) for i,r in enumerate(rows) if r['operator']['op']=='wait']
            assert all(close(x,y) for x,y in zip(incremental,[a0,an,a1]));assert close(final[0]['elapsed_seconds'],1800)
            ph=measures(rows,'ph_meter');assert len(ph)==2 and 6<ph[0][1]['value']<7 and ph[1][1]['value']<3
            independent['incremental_hydrolysed_g']=incremental
            note='Incremental conversion nearly pauses near pH6.847 and resumes after reacidification to2.051. The third increment is smaller than first because less unconverted substrate remains; no irreversible high-pH destruction or activity reset is inferred.'
    elif k=='li-002':
        ph=measures(rows,'ph_meter');uv=measures(rows,'spectrophotometer');assert len(ph)==3 and len(uv)==3;assert ph[1][1]['value']<3 and abs(ph[0][1]['value']-ph[2][1]['value'])<.01
        assert [e['unit'] for _,e in uv]==['absorbance at 565 nm','absorbance at 525 nm','absorbance at 565 nm'];assert abs(uv[0][1]['value']-uv[2][1]['value'])<.001
        evidence=ph+uv;note='Anthocyanin surrogate band changes565→525→565 nm with measured neutral→acid→neutral pH. Equal retained pigment inventory is independently conserved; slight final dilution/heating remains in raw state. This is the installed surrogate response, not a universal molecular pKa or degradation model.'
    elif k=='li-008':
        scene=scenes[-1]['scene']['vessels'];actual=[v['liquid']['srgb'] for v in scene]
        predicted=[paint([('school/acrylic-red-surrogate',1)]),paint([('school/acrylic-blue-surrogate',1)]),paint([('school/acrylic-red-surrogate',1),('school/acrylic-blue-surrogate',1)])]
        assert actual==predicted
        assert all(v['liquid']['cloudiness']==1 for v in scene)
        assert close(final[0]['unresolved_materials'][0]['amount'],1.1) and close(final[1]['unresolved_materials'][0]['amount'],1.1)
        assert len(final[2]['unresolved_materials'])==2 and all(close(p['amount'],.55) for p in final[2]['unresolved_materials'])
        average=[(actual[0][i]+actual[1][i])/2 for i in range(3)];assert actual[2]!=average
        independent={'independent_pinned_Kubelka_Munk_and_CIE_srgb':predicted,'arithmetic_rgb_average':average};note='All3 actual paint colours match independently reconstructed pinned16-band absorption/scattering plus CIE/sRGB computation; mixed colour differs from arithmeticRGB. Equal original pigment masses remain. Optically thick teaching film only; no curing, substrate, thickness or proprietary formulation prediction. Balance omits unresolved binder/pigment and is not treated as total physical mass.'
    elif k=='co-003':
        scene=scenes[-1]['scene']['vessels'];assert not final[0].get('unresolved_materials') and final[1]['unresolved_materials'][0]['amount']==2
        assert final[0]['contents'][1]['species']=='ethanol';assert not final[0].get('emulsion') and not final[1].get('emulsion')
        assert len(scene[0]['layers'])==1 and len(scene[1]['layers'])==2;assert all(close(v['elapsed_seconds'],310) for v in final)
        evidence=events(rows,'material_layers_formed')+measures(rows,'densitometer');note='Alcohol remains mixed solvent inventory with one liquid layer; the retained unresolved oil produces a distinct immiscible layer despite stirring and300 s rest. Separate10 s stirs mean both clocks310 s. No universal oil partition coefficient or precise triglyceride identity is inferred.'
    elif k=='mi-006':
        e=events(rows,'magnet_separated');assert len(e)==1 and e[0][1]['attracted']==['Fe'];assert set(e[0][1]['remained'])=={'Fe2O3','SiO2'}
        nmetal=final[1]['contents'][0]['moles'];noxide=next(p['moles'] for p in final[0]['contents'] if p['species']=='Fe2O3');independent={'recovered_Fe_mol':nmetal,'remaining_oxide_Fe_mol':2*noxide,'fraction_of_initial_Fe_atoms_recovered':nmetal/(nmetal+2*noxide)}
        assert 0<independent['fraction_of_initial_Fe_atoms_recovered']<1;evidence=e+measures(rows,'balance');note='Only metallic Fe transfers; hematite retains twice its formula-unit amount ofFe atoms and silica stays behind. Per-vessel elemental ledger closes at every row, so recovered metallic mass is not misreported as total elemental iron. Binary selectivity only, no susceptibility calibration.'
    elif k=='mi-001':
        n0={p['species']:p['moles'] for p in final[0]['contents']};n1={p['species']:p['moles'] for p in final[1]['contents']}
        assert close(n0['Mg+2'],.005) and close(n1['Mg+2'],.005) and close(n0['SO4-2'],.005) and close(n1['SO4-2'],.005);assert close(n0['water']-n1['water'],7*.005)
        b=measures(rows,'balance');assert close(b[0][1]['value']-b[1][1]['value'],.035*MASSES['water']);evidence=b+measures(rows,'conductivity_meter');independent={'hydrate_released_water_mol':n0['water']-n1['water'],'extra_water_mass_g':.035*MASSES['water']};note='Equal5 mmol MgSO4 formula-unit doses retain equalMg/S while epsomite contributes35 mmol additional water. Entire per-row elemental/charge inventories and balance difference agree independently. Conductivity is concentration/temperature dependent and carries its2:2 empirical-correction limitation, not differentMg stoichiometry.'
    elif k=='th-005':
        energy=events(rows,'energy_transferred');assert len(energy)==2
        integrals=[]
        for line,e in energy:
            i=next(i for i,r in enumerate(rows) if r['raw_jsonl_line']==line);v=rows[i-1]['bench']['vessels'][e['vessel']];after=rows[i]['bench']['vessels'][e['vessel']];t0=v['temperature'];t1=after['temperature'];q=sum(p['moles']*(gas_enthalpy(p['species'],t0,t1)-R*(t1-t0)) for p in v['contents']);assert abs(q-1)<1e-9;integrals.append(q)
        pressures=measures(rows,'pressure_gauge')
        for _,e in pressures:
            v=final[e['vessel']];n=sum(p['moles'] for p in v['contents']);assert abs(e['value']-n*R*v['temperature']/.5)<1e-8
        assert all(v['headspace']['boundary']=='sealed' for v in final)
        assert final[0]['temperature']>final[1]['temperature'];evidence=energy+measures(rows,'thermometer')+pressures;independent={'integrated_Cv_energy_J':integrals,'integration':'Pinned NASA9 polynomials forN2/O2/CO2, tabulated heliumCp, subtractR for rigid sealed gas; every species and trapped dry air counted.'};note='Each vessel contains sealed dry air plus5 mmol extra gas. Full owned inventory and independent integralCv reproduce each1 J increment; helium arm warms more than nitrogen. Pressure reconstructs from totalnRT/V. No evacuated headspace or naive pure-gas temperature ratio assumed.'
    return {'interpretation':note,'independent_calculations':independent,'event_evidence':[{'raw_jsonl_line':l,'event':e} for l,e in evidence]}

def main():
    archive=BASE/'native-screen-results/raw-artifact.zip';verification=json.loads((BASE/'native-screen-results/verification.json').read_text());assert sha(archive.read_bytes())==verification['artifact_sha256'];binding=json.loads((BASE/'native-screen-results/binding.json').read_text());index=[]
    intake=json.loads((BASE/'intake.json').read_text())
    with zipfile.ZipFile(archive) as z:
        for k in sorted(SELECT):
            oid='batch3-diverse:'+k;original=next(x for x in intake if x['original_id']==oid);prefix=oid.replace(':','_');summary=json.loads((BASE/'native-screen-results/cases'/f'{prefix}.json').read_text());execution=json.loads(z.read(prefix+'/execution.json'));assert summary['engine']==binding['engine'];hashes={}
            for f,key in [('experiment.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
                h=sha(z.read(prefix+'/'+f));assert h==execution[key]==summary['execution'][key];hashes[f]=h
            assert z.read(prefix+'/experiment.lab')==(BASE/original['original_script']).read_bytes();assert hashes['experiment.lab']==original['script_sha256'];assert execution['exit_code']==0 and not summary['solver_failures']
            rr=[json.loads(l) for l in z.read(prefix+'/stdout.jsonl').splitlines()];rows=[dict(r,raw_jsonl_line=i+1) for i,r in enumerate(rr) if 'bench'in r];scenes=[{'raw_jsonl_line':i+1,'scene':r['scene']} for i,r in enumerate(rr) if 'scene'in r]
            inventory=ledger(rows);review=assess(k,rows,scenes)
            report={'original_id':oid,'title':original['title'],'original_forecast':original['original_expected_result'],'original_limitations':original['original_limitations'],'original_script_sha256':original['script_sha256'],'raw_member_prefix':prefix,'raw_sha256':hashes,'engine':binding['engine'],'status':'candidate_supported_original_bounded_criteria_met_pending_parent_review','no_catalog_selection':True,'criteria':{c['id']:{'original_criterion':c['criterion'],'original_instruction':c['review_instruction'],'assessment':review['interpretation'] if c['id']!='inventory' else inventory['domain'],'result':'candidate_met_pending_parent_review'} for c in original['original_acceptance_checks']},'inventory':inventory,**review,'final_scene':scenes[-1] if scenes else None};dump(OUT/(k+'.json'),report);index.append({'original_id':oid,'assessment':k+'.json'})
    paths=['data/registry/registry-source-v1.json']+['crates/kerotakis-core/src/'+p for p in ['bench.rs','vessel.rs','enzyme_activity.rs','enzyme.rs','constants.rs','heat_capacity.rs','pigment.rs','spectrum.rs','appearance.rs','states.rs']]
    dump(OUT/'source-binding.json',{'engine':binding['engine'],'raw_archive_sha256':sha(archive.read_bytes()),'raw_Cargo_lock_not_available':True,'saved_reader_sha256':sha(pathlib.Path(__file__).read_bytes()),'first_batch_reader_sha256':sha(FIRST.read_bytes()),'source_blobs':[{'path':p,'sha256':sha(source(p))} for p in paths],'no_engine_execution':True});dump(OUT/'candidate-index.json',index);print(json.dumps({'new_supported_candidates_for_parent_review':len(index),'engine_runs':0,'catalog_edits':0}))
if __name__=='__main__':main()
