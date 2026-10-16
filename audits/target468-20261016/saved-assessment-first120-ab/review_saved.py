#!/usr/bin/env python3
"""Saved bytes only: authenticate and assess original first120 acid-base cases."""
import collections
import hashlib
import json
import math
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / 'common-engine-completion'
ORIGINAL_SHA = '8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd'
ZIP_SHA = '9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f'
ENGINE = {'source_head': 'c44692053a94c59dcf62e7fb67549f84c7d18f83',
          'source_tree': 'f17216653825f3149bb7a0e46946834ee5c7e3df',
          'binary_sha256': '670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f',
          'cargo_lock_sha256': '82f3196fe4feca05ff6eac43cfe7d46163d90e3f5c6ce0444edcf0245a023229'}
# Formula counts for represented portions and literal operator inputs only.
# Measured free_proton/free_hydroxide, analytical solute_charge, solution species,
# historical lots and scene descriptions are NOT additional owned matter.
FORMULAS = {
 'water': {'H':2,'O':1}, 'H+':{'H':1}, 'base_equivalents':{'H':1,'O':1},
 'Na+':{'Na':1},'Cl-':{'Cl':1},'Ca+2':{'Ca':1},'Mg+2':{'Mg':1},'Fe+3':{'Fe':1},
 'CH3COOH':{'C':2,'H':4,'O':2},'CH3COO-':{'C':2,'H':3,'O':2},
 'CO2':{'C':1,'O':2},'CO2(aq)':{'C':1,'O':2},'HCO3-':{'H':1,'C':1,'O':3},'CO3-2':{'C':1,'O':3},
 'NH3':{'N':1,'H':3},'NH4+':{'N':1,'H':4},'N2':{'N':2},'O2':{'O':2},
 'H3PO4':{'H':3,'P':1,'O':4},'H2PO4-':{'H':2,'P':1,'O':4},'HPO4-2':{'H':1,'P':1,'O':4},'PO4-3':{'P':1,'O':4},
 'SO4-2':{'S':1,'O':4},'Fe(OH)3':{'Fe':1,'O':3,'H':3},
 'HCl':{'H':1,'Cl':1},'NaOH':{'Na':1,'O':1,'H':1},'NaCl':{'Na':1,'Cl':1},
 'NaOAc':{'Na':1,'C':2,'H':3,'O':2},'NH4Cl':{'N':1,'H':4,'Cl':1},
 'NaHCO3':{'Na':1,'H':1,'C':1,'O':3},'Na2CO3':{'Na':2,'C':1,'O':3},
 'H2SO4':{'H':2,'S':1,'O':4},'FeCl3':{'Fe':1,'Cl':3},'CaCl2':{'Ca':1,'Cl':2},
 'Ca(OH)2':{'Ca':1,'O':2,'H':2},'Mg(OH)2':{'Mg':1,'O':2,'H':2},'CaCO3':{'Ca':1,'C':1,'O':3}}
CHARGE = {'H+':1,'base_equivalents':-1,'Na+':1,'Cl-':-1,'Ca+2':2,'Mg+2':2,'Fe+3':3,
 'CH3COO-':-1,'HCO3-':-1,'CO3-2':-2,'NH4+':1,'H2PO4-':-1,'HPO4-2':-2,'PO4-3':-3,'SO4-2':-2}
KEYS = ['H','O','C','N','P','S','Na','Cl','Ca','Mg','Fe','charge']

def sha(b): return hashlib.sha256(b).hexdigest()
def add(out, species, moles):
    assert math.isfinite(moles)
    for k,n in FORMULAS[species].items(): out[k] += n*moles
    out['charge'] += CHARGE.get(species,0)*moles

def inventory(vessels):
    out = dict.fromkeys(KEYS,0.0)
    for v in vessels:
        assert not v['surfaces'] and not v.get('unresolved_materials')
        for p in v['contents']:
            assert math.isfinite(p['moles']) and p['moles'] >= 0
            add(out,p['species'],p['moles'])
    return out

