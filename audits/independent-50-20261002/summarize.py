import json
from pathlib import Path
root=Path(__file__).resolve().parent
summaries=[]
for c in json.loads((root/'predictions.json').read_text()):
    p=root/'results'/f"{c['id']}.json.stdout"
    steps=[json.loads(s) for s in p.read_text().splitlines() if s.strip()]
    row=dict(id=c['id'],title=c['title'],steps=len(steps),measurements=[],special_events=[],final=[])
    for s in steps:
        if 'measurement' in s: row['measurements'].append(s['measurement'])
        for e in s.get('events',[]):
            if e.get('event') in ('measured','not_modelled','cell','electrolysed','gas_transferred','gas_evolved','thermal_budget'): row['special_events'].append(e)
    if steps:
        row['last_keys']=list(steps[-1])
        for v in steps[-1].get('bench',{}).get('vessels',[]):
            row['final'].append(dict(id=v['id'],temperature_c=v['temperature']-273.15,ph=(v.get('solution') or {}).get('ph'),pressure_pa=v['pressure'],contents=v['contents'],headspace=v['headspace'],solution_scope=(v.get('solution') or {}).get('scope')))
    row['stderr']=(root/'results'/f"{c['id']}.json.stderr").read_text()
    summaries.append(row)
(root/'summary.json').write_text(json.dumps(summaries,indent=2)+'\n')
for r in summaries:
 print(r['id'],r['title'])
 for v in r['final']:
  print(' v',v['id']+1,'T',round(v['temperature_c'],5),'pH',v['ph'],'P',round(v['pressure_pa'],2),'scope',v['solution_scope'])
  print(' ',[(x['species'],round(x['moles'],9),x['phase']) for x in v['contents']])
 if r['stderr']: print(' ERROR',r['stderr'].strip())
