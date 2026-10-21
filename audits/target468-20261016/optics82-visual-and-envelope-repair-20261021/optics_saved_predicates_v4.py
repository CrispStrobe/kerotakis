"""Pure prospective optics18 v4 actual-register predicates; no browser/provider/runtime imports.

This defines the NEW observer record schema, not a claim existing raw evidence
has these fields. Source/asset/worker/issue/manifest authentication stays in the
retained v1 chain. Missing prerequisites HOLD the whole group; never skip rows.
"""
import json,math
class PrerequisiteHold(ValueError):pass
def require(value,message):
 if not value:raise ValueError(message)
def finite(x):return type(x)in(int,float)and math.isfinite(x)
def events(result):return [event for step in result['steps']for event in step['events']]
def typed_species(reason):
 slot=next(value for key,value in reason['slots']if key=='species')
 return [item['term']for item in slot['list']['items']]
def v(result):
 vs=result['bench']['vessels'];require(len(vs)==1 and vs[0]['id']==0,'one actual owned vessel required');return vs[0]
def pool(vessel,key):return sum(x['moles']for x in vessel['contents']if x['species']==key and x['phase']in ['aqueous','liquid'])
def scene_v(scene):
 rows=[x for x in scene['vessels']if x['id']==0];require(len(rows)==1 and rows[0]['liquid']is not None,'actual liquid scene required');return rows[0]
def gui_result(pair,line):
 request,response=pair['request'],pair['response'];require(request['cmd']=='run_script'and request['script'].strip()==line,'actual GUI command mismatch');require(response['type']=='done'and response['id']==request['id'],'actual matched worker done required');return json.loads(response['result_json'])
def domain(case,before):
 ph=(before.get('solution')or{}).get('ph');missing=case['expected_missing_ph'];indicator=case['id']not in ['non-ph-blue','blank-pure-water','empty-sample']
 if case['id']=='empty-sample':
  if before['contents'] or finite(ph):raise PrerequisiteHold('fresh empty-sample prerequisite absent')
 if missing and finite(ph):raise PrerequisiteHold('live solver supplied finite pH: missing-pH arm not demonstrated')
 if not missing and case['id']in ['solved-indicator','acidic-solved-indicator']and not finite(ph):raise PrerequisiteHold('live represented pH absent: supported-positive prerequisite not demonstrated')
 if indicator and not pool(before,'methyl_orange')>0:raise PrerequisiteHold('positive aqueous/liquid indicator absent')
 if case['id']in ['non-ph-blue','mixed-partial']and not pool(before,'indigo_carmine')>0:raise PrerequisiteHold('actual blue carrier/dye role absent')
 if case['id']=='non-ph-blue'and finite(ph):raise PrerequisiteHold('non-pH absorber independence requires actual absent finite pH')
 if case['id']=='acidic-solved-indicator'and not ph<3:raise PrerequisiteHold('independent HCl positive did not establish acidic represented pH')
