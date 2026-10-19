"""Check catalog preservation, script digests and unique fixed-target membership."""
import hashlib,json,pathlib
H=pathlib.Path(__file__).resolve().parent;R=H.parents[2];A=H.parent
sha=lambda b:hashlib.sha256(b).hexdigest()
receipt=json.loads((H/'selection.json').read_bytes());oldraw=(R/'audits/working-experiments-20261019-ammonia.json').read_bytes();newraw=(R/'audits/working-experiments-20261019-reviewed-separation.json').read_bytes()
assert sha(oldraw)==receipt['previous_catalog_sha256'] and sha(newraw)==receipt['new_catalog_sha256']
old=json.loads(oldraw);new=json.loads(newraw);assert len(old['experiments'])==238 and new['experiments'][:238]==old['experiments'];assert len(new['experiments'])==new['count']==246
ids=set();mapping=[];outside=[]
inventory_raw=(A/'common-engine-all668.json').read_bytes();inventory=json.loads(inventory_raw);all_ids={c['id'] for c in inventory['cases']};target={i for i in all_ids if not i.startswith('curriculum200:')};assert len(target)==468
for e in new['experiments']:
 assert e['id'] not in ids;ids.add(e['id']);assert sha(e['script'].encode())==e['script_sha256']
 original=e.get('original_id',e['id']);matches={original,'baseline147:'+original,original.removeprefix('new201-original-assessed:')} & all_ids
 if not matches:outside.append(e['id']);continue
 assert len(matches)==1 and matches<=target
 mapping.append({'catalog_id':e['id'],'original_target_id':matches.pop()})
assert len(mapping)==len({m['original_target_id'] for m in mapping})==242
assert outside==json.loads((A/'scoreboard-20261019-ammonia.json').read_bytes())['out_of_target_catalog_entries']
report={'catalog_sha256':sha(newraw),'original_inventory_sha256':sha(inventory_raw),'previous238_entries_preserved_exactly':True,'all246_scripts_authenticated':True,'fixed468_unique_selected':242,'out_of_target':outside,'mapping':mapping,'new_engine_or_API_executions':0}
p=H/'catalog-membership.json'
if p.exists():assert json.loads(p.read_bytes())==report
else:p.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'catalog_entries':246,'unique_fixed468':242,'previous238_unchanged':True,'additional_to401':159,'new_executions':0}))