def state_at(rows, index=-1):
    state = {}
    for r in rows[:len(rows) if index == -1 else index+1]:
        published={v['id']:v for v in r['bench']['vessels']}
        if state.keys()-published.keys():
            assert r['operator']['op']=='inspect' and not r['events']
            assert all(v==state[k] for k,v in published.items())
        state.update(published)
    return state

def amount(v, keys, phase=None):
    if isinstance(keys,str): keys={keys}
    return sum(p['moles'] for p in v['contents'] if p['species'] in keys and (phase is None or p['phase']==phase))
def family(v): return amount(v,{'CH3COOH','CH3COO-'})
def measured(rows, instrument):
    return [(i,e) for i,r in enumerate(rows) for e in r['events'] if e['event']=='measured' and e['instrument']==instrument]
def phs(rows): return [e['value'] for _,e in measured(rows,'ph_meter')]
def at_measure(rows,instrument,n): return state_at(rows,measured(rows,instrument)[n][0])

def ledger(rows):
    incoming=dict.fromkeys(KEYS,0.0); outgoing=dict.fromkeys(KEYS,0.0)
    air=[]; exports=[]
    expected_by_vessel={}
    def book(vessel, species, n):
        expected_by_vessel.setdefault(vessel,dict.fromkeys(KEYS,0.0))
        add(expected_by_vessel[vessel],species,n)
    for i,r in enumerate(rows):
        op=r['operator']
        if op['op']=='add':
            add(incoming,op['species'],op['moles'])
            book(op['vessel'],op['species'],op['moles'])
        # Only AB012 Decant and AB022 Mix use transfers in this fixed cohort.
        # Their donors contain liquid/aqueous portions only. Account exact
        # declared fractions; do not infer transfers from a desired final state.
        previous=state_at(rows,i-1) if i else {}
        if op['op'] in ['decant','mix']:
            transfers=[(op['from'],op['to'],op['fraction'])] if op['op']=='decant' else [
                (op['a'],op['into'],op['fraction_a']),(op['b'],op['into'],op['fraction_b'])]
            for source,destination,fraction in transfers:
                for portion in previous[source]['contents']:
                    assert portion['phase'] in ['liquid','aqueous']
                    n=portion['moles']*fraction
                    book(source,portion['species'],-n)
                    book(destination,portion['species'],n)
        for e in r['events']:
            if e['event']=='diluted':
                add(incoming,'water',e['moles']);book(e['vessel'],'water',e['moles'])
            if e['event']=='vessel_sealed':
                n=e['trapped_air']; air.append({'row':i,'vessel':e['vessel'],'moles':n})
                # c446 bench.rs trap_boundary_gas: 0.7901 N2, .2095 O2, .0004 CO2.
                for species,x in [('N2',.7901),('O2',.2095),('CO2',.0004)]:
                    add(incoming,species,n*x);book(e['vessel'],species,n*x)
            if e['event']=='gas_evolved':
                add(outgoing,e['species'],e['moles']);book(e['vessel'],e['species'],-e['moles']);exports.append({'row':i,**e})
            if e['event']=='gas_absorbed':
                add(incoming,e['species'],e['moles']);book(e['vessel'],e['species'],e['moles'])
    retained=inventory(state_at(rows).values())
    residual={k:retained[k]+outgoing[k]-incoming[k] for k in KEYS}
    per_vessel=[]
    for vessel_id,vessel in state_at(rows).items():
        actual=inventory([vessel]);expected=expected_by_vessel.get(vessel_id,dict.fromkeys(KEYS,0.0))
        delta={k:actual[k]-expected[k] for k in KEYS}
        per_vessel.append({'vessel':vessel_id,'retained_mol_atoms_or_charge':actual,
            'expected_after_inputs_boundary_and_internal_transfers':expected,
            'residual_mol_atoms_or_charge':delta,'max_abs_residual':max(abs(x) for x in delta.values())})
    return {'scope':'Whole-script represented portions, literal additions, diluted water, trapped boundary air and explicit gas transfers. No measured-field/solution/lot double counting. Not an energy ledger.',
            'incoming_mol_atoms_or_charge':incoming,'retained_mol_atoms_or_charge':retained,
            'exported_mol_atoms_or_charge':outgoing,'residual_mol_atoms_or_charge':residual,
            'trapped_air':air,'explicit_exports':exports,
            'per_vessel_ledgers':per_vessel,
            'max_abs_residual':max([abs(x) for x in residual.values()]+[v['max_abs_residual'] for v in per_vessel]),
            'audit_resolution_mol':1e-8,
            'within_existing_inventory_audit_resolution':all(abs(x)<=1e-8 for x in residual.values()) and all(v['max_abs_residual']<=1e-8 for v in per_vessel)}


