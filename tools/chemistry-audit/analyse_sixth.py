#!/usr/bin/env python3
"""Independent sixth-fleet checks; thresholds frozen before observations."""
import argparse
import json
import math
from pathlib import Path
import re
import tempfile

ROOT = Path(__file__).resolve().parents[2]

def close(a, b, absolute=1e-8, relative=1e-6):
    return math.isfinite(a) and math.isfinite(b) and abs(a-b) <= absolute + relative*max(abs(a),abs(b))

def analyse(directory):
    summary=json.loads((directory/'summary.json').read_text())
    records={int(r['id'].split('-')[0]):r for r in summary['cases']}
    if len(summary['cases']) != 24 or set(records) != set(range(195,219)):
        raise ValueError('Exactly 24 distinct cases 195–218 required')
    registry=json.loads((ROOT/'data/registry/registry-source-v1.json').read_text())
    composition={r['species_id']:{e['element']:e['count']['value'] for e in r['elements']} for r in registry['compositions']}
    rows, scripts, malformed={}, {}, {}
    for k,r in records.items():
        source=directory/r['id']
        rows[k],malformed[k]=[],[]
        try:
            scripts[k]=(source/'experiment.lab').read_text()
            for line in (source/'stdout.ndjson').read_text().splitlines():
                try:
                    row=json.loads(line)
                    if not isinstance(row,dict):
                        raise ValueError('row not object')
                    rows[k].append(row)
                except ValueError as exc:
                    malformed[k].append(str(exc))
        except OSError as exc:
            scripts[k]=''
            malformed[k].append(str(exc))
    checks=[]
    def check(name, fn, kind='model-law'):
        try:
            passed,evidence=fn()
        except (KeyError,IndexError,StopIteration,TypeError,ValueError,ZeroDivisionError,OverflowError) as exc:
            passed,evidence=False,{'missing_or_invalid_output':str(exc)}
        checks.append(dict(check=name,kind=kind,passed=bool(passed),evidence=evidence))
    def vessel(bench):
        return next(v for v in bench['vessels'] if v['id']==0)
    def final(k):
        return vessel(records[k]['final'])
    def events(k,name):
        return [e for row in rows[k] for e in row.get('events',[]) if e['event']==name]
    def atoms(v,e):
        return sum(p['moles']*composition[p['species']].get(e,0) for p in v['contents'])
    def amounts(v):
        result={}
        for p in v['contents']:
            key=(p['species'],p['phase'])
            result[key]=result.get(key,0)+p['moles']
        return result
    def before(k,op):
        i=next(i for i,r in enumerate(rows[k]) if r.get('operator',{}).get('op')==op)
        return vessel(next(r['bench'] for r in reversed(rows[k][:i]) if 'bench' in r))
    def feed(k,key):
        return sum(float(n) for name,n in re.findall(r'^add v1 (\S+) ([0-9.eE+-]+)mol$',scripts[k],re.M) if name==key)
    for k,r in records.items():
        def execution(k=k,r=r):
            last=next(x['bench'] for x in reversed(rows[k]) if 'bench' in x)
            valid=(r['exit']==0 and bool(rows[k]) and bool(scripts[k]) and not r['non_json_lines'] and not malformed[k]
                   and last==r['final'] and bool(last['vessels']) and not events(k,'solver_failed')
                   and all(p['species'] in composition and math.isfinite(p['moles']) and p['moles']>=0 for v in last['vessels'] for p in v['contents']))
            return valid,dict(exit=r['exit'],rows=len(rows[k]),malformed=malformed[k],solver_failures=events(k,'solver_failed'))
        check(f'{k}: finite registered complete execution',execution,'execution')
    for k in range(195,201):
        def endpoint(k=k):
            e=events(k,'titrated')[-1]
            expected=feed(k,'NaOH')-feed(k,'HCl')
            actual=e['concentration']*e['total_volume']
            return (e['titrant']=='HCl' and e['endpoint_reached'] is True and abs(e['final_ph']-7)<.02
                    and close(actual,expected,absolute=2e-7,relative=.002)),dict(expected_acid_mol=expected,delivered_acid_mol=actual,event=e)
        check(f'{k}: descending endpoint and net base equivalents',endpoint)
        def inventory(k=k):
            dose=events(k,'titrated')[-1]
            expected={'Na':feed(k,'NaOH'),'Cl':feed(k,'HCl')+dose['concentration']*dose['total_volume']}
            actual={e:atoms(final(k),e) for e in expected}
            return all(close(actual[e],n) for e,n in expected.items()),dict(expected=expected,actual=actual)
        check(f'{k}: titrant and starting solute element ledger',inventory,'conservation')
    def conductivity(k):
        e=events(k,'measured')[-1]
        if e['unit']!='µS/cm' or e['instrument']!='conductivity_meter':
            raise ValueError('aqueous conductivity unit missing or wrong')
        return e['value']
    for k in range(201,207):
        check(f'{k}: explicit finite nonnegative conductivity',lambda k=k:(math.isfinite(conductivity(k)) and conductivity(k)>=0,conductivity(k)))
    check('201/202/203/206: charge-carrier dose and electrolyte controls',lambda:
          (conductivity(203)>conductivity(202)>conductivity(201)+.01 and conductivity(206)>conductivity(201)+.01,
           {k:conductivity(k) for k in (201,202,203,206)}))
    check('202/204: intensive conductivity under extensive scaling',lambda:
          (close(conductivity(202),conductivity(204),absolute=.01,relative=1e-4),[conductivity(202),conductivity(204)]))
    check('201/202/205: neutral-solute negative electrical control',lambda:
          (abs(conductivity(205)-conductivity(201))<.01*(conductivity(202)-conductivity(201)),
           {k:conductivity(k) for k in (201,202,205)}))
    for k in range(207,213):
        def gaslaw(k=k):
            v=final(k)
            gas=sum(p['moles'] for p in v['contents'] if p['phase']=='gas')
            expected=gas*8314.46261815324*v['temperature']/v['headspace']['volume']
            initial=before(k,'heat')
            conserved=all(close(atoms(v,e),atoms(initial,e),absolute=1e-10) for e in ('N','O','H'))
            return (v['headspace']['boundary']=='sealed' and gas>0 and conserved and close(v['pressure'],expected,absolute=.001)),dict(expected_pressure_Pa=expected,pressure_Pa=v['pressure'],gas_mol=gas,conserved=conserved)
        check(f'{k}: dry sealed gas pressure and retained atoms',gaslaw,'conservation')
    check('207/209/211: partitioned and extensive energy controls',lambda:
          (all(close(final(207)['temperature'],final(k)['temperature'],absolute=.002,relative=1e-6) for k in (209,211)),{k:final(k)['temperature'] for k in (207,209,211)}))
    check('207/210: larger energy raises dry gas temperature',lambda:
          (final(210)['temperature']>final(207)['temperature'],[final(207)['temperature'],final(210)['temperature']]))
    check('212: equal positive/negative energy restores temperature',lambda:
          (close(final(212)['temperature'],before(212,'heat')['temperature'],absolute=.002),[before(212,'heat')['temperature'],final(212)['temperature']]))
    for k in range(213,219):
        def ester(k=k):
            initial=before(k,'react')
            es=[e for e in events(k,'org_reacted') if e['name']=='esterification']
            expected=scripts[k].count('react v1 esterification')
            extent=sum(e['extent'] for e in es)
            n={key:sum(p['moles'] for p in initial['contents'] if p['species']==key) for key in ('CH3COOH','ethanol','ethyl_acetate','water')}
            q=(n['ethyl_acetate']+extent)*(n['water']+extent)/((n['CH3COOH']-extent)*(n['ethanol']-extent))
            return (len(es)==expected and all(e.get('boundary') for e in es) and abs(q-4)<1e-4
                    and all(abs(e['extent'])<1e-8 for e in es[1:]) and all(close(atoms(initial,e),atoms(final(k),e)) for e in ('C','H','O'))),dict(events=es,current_state_Q=q)
        check(f'{k}: equilibrium law and explicit repeated zero-extent outcome',ester)
        def unchanged(k=k):
            reactions=[r for r in rows[k] if r.get('operator',{}).get('op')=='react']
            if len(reactions)<2:
                raise ValueError('missing repeated react observations')
            first,last=vessel(reactions[0]['bench']),final(k)
            a,b=amounts(first),amounts(last)
            valid=all(close(a.get(key,0),b.get(key,0),absolute=1e-8,relative=1e-6) for key in a.keys()|b.keys())
            valid=valid and close(first['temperature'],last['temperature'],absolute=.002) and close(first['pressure'],last['pressure'],absolute=.001)
            return valid,dict(max_mole_change=max(abs(a.get(key,0)-b.get(key,0)) for key in a.keys()|b.keys()))
        check(f'{k}: no-op material state unchanged',unchanged,'conservation')
    return dict(fleet='sixth-195-218',checks=checks,passed=sum(c['passed'] for c in checks),unmet=sum(not c['passed'] for c in checks))

