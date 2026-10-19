"""Reproduce preservation, script digests and unique original-target membership."""
import hashlib,json,pathlib
H=pathlib.Path(__file__).resolve().parent;R=H.parents[2];A=H.parent
sha=lambda raw:hashlib.sha256(raw).hexdigest()
selection=json.loads((H/'selection.json').read_bytes())
oldraw=(R/'audits/working-experiments-20261019-reviewed-separation.json').read_bytes()
newraw=(R/'audits/working-experiments-20261019-hydrolysis.json').read_bytes()
assert sha(oldraw)==selection['previous_catalog_sha256'] and sha(newraw)==selection['new_catalog_sha256']
old=json.loads(oldraw);new=json.loads(newraw)
assert len(old['experiments'])==246 and new['experiments'][:246]==old['experiments']
assert new['count']==len(new['experiments'])==247
raw=(A/'common-engine-all668.json').read_bytes();inventory=json.loads(raw)
all_ids={c['id'] for c in inventory['cases']};target={k for k in all_ids if not k.startswith('curriculum200:')};assert len(target)==468
ids=set();mapping=[];outside=[]
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert sha(e['script'].encode())==e['script_sha256']
 original=e.get('original_id',e['id']);matches={original,'baseline147:'+original,original.removeprefix('new201-original-assessed:')}&all_ids
 if not matches:outside.append(e['id']);continue
 assert len(matches)==1 and matches<=target
 mapping.append({'catalog_id':e['id'],'original_target_id':matches.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==243
assert outside==json.loads((A/'scoreboard-20261019-reviewed-separation.json').read_bytes())['out_of_target_catalog_entries']
report={'catalog_sha256':sha(newraw),'original_inventory_sha256':sha(raw),'previous246_entries_preserved_exactly':True,'all247_scripts_authenticated':True,'fixed468_unique_selected':243,'out_of_target':outside,'mapping':mapping,'new_engine_or_API_executions':0}
p=H/'catalog-membership.json'
if p.exists():assert json.loads(p.read_bytes())==report
else:p.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'catalog_entries':247,'unique_fixed468':243,'additional_to401':158,'new_executions':0}))
