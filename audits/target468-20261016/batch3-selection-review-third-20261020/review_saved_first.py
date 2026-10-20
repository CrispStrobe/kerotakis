#!/usr/bin/env python3
"""Third disjoint saved-byte review only. No engine, source mutation or catalog selection."""
import collections,hashlib,json,math,pathlib,runpy,zipfile
OUT=pathlib.Path(__file__).resolve().parent
ROOT=OUT.parents[2]
DEPENDENCY=ROOT/'audits/target468-20261016/batch3-selection-review-second-20261020/review_saved.py'
a=runpy.run_path(str(DEPENDENCY));sha=a['sha'];source=a['source'];REG=a['REG'];RECIPES=a['RECIPES'];MASSES=a['MASSES'];close=a['close'];dump=a['dump'];SOURCE=a['SOURCE'];BASE=a['BASE'];COMPOSITION=a['COMPOSITION'];CHARGE=a['CHARGE'];R=a['R'];vecadd=a['vecadd'];atoms=a['atoms'];events=a['events'];measures=a['measures'];conv=a['conv']
SELECT={'mi-005','li-005','li-006','li-007','co-008','en-005'}
ALLOWED={'added','material_added','vessel_created','solution_routed','solution_characterized','observed','measured','neutralised','dissolved','temperature_changed','energy_transferred','enzyme_hydrolysed','stirred','material_layers_formed','vessel_sealed','precipitated','headspace_equilibrated','gas_contained','hazard_warning','chemiluminescence_observed','irradiated','uv_attenuated','soap_scum_formed'}
def ledger(k,rows):
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
                if op['recipe_id']=='prepared/fatty-soap-equivalent':
                    eq=next(r['moles_per_gram'] for r in recipe['roles'] if r['kind']=='fatty_soap_equivalent')
                    expected[vid]['Na']+=u*eq
                    expected[vid]['soap_ligand_equivalent']+=u*eq
        snapshots=[];maxr=0;maxu=0
        for v in row['bench']['vessels']:
            vi=v['id'];actual=collections.defaultdict(float)
            for p in v['contents']:vecadd(actual,p['species'],p['moles'])
            soap=next((p for p in v.get('unresolved_materials',[]) if p['recipe_id']=='prepared/fatty-soap-equivalent'),None)
            scum=v.get('soap_scum')
            if soap is not None:
                eq=next(r['moles_per_gram'] for r in RECIPES[('prepared/fatty-soap-equivalent',1)]['roles'] if r['kind']=='fatty_soap_equivalent')
                actual['Na']+=soap['amount']*eq
                actual['soap_ligand_equivalent']+=soap['amount']*eq
            if scum:
                assert k=='co-008'
                actual['Ca']+=scum['divalent_ion_moles']
                actual['soap_ligand_equivalent']+=scum['soap_equivalent_moles']
            for e in set(actual)|set(expected[vi]):
                err=abs(actual[e]-expected[vi][e]);maxr=max(maxr,err)
                assert close(actual[e],expected[vi][e]),(i,vi,e,actual[e],expected[vi][e])
            uv=collections.defaultdict(float)
            for p in v.get('unresolved_materials',[]):uv[p['recipe_id']]+=p['amount']
            if scum:
                uv['prepared/fatty-soap-equivalent']+=scum['soap_equivalent_moles']/eq
            for material_key in set(uv)|set(unresolved[vi]):
                maxu=max(maxu,abs(uv[material_key]-unresolved[vi][material_key]));assert close(uv[material_key],unresolved[vi][material_key])
            assert close(v['elapsed_seconds'],clock[vi]),(i,vi,'clock')
            snapshots.append({'id':vi,'temperature_k':v['temperature'],'elapsed_s':v['elapsed_seconds'],'actual_elemental_and_macromolecule_carrier_totals':dict(actual),'expected_explicit_input_totals':dict(expected[vi]),'unresolved_g':dict(uv),'contents':v['contents']})
        trace.append({'raw_jsonl_line':row['raw_jsonl_line'],'step':row['step'],'operator':op,'max_element_or_charge_residual':maxr,'max_unresolved_g_residual':maxu,'vessels':snapshots})
    return {'domain':'Explicit additions, pinned recipe expansion, all resolved elemental/charge ownership, exact unresolved aggregate mass, actual contained gas and sealed dry air. CO008 additionally owns allCa/Na/declared soap-ligand equivalents, with no invented aggregate molecularformula. No gas export or unknown event admitted. Free-ion readback is not counted twice over analytical basis contents.','trace':trace,'max_element_or_charge_residual':max(x['max_element_or_charge_residual'] for x in trace),'max_unresolved_g_residual':max(x['max_unresolved_g_residual'] for x in trace)}