def self_test():
    import sixth_batch
    cases=sixth_batch.recorder.CASES
    assert len(cases)==len({c['script'] for c in cases})==24
    with tempfile.TemporaryDirectory(prefix='kero-sixth-negative-') as temporary:
        directory=Path(temporary)
        records=[]
        for c in cases:
            source=directory/c['id']; source.mkdir()
            (source/'experiment.lab').write_text(c['script'])
            (source/'stdout.ndjson').write_text('')
            records.append(dict(id=c['id'],exit=0,non_json_lines=[],final=None,stderr=''))
        (directory/'summary.json').write_text(json.dumps(dict(cases=records)))
        result=analyse(directory)
        assert result['passed']==0 and result['unmet']>=24,result
    print(json.dumps(dict(distinct_inputs=24,missing_output_negative_control='passed',rejected_checks=result['unmet'])))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory',type=Path,nargs='?')
    parser.add_argument('--out',type=Path)
    parser.add_argument('--self-test',action='store_true')
    args=parser.parse_args()
    if args.self_test:
        self_test()
    else:
        if args.directory is None:
            parser.error('directory required')
        result=analyse(args.directory)
        text=json.dumps(result,indent=2,allow_nan=False)+'\n'
        if args.out:
            with args.out.open('x') as handle:
                handle.write(text)
        print(text,end='')
        raise SystemExit(bool(result['unmet']))