def review_result(case,before,result,words,oracle):
 after=v(result);require(before==after,'passive look/UVvis changed owned state, phase, derived cache or clock');domain(case,before)
 es=events(result);require(not any(e['event']=='solver_failed'for e in es),'actual solver failure');measured=[e for e in es if e['event']=='measured'and e['instrument']=='spectrophotometer'];reason=[e['reason']for e in es if e['event']=='not_yet_modeled'and e.get('reason',{}).get('key')=='not-modeled.indicator-optics-missing-ph'];look=[e for e in es if e['event']=='observed'];require(len(look)==1,'actual look observation required');notes=look[0]['appearance']['notes'];phnotes=[n for n in notes if n['key']=='look.indicator-optics-missing-ph'];rendered='\n'.join(x for step in result['steps']for x in step.get('rendered',[]))
 if case['id']=='empty-sample':
  no_sample=[e for e in es if e['event']=='not_yet_modeled'and e.get('reason',{}).get('key')=='not-modeled.spectrophotometer-has-no-solution'];require(len(no_sample)==1 and not measured and not reason and not phnotes,'empty sample requires typed refusal, never measured zero');require(words['core_no_sample']in rendered,'empty sample refusal not localized');return after
 if case['expected_missing_ph']:
  require(not measured and len(reason)==len(phnotes)==1,'typed missing-pH refusal and look note required; never zero');require(typed_species(reason[0])==typed_species(phnotes[0])==[{'section':'species','en':'methyl_orange'}],'canonical typed species identity changed');require(words['core_refusal']in rendered and words['core_look']in rendered,'localized actual core words/species missing');require('methyl_orange'not in rendered and '{species}'not in rendered,'raw known-species identifier or placeholder leaked into prose')
 else:
  require(not reason and not phnotes and len(measured)==1 and finite(measured[0]['value']),'complete supported finite instrument reading required')
  require(abs(measured[0]['value'])<=1e-12 if case['expected_measurement']=='zero'else measured[0]['value']>0,'blank/positive distinction lost')
 # Independently reproduce supported peak absorbance from frozen band data,
 # actual owned liquid volume and actual finite pH. No production optics
 # helper or expected_result is used as an observation.
 if measured:
  props=oracle['species'];litres=sum(x['moles']*props[x['species']]['molar_mass_g_mol']/props[x['species']]['density_g_ml']/1000 for x in before['contents']if x['phase']=='liquid');require(finite(litres)and litres>0,'owned liquid volume absent');bands=[0.0]*16
  indigo=pool(before,'indigo_carmine')
  for i,epsilon in enumerate(oracle['indigo_epsilon']):bands[i]+=epsilon*indigo/litres
  indicator=pool(before,'methyl_orange')
  if indicator:
   ph=before['solution']['ph'];fraction=1/(1+10**(oracle['methyl_orange_pka']-ph))
   for i,nm in enumerate(oracle['band_nm']):
    acid=24000*math.exp(-.5*((nm-505)/50)**2);base=26000*math.exp(-.5*((nm-464)/55)**2);bands[i]+=((1-fraction)*acid+fraction*base)*indicator/litres
  expected=max(bands);require(abs(measured[0]['value']-expected)<=max(1e-12,abs(expected)*1e-10),'actual UVvis differs from independent declared16-band peak')
 return after

def traffic_pairs(entries):
 """Independently reconstruct matched real envelopes; never trust copied pairs."""
 require(type(entries)is list and entries,'retained genuine worker traffic absent')
 workers=set();requests={};terminals={};previous=0
 for entry in entries:
  seq=entry['sequence'];worker=entry['worker'];direction=entry['direction']
  require(type(seq)is int and seq>previous,'worker traffic must have strictly increasing integer sequences');previous=seq
  require(type(worker)is int and worker>=0,'invalid actual worker identity')
  if direction=='created':
   require(worker not in workers,'duplicate worker creation');workers.add(worker);continue
  require(worker in workers,'message precedes actual worker creation')
  require(direction in ['request','response'],'unknown worker traffic direction')
  value=entry['value'];require(type(value)is dict and type(value.get('id'))is int,'typed actual envelope ID required');key=(worker,value['id'])
  if direction=='request':
   require(key not in requests,'request ID reused within worker')
   require(type(value.get('cmd'))is str,'request command absent')
   origin=entry['origin'];require(origin in ['app','observer-read-only-snapshot','observer-read-only-scene'],'unknown request origin')
   require(value['id']>0 if origin=='app'else value['id']<0,'request origin/ID ownership differs')
   if origin!='app':require(origin=='observer-read-only-'+value['cmd']and value['cmd']in ['snapshot','scene'],'observer issued non-read-only operation')
   requests[key]=entry
  else:
   require(key in requests and requests[key]['sequence']<seq,'response without earlier owned request')
   require(value.get('type')in ['progress','done','error','cancelled'],'unknown actual response type')
   require(key not in terminals,'response after terminal or duplicate terminal ID within worker')
   if value['type']!='progress':terminals[key]=entry
 pairs=[]
 for key,q in requests.items():
  r=terminals.get(key)
  if r is not None:pairs.append({'request':q['value'],'response':r['value'],'request_sequence':q['sequence'],'response_sequence':r['sequence'],'worker':q['worker'],'origin':q['origin']})
 return pairs

