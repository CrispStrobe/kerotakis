"""Grouped saved-evidence authentication only. No provider/client/network imports."""
import hashlib,json,pathlib
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(pathlib.Path(p).read_bytes())

def digest(path):
 with pathlib.Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def safe(root,name):
 p=pathlib.PurePosixPath(name);assert name and not p.is_absolute() and '..' not in p.parts and '\\' not in name and str(p)==name
 target=root/pathlib.Path(*p.parts);assert all(not q.is_symlink() for q in [target,*target.parents] if q==root or root in q.parents);assert target.is_file();return target

def manifest(root):
 root=pathlib.Path(root);m=load(root/'SHA256.json');assert isinstance(m,dict) and m
 actual=set()
 for p in root.rglob('*'):
  assert not p.is_symlink(),str(p)
  if p.is_file() and p!=root/'SHA256.json':actual.add(p.relative_to(root).as_posix())
 assert set(m)==actual,{'manifest_omissions':sorted(actual-set(m)),'missing':sorted(set(m)-actual)}
 for n,d in m.items():assert digest(safe(root,n))==d,n
 return m

class ExplicitAliases:
 """Read-only paths to actual explicit inputs for unchanged legacy AST expressions."""
 def __init__(self,mapping):self.mapping={k:pathlib.Path(v) for k,v in mapping.items()}
 def __truediv__(self,name):assert name in self.mapping,name;return self.mapping[name]