def validate_snapshot_integrity(case, rows):
    commands=[line.strip() for line in case['script'].splitlines()
              if line.strip() and not line.lstrip().startswith('#') and not line.strip().startswith('register ')]
    assert len(commands)==len(rows)
    state={}
    for command,row in zip(commands,rows):
        published={v['id']:v for v in row['bench']['vessels']}
        assert len(published)==len(row['bench']['vessels'])
        if state.keys()-published.keys():
            words=command.split()
            assert len(words)==2 and words[0]=='inspect' and words[1].startswith('v')
            target=int(words[1][1:])-1
            assert set(published)=={target} and row['operator']['op']=='inspect' and not row['events']
            assert published[target]==state[target]
        state.update(published)

def authenticate():
    original_bytes=(INPUT/'originals/new-experiments-20261016.json').read_bytes()
    assert sha(original_bytes)==ORIGINAL_SHA
    original=json.loads(original_bytes)
    archive=INPUT/'results/raw-artifact.zip'
    meta=json.loads((INPUT/'results/artifact-metadata.json').read_text())
    assert sha(archive.read_bytes())==ZIP_SHA==meta['digest'].removeprefix('sha256:')
    verified=json.loads((INPUT/'results/verified-results.json').read_text())
    assert verified['raw_artifact_sha256']==ZIP_SHA and verified['engine']==ENGINE
    cases={c['id']:c for c in original['experiments']}
    receipts={c['id']:c for c in verified['cases']}
    result=[]
    with zipfile.ZipFile(archive) as z:
        binding=json.loads(z.read('binding.json'))
        assert binding==verified['binding'] and binding['engine']==ENGINE
        for n in range(1,25):
            key=f'new120:ab-{n:03}'; prefix=key.replace(':','_')
            c=cases[key]; receipt=receipts[key]
            raw=z.read(prefix+'/stdout.jsonl'); script=z.read(prefix+'/experiment.lab'); err=z.read(prefix+'/stderr.txt')
            actual=json.loads(z.read(prefix+'/execution.json'))
            assert sha(script)==c['script_sha256']==receipt['script_sha256'] and script==c['script'].encode()
            assert sha(raw)==receipt['stdout_sha256'] and sha(err)==receipt['stderr_sha256']
            for k in ['id','exit_code','script_sha256','stdout_sha256','stderr_sha256','stdout_parse_error']:
                assert actual[k]==receipt[k]
            rows=[json.loads(line) for line in raw.splitlines()]
            assert receipt['exit_code']==0 and receipt['stdout_parse_error'] is None
            assert len(rows)==receipt['parsed_observations']
            validate_snapshot_integrity(c,rows)
            result.append((c,receipt,rows))
    return result,binding


