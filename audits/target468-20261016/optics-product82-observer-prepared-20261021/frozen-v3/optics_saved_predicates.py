"""Pure prospective optics18 predicates; no browser/provider/runtime imports.

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

def review_case(row,case,words,oracle):
 require(row['case_id']==case['id']and row['locale']in case['locales'],'case/locale mismatch');lab=row['lab'];require(lab['setup_script']==case['setup']and lab['continuation_script']==case['continuation'],'frozen protocol changed');require(lab['calls_before_measure']==lab['calls_after_measure'],'passive observation invoked chemistry');before=v(lab['setup_result']);after=review_result(case,before,lab['continuation_result'],words,oracle)
 # GUI importLab submits each actual line separately; it is not a full-script
 # runScript call. Preserve transparent request/done pairs in original order.
 lines=[x.strip()for x in(case['setup']+case['continuation']).splitlines()if x.strip()];pairs=row['gui']['protocol_pairs'];require(len(pairs)==len(lines),'missing/extra GUI command');require(len({p['request']['id']for p in pairs})==len(pairs),'duplicate GUI request ID');results=[gui_result(p,line)for p,line in zip(pairs,lines,strict=True)];gbefore=v(results[-3]);domain(case,gbefore);require(v(results[-2])==v(results[-1])==gbefore,'GUI passive look/measure changed actual owned vessel');require(row['gui']['locale_request']['request']['cmd']=='set_locale'and row['gui']['locale_request']['request']['code']==row['locale'],'GUI locale not sent');require(row['gui']['locale_request']['response']['type']=='done'and row['gui']['locale_request']['request']['id']==row['gui']['locale_request']['response']['id'],'GUI locale not completed');require(row['gui']['locale_request']['response_sequence']<pairs[0]['request_sequence'],'locale was not completed before chemistry')
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
