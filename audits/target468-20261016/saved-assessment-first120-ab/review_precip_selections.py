#!/usr/bin/env python3
"""Independent saved-only sibling review. Prints receipt; never writes sibling files."""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import zipfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
BASE=HERE.parent/'common-engine-completion'
SIBLING=HERE.parent/'saved-assessment-first120-precip'
PIN='c44692053a94c59dcf62e7fb67549f84c7d18f83'
def sha(raw):return hashlib.sha256(raw).hexdigest()
def strict_pairs(pairs):
    result={}
    for k,v in pairs:
        assert k not in result
        result[k]=v
    return result

def loads(raw):
    def bad(value):raise ValueError(value)
    return json.loads(raw,object_pairs_hook=strict_pairs,parse_constant=bad)

snapshots={name:(SIBLING/name).read_bytes() for name in ['README.md','assessment.json','assess.py']}
sibling=loads(snapshots['assessment.json'])
original_raw=(BASE/'originals/new-experiments-20261016.json').read_bytes()
assert sha(original_raw)=='8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd'
originals={c['id']:c for c in loads(original_raw)['experiments']}
archive_raw=(BASE/'results/raw-artifact.zip').read_bytes()
assert sha(archive_raw)=='9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f'
verified=loads((BASE/'results/verified-results.json').read_bytes())
metadata=loads((BASE/'results/artifact-metadata.json').read_bytes())
assert metadata['digest']=='sha256:'+sha(archive_raw)== 'sha256:'+verified['raw_artifact_sha256']
receipts={c['id']:c for c in verified['cases']}
# Shared saved-analysis helpers; no main(), engines, imports of product code or writes.
helper_raw=(HERE/'review_saved.py').read_bytes()
helper=runpy.run_path(str(HERE/'review_saved.py'))
assert verified['engine']==helper['ENGINE']
helper['FORMULAS'].update({
 'Zn+2':{'Zn':1},'Zn(OH)2':{'Zn':1,'O':2,'H':2},'ZnSO4':{'Zn':1,'S':1,'O':4},
 'Cu+2':{'Cu':1},'CuO':{'Cu':1,'O':1},'CuSO4':{'Cu':1,'S':1,'O':4},'Cu(OH)2':{'Cu':1,'O':2,'H':2},
 'brochantite':{'Cu':4,'S':1,'O':10,'H':6},'antlerite':{'Cu':3,'S':1,'O':8,'H':4},
 'atacamite':{'Cu':2,'Cl':1,'O':3,'H':3},'Ag+':{'Ag':1},'AgCl':{'Ag':1,'Cl':1},
 'AgNO3':{'Ag':1,'N':1,'O':3},'NaNO3':{'Na':1,'N':1,'O':3},'NO3-':{'N':1,'O':3},
 'NaCl':{'Na':1,'Cl':1},'Na2SO4':{'Na':2,'S':1,'O':4},
 'Ba+2':{'Ba':1},'BaSO4':{'Ba':1,'S':1,'O':4},'BaCl2':{'Ba':1,'Cl':2},
 'SiO2':{'Si':1,'O':2},'ethanol':{'C':2,'H':6,'O':1}})
helper['CHARGE'].update({'Zn+2':2,'Cu+2':2,'Ag+':1,'Ba+2':2,'NO3-':-1})
# inventory() reads KEYS from the original globals; mutate list in place.
helper['KEYS'].extend(['Zn','Cu','Ag','Ba','Si'])
amount=helper['amount'];state_at=helper['state_at'];inventory=helper['inventory'];add=helper['add'];keys=helper['KEYS']
source_pins={}
for path,expected in [('crates/kerotakis-core/src/nonaqueous.rs','df2dc81f729cafd5312df546e868f9b0ff950818bcb2f84e332c8a9b022b4d3d'),('crates/kerotakis-core/src/curated.rs','20db49d2b0679fb77e1df7205f6a4ae8878f8becccf7c30cebb1c9796bcd27a5')]:
    raw=subprocess.check_output(['git','-C',str(ROOT),'show',PIN+':'+path]);assert sha(raw)==expected
    source_pins[path]=expected
