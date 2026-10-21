"""Independent source-only calculation. No engine run or empirical validation."""
import json, subprocess, hashlib, argparse
from decimal import Decimal as D, localcontext
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo',default='.')
parser.add_argument('--output',type=Path)
args=parser.parse_args()
SOURCE='28277d415b68fedda2f2efccf702811fe160fee9'
REPO=args.repo
def blob(p):
    return subprocess.check_output(['git','-C',REPO,'show',SOURCE+':'+p])
raw=blob('crates/kerotakis-core/tests/golden/registry.json')
water=next(s for s in json.loads(raw,parse_float=D) if s['key']=='water')
curves={x['phase']:x['intervals'] for x in water['heat_capacity_polys']}
def cp_terms(phase):
    # Endpoint holding is the actual model boundary, not physical supercooling data.
    interval=curves[phase][0]
    assert interval['form']=='nasa9'
    terms=list(zip(range(-2,5),map(D,interval['coefficients'])))
    if phase=='liquid':
        assert D(interval['t_min'])>=D('273.15')
        t=D(interval['t_min'])
        return [(0,R*sum(c*t**p for p,c in terms))]
    assert D(interval['t_min'])<=D(249)<D('273.15')<=D(interval['t_max'])
    return [(p,R*c) for p,c in terms]
def integral(terms,a,b,shift=0):
    return sum(c*((b.ln()-a.ln()) if p+shift==-1 else
                  (b**(p+shift+1)-a**(p+shift+1))/D(p+shift+1)) for p,c in terms)
def run(precision):
    global R
    with localcontext() as ctx:
        ctx.prec=precision
        R=D('8.31446261815324'); ref=D('273.15'); fusion=D(6010)
        delta=cp_terms('liquid')+[(p,-c) for p,c in cp_terms('solid')]
        def h(t): return fusion+integral(delta,ref,t)
        def logaw(t):
            return (fusion/ref-h(t)/t+integral(delta,ref,t,-1))/R
        rows=[]
        for label in ['273.15','270','260','252']:
            t=D(label)
            oldlog=fusion/R*(1/ref-1/t)
            lo,hi=D(249),ref
            assert logaw(lo)<=oldlog<=logaw(hi)
            for _ in range(220):
                mid=(lo+hi)/2
                if logaw(mid)<oldlog: lo=mid
                else: hi=mid
            new=(lo+hi)/2
            rows.append(dict(constant_fusion_temperature_k=str(t),
                activity=str(oldlog.exp()), source_fusion_j_per_mol=str(h(t)),
                source_fusion_fraction_change=str((h(t)-fusion)/fusion),
                variable_fusion_temperature_k=str(new),
                temperature_difference_k=str(new-t),
                root_log_activity_residual=str(logaw(new)-oldlog)))
        return rows
rows60=run(60); rows80=run(80)
assert all(abs(D(a['temperature_difference_k'])-D(b['temperature_difference_k']))<D('1e-50') for a,b in zip(rows60,rows80))
result={'source':SOURCE,'scope':'SOURCE_ONLY_NO_ENGINE_OR_EMPIRICAL_VALIDATION',
 'registry_sha256':hashlib.sha256(raw).hexdigest(),
 'assumptions':['Pure ice; temperature-independent imposed water activity for comparison.',
  'Use source liquid Cp endpoint holding below tabulated liquid interval; this is a model extension, not measured supercooled-water authority.',
  'Use source ice NASA9 within its shipped200..273.15K interval.',
  'Integrate ln(a_w)=integral(Tref,T) deltaHfusion(u)/(R*u^2) du.',
  'Reference fusion6010J/mol and Tref273.15K exactly as source.'],
 'precision_refinement':'60/80 decimal digits agree within1e-50K',
 'rows':rows80,'scientific_corpus_promotions':0,
 'current_cap_refusal_acceptance_changed':False}
if args.output: args.output.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