def content(v,sp,phase=None):return sum(p['moles'] for p in v['contents'] if p['species']==sp and (phase is None or p['phase']==phase))
def intensity(v):
    g=sum(p['amount'] for p in v.get('unresolved_materials',[]) if p['recipe_id']=='school/luminol-glow-solution');n=content(v,'H2O2')
    if g<=1e-9 or n<=1e-9:return None
    activation=min(p['added_at'] for p in v['lots'] if p['species']=='H2O2');elapsed=max(0,v['elapsed_seconds']-activation);t=min(333.15,max(273.15,v['temperature']));ratio=math.exp(35000/8.314462618*(1/293.15-1/t));half=60/ratio
    return {'elapsed_s':elapsed,'half_life_s':half,'relative_intensity':min(4,ratio*2**(-elapsed/half)*min(1,n/.001)),'oxidant_moles':n,'temperature':t}
def phase_energy(sp,phase,t0,t1):
    curves=[p for p in REG['heat_capacity_polynomials'] if p['species_id']==sp and p['phase']==phase]
    if not curves:
        cp=next(p['quantity']['value'] for p in REG['phase_thermodynamics'] if p['species_id']==sp and p['property']=='molar_heat_capacity');return cp*(t1-t0)
    p=curves[0];assert p['form']=='nasa9';inter=next(x for x in p['intervals'] if x['t_min_k']<=min(t0,t1) and x['t_max_k']>=max(t0,t1));c=inter['coefficients']
    return R*(c[0]*(1/t0-1/t1)+c[1]*math.log(t1/t0)+c[2]*(t1-t0)+sum(c[n]*(t1**(n-1)-t0**(n-1))/(n-1) for n in range(3,7)))