records=[]
with zipfile.ZipFile(BASE/'results/raw-artifact.zip') as z:
    assert len(z.namelist())==len(set(z.namelist()))
    assert loads(z.read('binding.json'))==verified['binding']
    design_raw=(BASE/'native-screen-design.json').read_bytes();archived=z.read('design.json')
    assert sha(design_raw)==verified['binding']['design_sha256']
    assert loads(design_raw)==loads(archived)
    design_pins={'original_byte_sha256':sha(design_raw),'archived_byte_sha256':sha(archived),'full_json_value_equal':True}
    selected=[c for c in sibling['cases'] if c['case_status']=='met']
    assert len(selected)==14
    for c in selected:
        original=originals[c['id']];prefix=c['id'].replace(':','_');receipt=loads(z.read(prefix+'/execution.json'))
        raw=z.read(prefix+'/stdout.jsonl');script=z.read(prefix+'/experiment.lab');stderr=z.read(prefix+'/stderr.txt')
        assert sha(raw)==receipt['stdout_sha256']==receipts[c['id']]['stdout_sha256']
        assert sha(stderr)==receipt['stderr_sha256']==receipts[c['id']]['stderr_sha256']
        assert sha(script)==receipt['script_sha256']==original['script_sha256'] and script==original['script'].encode()
        assert all(receipt[k]==receipts[c['id']][k] for k in receipt)
        assert c['original_forecast']==original['expected_result'] and c['script']==original['script']
        for criterion,source in zip(c['criteria'],original['acceptance_checks']):assert all(criterion[k]==v for k,v in source.items())
        rows=[loads(line) for line in raw.splitlines()];assert len(rows)==receipt['parsed_observations'] and receipt['exit_code']==0
        helper['validate_snapshot_integrity'](original,rows)
        assert not any(e['event']=='solver_failed' for r in rows for e in r['events'])
        # Final represented whole-script ledger only, not a per-step/energy proof.
        incoming=dict.fromkeys(keys,0.0);outgoing=dict.fromkeys(keys,0.0)
        for r in rows:
            op=r['operator']
            if op['op']=='add':add(incoming,op['species'],op['moles'])
            for e in r['events']:
                if e['event']=='vessel_sealed':
                    for species,x in [('N2',.7901),('O2',.2095),('CO2',.0004)]:add(incoming,species,e['trapped_air']*x)
                if e['event']=='gas_evolved':add(outgoing,e['species'],e['moles'])
                if e['event']=='gas_absorbed':add(incoming,e['species'],e['moles'])
        final=state_at(rows);retained=inventory(final.values())
        residual={k:retained[k]+outgoing[k]-incoming[k] for k in keys}
        limits=[{'row':i,**e} for i,r in enumerate(rows) for e in r['events'] if e['event']=='not_yet_modeled']
        n=int(c['id'][-3:]);evidence={}
        def at(i,v=0):return state_at(rows,i)[v]
        def solid(v,key):return amount(v,key,'solid')
        def aqueous(v,key):return amount(v,key,'aqueous')
        if n==2:
            solids=[solid(at(i),'Mg(OH)2') for i in [2,4,6]]
            assert solids[0]>solids[1]==0 and solids[2]>solids[1]
            evidence['MgOH_solid_cycle_mol']=solids
        elif n==3:
            assert solid(at(3),'Fe(OH)3')>solid(at(5),'Fe(OH)3')==0 and aqueous(final[0],'Fe+3')>0
            evidence['FeOH_solid_before_after_mol']=[solid(at(i),'Fe(OH)3') for i in [3,5]]
        elif n==4:
            def cu_solid(v):return sum(inventory([{**v,'contents':[q]}])['Cu'] for q in v['contents'] if q['phase']=='solid')
            solids=[cu_solid(at(i)) for i in [4,6]]
            ammines=[{q['name']:q['molality'] for q in at(i)['solution']['species'] if q['name'].startswith('Cu(NH3)')} for i in [4,6]]
            assert solids[0]>solids[1] and not ammines[0] and sum(ammines[1].values())>0
            reagent_n=amount(final[0],{'NH3','NH4+'})
            evidence.update(solid_Cu_before_after_mol=solids,reported_ammine_molalities=ammines,reagent_nitrogen_all_phases_mol=reagent_n)
        elif n==6:
            solids=[solid(final[v],'CaCO3') for v in [0,1]]
            assert solids[0]>solids[1]==0 and aqueous(final[1],'H+')<.003 and outgoing['C']>0
            evidence.update(CaCO3_solid_mol_by_vessel=solids,acidified_residual_H_mol=aqueous(final[1],'H+'),exported_C_mol=outgoing['C'])
        elif n==7:
            solids=[solid(at(i),'CaCO3') for i in [4,6]]
            assert solids[0]>solids[1] and amount(final[0],'CO2','gas')>0
            evidence.update(CaCO3_solid_before_after_mol=solids,sealed_final_C_mol=retained['C'],sealed_final_Ca_mol=retained['Ca'])
        elif n==9:
            ca=solid(final[0],'CaSO4');ba=solid(final[1],'BaSO4')
            assert ca==0 and ba>0 and aqueous(final[0],'Ca+2')>0 and aqueous(final[0],'SO4-2')>0
            evidence.update(CaSO4_solid_mol=ca,BaSO4_solid_mol=ba,dissolved_Ca_mol=aqueous(final[0],'Ca+2'),dissolved_sulfate_mol=aqueous(final[0],'SO4-2'))
        elif n==10:
            silver=[aqueous(final[v],'Ag+') for v in [0,1,2]]
            activities=[next(q['activity'] for q in final[v]['solution']['species'] if q['name']=='Ag+') for v in [0,1,2]]
            assert all(x>0 for x in silver+activities) and silver[0]-silver[2]>abs(silver[1]-silver[0]) and activities[2]<activities[0]
            evidence.update(dissolved_Ag_water_nitrate_chloride_mol=silver,reported_free_Ag_activity=activities)
        elif n==11:
            assert solid(at(3),'AgCl')==0 and solid(at(6),'AgCl')>0 and aqueous(at(6),'Ag+')>0 and aqueous(at(6),'Cl-')>0
            evidence['AgCl_solid_trace_later_mol']=[solid(at(i),'AgCl') for i in [3,6]]
        elif n==12:
            assert aqueous(final[1],'Ca+2')>aqueous(final[0],'Ca+2') and solid(final[0],'BaSO4')>aqueous(final[1],'Ba+2') and outgoing['C']>0
            evidence.update(filtrate_Ca_mol=aqueous(final[1],'Ca+2'),retained_BaSO4_mol=solid(final[0],'BaSO4'),exported_C_mol=outgoing['C'])
        elif n==13:
            assert solid(final[0],'AgCl')>0 and solid(final[0],'BaSO4')>0 and all(aqueous(final[1],key)>0 for key in ['Na+','NO3-','Cl-'])
            evidence.update(source_AgCl_mol=solid(final[0],'AgCl'),source_BaSO4_mol=solid(final[0],'BaSO4'),filtrate_spectators_mol={key:aqueous(final[1],key) for key in ['Na+','NO3-','Cl-']})
        elif n==14:
            assert amount(final[0],'SiO2')>amount(final[1],'SiO2') and solid(final[1],'CaCO3')>0
            evidence.update(Si_source_receiver_mol=[amount(final[v],'SiO2') for v in [0,1]],receiver_CaCO3_mol=solid(final[1],'CaCO3'))
        elif n==15:
            assert solid(at(1),'CuO')>solid(at(3),'CuO')==0 and solid(final[0],'Cu(OH)2')>0
            evidence.update(CuO_before_after_acid_mol=[solid(at(i),'CuO') for i in [1,3]],final_CuOH_solid_mol=solid(final[0],'Cu(OH)2'))
        record={'id':c['id'],'original_forecast':original['expected_result'],'original_criteria':original['acceptance_checks'],
            'verified_hashes':{'script':sha(script),'stdout':sha(raw),'stderr':sha(stderr),'execution_receipt':sha(z.read(prefix+'/execution.json'))},
            'named_limits':limits,'independent_positive_numeric_checks':evidence,'final_whole_script_material_charge_residual_mol':residual,
            'max_abs_residual_mol':max(abs(v) for v in residual.values()),
            'phase_ownership':[{'vessel':v['id'],'contents':v['contents']} for v in final.values()],
            'independent_disposition':'bounded_criteria_supported_for_root_consideration',
            'scope':'Represented positive phases/metal ownership only; all original limitations retained. No full phase/heat/empirical validation.'}
        if c['id']=='new120:pr-001':
            before=rows[3]['bench']['vessels'][0];after=rows[5]['bench']['vessels'][0]
            b=amount(before,'Zn(OH)2','solid');a=amount(after,'Zn(OH)2','solid')
            zinc=[amount(v,{'Zn+2','Zn(OH)2'}) for v in [before,after]]
            complexes={p['name']:p['molality'] for p in after['solution']['species'] if p['name'] in ['Zn(OH)3-','Zn(OH)4-2']}
            assert b>a>0 and all(v>0 for v in complexes.values()) and set(complexes)=={'Zn(OH)3-','Zn(OH)4-2'}
            record['partial_redissolution']={'solid_before_mol':b,'solid_after_mol':a,'decrease_mol':b-a,'decrease_fraction':(b-a)/b,'total_owned_zinc_mol':zinc,'reported_hydroxo_complex_molality':complexes}
            record['reason']='Original C2 requires a decrease, not clearing or a minimum fraction. A 2.4904% decrease with supported hydroxo-complex distribution and zinc retention is substantive bounded partial redissolution. Neither the original forecast nor checks require complete clearing; do not rename this result complete dissolution.'
        elif c['id']=='new120:pr-016':
            v=final[1];conversion=amount(v,'AgCl','solid')/.001
            record['independent_disposition']='hold_original_C1_inconclusive_and_nonaqueous_provenance_unsupported'
            record['reason']='AgCl products and unequal represented conversion are positive, but C1 cannot establish a separately owned undissolved-salt fraction from residual liquid-labelled salts. Source c446 nonaqueous.rs36-65 explicitly withdraws all historical handbook citations; NaCl/ethanol.065g/100mL and AgNO3/ethanol2.1g/100mL rows77-99 claim NO SOURCE. curated.rs682-722 still uses those magnitudes in available_dissolved to cap the metathesis extent. No named raw limit is emitted here. This is unearned quantitative-solubility support, not absence of all chemistry or a conservation defect; do not select a working chemistry case from honesty/reporting alone.'
            record['actual_ethanol_conversion_fraction']=conversion
            record['residual_starting_salts']=[p for p in v['contents'] if p['species'] in ['AgNO3','NaCl']]
            record['next_action']='Keep first output. Separately review nonaqueous solubility provenance admission and phase-resolved residual ownership; no new experiment or numerical tolerance is authorised by this review.'
        elif limits:
            record['scope']='Criterion-scoped represented equilibrium/phase trajectory only. All listed phase exclusions remain; no complete physical phase assemblage, missing spectra, kinetic or thermal claim.'
        records.append(record)
