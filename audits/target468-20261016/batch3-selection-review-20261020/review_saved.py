#!/usr/bin/env python3
"""Bounded saved-byte review only: never imports or invokes a chemistry engine."""
import collections, hashlib, json, pathlib, subprocess, zipfile
ROOT = pathlib.Path(__file__).resolve().parents[3]
OUT = pathlib.Path(__file__).resolve().parent
BASE = ROOT / 'audits/new-experiments-201-20261016'
SOURCE = 'c44692053a94c59dcf62e7fb67549f84c7d18f83'
sha = lambda b: hashlib.sha256(b).hexdigest()
def dump(p, x): p.write_text(json.dumps(x, indent=2, ensure_ascii=False) + '\n')
def source(path): return subprocess.check_output(['git','show',SOURCE+':'+path], cwd=ROOT)
def close(a,b): return abs(a-b) < 1e-10 * max(1,abs(a),abs(b))
REGISTRY=json.loads(source('data/registry/registry-source-v1.json'))
RECIPES={(x['id'],x['version']):x for x in REGISTRY['material_recipes']}
MASSES={x['species_id']:x['quantity']['value'] for x in REGISTRY['phase_thermodynamics'] if x['property']=='molar_mass'}

def additive_ledger(rows):
    """These cases have no chemical conversion of resolved carriers.

    Reconstruct explicit operator additions and material event components;
    audit aggregate unresolved mass separately, never add provenance lots a
    second time. NaCl/NaBr dissolution conserves each introduced ion. Reject
    any unhandled source/sink instead of silently assuming it is zero.
    """
    expected = collections.defaultdict(lambda: collections.defaultdict(float))
    unresolved = collections.defaultdict(lambda: collections.defaultdict(float))
    clocks = collections.defaultdict(float)
    trace=[]
    # Exhaustive for this no-export/no-resolved-reaction review domain.
    # Unknown future bookkeeping/results stop rather than become authority.
    allowed_events={'added','observed','material_added','solution_routed','measured','material_layers_formed','vessel_created','temperature_changed','enzyme_hydrolysed','polymer_heated','emulsion_changed','stirred','energy_transferred','dissolved','solution_characterized','polymer_swelled','surface_spread','chromatographed'}
    for i,row in enumerate(rows):
        op=row['operator']; vid=op.get('vessel')
        if op['op']=='add':
            sp=op['species']; n=op['moles']
            if sp in ('NaCl','NaBr'):
                expected[vid]['Na+']+=n; expected[vid]['Cl-' if sp=='NaCl' else 'Br-']+=n
            else: expected[vid][sp]+=n
        for e in row['events']:
            assert e['event'] in allowed_events, (i,e)
            if e['event']=='dissolved': assert e['species'] in {'NaCl','NaBr','sucrose'}, (i,e)
            if e['event']=='material_added':
                assert e['basis']=='mass_fraction'
                recipe=RECIPES[(e['recipe_id'],e['recipe_version'])]
                dose=op['total_amount']; assert op['op']=='add_material'
                assert close(dose,e['total_amount'])
                assert len(recipe['components'])==len(e['components'])
                for comp,observed in zip(recipe['components'],e['components']):
                    fraction=comp['fraction']; assert fraction['lower']==fraction['upper']
                    assert observed['species']==comp['species_id']
                    assert close(observed['moles'],dose*fraction['lower']/MASSES[comp['species_id']])
                uf=recipe['unresolved_fraction']; assert uf is None or uf['lower']==uf['upper']
                assert close(e['unresolved_amount'],dose*(uf['lower'] if uf else 0))
                for p in e['components']: expected[e['vessel']][p['species']]+=p['moles']
                unresolved[e['vessel']][(e['recipe_id'],e['recipe_version'],e['basis'])]+=e['unresolved_amount']
        if op['op']=='wait':
            for v in row['bench']['vessels']: clocks[v['id']]+=op['seconds']
        elif op['op']=='stir': clocks[vid]+=op['seconds']
        residual=0.; ures=0.
        snapshots=[]
        for v in row['bench']['vessels']:
            vi=v['id']; actual=collections.defaultdict(float)
            for p in v['contents']: actual[p['species']]+=p['moles']
            for sp in set(actual)|set(expected[vi]):
                diff=abs(actual[sp]-expected[vi][sp]); residual=max(residual,diff)
                assert close(actual[sp],expected[vi][sp]), (i,vi,sp,actual[sp],expected[vi][sp])
            au=collections.defaultdict(float)
            for p in v.get('unresolved_materials',[]): au[(p['recipe_id'],p['recipe_version'],p['basis'])]+=p['amount']
            for key in set(au)|set(unresolved[vi]):
                diff=abs(au[key]-unresolved[vi][key]); ures=max(ures,diff)
                assert close(au[key],unresolved[vi][key]), (i,vi,key,au[key],unresolved[vi][key])
            assert close(v['elapsed_seconds'],clocks[vi]),(i,vi,'clock',v['elapsed_seconds'],clocks[vi])
            assert not any(p['phase']=='gas' for p in v['contents'])
            snapshots.append({'vessel':vi,'elapsed_s':v['elapsed_seconds'],'temperature_k':v['temperature'],'resolved_moles':dict(actual),'unresolved_g':{'|'.join(map(str,k)):n for k,n in au.items()}})
        trace.append({'row':i,'raw_jsonl_line':row['raw_jsonl_line'],'step':row['step'],'operator':op,'max_resolved_residual_mol':residual,'max_unresolved_residual_g':ures,'vessels':snapshots})
    return {'method':'independent per-row operator/component/unresolved ledger with recipe fractions and molecular masses independently rederived from pinned registry;  no resolved reactions, transfers, gas or hidden export admitted','max_resolved_residual_mol':max(t['max_resolved_residual_mol'] for t in trace),'max_unresolved_residual_g':max(t['max_unresolved_residual_g'] for t in trace),'all_operator_clocks_checked':True,'trace':trace}

