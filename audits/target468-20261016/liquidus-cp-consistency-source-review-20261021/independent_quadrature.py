"""Independent numerical-quadrature source audit; no product/engine/calibration."""
from pathlib import Path
import hashlib,json,math,subprocess,argparse
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo',default='.')
parser.add_argument('--reference',type=Path,required=True)
parser.add_argument('--output',type=Path)
args=parser.parse_args()
SOURCE='28277d415b68fedda2f2efccf702811fe160fee9'
REPO=Path(args.repo)
R=8.31446261815324;REFERENCE=273.15;FUSION=6010.0
raw=subprocess.check_output(['git','show',SOURCE+':crates/kerotakis-core/tests/golden/registry.json'],cwd=REPO)
water=next(s for s in json.loads(raw)if s['key']=='water')
curves={p['phase']:p for p in water['heat_capacity_polys']if p['phase']in['liquid','solid']}
assert curves['liquid']['intervals'][0]['t_min']==REFERENCE
assert curves['solid']['intervals'][0]['t_min']==200 and curves['solid']['intervals'][0]['t_max']==REFERENCE

def cp(phase,temperature):
 intervals=curves[phase]['intervals'];t=max(intervals[0]['t_min'],min(intervals[-1]['t_max'],temperature))
 row=next(i for i in intervals if i['t_min']<=t<=i['t_max']);assert row['form']=='nasa9'
 return R*math.fsum(c*t**power for power,c in zip(range(-2,5),row['coefficients']))

def simpson(f,a,b,n):
 assert n%2==0
 d=(b-a)/n
 return d/3*math.fsum((1 if i in[0,n]else 4 if i%2 else 2)*f(a+i*d)for i in range(n+1))

def delta_cp(t):return cp('liquid',t)-cp('solid',t)

def fusion(t,n):return FUSION+simpson(delta_cp,REFERENCE,t,n)

def log_activity(t,n):
 # Swap the order of the two integrals in integral Hfus(u)/(R*u²)du.
 # No NASA9 primitive, H primitive or production integration/root is used.
 # This is exactly F/R*(1/Tref-1/T)+1/R*integral deltaCp(u)*(1/u-1/T)du.
 return FUSION/R*(1/REFERENCE-1/t)+simpson(lambda u:delta_cp(u)*(1/u-1/t),REFERENCE,t,n)/R

def row(old,n):
 log_aw=FUSION/R*(1/REFERENCE-1/old)
 lo,hi=249.,REFERENCE
 for _ in range(64):
  mid=(lo+hi)/2
  if log_activity(mid,n)<log_aw:lo=mid
  else:hi=mid
 root=(lo+hi)/2
 return {'constant_fusion_temperature_K':old,'source_activity':math.exp(log_aw),'source_fusion_J_per_mol':fusion(old,n),'fusion_fraction_change':fusion(old,n)/FUSION-1,'variable_fusion_hypothetical_root_K':root,'root_shift_K':root-old,'quadrature_logaw_residual':log_activity(root,n)-log_aw,'root_below_current252K_cap':root<252}

a=[row(t,512)for t in[273.15,270.,260.,252.]];b=[row(t,1024)for t in[273.15,270.,260.,252.]]
parent=json.loads(args.reference.read_text())
comparison=[]
for first,second,other in zip(a,b,parent['rows']):
 comparison.append({'old_temperature_K':second['constant_fusion_temperature_K'],'mesh_refinement_root_difference_K':second['variable_fusion_hypothetical_root_K']-first['variable_fusion_hypothetical_root_K'],'mesh_refinement_fusion_difference_J_per_mol':second['source_fusion_J_per_mol']-first['source_fusion_J_per_mol'],'independent_vs_parent_analytic_root_K':second['variable_fusion_hypothetical_root_K']-float(other['variable_fusion_temperature_k']),'independent_vs_parent_analytic_fusion_J_per_mol':second['source_fusion_J_per_mol']-float(other['source_fusion_j_per_mol'])})
assert max(abs(r['mesh_refinement_root_difference_K'])for r in comparison)<1e-8
assert max(abs(r['independent_vs_parent_analytic_root_K'])for r in comparison)<1e-7
assert max(abs(r['independent_vs_parent_analytic_fusion_J_per_mol'])for r in comparison)<1e-5
result={'scope':'SOURCE_MODEL_CONSISTENCY_ONLY_NO_ENGINE_NO_EMPIRICAL_CALIBRATION','source':SOURCE,'registry_sha256':hashlib.sha256(raw).hexdigest(),'method':'Independent composite Simpson Cp quadrature; exchanged-integral kernel for Gibbs/chemical-potential relation, distinct from parent Decimal analytic primitives','constants':{'R_J_mol_K':R,'reference_K':REFERENCE,'fusion_reference_J_mol':FUSION},'phase_ranges':{k:[(i['t_min'],i['t_max'])for i in v['intervals']]for k,v in curves.items()},'liquid_below_table':'Endpoint-held Cp at273.15K, source algorithmic extension; not measured metastable liquid data','rows':b,'independent_comparisons':comparison,'inference':'Current source constant6010 liquidus and temperature-adjusted phase-transfer enthalpy are thermodynamically inconsistent for the imposed-aw comparison. The4percent/under0.1K comment is contradicted by current registry model, not established physical data.','bounds':['No temperature-dependent real-solution aw model was supplied; activity is held fixed for each comparison.','250.3997K is below current252K model cap and is only a hypothetical mathematical root.','Current frozen cap controls intentionally validate existing constant-fusion relation and conservative energy/refusal; they are not silently changed.'],'current_PR_or_frozen_controls_changed':False,'scientific_selections':0}
if args.output: args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,indent=2))