def review_traffic(gui):
 derived=traffic_pairs(gui['worker_traffic'])
 consumed=[*gui['protocol_pairs'],gui['locale_request'],gui['setup_snapshot_pair'],gui['scene_pair'],gui['hello']]
 require(consumed and all(pair in derived for pair in consumed),'copied GUI pair differs from retained request/terminal response')
 require(len({(p['worker'],p['request']['id'])for p in consumed})==len(consumed),'one raw pair consumed under multiple roles')
 workers={p['worker']for p in consumed};require(len(workers)==1,'consumed GUI protocol/locale/snapshot/scene/hello have different worker owners')
 worker=next(iter(workers));protocol=gui['protocol_pairs'];require(len(protocol)>=3,'register/look/measure protocol incomplete')
 require(all(p['origin']=='app'and p['response']['type']=='done'for p in protocol),'GUI protocol must originate in app and genuinely complete')
 require(all(a['response_sequence']<b['request_sequence']for a,b in zip(protocol,protocol[1:])),'GUI protocol command order overlaps or reverses')
 locale=gui['locale_request'];require(locale['origin']=='app'and locale['response_sequence']<protocol[0]['request_sequence'],'locale was not genuinely completed before protocol')
 snap=gui['setup_snapshot_pair'];scene=gui['scene_pair']
 require(snap['origin']=='observer-read-only-snapshot'and snap['request']['cmd']=='snapshot'and snap['request']['id']<0,'snapshot ownership is not explicit read-only observer')
 require(scene['origin']=='observer-read-only-scene'and scene['request']['cmd']=='scene'and scene['request']['id']<0,'scene ownership is not explicit read-only observer')
 require(protocol[-3]['response_sequence']<snap['request_sequence']<snap['response_sequence']<protocol[-2]['request_sequence'],'snapshot not completed after final setup and before look')
 require(protocol[-1]['response_sequence']<scene['request_sequence']<scene['response_sequence'],'scene not obtained after final actual measure')
 start=protocol[0]['request_sequence'];end=scene['response_sequence']
 actual_protocol=[p for p in derived if p['worker']==worker and p['origin']=='app'and start<=p['request_sequence']<end and p['request']['cmd']in ['set_register','run_script']]
 require(actual_protocol==protocol,'missing/extra app chemistry or register commands in retained interval')
 raw_requests=[e for e in gui['worker_traffic']if e['direction']=='request'and e['worker']==worker and start<=e['sequence']<end]
 raw_protocol=[{'request':e['value'],'request_sequence':e['sequence'],'origin':e['origin']}for e in raw_requests if e['origin']=='app'and e['value']['cmd']in ['set_register','run_script']]
 require(raw_protocol==[{'request':p['request'],'request_sequence':p['request_sequence'],'origin':p['origin']}for p in protocol],'extra unfinished chemistry/register request in retained interval')
 require(not any(e['value']['cmd']in ['set_locale','restore','reset','step']for e in raw_requests),'locale or alternate mutation changed the observed protocol interval')
 hello=gui['hello'];require(hello['origin']=='app'and hello['request']['cmd']=='hello'and hello['response']['type']=='done'and hello['response_sequence']<start,'actual owned startup hello absent')
 return derived

