"""Strict canonical public metadata for future frozen optics references."""
def validate_optics_references(plan, public_bytes=None):
 import base64,hashlib,pathlib,re
 expected={'VesselOpticalPhGap.test.ts','VesselOpticalSpeciesLabels.test.ts','forecast.json','indicator_optical_coverage.rs','label-forecast.json','optical_sample_admission.rs','optical_species_labels.rs','sample-forecast.json'}
 entries=plan['optics_frozen_inputs'];assert type(entries)is dict and set(entries)==expected,'Exact eight frozen optics references required'
 rows={}
 for name,entry in entries.items():
  assert type(entry)is dict and pathlib.PurePosixPath(name).name==name
  keys={'base64','sha256','public_commit','public_path'}
  if name=='label-forecast.json':keys.add('origin_kind')
  assert set(entry)==keys, 'Unsupported canonical frozen-reference schema: '+name
  assert type(entry['public_commit'])is str and re.fullmatch('[a-f0-9]{40}',entry['public_commit'])
  path=entry['public_path'];assert type(path)is str and path and not path.startswith('/') and all(p not in['.','..','']for p in path.split('/')) and '\\'not in path
  assert type(entry['sha256'])is str and re.fullmatch('[a-f0-9]{64}',entry['sha256'])
  raw=base64.b64decode(entry['base64'],validate=True);assert base64.b64encode(raw).decode()==entry['base64']and hashlib.sha256(raw).hexdigest()==entry['sha256']
  if name=='label-forecast.json':
   assert entry['origin_kind']=='pre-runtime-edit-frozen-forecast'
   assert entry['sha256']=='eec58140902f1bdc9395479448a5dc224f718fc0eff43d02fd2b4f36d99b00de'
   assert entry['public_commit']=='e81b80a669de42444d20c60164965cd4d9ea9d76'
   assert path=='audits/target468-20261016/optical-species-label-frozen-controls-20261021/forecast.json'
  if public_bytes is not None:assert public_bytes(entry['public_commit'],path)==raw,'Frozen public bytes differ: '+name
  rows[name]={'sha256':entry['sha256'],'public_commit':entry['public_commit'],'public_path':path,'bytes':len(raw)}
 return rows
