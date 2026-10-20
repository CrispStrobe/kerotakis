"""Pure saved-raw presentation predicates. No browser or chemistry execution."""
import math
WORDS={
 'de':{'temperature':'Temperatur','minimum':'mindestens','catalyst':'Katalysator','alternatives':['Schwefelsäure','Wasserstoffionen',' oder ']},
 'fr':{'temperature':'température','minimum':'au moins','catalyst':'catalyseur','alternatives':['acide sulfurique','ions hydrogène',' ou ']},
}
def phrase(value,key):
 if isinstance(value,dict):
  if value.get('key')==key:return value
  children=value.values()
 elif isinstance(value,list):children=value
 else:return None
 for child in children:
  found=phrase(child,key)
  if found is not None:return found
 return None
def slot(value,name):return next((v for k,v in value.get('slots',[])if k==name),None)
def events(result):return [event for step in result['steps']for event in step['events']]
def rendered(result):
 lines=[]
 for step in result['steps']:
  value=step.get('rendered',[]);assert isinstance(value,list)and all(isinstance(x,str)for x in value);lines.extend(value)
 return '\n'.join(lines)
def terms(value):
 found=[]
 if isinstance(value,dict):
  if 'term'in value:found.append(value['term']['en'])
  for child in value.values():found.extend(terms(child))
 elif isinstance(value,list):
  for child in value:found.extend(terms(child))
 return found
def amount(v,key):return sum(p['moles']for p in v['contents']if p['species']==key)
def validate_family_rows(raw):
 checked=[]
 for locale,words in WORDS.items():
  cold=[]
  for name,gate in [('cold15','temperature_k'),('cold25','temperature_k'),('warm65','catalyst')]:
   row=raw[locale+'-family-'+name];result=row['preparation'];vessels=result['bench']['vessels'];assert len(vessels)==1;v=vessels[0];ee=events(result);text=rendered(result)
   assert v['id']==0 and math.isfinite(v['temperature'])and abs(v['elapsed_seconds']-1)<1e-12
   assert all(math.isfinite(p['moles'])and p['moles']>=0 for p in v['contents'])
   assert abs(amount(v,'CH3COOH')-.01)<1e-12 and abs(amount(v,'ethanol')-.01)<1e-12 and amount(v,'ethyl_acetate')==0
   assert not any(e['event']=='org_reacted'and e.get('name')=='fischer-esterification'for e in ee)
   matches=[e for e in ee if e['event']=='not_yet_modeled'and e.get('reason',{}).get('key')=='not-modeled.family-declined-at-gate'and slot(e['reason'],'family')=={'text':{'text':'fischer-esterification'}}];assert matches
   for event in matches:
    assert event['vessel']==v['id'];reason=event['reason'];assert slot(reason,'gate')['term']['en']==gate
    if gate=='temperature_k':
     detail=phrase(reason,'family-gate.temperature');assert detail is not None
     actual=slot(detail,'actual')['number']['text'];window=phrase(detail,'family-gate.window-min');assert window is not None;minimum=slot(window,'min')['number']['text']
     assert math.isfinite(float(actual))and abs(float(actual)-v['temperature'])<.06 and abs(float(minimum)-333.15)<.06
     # Independent formatting of the owned numeric state and source-declared window.
     # Pinned FamilyRouter uses one decimal; never trust a self-reported actual/min alone.
     assert actual==format(v['temperature'],'.1f') and minimum==format(333.15,'.1f') and v['temperature']<333.15
     assert words['temperature']in text and words['minimum']in text and actual.replace('.',',')in text and minimum.replace('.',',')in text
     assert 'the vessel stands at'not in text and 'temperature_k'not in text
    else:
     detail=phrase(reason,'family-gate.catalyst-missing');assert detail is not None
     wanted=slot(detail,'wanted');alternative=phrase(wanted,'family-gate.alternative');assert alternative is not None
     assert set(terms(wanted))=={'H2SO4','H+'},'Require actual nested alternatives, not a stray string'
     assert words['catalyst']in text and all(w in text for w in words['alternatives'])
     assert 'the family needs its catalyst present'not in text
     assert v['temperature']>=333.15 and v['free_proton']==0 and amount(v,'H2SO4')==0
   if gate=='temperature_k':cold.append(v['temperature'])
   checked.append({'case':locale+'-family-'+name,'actual_temperature_k':v['temperature'],'retained_acid_mol':amount(v,'CH3COOH'),'retained_ethanol_mol':amount(v,'ethanol'),'structured_gate':gate,'localized_rendered':text})
  assert cold[0]!=cold[1]
 return checked