def event_list(rows,name): return [(i,e) for i,r in enumerate(rows) for e in r['events'] if e['event']==name]
def measured(rows,instrument): return [(i,e) for i,e in event_list(rows,'measured') if e['instrument']==instrument]

def specific(k,rows):
    ev=lambda n:event_list(rows,n)
    final=rows[-1]['bench']['vessels']
    # Events must agree with actual owned state, not merely repeat a number.
    for row_i,row in enumerate(rows):
        for e in row['events']:
            v=next((v for v in row['bench']['vessels'] if v['id']==e.get('vessel')),None)
            if e['event']=='enzyme_hydrolysed':
                matches=[p for p in v.get('unresolved_materials',[]) if p.get('enzyme_hydrolysis',{}).get('family')==e['family'] and close(p.get('enzyme_hydrolysis',{}).get('converted_fraction',-1),e['converted_fraction'])]
                assert matches,(row_i,e,'missing owned hydrolysis')
            if e['event']=='emulsion_changed':
                # Multiple transitions may occur inside one operation; last one owns the snapshot.
                later=[x for x in row['events'] if x['event']=='emulsion_changed' and x['vessel']==e['vessel']]
                last=later[-1]
                assert close(v['emulsion']['dispersed_volume_l'],last['dispersed_volume_l'])
            if e['event']=='surface_spread': assert close(v['surface_particles']['cleared_fraction'],e['to_cleared_fraction'])
            if e['event']=='polymer_heated':
                assert close(v['temperature'],e['temperature'])
                recipe=RECIPES[('polymer/thermoplastic-sheet' if e['vessel']==0 else 'polymer/thermoset-resin',1)]
                role=next(x for x in recipe['roles'] if x['kind']=='polymer_heat_response')
                if e['state']=='softened':assert role['softens_above_k'] is not None and v['temperature']>=role['softens_above_k']
                if e['state']=='rigid':assert v['temperature']< (role['softens_above_k'] or role['chars_above_k'])

    if k=='en-003':
        e=ev('enzyme_hydrolysed');assert len(e)==1 and e[0][1]['vessel']==0 and e[0][1]['family']=='protease';a=e[0][1];assert close(a['hydrolysed_mass_g'],.9*a['converted_fraction']) and 0<a['converted_fraction']<1
        assert all(not v.get('unresolved_materials',[{}])[0].get('enzyme_hydrolysis') for v in final[1:])
        return 'Protease alone updates the retained 0.9 g gelatin aggregate; lactase and blank retain no conversion state. No gel-liquefaction, peptide-product or empirical-rate claim.',e
    if k=='en-007':
        e=ev('enzyme_hydrolysed');assert len(e)==3 and all(x[1]['vessel']==0 for x in e)
        total=sum(x[1]['hydrolysed_mass_g'] for x in e);p=final[0]['unresolved_materials'];assert len(p)==2;assert close(total,sum(x['amount']*x.get('enzyme_hydrolysis',{}).get('converted_fraction',0) for x in p));assert .224<total<.45
        assert not any(x.get('enzyme_hydrolysis') for x in final[1]['unresolved_materials'])
        return 'Two distinct 0.225 g retained gelatin pools own separate conversion states. Summed incremental hydrolysed mass equals final mass-weighted conversion; fresh substrate responds, blank does not. No reversed conversion or enzyme growth.',e
    if k=='co-001':
        e=ev('emulsion_changed');assert len(e)==1 and e[0][1]['vessel']==0 and close(e[0][1]['to_dispersed_fraction'],.92);assert all(not v.get('emulsion') for v in final[1:])
        return 'Only oil+surfactant+stir yields dispersion; oil-only stirring and unstirred oil+surfactant remain negative. Oil remains conserved unresolved material.',e
    if k=='co-002':
        e=ev('emulsion_changed');assert len(e)==4;assert close(e[1][1]['to_dispersed_fraction'], e[0][1]['to_dispersed_fraction']*.25);assert close(e[-1][1]['to_dispersed_fraction'],e[0][1]['to_dispersed_fraction']);assert close(final[0]['elapsed_seconds'],660)
        return 'Dispersion falls by 2 half-lives over600 s, then declines during the second30 s stir before being restored; no new oil is added or destroyed.',e
    if k=='co-007':
        e=ev('surface_spread');assert len(e)==1 and e[0][1]['vessel']==0 and 0<e[0][1]['to_cleared_fraction']<1;assert final[1]['surface_particles']['cleared_fraction']==0
        return 'Detergent creates a bounded surface-clearing state; equal-mass water control leaves cleared fraction zero. Pepper stays unresolved, no chemical reaction or particle velocity inferred.',e
    if k=='po-005':
        e=ev('polymer_heated');a=[x[1]['state'] for x in e if x[1]['vessel']==0];b=[x[1]['state'] for x in e if x[1]['vessel']==1];assert a==['rigid','softened','rigid'] and b==['rigid']*3
        assert all(close(v['temperature'],293.15) for v in final);energy=ev('energy_transferred');assert len(energy)==4 and all(close(x[1]['delivered_j'],1300) and close(x[1]['sensible_j'],1300) for x in energy)
        # Independently reconstruct all four sensible energies from owned pre/post state.
        for row_i,receipt in energy:
            before=rows[row_i-1]['bench']['vessels'][receipt['vessel']];after=rows[row_i]['bench']['vessels'][receipt['vessel']]
            if receipt['vessel']==0:
                cp=next(x['quantity']['value'] for x in REGISTRY['phase_thermodynamics'] if x['species_id']=='PE' and x['property']=='molar_heat_capacity')
                capacity=sum(p['moles']*cp for p in before['contents'])
            else:
                role=next(x for x in RECIPES[('polymer/thermoset-resin',1)]['roles'] if x['kind']=='polymer_heat_response')
                capacity=sum(p['amount'] for p in before['unresolved_materials'])*role['specific_heat_j_per_g_k']
            assert close(abs(after['temperature']-before['temperature'])*capacity,1300)
        return 'Named PE article crosses403.15 K then returns rigid; thermoset remains below573.15 K and never becomes melt-processable. Both initial temperatures and all4 energy receipts retained. Editorial thresholds, no geometry/DSC prediction.',e+energy
    if k=='po-006':
        e=ev('polymer_swelled');assert len(e)==2 and close(e[0][1]['retained_water_g'],10) and close(e[1][1]['retained_water_g'],20);assert not e[0][1]['saturated'] and e[1][1]['saturated'];assert close(final[0]['contents'][0]['moles']*18.015,40)
        return '0.2 g unresolved polymer retains10 g then capacity-limited20 g of40 g final water. Retention is a nonconsuming projection; balance excludes0.2 g unresolved polymer and is not treated as total physical mass.',e
    if k=='po-010':
        e=ev('polymer_heated');assert len(e)==2 and all(x[1]['vessel']==0 for x in e);assert e[-1][1]['state']=='softened';assert close(final[0]['temperature'],final[1]['temperature']);assert close(final[0]['contents'][0]['moles'],final[1]['contents'][0]['moles'])
        return 'Named article and bare PE retain equal PE dose and final temperature; only named article has material-role annotation. No claim that bare real PE cannot melt.',e+measured(rows,'thermometer')
    if k=='su-006':
        c=measured(rows,'conductivity_meter');d=measured(rows,'densitometer');assert len(c)==3 and c[0][1]['value']<c[1][1]['value']<c[2][1]['value'];assert len(d)==2 and all(close(x[1]['value'],2.33) for x in d)
        return 'Specified doped teaching-grade conductivity exceeds intrinsic silicon, copper exceeds both; both silicon densities2.33 g/mL. Unresolved1 g specimens retained. No Hall/carrier/activation or arbitrary doping claim.',c+d
    if k=='an-008':
        c=measured(rows,'conductivity_meter');uv=measured(rows,'spectrophotometer');assert len(c)==3 and min(x[1]['value'] for x in c[:2])>1000*c[2][1]['value'];assert len(uv)==3 and all(x[1]['value']==0 for x in uv)
        return 'Both halide controls conduct far above sucrose and all405 nm absorbances arezero; Cl andBr inventories stay distinct. Actual298.131/298.150 K temperatures retained, no inverse identification/calibration claim.',c+uv
    if k=='bd-001':
        e=ev('polymer_swelled');c=measured(rows,'conductivity_meter');assert len(e)==2 and all(close(x[1]['retained_water_g'],20) for x in e);assert c[1][1]['value']>1000*c[0][1]['value'];assert final[1]['solution']['ionic_strength']>.5
        return 'Original explicitly scoped boundary: retained water stays20 g after salt despite independent ionic-strength/conductivity contrast; formula has no salt argument. Does not establish real gel salt insensitivity.',e+c
    if k=='bd-002':
        e=ev('enzyme_hydrolysed');m=ev('emulsion_changed');assert len(e)==4 and len(m)==2;assert close(e[0][1]['converted_fraction'],e[1][1]['converted_fraction']);assert close(e[2][1]['converted_fraction'],e[3][1]['converted_fraction']);assert all(x[1]['vessel']==0 for x in m);assert all(close(v['elapsed_seconds'],1830) for v in final)
        return 'Original explicitly scoped boundary:1830 s conversion is equal despite surfactant-only dispersion; own-vessel30 s stirs followed by shared1800 s wait. No interfacial rate law or detergent compatibility implied.',e+m
    if k=='bd-004':
        e=ev('chromatographed');assert len(e)==1;a=e[0][1];assert [p['species'] for p in a['peaks']]==['ethanol'];assert set(a['outside_method'])=={'Na+','Cl-'};assert close(final[0]['unresolved_materials'][0]['amount'],.188)
        return 'Original explicitly scoped boundary: ethanol-only peak coexists with1 mmol each Na/Cl and0.188 g unresolved protein plus0.012 g recipe water. Peak omission cannot establish ingredient absence. Balance excludes unresolved protein; no total-mass certification.',e+measured(rows,'conductivity_meter')+measured(rows,'balance')
    raise ValueError(k)

