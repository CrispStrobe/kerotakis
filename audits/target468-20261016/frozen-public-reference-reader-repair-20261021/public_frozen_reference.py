"""Strict public authority for both original frozen-input metadata schemas."""
LABEL_NAME='label-forecast.json'
LABEL_SHA='eec58140902f1bdc9395479448a5dc224f718fc0eff43d02fd2b4f36d99b00de'
LABEL_COMMIT='e81b80a669de42444d20c60164965cd4d9ea9d76'
LABEL_PATH='audits/target468-20261016/optical-species-label-frozen-controls-20261021/forecast.json'
def public_authority(name,entry):
 if name==LABEL_NAME:
  assert set(entry)=={'base64','sha256','origin_kind'},'Unexpected historical label forecast schema'
  assert entry['origin_kind']=='pre-runtime-edit-frozen-forecast' and entry['sha256']==LABEL_SHA
  return LABEL_COMMIT,LABEL_PATH
 assert set(entry)=={'base64','sha256','public_commit','public_path'},'Unknown public frozen-input schema'
 return entry['public_commit'],entry['public_path']