def review_case(row,case,words,oracle):
 require(row['case_id']==case['id']and row['locale']in case['locales'],'case/locale mismatch');lab=row['lab'];require(lab['setup_script']==case['setup']and lab['continuation_script']==case['continuation'],'frozen protocol changed');require(lab['calls_before_measure']==lab['calls_after_measure'],'passive observation invoked chemistry');before=v(lab['setup_result']);after=review_result(case,before,lab['continuation_result'],words,oracle)
 review_traffic(row['gui'])
 # GUI importLab submits each actual line separately; it is not a full-script
 # runScript call. Preserve transparent request/done pairs in original order.
 lines=[x.strip()for x in(case['setup']+case['continuation']).splitlines()if x.strip()];require(lines[0]=='register lv3','frozen register protocol changed');pairs=row['gui']['protocol_pairs'];require(len(pairs)==len(lines),'missing/extra GUI command');require(len({p['request']['id']for p in pairs})==len(pairs),'duplicate GUI request ID');reg=pairs[0];require(reg['request']['cmd']=='set_register'and reg['request']['level']=='lv3'and reg['response']['type']=='done'and reg['request']['id']==reg['response']['id']and json.loads(reg['response']['result_json'])=={},'actual GUI register contract');results=[gui_result(p,line)for p,line in zip(pairs[1:],lines[1:],strict=True)];snap=row['gui']['setup_snapshot_pair'];require(snap['request']['cmd']=='snapshot'and snap['request']['id']==snap['response']['id']and snap['response']['type']=='done'and snap['request_sequence']>pairs[-3]['response_sequence']and snap['response_sequence']<pairs[-2]['request_sequence'],'genuine post-setup/pre-observation snapshot required');gb=json.loads(json.loads(snap['response']['result_json'])['snapshot']);require(len(gb['vessels'])==1 and gb['vessels'][0]['id']==0,'actual owned snapshot vessel');gbefore=gb['vessels'][0];domain(case,gbefore);require(v(results[-2])==v(results[-1])==gbefore,'GUI passive look/measure changed actual owned vessel');require(len(results)==2 or v(results[-3])==gbefore,'setup result and genuine snapshot differ');require(row['gui']['locale_request']['request']['cmd']=='set_locale'and row['gui']['locale_request']['request']['code']==row['locale'],'GUI locale not sent');require(row['gui']['locale_request']['response']['type']=='done'and row['gui']['locale_request']['request']['id']==row['gui']['locale_request']['response']['id'],'GUI locale not completed');require(row['gui']['locale_request']['response_sequence']<pairs[0]['request_sequence'],'locale was not completed before chemistry')
 merged={'bench':results[-1]['bench'],'steps':results[-2]['steps']+results[-1]['steps']};review_result(case,gbefore,merged,words,oracle)
 sp=row['gui']['scene_pair'];require(sp['request']['cmd']=='scene'and sp['request']['id']==sp['response']['id']and sp['response']['type']=='done','actual GUI scene response required');scene=json.loads(sp['response']['result_json']);
 if case['id']=='empty-sample':
  sv=next(x for x in scene['vessels']if x['id']==0);require(sv['liquid']is None and not sv.get('layers'),'empty GUI scene invented liquid');require(next(x for x in lab['scene_after']['vessels']if x['id']==0)['liquid']is None,'empty Lab scene invented liquid');dom=row['gui']['dom'];require(dom['vessel_id']=='0'and dom['locale_value']==row['locale']and not dom['layers']and not dom['hatches']and not dom['ph_readouts']and not dom['legacy_gap_readouts'],'empty sample fabricated liquid/hatch/readout');return {'case_id':case['id'],'locale':row['locale'],'all_conjunctive_predicates_met':True,'scientific_selections':0,'empirical_calibration_claim':False}
 sv=scene_v(scene);liquid=sv['liquid'];missing=case['expected_missing_ph'];require(liquid.get('optical_ph_gaps',[])==(['methyl_orange']if missing else[]),'scene typed pH gaps differ');require(not liquid.get('spectral_gaps',[]),'unexpected spectral gap; do not conflate causes');require(liquid==scene_v(lab['scene_after'])['liquid'],'GUI and independent Lab liquid presentation disagree')
 dom=row['gui']['dom'];require(dom['vessel_id']=='0'and dom['locale_value']==row['locale'],'actual DOM owner/locale mismatch');layers=dom['layers'];require(layers,'actual liquid DOM layer absent');expected=sv.get('layers')or[liquid];require(len(layers)==len(expected),'DOM/scene layer count differs')
 for layer,source in zip(layers,expected,strict=True):
  require(layer['data_srgb']==','.join(str(c)for c in source['srgb']),'underlying actual computed colour changed');require(layer['fill']=='rgb('+layer['data_srgb']+')','liquid fill differs from scene mapping');require(all(finite(layer[k])for k in ['x','y','width','height'])and layer['height']>0,'invalid actual liquid geometry')
 require(not dom['legacy_gap_readouts'],'missing pH mislabeled missing spectrum');require(len(dom['hatches'])==len(dom['ph_readouts'])==(1 if missing else 0),'one pH hatch/readout or none required')
 if missing:
  hatch=dom['hatches'][0];bounds={'x':min(x['x']for x in layers),'y':min(x['y']for x in layers),'width':max(x['x']+x['width']for x in layers)-min(x['x']for x in layers),'height':max(x['y']+x['height']for x in layers)-min(x['y']for x in layers)}
  require(all(finite(hatch[k])and abs(hatch[k]-bounds[k])<=1e-9 for k in bounds),'hatch must cover actual liquid column only, not whole vessel');require(hatch['data_ph_gaps']=='methyl_orange'and hatch['data_gaps']=='','canonical gaps belong in dataattrs');readout=dom['ph_readouts'][0];require(readout['small']==words['ui_small']and readout['strong']==words['ui_species']and readout['aria_label']==words['ui_aria']and hatch['title']==words['ui_aria'],'persistent/UI/core translated known-species labels incomplete');require('methyl_orange'not in readout['visible_text']and '{species}'not in readout['visible_text'],'identifier/placeholder leaked')
 if case['id']=='mixed-partial':require(len(set(liquid['srgb']))>1,'supported partial colour was lost to white/gray')
 return {'case_id':case['id'],'locale':row['locale'],'all_conjunctive_predicates_met':True,'scientific_selections':0,'empirical_calibration_claim':False}
