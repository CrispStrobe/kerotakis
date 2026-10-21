"""Validate passive physical state and the complete expected measurement journal."""
def require(value,message):
 if not value:raise ValueError(message)
def check_passive_journal(lab,english_lab,locale):
 before=lab['snapshot_before'];after=lab['snapshot_after']
 require(locale in ['en','de','fr']and before==english_lab['snapshot_before'],'canonical same-case English physical/journal baseline differs')
 require(type(before)is dict and type(after)is dict and set(before)==set(after),'snapshot schema changed')
 require('log'in before and all(before[key]==after[key]for key in before if key!='log'),'passive physical state, clock, cache or other snapshot field changed')
 previous=before['log'];journal=after['log'];require(type(previous)is list and type(journal)is list,'journal must be a list')
 require(len(journal)==len(previous)+2 and journal[:len(previous)]==previous,'journal prefix changed, truncated or extra commands appended')
 require(journal==english_lab['snapshot_after']['log'],'canonical journal differs across locales')
 steps=lab['continuation_result']['steps'];require(type(steps)is list and len(steps)==2,'exactly two measurement steps required')
 operators=[{'op':'measure','vessel':0,'instrument':'eyes'},{'op':'measure','vessel':0,'instrument':'spectrophotometer'}]
 for index,(step,operator)in enumerate(zip(steps,operators)):
  require(step['operator']==operator,'non-prescribed journal operator')
  events=step['events'];require(type(events)is list and len(events)==1,'exactly one measurement/refusal event required')
  require(events[0]['event']in(['observed']if index==0 else['measured','not_yet_modeled']),'non-observation journal event')
  canonical_events=english_lab['continuation_result']['steps'][index]['events']
  normalized=[]
  for event,canonical_event in zip(events,canonical_events,strict=True):
   event=__import__('copy').deepcopy(event)
   if locale!='en'and event['event']=='observed':
    require(type(event['appearance'].get('words'))is str and bool(event['appearance']['words']),'localized appearance words absent')
    event['appearance']['words']=canonical_event['appearance']['words']
   elif locale!='en'and event['event']=='not_yet_modeled':
    require(type(event.get('what'))is str and bool(event['what'])and type(event.get('reason'))is dict,'localized typed refusal absent')
    event['what']=canonical_event['what']
   normalized.append(event)
  require(normalized==canonical_events,'non-presentation event field differs from canonical English event')
  expected={'step':len(previous)+index,'operator':operator,'events':canonical_events}
  entry=journal[len(previous)+index]
  require(type(entry)is dict and type(entry.get('step'))is int and entry==expected,'journal entry/step/event differs from actual continuation')
 return {'physical_snapshot_exact':True,'original_journal_prefix_exact':True,'exact_prescribed_measurement_records':2}