def assess(c,receipt,rows):
    n=int(c['id'][-3:]); p=phs(rows); final=state_at(rows); events=[e for r in rows for e in r['events']]
    limits=[{'row':i,**e} for i,r in enumerate(rows) for e in r['events'] if e['event'] in ['not_yet_modeled','solver_failed']]
    obs={'ph_measurements':[{'row':i,**e,'temperature_K':at_measure(rows,'ph_meter',j)[e['vessel']]['temperature']} for j,(i,e) in enumerate(measured(rows,'ph_meter'))]}
    notes=[]; outcomes=[]
    def check(status,evidence):outcomes.append((status,evidence))
    if n==1:
        k=[e['value'] for _,e in measured(rows,'conductivity_meter')];obs['conductivity_uS_cm']=k
        assert k[2]<k[0] and k[2]<k[3] and all(x>0 for x in k) and p[0]<p[1]<p[2]
        check('met',f'Equivalence conductivity {k[2]:.12g} < initial {k[0]:.12g} and excess-base {k[3]:.12g} uS/cm.')
        check('met',f'Measured pH {p}; equivalence near neutral, subsequent base alkaline; all conductivities positive.')
    elif n==2:
        k=[e['value'] for _,e in measured(rows,'conductivity_meter')];obs['conductivity_uS_cm']=k
        vols=[(i,[{'vessel':v['id'],'liquid_volume_l':v['liquid']['volume_l']} for v in r.get('scene',{}).get('vessels',[]) if v.get('liquid')]) for i,r in enumerate(rows) if r['operator']['op']=='measure']
        obs.update(measurement_scene_volumes=vols,final_half_mmol_conductivity_rise=k[-1]-k[-2],preceding_half_mmol_rise=k[-2]-k[-3],first_two_quarter_mmol_combined_rise=k[2]-k[0])
        assert k[3]>k[0]
        check('met',f'Equivalence {k[3]:.12g} > initial {k[0]:.12g} uS/cm.')
        check('inconclusive','Final 0.5 mmol addition raises conductivity by 1162.2639 uS/cm versus 226.8608 for the preceding 0.5 mmol ending AT equivalence. Earlier well-before-equivalence increments are 0.25 mmol each, not the single equal-size addition specified. Normalised slope supports forecast but does not supply missing matched single-dose control.')
        notes.append('Freeze a separate matched early/final 0.5 mmol-control design, or retain only C1; do not revise the original criterion after observation.')
    elif n==3:
        totals=[sum(amount(v,{'H3PO4','H2PO4-','HPO4-2','PO4-3'}) for v in at_measure(rows,'ph_meter',i).values()) for i in range(len(p))]
        obs['total_phosphorus_mol_at_measurements']=totals
        assert all(a<b for a,b in zip(p,p[1:])) and p[3]>p[1]
        check('met',f'pH increases {p}; half-equivalent states {p[1]} and {p[3]} differ by {p[3]-p[1]:.9g}.')
        check('met' if all(abs(x-.001)<=1e-8 for x in totals) else 'failed',f'Phosphorus amounts {totals}; canonical HPO4-2/PO4-3 counted once.')
    elif n==4:
        assert p[0]<p[1] and p[0]<7
        check('met',f'One-equivalent pH {p[0]}, two-equivalent {p[1]}, rise {p[1]-p[0]:.9g}.')
        s=amount(final[0],'SO4-2');obs['final_sulfate_mol']=s
        check('met' if abs(s-.001)<=1e-8 else 'failed',f'Sulfur retained as sulfate {s:.15g} mol; final pH {p[1]} is not forced to seven.')
    elif n==5:
        assert p[1]>p[0]
        check('met',f'Carbonate pH {p[1]} > bicarbonate {p[0]}.')
        carbon=[amount(final[v],{'CO2','CO2(aq)','HCO3-','CO3-2'}) for v in [0,1]];exports=[sum(e['moles'] for e in events if e['event']=='gas_evolved' and e['species']=='CO2' and e['vessel']==v) for v in [0,1]]
        obs.update(retained_carbon_mol=carbon,exported_CO2_mol=exports,sodium_mol=[amount(final[v],'Na+') for v in [0,1]])
        check('met' if all(abs(a+b-.001)<=1e-8 for a,b in zip(carbon,exports)) else 'failed',f'Retained carbon {carbon}, exported CO2 {exports}; each closes to 1 mmol. Sodium is 1 vs 2 mmol, not matched.')
    elif n==6:
        before=at_measure(rows,'ph_meter',0)[0];obs['buffer_before_acid_mol']={k:amount(before,k) for k in ['NH3','NH4+']};obs['buffer_after_acid_mol']={k:amount(final[0],k) for k in ['NH3','NH4+']}
        assert all(x>0 for x in obs['buffer_before_acid_mol'].values())
        check('met',f'Half-neutralisation retains {obs["buffer_before_acid_mol"]}, including all-phase NH3.')
        assert all(x>0 for x in obs['buffer_after_acid_mol'].values())
        nitrogen=amount(final[0],{'NH3','NH4+'});obs['reagent_nitrogen_all_phases_mol']=nitrogen
        check('met' if p[1]<p[0] and abs(nitrogen-.002)<=1e-8 else 'failed',f'pH {p[0]} -> {p[1]}; NH3 gas {amount(final[0],"NH3","gas"):.12g} mol included; reagent nitrogen {nitrogen:.15g} plus separately trapped N2.')
    elif n==7:
        assert p[1]<p[0]
        check('met',f'NH3/NH4Cl pH {p[1]} < NH3-only {p[0]}.')
        nitrogen=[amount(final[v],{'NH3','NH4+'}) for v in [0,1]];obs['reagent_nitrogen_all_phases_mol']=nitrogen
        check('met' if abs(nitrogen[0]-.001)<=1e-8 and abs(nitrogen[1]-.002)<=1e-8 else 'failed',f'NH3 + NH4+ including gas: {nitrogen} mol; trapped N2 is separate in ledger.')
    elif n==8:
        shifts=[p[2]-p[0],p[3]-p[1]];obs['own_baseline_ph_shifts']=shifts
        check('met' if abs(shifts[0])<abs(shifts[1]) else 'failed',f'Own-baseline shifts buffered {shifts[0]} vs water {shifts[1]}.')
        assert amount(final[0],'NH4+')>0
        check('met',f'NH4+ remaining {amount(final[0],"NH4+"):.12g} mol.')
    elif n==9:
        before=at_measure(rows,'ph_meter',0)[0];obs['acetate_family_before_after_mol']=[{k:amount(v,k) for k in ['CH3COOH','CH3COO-']} for v in [before,final[0]]]
        check('met' if amount(before,'CH3COOH')>amount(final[0],'CH3COOH') and amount(before,'CH3COO-')<amount(final[0],'CH3COO-') and abs(family(final[0])-.002)<=1e-8 else 'failed',f'Before/after {obs["acetate_family_before_after_mol"]}; total family 2 mmol, carbon 4 mmol.')
        assert p[0]<p[1]<7
        check('met',f'pH {p[0]} -> {p[1]}, remains acidic.')
    elif n==10:
        delta=p[1]-p[0];obs.update(ph_difference=delta,ideal_log10_81=math.log10(81),difference_from_ideal=delta-math.log10(81))
        assert p[1]>p[0]
        check('met',f'Acetate-rich pH {p[1]} > acid-rich {p[0]}.')
        check('met',f'Difference {delta:.12g} vs ideal log10(81)={math.log10(81):.12g}, deviation {delta-math.log10(81):.9g}; qualified approximate comparison, no exact calibration or newly imposed tolerance.')
    elif n==11:
        shifts=[p[2]-p[0],p[3]-p[1]];obs.update(initial_ph_difference=abs(p[0]-p[1]),own_baseline_ph_shifts=shifts)
        check('met',f'Initial pH {p[:2]}, difference {abs(p[0]-p[1]):.9g}; similar, not identical.')
        check('met' if abs(shifts[1])<abs(shifts[0]) else 'failed',f'Dilute/concentrated shifts {shifts}; concentrated buffer smaller.')
    elif n==12:
        shifts=[p[2]-p[0],p[3]-p[1]];obs.update(initial_ph_difference=abs(p[0]-p[1]),own_baseline_ph_shifts=shifts)
        check('met',f'Postdilution pH {p[:2]}, difference {abs(p[0]-p[1]):.9g}.')
        check('met' if shifts[1]<shifts[0]<0 else 'failed',f'0.05/0.25 mmol acid challenges; concentrated/dilute shifts {shifts}. Approximate concentration-normalised protocol retained.')
    elif n==13:
        check('met' if p[1]<=p[0] else 'failed',f'1 nmol HCl: pH {p[0]} -> {p[1]}, not alkaline.')
        assert final[0]['headspace']=={'boundary':'open'} and math.isfinite(final[0]['temperature'])
        check('met',f'Actual water temperature {final[0]["temperature"]:.12g} K; open model boundary, initial pH {p[0]} is the measured neutral reference. No atmospheric calibration claimed.')
    elif n==14:
        check('met' if p[2]>7 and p[3]<7 else 'failed',f'After equal20 umol base: HCl pH {p[2]}, weak acid {p[3]}.')
        obs['initial_ph_difference']=abs(p[0]-p[1]);check('met',f'Initial pH {p[:2]} difference {abs(p[0]-p[1]):.9g}; match reported only as approximate.')
    elif n==15:
        before=at_measure(rows,'ph_meter',0)[0];fractions=[amount(v,'CH3COO-')/family(v) for v in [before,final[0]]];totals=[family(v) for v in [before,final[0]]]
        obs.update(ionised_fractions=fractions,acetate_family_mol=totals,ph_rise=p[1]-p[0])
        check('met' if fractions[1]>fractions[0] else 'failed',f'Ionised fraction {fractions}; pH rise {p[1]-p[0]:.9g}, less than one.')
        check('met' if all(abs(x-.001)<=1e-8 for x in totals) else 'failed',f'Family amounts {totals} mol after added900mL water.')
    elif n==16:
        assert p[0]>p[1]
        check('met',f'Acetate pH {p[0]} > saltcontrol {p[1]}.')
        check('met' if amount(final[0],'CH3COOH')>0 and amount(final[0],'base_equivalents')>0 and amount(final[0],'NaOH')==0 and final[0]['free_hydroxide']>0 else 'failed',f'CH3COOH {amount(final[0],"CH3COOH"):.12g}; canonical base_equivalents {amount(final[0],"base_equivalents"):.12g}; measured freeOH {final[0]["free_hydroxide"]:.12g} mol. No NaOH bottle species created.')
    elif n==17:
        prior=rows[2]['bench']['vessels'][0];obs['same_vessel_NH4Cl_only_ph']=prior['solution']['ph'];obs['NH3_aqueous_gas_mol']={phase:amount(final[0],'NH3',phase) for phase in ['aqueous','gas']}
        check('inconclusive',f'Final pH {p[0]} near-neutral and above intermediate NH4Cl-only pH {prior["solution"]["ph"]}; no same-boundary NaOAc-only control in this script. AB016 NaOAc-only is open, so not an exact sealed comparator.')
        assert all(x>0 for x in obs['NH3_aqueous_gas_mol'].values()) and abs(amount(final[0],{'NH3','NH4+'})-.001)<=1e-8
        check('met',f'NH3 phase partition {obs["NH3_aqueous_gas_mol"]}; final pH {p[0]} not asserted seven, reagent N {amount(final[0],{"NH3","NH4+"}):.12g} mol.')
        notes.append('Freeze a matching sealed NaOAc-only comparator before closing C1; do not infer a measured missing control from theory.')
    elif n==18:
        assert p[0]<p[1]
        check('met',f'Ferric pH {p[0]} < calcium {p[1]}.')
        obs['ferric_hydroxide_solid_mol']=amount(final[0],'Fe(OH)3','solid')
        assert not any(r['operator']['op']=='add' and r['operator']['species'] not in ['water','FeCl3','CaCl2'] for r in rows)
        check('met',f'Fe(OH)3 {obs["ferric_hydroxide_solid_mol"]:.12g} mol with FeCl3/water inputs only; no external base operator. Aqueous Fe+3 is canonical total; do not treat it as free aquo ion concentration.')
        notes.append('Supersaturated Goethite/Hematite/Maghemite and saturation-index coverage notices limit phase-complete prediction; only the bounded acidity comparison is assessed.')
    elif n==19:
        pools={k:amount(final[0],k) for k in ['H2PO4-','HPO4-2']};obs['postpulse_phosphate_mol']=pools
        check('met' if all(x>0 for x in pools.values()) else 'failed',f'Postpulse species {pools}.')
        shifts=[p[2]-p[0],p[3]-p[1]];obs['own_baseline_ph_shifts']=shifts
        check('met' if abs(shifts[0])<abs(shifts[1]) else 'failed',f'Buffered/water ownbaseline shifts {shifts}.')
    elif n==20:
        before=at_measure(rows,'ph_meter',0)[0];ratios=[amount(v,'HCO3-')/amount(v,'CO3-2') for v in [before,final[0]]];obs['bicarbonate_to_carbonate_before_after']=ratios
        check('met' if ratios[1]>ratios[0] else 'failed',f'Bicarbonate/carbonate ratio {ratios}; pH {p}.')
        carbon=amount(final[0],{'CO2','CO2(aq)','HCO3-','CO3-2'});initialair=sum(e['trapped_air']*.0004 for e in events if e['event']=='vessel_sealed');obs.update(inorganic_carbon_all_phases_mol=carbon,initial_boundary_carbon_mol=initialair)
        check('met' if abs(carbon-(.002+initialair))<=1e-8 else 'failed',f'Total carbon {carbon:.15g} vs .002 + initial air {initialair:.15g}; no explicit exports.')
    elif n==21:
        balances=[{'vessel':v,'temperature_K':final[v]['temperature'],'free_proton_mol':final[v]['free_proton'],'free_hydroxide_mol':final[v]['free_hydroxide']} for v in [0,1]];obs['water_neutrality']=balances
        check('met' if all(b['free_proton_mol']==b['free_hydroxide_mol'] for b in balances) else 'failed',f'H/O measured inventories match in both states {balances}; these fields are diagnostics, not extra portions.')
        check('met' if p[1]<p[0] else 'failed',f'Actual25/60C pure-water pH {p}; thermal effect present, fixed-Kw premise false. No empirical pKw calibration claimed.')
    elif n==22:
        mixed=p[2];dist=[abs(mixed-p[0]),abs(mixed-p[1])];obs.update(distances_to_initial_ph=dist,arithmetic_mean_ph=(p[0]+p[1])/2,ideal_mixed_ph=-math.log10(.000505/.1))
        check('met' if dist[0]<dist[1] else 'failed',f'Initial pH {p[:2]}, mixed {mixed}; distances {dist}, arithmetic mean {(p[0]+p[1])/2:.12g}.')
        assert not final[0]['contents'] and not final[1]['contents']
        check('met' if abs(amount(final[2],'H+')-.000505)<=1e-8 and abs(amount(final[2],'Cl-')-.000505)<=1e-8 else 'failed',f'Mixed H+/Cl- each {amount(final[2],"H+"):.12g}/{amount(final[2],"Cl-"):.12g} mol; parents empty after fullmix; ideal estimate {obs["ideal_mixed_ph"]:.12g} qualified by activity.')
    elif n==23:
        check('met' if p[0]<p[1] and amount(final[0],'H+')>0 and abs(p[1]-7)<abs(p[0]-7) else 'failed',f'NaOH pH {p[0]} with H+ {amount(final[0],"H+"):.12g} mol; calciumhydroxide pH {p[1]}.')
        check('met' if abs(amount(final[1],'Ca+2')-.001)<=1e-8 and amount(final[1],'Ca(OH)2')==0 else 'failed',f'Calcium {amount(final[1],"Ca+2"):.12g} mol aqueous; no remaining Ca(OH)2. Formula unit supplies2OH; chloride2mmol, no new reagent.')
    elif n==24:
        check('met' if amount(final[0],'H+')<amount(final[1],'H+') and p[0]>p[1] else 'failed',f'MgOH residual H+ {amount(final[0],"H+"):.12g} mol vs carbonate {amount(final[1],"H+"):.12g}; pH {p}.')
        gas=[sum(e['moles'] for e in events if e['event']=='gas_evolved' and e['species']=='CO2' and e['vessel']==v) for v in [0,1]];obs['exported_CO2_mol_by_vessel']=gas
        check('met' if gas[0]==0 and gas[1]>0 else 'failed',f'Explicit CO2 exports Mg/Ca vessels {gas}; carbonate retained aqueous carbon also counted in ledger.')
    assert len(outcomes)==len(c['acceptance_checks'])==2
    # Original 'similar', 'near', 'substantial' and activity-qualified approximate
    # words did not freeze numeric pass bands. These are transparent manual
    # semantic decisions on the authenticated fixed stream, not new tolerances.
    semantic={(1,2),(3,1),(4,1),(9,2),(10,2),(11,1),(12,1),(13,2),(14,2),(17,2),(18,2),(21,2),(22,1)}
    checks=[{'original':original,'assessment':status,'evidence':why,
             'assessment_basis':'manual_semantic_review_of_original_qualitative_criterion_no_new_pass_band'
                if (n,j+1) in semantic else 'explicit_numeric_inventory_or_order_comparison_or_named_missing_control'}
            for j,(original,(status,why)) in enumerate(zip(c['acceptance_checks'],outcomes))]
    statuses=[x['assessment'] for x in checks]
    mat=ledger(rows)
    if not mat['within_existing_inventory_audit_resolution']:
        notes.append('Represented material/charge ledger exceeds existing 1e-8 mol audit resolution. Matching pH/criteria alone is not evidence of whole-state material closure; independently review before catalogue selection. No tolerance changed.')
    verdict='original_bounded_criteria_met' if statuses==['met','met'] else 'original_criteria_incomplete'
    selection='proposed_bounded_selection' if statuses==['met','met'] and mat['within_existing_inventory_audit_resolution'] and not limits else 'hold_for_named_followup'
    return {'id':c['id'],'title':c['title'],'original_expected_result':c['expected_result'],'script_sha256':c['script_sha256'],
            'stdout_sha256':receipt['stdout_sha256'],'stderr_sha256':receipt['stderr_sha256'],'exit_code':receipt['exit_code'],
            'criteria':checks,'observations':obs,'ledger':mat,'named_limits':limits,'notes':notes,'verdict':verdict,
            'proposed_selection':selection,'catalog_case_added':False,
            'latest_vessel_snapshots':[{'vessel':v['id'],'temperature_K':v['temperature'],'headspace':v['headspace'],'contents':v['contents'],'free_proton_mol':v['free_proton'],'free_hydroxide_mol':v['free_hydroxide']} for v in final.values()]}


def main():
    inputs,binding=authenticate()
    cases=[assess(*items) for items in inputs]
    result={'scope':'Independent saved-only original criterion assessment; no engine executions, rewritten forecasts, empirical calibration, general phase completeness or full energy certification.',
            'original_json_sha256':ORIGINAL_SHA,'raw_artifact_sha256':ZIP_SHA,'binding':binding,
            'engine':ENGINE,'lock_qualification':'Producer-reported manifest hash; raw Cargo.lock absent from producer artifact.',
            'new_executions':0,'catalog_additions':0,'inventory_audit_resolution_mol':1e-8,
            'resolution_qualification':'Existing represented-inventory audit reporting resolution, not a new physical forecast tolerance. Qualitative original criteria evaluated with explicit numbers and no invented pass bands.',
            'counts':dict(collections.Counter(c['verdict'] for c in cases)),
            'proposed_bounded_selections':[c['id'] for c in cases if c['proposed_selection']=='proposed_bounded_selection'],
            'held_cases':[c['id'] for c in cases if c['proposed_selection']!='proposed_bounded_selection'], 'cases':cases}
    print(json.dumps(result,indent=2,allow_nan=False))

if __name__=='__main__': main()