def assess(k,rows,scenes):
    final=rows[-1]['bench']['vessels'];calcs={};evidence=[]
    if k=='mi-005':
        air=next(e for _,e in events(rows,'vessel_sealed'))['trapped_air']
        totalC=[]
        for v in final:
            c=sum(atoms(p['species'],p['moles']).get('C',0) for p in v['contents']);totalC.append(c)
            assert close(content(v,'Ca+2'),.0005) and close(content(v,'Cl-'),.0012)
            p=sum(x['moles'] for x in v['contents'] if x['phase']=='gas')*R*v['temperature']/.5
            assert abs(v['pressure']/1000-p)<1e-8
        assert close(totalC[0]-.0005,air*.0004) and close(totalC[1],air*.0004)
        assert content(final[0],'CO2','gas')>50*content(final[1],'CO2','gas')
        assert close(content(final[1],'SO4-2'),.0005);assert not any(p['species']=='CaCO3' for p in final[0]['contents'])
        calcs={'carbonate_arm_total_C_mol':totalC[0],'gypsum_arm_total_C_mol':totalC[1],'trapped_air_C_per_arm_mol':air*.0004,'gas_CO2_mol':[content(v,'CO2','gas') for v in final],'aqueous_Ca_mol':[content(v,'Ca+2') for v in final]}
        evidence=events(rows,'gas_contained')+measures(rows,'pressure_gauge')
        note='Carbonate contributes0.5 mmol additional inorganicC while gypsum contributes0.5 mmol sulfate; both retain separately counted airCO2. Actual gas identity/inventory, not pressure alone, distinguishes arms. Full original sealed C/Ca/S/H/O/Cl/N/charge closes every row, with actual temperature andnRT/V pressures. No carbonate dissolving trajectory or gypsum solubility equality is assumed.'
    elif k in {'li-005','li-006'}:
        e=events(rows,'chemiluminescence_observed');calcs['independent_emission_model_samples']=[]
        for line,event in e:
            row=next(r for r in rows if r['raw_jsonl_line']==line);v=row['bench']['vessels'][event['vessel']];prediction=intensity(v);assert prediction is not None
            for name,value in prediction.items():assert close(value,event[name]),(k,line,name,value,event[name])
            calcs['independent_emission_model_samples'].append({'raw_jsonl_line':line,'vessel':v['id'],**prediction})
        peroxide=[]
        for i,row in enumerate(rows):
            if row['operator']['op']!='wait':continue
            for vi,v in enumerate(row['bench']['vessels']):
                before=rows[i-1]['bench']['vessels'][vi];dn=content(before,'H2O2')-content(v,'H2O2');dw=content(v,'water')-content(before,'water');do=content(v,'O2','gas')-content(before,'O2','gas')
                assert abs(dw-dn)<1e-12 and abs(do-dn/2)<1e-12
                peroxide.append({'raw_jsonl_line':row['raw_jsonl_line'],'vessel':vi,'peroxide_consumed_mol':dn,'water_formed_mol':dw,'retained_O2_formed_mol':do,'boundary':v['headspace']['boundary']})
        calcs['owned_peroxide_decomposition']=peroxide
        final_scene=scenes[-1]['scene']['vessels']
        for v in final:
            predicted=intensity(v);actual=final_scene[v['id']].get('chemiluminescence')
            if predicted is None:assert actual is None
            else:
                assert actual is not None
                for key in ['elapsed_s','half_life_s','relative_intensity']:assert close(actual[key],predicted[key])
        if k=='li-005':
            assert len(e)==2 and all(event['vessel']==0 for _,event in e);assert intensity(final[1]) is None and intensity(final[2]) is None
            assert all(close(v['elapsed_seconds'],60) for v in final)
            assert close(content(final[0],'H2O2'),content(final[2],'H2O2'))
            note='Only complete formulation+activator has a supported emission state; no-activator and no-formulation controls remain negative even though both peroxide arms undergo equal smalldecomposition. Independent state-derived Arrhenius/display decay reproduces both actual samples and persistent scene. AllH/O and retainedO2 close; no incoming-light command, molecular luminol reaction ledger, photon budget or commercial glow-stick claim.'
        else:
            assert len(e)==6;initial=[event for _,event in e[:2]];assert initial[1]['relative_intensity']>initial[0]['relative_intensity'] and initial[1]['half_life_s']<initial[0]['half_life_s']
            late=[event for _,event in e[-2:]];assert late[0]['relative_intensity']>late[1]['relative_intensity'];assert all(close(v['elapsed_seconds'],600) for v in final)
            note='Warm prepared formulation is initially brighter with shorter declared half-life; later cold arm persists more strongly. Each sample is independently reconstructed from owned formulation,peroxide,activation lot,time andactualchangingtemperature, not nominal fixed15/40C. Separate peroxide/water/retainedO2 ledger closes; no total accumulated photons, molecular yield or glow energy balance is inferred.'
        evidence=e+measures(rows,'thermometer')
    elif k=='li-007':
        role=next(r for r in RECIPES[('cosmetic/broad-spectrum-sunscreen',1)]['roles'] if r['kind']=='uv_attenuation')
        ir=events(rows,'irradiated');uv=events(rows,'uv_attenuated');assert len(ir)==4 and len(uv)==2
        assert [e['vessel'] for _,e in uv]==[0,1] and [e['band'] for _,e in uv]==['UV-B','UV-A']
        assert close(uv[0][1]['transmitted_fraction'],1/role['spf']) and close(uv[1][1]['transmitted_fraction'],1/role['uva_protection_factor'])
        assert all(close(e['irradiance_w_m2'],1) for _,e in ir);assert [e['wavelength_nm'] for _,e in ir]==[305,365,550,305]
        assert all(close(v['unresolved_materials'][0]['amount'],.4) for v in final[:3]);assert not final[3].get('unresolved_materials')
        assert all(v['elapsed_seconds']==0 for v in final)
        calcs={'UVB_transmitted_fraction':1/role['spf'],'UVA_transmitted_fraction':1/role['uva_protection_factor'],'visible550nm':'outside declared280–400nm model; absence is not unitytransmission','water_control305nm':'no installed UVattenuation materialrole','standard_film_mg_per_cm2':role['film_mg_per_cm2']}
        evidence=ir+uv;note='Equal incident irradiance receives distinct supportedUVB/UVA declared standard-film attenuation factors. No visible550nm attenuation or waterUV absorption is inferred from absent events. Original model-domain question is answered within280–400nm only; no measured continuous spectrum, thickness, photostability, exposure or safety claim.'
    elif k=='co-008':
        eq=next(r['moles_per_gram'] for r in RECIPES[('prepared/fatty-soap-equivalent',1)]['roles'] if r['kind']=='fatty_soap_equivalent');scum=final[0]['soap_scum'];assert close(scum['divalent_ion_moles'],.0005) and close(scum['soap_equivalent_moles'],.001)
        assert content(final[0],'Ca+2')==0 and close(content(final[0],'Na+'),.001);assert close(content(final[1],'Ca+2'),.0005) and not final[1].get('soap_scum');assert not final[2].get('soap_scum')
        checks=[]
        for row in rows:
            for v in row['bench']['vessels']:
                ss=v.get('soap_scum')
                if not ss:continue
                original_consumed=ss['soap_equivalent_moles']/eq
                mass=original_consumed+ss['divalent_ion_moles']*MASSES['Ca+2']-ss['soap_equivalent_moles']*MASSES['Na+']
                assert close(mass,ss['aggregate_mass_g'])
                remaining=next(p['amount'] for p in v['unresolved_materials'] if p['recipe_id']=='prepared/fatty-soap-equivalent');assert close(remaining+original_consumed,.4)
                assert close(content(v,'Na+'),ss['soap_equivalent_moles']);assert close(content(v,'Ca+2')+ss['divalent_ion_moles'],.0005)
                checks.append({'raw_jsonl_line':row['raw_jsonl_line'],'consumed_original_soap_g':original_consumed,'remaining_original_soap_g':remaining,'computed_scum_g':mass,'released_Na_mol':content(v,'Na+'),'bound_Ca_mol':ss['divalent_ion_moles']})
        scene=scenes[-1]['scene']['vessels'];assert close(scene[0]['soap_scum']['aggregate_mass_g'],scum['aggregate_mass_g']);assert not scene[1].get('soap_scum') and not scene[2].get('soap_scum')
        calcs={'per_row_mass_equivalent_and_metal_exchange':checks};evidence=events(rows,'soap_scum_formed')
        note='Only declaredfattysoap+Ca produces conserved aggregate scum; genericdishsoap+Ca retainsCa and no scum, fattysoapwithoutCa remains unchanged. Exactly2soap equivalents/Ca return2Na, consumedoriginalsoap+Ca−releasedNa reproduces0.303509gscum and remaining0.09354gsoap. Fullresolvedelement/charge plusCa/Na/ligand-equivalent/aggregate-mass ownership, no invented unresolved molecularformula. Controls have10/10/0s clocks; no equalstir history invented.'
    elif k=='en-005':
        heated=next(r for r in rows if r['operator']['op']=='heat');vhot=heated['bench']['vessels'][0];role=next(r for r in RECIPES[('food/pineapple',1)]['roles'] if r['kind']=='enzyme_source');assert vhot['temperature']>role['denatures_above_k']
        p0=next(p for p in final[0]['unresolved_materials'] if p['recipe_id']=='food/pineapple');p1=next(p for p in final[1]['unresolved_materials'] if p['recipe_id']=='food/pineapple')
        assert p0['enzyme_hydrolysis']['carried_enzyme_denatured'] and not p1.get('enzyme_hydrolysis',{}).get('carried_enzyme_denatured',False)
        assert conv(final[0])==0 and conv(final[1])>.3;assert all(close(v['elapsed_seconds'],2820) for v in final)
        heatchecks=[]
        for i,row in enumerate(rows):
            if row['operator']['op']not in ['heat','cool']:continue
            before=rows[i-1]['bench']['vessels'][0];after=row['bench']['vessels'][0];t0=before['temperature'];t1=after['temperature'];q=sum(p['moles']*phase_energy(p['species'],p['phase'],t0,t1) for p in before['contents'])
            assert abs(abs(q)-9000)<1e-6;heatchecks.append({'raw_jsonl_line':row['raw_jsonl_line'],'independent_sensible_j':q,'t0_k':t0,'t1_k':t1})
        calcs={'independent_heat_cool_energy':heatchecks,'denatures_above_k':role['denatures_above_k'],'final_temperatures_k':[v['temperature'] for v in final],'final_hydrolysed_g':[conv(v) for v in final]};evidence=events(rows,'enzyme_hydrolysed')+measures(rows,'thermometer')
        note='Heatedenzyme source crossesdeclared343.15K threshold before gelatinaddition; carried irreversible denaturationflagpersists aftercooling and no gelatin conversion appears, while freshsourcecontrol converts0.381800g. Independent±9000J energy andallstate/recipe/acidcoordinate inventoriesclose. Heating/cooling+ambient120swaitleave16.577C vs25C beforeprotein and final24.673vs25C, so no isothermalratecomparison orgeltextureclaim.'
    return {'interpretation':note,'independent_calculations':calcs,'event_evidence':[{'raw_jsonl_line':line,'event':event} for line,event in evidence]}

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
            inventory=ledger(k,rows);review=assess(k,rows,scenes)
            report={'original_id':oid,'title':original['title'],'original_forecast':original['original_expected_result'],'original_limitations':original['original_limitations'],'original_script_sha256':original['script_sha256'],'raw_member_prefix':prefix,'raw_sha256':hashes,'engine':binding['engine'],'status':'candidate_supported_original_bounded_criteria_met_pending_parent_review','no_catalog_selection':True,'criteria':{c['id']:{'original_criterion':c['criterion'],'original_instruction':c['review_instruction'],'assessment':review['interpretation'] if c['id']!='inventory' else inventory['domain'],'result':'candidate_met_pending_parent_review'} for c in original['original_acceptance_checks']},'inventory':inventory,**review,'final_scene':scenes[-1] if scenes else None};dump(OUT/(k+'.json'),report);index.append({'original_id':oid,'assessment':k+'.json'})
    paths=['data/registry/registry-source-v1.json']+['crates/kerotakis-core/src/'+p for p in ['bench.rs','vessel.rs','enzyme_activity.rs','enzyme.rs','constants.rs','heat_capacity.rs','chemiluminescence.rs','uv.rs','kitchen_biology.rs','clock.rs','states.rs']]
    dump(OUT/'source-binding.json',{'engine':binding['engine'],'raw_archive_sha256':sha(archive.read_bytes()),'raw_Cargo_lock_not_available':True,'saved_reader_sha256':sha(pathlib.Path(__file__).read_bytes()),'second_batch_reader_sha256':sha(DEPENDENCY.read_bytes()),'source_blobs':[{'path':p,'sha256':sha(source(p))} for p in paths],'no_engine_execution':True});dump(OUT/'candidate-index.json',index);print(json.dumps({'new_supported_candidates_for_parent_review':len(index),'engine_runs':0,'catalog_edits':0}))
if __name__=='__main__':main()