def authenticate(*,evidence,design_path,worker_path,issue_path,api_path,verified_path,native_reference_path,group):
 """Frozen grouped schema: CONFIG inputs/ordered groups, explicit external issue."""
 import base64
 E=pathlib.Path(evidence);design=load(design_path);top=manifest(E);assert (E/'execution-admission.json').read_bytes()==pathlib.Path(design_path).read_bytes()
 canonical=json.dumps(design,sort_keys=True,separators=(',',':'),allow_nan=False).encode('utf-8');config_sha=sha(canonical);worker_sha=digest(worker_path);issue=load(issue_path)
 assert issue['worker_sha256']==worker_sha and issue['config_sha256']==config_sha and issue['runtime_source']==design['runtime_source'];assert type(issue['version']) is int and issue['one_issue_only'] is True and issue['parent_reviewed_exact_producer_and_readers'] is True and issue['version']==1 and isinstance(issue['kernel_id'],str) and issue['kernel_id'].strip()
 assert pathlib.Path(issue_path).resolve()!= (E/'ISSUED.json').resolve() and E.resolve() not in pathlib.Path(issue_path).resolve().parents,'Issue authority must be external to producer output'
 db=load(E/'design-binding.json');assert db['config_sha256']==config_sha and db['worker_sha256']==worker_sha and db['parent_acceptance_assigned'] is False;assert load(E/'STARTED.json')['script_sha256']==worker_sha
 groups=design['groups'];assert [(g['id'],g['entry'],g['expected_checks'],g['timeout_seconds']) for g in groups]==[('browser37','live-client.mjs',37,240),('targeted7','targeted.mjs',7,240),('locale18','run-locale18.mjs',18,300),('heat2','run-heat.mjs',2,300)]
 binding=design['binding'];S=design['runtime_source'];assert binding['runtime_source']==S and binding['harness_head']==design['harness_head'] and binding['harness_tree']==design['harness_tree'] and binding['binding_sha256']==design['binding_sha256'];assert binding['engine']['source_head']==S and binding['never_merge_validation_ref'] is True and binding['never_merge_evidence_branch'] is True and binding['new_owner_keys_in_browser_or_kaggle'] is False
 inputs=design['inputs'];input_paths=[];input_hashes={}
 for name,row in inputs.items():
  assert set(row)=={'base64','sha256'};b=base64.b64decode(row['base64'],validate=True);assert base64.b64encode(b).decode('ascii')==row['base64'] and sha(b)==row['sha256'];archived=safe(E,'frozen-inputs/'+name);assert archived.read_bytes()==b;input_paths.append(archived);input_hashes[name]=row['sha256']
 original_modules={'live-client.mjs':'1f0ddea0b8a9e16d78cb15cfaaa319abbe2e6cd817999d7075c4e9c235a903a0','targeted.mjs':'f17c5c7fe143d9930efeeafa8cf9c14c1a873ba10ea4927ca5fc30fa91cba4f9','locale-controls.mjs':'7290327a0d59d981bcc8bc0e35b5d0941c3559d14067ecdf488ae4f24c0f8819','run-locale18.mjs':'8456b089e8389c2995fd1c58bcd316c64fffcfcad4db002927f336fa58a8bd21','heat-controls.mjs':'cf19825bd6a273115eb686feab0b3242dfebb5533e571fed4765b8f49c371d6f','run-heat.mjs':'316980a04ba85c94a54665f1ad5418a310f8485892afe97b85c9ba97daa6d592'}
 for name,h in original_modules.items():assert input_hashes[name]==h
 projection=load(E/'frozen-inputs/forecasts/parent-source-review-projection.json');assert projection['execution_admission_order_approved'] is True and projection['original_modules_byte_identical'] is True and projection['all64_final_acceptance_mandatory'] is True
 for name,h in projection['input_aliases_sha256'].items():assert input_hashes[name]==h
 assert inputs['release-binding.json']['sha256']==design['binding_sha256'] and load(E/'frozen-inputs/release-binding.json')==binding['binding'] and (E/'release-binding.json').read_bytes()==(E/'frozen-inputs/release-binding.json').read_bytes()
 api=load(api_path);verified=load(verified_path);native=load(native_reference_path);prior=design['prior_release'];assert digest(api_path)==prior['API_receipt_sha256']==inputs['prior-acceptance/API-ACCEPTED.json']['sha256'];assert digest(verified_path)==prior['verified_artifact_receipt_sha256']==inputs['prior-acceptance/verified-release.json']['sha256'];assert digest(native_reference_path) in set(input_hashes.values()),'Independent methanol reference not frozen before execution'
 assessment=load(E/'frozen-inputs/prior-acceptance/script-assessment.json');assert assessment['engine']==api['engine']==binding['engine'] and assessment['all_script_controls_pass'] is True and assessment['native_browser_reference']==binding['binding']['native_bridge_reference'];assert binding['binding']['native_bridge_reference']['engine']==binding['engine']
 reference_case=[c for c in assessment['cases'] if c['id']=='uncached-native-browser-reference'];assert len(reference_case)==1;reference_case=reference_case[0];reference=assessment['native_browser_reference'];assert reference_case['script_sha256']==reference['script_sha256'] and reference_case['response_sha256']==reference['response_sha256'];assert all(reference_case['metrics'][k]==reference['solution'][k] for k in ['ph','ionic_strength'])
 assert api['source']==S and api['eleven_original_protocols_pass'] and api['concurrency']==[200,429];assert verified['actual_binary_bytes_rehashed'] and verified['actual_producer_lock_bytes_rehashed'];assert all(verified['engine'][k]==v for k,v in binding['engine'].items());assert verified['engine']['source_tree']==design['runtime_tree'];assert prior['native_binary_sha256']==verified['engine']['binary_sha256'] and prior['native_actual_producer_lock_sha256']==verified['engine']['cargo_lock_sha256'];assert binding['binding']['producer_artifact_receipt_sha256']==digest(verified_path)
 assert binding['binding']['expected_engine_source']==S and binding['binding']['expected_binary_sha256']==verified['engine']['binary_sha256'];assets=binding['binding']['frontend_assets'];assert len(assets)==5
 for n,d in assets.items():assert verified['frontend_assets'][n]['sha256']==d
 assert verified['frontend_assets']['rest.html']['sha256']==binding['binding']['client_sha256']==prior['client_sha256'];assert prior['frontend_assets']==assets
 assert native['helper_sha256']==inputs['targeted.mjs']['sha256'] and native['targeted_checks']==7 and native['original_experiment_requests']==native['scientific_catalog_additions']==0
 group_design=next(g for g in groups if g['id']==group);g=E/'groups'/group;gm=manifest(g);gd=load(g/'group-design.json');started=load(g/'STARTED.json');done=load(g/'COMPLETED.json');source=load(g/'source.json');assert started['script_sha256']==gd['worker_sha256']==worker_sha;assert gd['source']==S and gd['binding']==binding and gd['execution_only_not_parent_acceptance'] is True;assert gd['expected_checks']==group_design['expected_checks'] and gd['entrypoint_path']==group_design['entry'] and gd['entrypoint_sha256']==inputs[group_design['entry']]['sha256'];assert gd['input_files_sha256']==input_hashes
 assert done['browser_exit_status']==0 and done['harness_head']==binding['harness_head'] and done['runtime_source']==S and done['raw_report_exists'] is True and done['root_review_required'] is True and done['owner_credentials_supplied'] is False and done['scientific_catalog_selections']==0 and done['parent_acceptance_assigned'] is False and done['group']==group
 for k in ['harness_head','harness_tree','binding_sha256','harness_script_sha256']:assert source[k]==binding[k]
 assert source['runtime_source']==S and source['group']==group and source['entrypoint_path']==group_design['entry'] and source['entrypoint_sha256']==gd['entrypoint_sha256'];assert source['modules_sha256']==gd['modules_sha256']
 assert gd['modules_sha256']==original_modules
 for name,h in gd['modules_sha256'].items():assert input_hashes[name]==h
 before=load(g/'source-before.json');after=load(g/'source-after.json');assert before==after and before['input_files_sha256']==input_hashes
 for k in ['harness_head','harness_tree','runtime_source','binding_sha256']:assert before[k]==source[k]
 assert load(g/'source-unchanged.json')['exit_status']==0;assert digest(g/'SHA256.json')==top['groups/'+group+'/SHA256.json']
 command=load(g/'actual-live-browser.json');assert command['exit_status']==0 and command['error_type'] is None and command['fresh_owned_session'] is True and len(command['argv'])==2 and command['argv'][0]=='node' and pathlib.PurePosixPath(command['argv'][1]).name==group_design['entry'];assert command['stdout_sha256']==digest(g/'actual-live-browser.stdout') and command['stderr_sha256']==digest(g/'actual-live-browser.stderr')
 assert not any(pathlib.PurePosixPath(n).name in ['PRIOR44-ACCEPTED.json','BROWSER37-ACCEPTED.json','ALL62-ACCEPTED.json','PRODUCTION-ACCEPTED.json'] and not n.startswith('frozen-inputs/') for n in top),'Producer must never assign parent acceptance'
 return {'E':E,'e':g,'design':design,'group_design':gd,'binding':binding,'S':S,'api':api,'verified':verified,'native':native,'input_paths':input_paths,'group_manifest_sha256':digest(g/'SHA256.json'),'top_manifest_sha256':digest(E/'SHA256.json'),'execution_design_sha256':digest(design_path),'config_canonical_sha256':config_sha,'external_issue_sha256':digest(issue_path),'worker_sha256':worker_sha,'api_sha256':digest(api_path),'verified_sha256':digest(verified_path),'native_reference_sha256':digest(native_reference_path)}