def main():
    binding=json.loads((BASE/'native-screen-results/binding.json').read_text());assert binding['engine']['source_head']==SOURCE
    originals=[x for x in json.loads((BASE/'intake.json').read_text()) if x['original_id'].startswith('batch3-diverse:')];assert len(originals)==100
    detailed={'en-003','en-007','co-001','co-002','co-007','po-005','po-006','po-010','su-006','an-008','bd-001','bd-002','bd-004'}
    review=[]; index=[]
    archive=BASE/'native-screen-results/raw-artifact.zip'
    verification=json.loads((BASE/'native-screen-results/verification.json').read_text())
    assert sha(archive.read_bytes())==verification['artifact_sha256']
    with zipfile.ZipFile(archive) as z:
        for original in originals:
            oid=original['original_id'];k=oid.split(':')[1];prefix=oid.replace(':','_')
            supplied=(BASE/original['original_script']).read_bytes();assert sha(supplied)==original['script_sha256']
            summary=json.loads((BASE/'native-screen-results/cases'/f'{prefix}.json').read_text());execution=json.loads(z.read(prefix+'/execution.json'))
            hashes={}
            for filename,key in [('experiment.lab','script_sha256'),('stdout.jsonl','stdout_sha256'),('stderr.txt','stderr_sha256')]:
                raw=z.read(prefix+'/'+filename);h=sha(raw);assert h==execution[key]==summary['execution'][key];hashes[filename]=h
            if k.startswith('so-00') and k[-1] in '1234':
                original_same=hashes['experiment.lab']==original['script_sha256']
            else:
                assert hashes['experiment.lab']==original['script_sha256']
                assert supplied==z.read(prefix+'/experiment.lab');original_same=True
            assert summary['engine']==binding['engine']
            entry={'original_id':oid,'original_script_sha256':original['script_sha256'],'executed_script_sha256':hashes['experiment.lab'],'unchanged_original_script':original_same,'raw_member_prefix':prefix,'raw_sha256':hashes,'screen_only_not_scientific_pass':k not in detailed,'model_limits':summary['named_model_limits'],'solver_failures':summary['solver_failures']}
            index.append(entry)
            if k not in detailed:continue
            raw_rows=[json.loads(line) for line in z.read(prefix+'/stdout.jsonl').splitlines()]
            rows=[dict(row,raw_jsonl_line=i+1) for i,row in enumerate(raw_rows) if 'bench' in row]
            assert execution['exit_code']==0 and not summary['solver_failures']
            ledger=additive_ledger(rows);interpretation,evidence=specific(k,rows)
            scenes=[{'raw_jsonl_line':i+1,'scene':r['scene']} for i,r in enumerate(raw_rows) if 'scene' in r]
            final_scene=scenes[-1]['scene']['vessels'] if scenes else []
            if k in {'po-006','bd-001'}:
                assert final_scene
                for v in final_scene: assert close(v['swelling']['retained_water_g'],20)
            if k=='en-003':
                assert close(final_scene[0]['enzyme_hydrolysis'][0]['converted_fraction'],rows[-1]['bench']['vessels'][0]['unresolved_materials'][0]['enzyme_hydrolysis']['converted_fraction'])
                assert all(not v.get('enzyme_hydrolysis') for v in final_scene[1:])
            if k in {'co-001','co-002','bd-002'}:
                assert final_scene
                for v in final_scene:
                    owned=rows[-1]['bench']['vessels'][v['id']].get('emulsion')
                    if owned: assert close(v['emulsion']['dispersed_volume_l'],owned['dispersed_volume_l'])
                    else: assert not v.get('emulsion')
            if k=='co-007':
                assert close(final_scene[0]['surface_particles']['cleared_fraction'],.09)
                assert final_scene[1]['surface_particles']['cleared_fraction']==0

            assessment={**entry,'title':original['title'],'original_forecast':original['original_expected_result'],'original_limitations':original['original_limitations'],'original_checks':original['original_acceptance_checks'],'verdict':('boundary_demonstration_verified_pending_parent_review_not_scientific_selection' if k.startswith('bd-') else 'candidate_all_original_bounded_criteria_met_pending_parent_independent_review'),'interpretation':interpretation,'per_criterion':{check['id']:{'original_criterion':check['criterion'],'original_review_instruction':check['review_instruction'],'assessment': ('All rows independently audited against explicit doses and pinned recipe components; retained unresolved masses separately conserved. See inventory_trace. No gas, transfer, sink or resolved reaction accepted.' if check['id']=='inventory' else 'Limited exactly to original model question and original limitations; editorial/aggregate descriptions are not empirical calibration or molecular products.' if check['id']=='coverage' else interpretation),'result':'candidate_met_pending_independent_parent_review'} for check in original['original_acceptance_checks']},'raw_coordinates':[{'row':i,'raw_jsonl_line':rows[i]['raw_jsonl_line'],'step':rows[i]['step'],'event':e} for i,e in evidence],'inventory_trace':ledger,'final_scene_evidence':scenes[-1] if scenes else None}
            dump(OUT/f'{k}.json',assessment);review.append({'original_id':oid,'assessment':k+'.json','status':assessment['verdict']})
    source_paths=['crates/kerotakis-core/src/'+s for s in ['enzyme_activity.rs','emulsion.rs','swelling.rs','surface_spread.rs','plastics.rs','conductivity.rs','vessel.rs','bench.rs','material.rs']]
    dump(OUT/'source-binding.json',{'engine':binding['engine'],'raw_archive_sha256':sha(archive.read_bytes()),'intake_sha256':sha((BASE/'intake.json').read_bytes()),'raw_Cargo_lock_not_available':True,'source_blobs':[{'path':p,'sha256':sha(source(p))} for p in source_paths+['data/registry/registry-source-v1.json']],'no_engine_execution':True,'no_catalog_selection':True})
    dump(OUT/'all100-authenticated-index.json',index);dump(OUT/'candidate-index.json',review)
    print(json.dumps({'all100_raw_authenticated':len(index),'detailed_candidates_for_parent_review':len(review),'engine_executions':0,'catalog_edits':0}))
if __name__=='__main__':main()