result={'scope':'Independent review of sibling14 proposed first120 precipitation selections. Retrospective source/saved JSON only, no engines/network/builds/catalog changes.',
        'sibling_snapshot_sha256':{k:sha(v) for k,v in snapshots.items()},'shared_AB_reader_sha256':sha(helper_raw),
        'original_sha256':sha(original_raw),'archive_sha256':sha(archive_raw),'design':design_pins,
        'engine':verified['engine'],'source_file_sha256_at_engine_commit':source_pins,
        'lock_binary_qualification':'Source/binary/lock identities are bound to the authenticated producer receipt; raw executable and Cargo.lock bytes absent from the case archive.',
        'proposed_supported_ids':sorted(r['id'] for r in records if r['independent_disposition']=='bounded_criteria_supported_for_root_consideration'),
        'held_ids':sorted(r['id'] for r in records if r['independent_disposition']!='bounded_criteria_supported_for_root_consideration'),
        'selection_qualification':'Root owns final selections. PR003/004/012/015 require the exact named phase/spectrum/reference-temperature limitations below; matching literal honesty guards alone never earns selection. These13 have positive phase/equilibrium evidence beyond honesty guards.',
        'material_qualification':'Final whole-script represented H/O/C/N/P/S/Na/Cl/Ca/Mg/Fe/Zn/Cu/Ag/Ba/Si and charge ledger, with neutral chemical formulae and actual trapped-air/exports. Diagnostic H/OH, ligand-distribution tables and lots are not extra owned matter. Not per-step, full physical-phase or energy closure.',
        'new_engine_executions':0,'catalog_additions':0,'cases':records}
print(json.dumps(result,indent=2,allow_nan=False))
