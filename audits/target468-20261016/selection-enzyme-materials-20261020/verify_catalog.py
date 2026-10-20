#!/usr/bin/env python3
"""Verify seven earned bounded selections; reproduce saved mathematics only, never an engine."""
import collections,hashlib,json,os,subprocess,sys
from pathlib import Path
from decimal import Decimal
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];A=HERE.parent;B2=A/'batch2-selection-review-eighth-20261020';B3=A/'batch3-selection-review-fourth-20261020';load=lambda p:json.loads(p.read_bytes());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
available=int(next(x.split()[1] for x in Path('/proc/meminfo').read_text().splitlines() if x.startswith('MemAvailable:')))//1024;assert available>=3200 and os.getloadavg()[0]<=3.5,'Saved archive gate closed; no rerun issued'
s=load(HERE/'selection.json');old=load(ROOT/s['previous_catalog']);new=load(ROOT/s['new_catalog']);assert sha(ROOT/s['previous_catalog'])==s['previous_catalog_sha256'] and sha(ROOT/s['new_catalog'])==s['new_catalog_sha256'];assert old['count']==len(old['experiments'])==325 and new['experiments'][:325]==old['experiments'];assert new['count']==len(new['experiments'])==332;assert new['counts_by_group']==dict(collections.Counter(e['group'] for e in new['experiments'])) and sum(new['counts_by_group'].values())==332
for path,digest in s['parent_adjudications'].items():assert sha(ROOT/path)==digest
adj2=load(B2/'parent-adjudication.json');adj3=load(B3/'parent-adjudication.json');a2=load(B2/'assessment.json');assessed2={c['id']:c for c in a2['cases']};assert adj2['reader_sha256']==sha(B2/'review_saved.py') and adj2['assessment_sha256']==sha(B2/'assessment.json');assert adj2['parent_reproduced_full_reader_exactly'] and adj2['original_criteria_preserved'];assert adj3['reader_sha256']==sha(B3/'review_saved.py') and adj3['parent_reproduced_all_saved_output_bytes_exactly'] and adj3['parent_read_all_original_forecasts_criteria_and_actual_independent_calculations']
accepted2={'batch2:ki-008'};accepted3={'batch3-diverse:'+i for i in ['en-006','co-004','po-007','fo-004','su-002','su-003']};assert set(adj2['accepted_originals'])==accepted2 and set(adj3['accepted_originals'])==accepted3;assert set(s['new_selected_originals'])==accepted2|accepted3;holds=set(adj2['held_originals']+adj3['held_originals']);assert len(holds)==9 and holds==set(s['retained_holds']);assert not holds&set(s['new_selected_originals'])
for key,digest in adj3['reports_sha256'].items():assert sha(B3/(key+'.json'))==digest
originals={o['original_id']:o for o in load(ROOT/'audits/new-experiments-201-20261016/intake.json')};assert len({e['id'] for e in new['experiments']})==332
for e in new['experiments']:assert hashlib.sha256(e['script'].encode()).hexdigest()==e['script_sha256']
for e in new['experiments'][325:]:
 oid=e['original_id'];o=originals[oid];assert oid in accepted2|accepted3 and e['script']==(ROOT/'audits/new-experiments-201-20261016'/o['original_script']).read_text();assert e['script_sha256']==o['script_sha256'] and e['expected_result']==o['original_expected_result'];assert e['limitations'][:len(o['original_limitations'])]==o['original_limitations'];assert len(e['acceptance_checks'])==len(o['original_acceptance_checks'])
 for x,y in zip(e['acceptance_checks'],o['original_acceptance_checks'],strict=True):assert {k:v for k,v in x.items() if k!='result'}=={k:v for k,v in y.items() if k!='result'} and x['result']=='met_in_original_supported_scope'
 if oid in accepted2:
  c=assessed2[oid];assert c['proposed_candidate'] and c['material_budget_pass'] and c['unknown_formula_conservation_proven_symbolically'];assert all(Decimal(p['unknown_atom_and_charge_vector_coefficient_residual_mol'])==0 for p in c['metrics']['every_step_symbolic_enzyme_conservation']);assert e['validated_on']==c['source'] and e['observed_result']['independent_calculations']==c['metrics']
 else:
  c=load(B3/(oid.split(':')[1]+'.json'));assert c['status']=='proposal_pending_parent_review' and not c['inventory']['strict_balance_failures'];assert c['original_criteria']==o['original_acceptance_checks'];assert e['validated_on']==c['engine'] and e['observed_result']['independent_calculations']==c['assessment']['independent_calculations']
 assert 'producer-reported' in e['limitations'][-1] and 'raw Cargo.lock bytes are absent' in e['limitations'][-1];assert all((ROOT/p).exists() for p in e['evidence'].values())
# Exact every prior entry also preserves actual producer-lock bindings where those exist;
# no historical producer-reported digest is promoted to independently rehashed bytes.
allids={c['id'] for c in load(A/'common-engine-all668.json')['cases']};target={i for i in allids if not i.startswith('curriculum200:')};mapping=[];outside=[]
for e in new['experiments']:
 oid=e.get('original_id',e['id']);match={oid,'baseline147:'+oid,oid.removeprefix('new201-original-assessed:')}&allids
 if not match:outside.append(e['id']);continue
 assert len(match)==1 and match<=target;mapping.append({'catalog_id':e['id'],'original_target_id':match.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==328 and len(outside)==4;assert load(HERE/'catalog-membership.json')=={'catalog_sha256':sha(ROOT/s['new_catalog']),'mapping':mapping,'outside_target':outside,'unique_fixed468':328};remaining=load(HERE/'remaining140.json');assert remaining['remaining']==len(remaining['cases'])==140 and remaining['additional_needed_to401']==401-328==73;assert {c['original_id'] for c in remaining['cases']}==target-{m['original_target_id'] for m in mapping}
p=subprocess.run([sys.executable,str(B2/'review_saved.py'),'--repo',str(ROOT)],cwd=ROOT,text=True,capture_output=True);assert p.returncode==0,p.stderr;assert json.loads(p.stdout)==a2
# This historical reader rewrites exactly these deterministic saved reports in place.
# Capture ALL ten output byte hashes first; require identical bytes after reproduction.
outputs=[B3/(key+'.json') for key in sorted(adj3['reports_sha256'])]+[B3/'source-binding.json',B3/'candidate-index.json'];before={str(p):sha(p) for p in outputs};p=subprocess.run([sys.executable,str(B3/'review_saved.py')],cwd=ROOT,text=True,capture_output=True);assert p.returncode==0,p.stderr;assert {str(p):sha(p) for p in outputs}==before
assert (B2/'first-unknown-enzyme-formula-hold-assessment.json').exists() and (B2/'first-reader-failure.json').exists() and (B3/'reader-first-failure.json').exists();assert load(B3/'en-008.json')['status']=='hold' and load(B3/'su-005.json')['status']=='hold'
print(json.dumps({'all325_predecessor_entries_exact':True,'all7_original_scripts_forecasts_criteria_exact':True,'actual_group_Counter_matches_metadata':True,'batch2_saved_JSON_reproduced':True,'batch3_all10_output_bytes_identical':True,'parent_adjudications_and_source_lock_qualifications_bound':True,'unknown_enzyme_conservation_symbolic_no_invented_formula':True,'catalog_entries':332,'unique_fixed468':328,'remaining':140,'additional_to401':73,'all9_holds_and_first_failures_preserved':True,'new_engine_or_API_executions':0}))
