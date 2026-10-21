"""SYNTHETIC WIRE-RECORD / SOURCE controls, no browser/engine observations."""
import ast,copy,json,re,unittest
from pathlib import Path
B=Path(__file__).parent
scope={}
for name,wanted in [('optics18/project_app_assets.py',{'require'}),('optics18/review_saved18.py',{'POST_SCENE_READ_ONLY','review_post_scene_commands','review_protocol_command_admission'})]:
 tree=ast.parse((B/name).read_bytes());nodes=[n for n in tree.body if isinstance(n,ast.FunctionDef)and n.name in wanted or isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id in wanted for t in n.targets)];exec(compile(ast.fix_missing_locations(ast.Module(body=nodes,type_ignores=[])),'EXTRACTED_SOURCE_RECORD_CONTROLS','exec'),scope)
class PostSceneSourceControls(unittest.TestCase):
 def setUp(self):
  self.scene={'worker':0,'request':{'cmd':'scene','id':-1},'response_sequence':3}
  self.entries=[{'sequence':1,'direction':'created','worker':0,'url':'SYNTHETIC_SOURCE_FIXTURE'}, {'sequence':2,'direction':'request','worker':0,'origin':'observer-read-only-scene','value':{'cmd':'scene','id':-1}}, {'sequence':3,'direction':'response','worker':0,'value':{'id':-1,'type':'done'}}]
 def append(self,cmd,origin='app',id=4,complete=False):
  self.entries.append({'sequence':4,'direction':'request','worker':0,'origin':origin,'value':{'id':id,'cmd':cmd}})
  if complete:self.entries.append({'sequence':5,'direction':'response','worker':0,'value':{'id':id,'type':'done'}})
 def review(self):scope['review_post_scene_commands'](self.entries,self.scene)
 def test_pending_step_rejected_at_request(self):
  self.append('step')
  with self.assertRaisesRegex(ValueError,'late mutating or unknown app command'):self.review()
 def test_completed_late_step_not_hidden_by_quiescence(self):
  self.append('step',complete=True)
  with self.assertRaisesRegex(ValueError,'late mutating or unknown app command'):self.review()
 def test_reset_restore_locale_and_all_other_source_mutators_rejected(self):
  base=copy.deepcopy(self.entries);names=json.loads((B/'WORKER-COMMAND-AUTHORITY.json').read_bytes())['mutating_or_initializing_commands'];self.assertEqual(len(names),12)
  for command in names:
   with self.subTest(command=command):
    self.entries=copy.deepcopy(base);self.append(command,complete=True)
    with self.assertRaisesRegex(ValueError,'late mutating or unknown app command'):self.review()
 def test_unknown_command_holds(self):
  self.append('future_unknown_command',complete=True)
  with self.assertRaisesRegex(ValueError,'late mutating or unknown app command'):self.review()
 def test_ordinary_unchanged_trailing_readonly_completion_allowed(self):
  self.append('grammar',complete=True);self.review()
 def test_owned_negative_readonly_scene_snapshot_allowed(self):
  base=copy.deepcopy(self.entries)
  for command in ['scene','snapshot']:
   self.entries=copy.deepcopy(base);self.append(command,'observer-read-only-'+command,-2,True);self.review()
 def test_unknown_request_origin_holds(self):
  self.append('scene','unowned-origin')
  with self.assertRaisesRegex(ValueError,'unknown trailing request owner'):self.review()
 def test_scene_worker_identity_must_match_actual_terminal(self):
  self.scene['worker']=1
  with self.assertRaisesRegex(ValueError,'owned final scene terminal missing'):self.review()
 def test_locale_selection_and_recording_scope_same_drawer_owner(self):
  source=(B/'optics18/optics-controls.mjs').read_text();self.assertIn("gui.locator('section.utility-drawer .locale select')",source);self.assertIn("document.querySelector('section.utility-drawer .locale select')",source);self.assertNotIn("gui.locator('.locale select')",source);self.assertNotIn("document.querySelector('.locale select')",source)
class FullIntervalSourceControls(unittest.TestCase):
 def setUp(self):
  self.entries=[{'sequence':1,'worker':0,'direction':'created','url':'SYNTHETIC_METADATA_ONLY'}]
  for id,cmd in [(10,'set_register'),(11,'run_script'),(12,'run_script')]:
   value={'id':id,'cmd':cmd};self.entries.extend([{'worker':0,'direction':'request','origin':'app','value':value},{'worker':0,'direction':'response','value':{'id':id,'type':'done'}}])
  self.entries.extend([{'worker':0,'direction':'request','origin':'observer-read-only-scene','value':{'id':-1,'cmd':'scene'}},{'worker':0,'direction':'response','value':{'id':-1,'type':'done'}}])
 def inject(self,cmd,id=100,worker=0,origin='app'):
  self.entries[3:3]=[{'worker':worker,'direction':'request','origin':origin,'value':{'id':id,'cmd':cmd}},{'worker':worker,'direction':'response','value':{'id':id,'type':'done'}}]
 def review(self):
  for seq,e in enumerate(self.entries,1):e['sequence']=seq
  protocol=[]
  for id in [10,11,12]:
   q=next(e for e in self.entries if e['worker']==0 and e['direction']=='request'and e['value']['id']==id);r=next(e for e in self.entries if e['worker']==0 and e['direction']=='response'and e['value']['id']==id)
   protocol.append({'worker':0,'origin':'app','request_sequence':q['sequence'],'response_sequence':r['sequence'],'request':q['value'],'response':r['value']})
  scene={'worker':0,'request':{'id':-1,'cmd':'scene'},'response_sequence':self.entries[-1]['sequence']};scope['review_protocol_command_admission'](self.entries,scene,protocol)
 def test_exact_prescribed_protocol_allowed(self):self.review()
 def test_load_pack_before_final_scene_rejected(self):
  self.inject('load_pack')
  with self.assertRaisesRegex(ValueError,'unprescribed mutation or unknown'):self.review()
 def test_quest_start_before_final_scene_rejected(self):
  self.inject('quest_start')
  with self.assertRaisesRegex(ValueError,'unprescribed mutation or unknown'):self.review()
 def test_init_before_final_scene_rejected(self):
  self.inject('init')
  with self.assertRaisesRegex(ValueError,'unprescribed mutation or unknown'):self.review()
 def test_unknown_before_final_scene_rejected(self):
  self.inject('future_unknown_command')
  with self.assertRaisesRegex(ValueError,'unprescribed mutation or unknown'):self.review()
 def test_mutator_impostor_wrong_worker_rejected(self):
  self.inject('run_script',id=11,worker=1)
  with self.assertRaisesRegex(ValueError,'unprescribed mutation or unknown'):self.review()
 def test_mutator_impostor_wrong_request_id_rejected(self):
  self.inject('run_script',id=100)
  with self.assertRaisesRegex(ValueError,'unprescribed mutation or unknown'):self.review()
 def test_interleaved_readonly_preserves_exact_protocol(self):self.inject('grammar');self.review()

if __name__=='__main__':unittest.main(verbosity=2)
