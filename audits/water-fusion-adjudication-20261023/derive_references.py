"""Independent Decimal/Simpson adjudication of old-law numerical expectations.
Reads captured scenes; never executes production Rust, a solver or a lesson.
"""
from pathlib import Path
from decimal import Decimal as D,getcontext
import argparse,hashlib,json
getcontext().prec=55
p=argparse.ArgumentParser();p.add_argument('--registry',type=Path,required=True);p.add_argument('--old-scene',type=Path,required=True);p.add_argument('--actual-scene',type=Path,required=True);args=p.parse_args()
R=D('8.31446261815324');TM=D('273.15');H=D(6010)
rows=json.loads(args.registry.read_text(),parse_float=D)['heat_capacity_polynomials']
c={x['phase']:x['intervals'][0] for x in rows if x['species_id']=='water' and x['phase'] in ['solid','liquid']}
def cp(row,t):
 t=max(D(row['t_min_k']),min(D(row['t_max_k']),t));a=list(map(D,row['coefficients']))
 return R*(a[0]/t**2+a[1]/t+a[2]+a[3]*t+a[4]*t**2+a[5]*t**3+a[6]*t**4)
def h_ice(t):
 a=list(map(D,c['solid']['coefficients']));return R*(-a[0]/t+a[1]*t.ln()+a[2]*t+a[3]*t**2/2+a[4]*t**3/3+a[5]*t**4/4+a[6]*t**5/5)
def s_ice(t):
 a=list(map(D,c['solid']['coefficients']));return R*(-a[0]/(2*t**2)-a[1]/t+a[2]*t.ln()+a[3]*t+a[4]*t**2/2+a[5]*t**3/3+a[6]*t**4/4)
CP=cp(c['liquid'],TM);HI=h_ice(TM);SI=s_ice(TM)
def h(t):return H+CP*(t-TM)-(h_ice(t)-HI)
def log_a(t):return (H*(1/TM-1/t)+CP*((t/TM).ln()-(t-TM)/t)-(s_ice(t)-SI)+(h_ice(t)-HI)/t)/R
def root(target):
 lo,hi=D(252),TM
 assert log_a(lo)<=target<=D(0)
 for _ in range(120):
  mid=(lo+hi)/2
  if log_a(mid)<target:lo=mid
  else:hi=mid
 return (lo+hi)/2

def quadrature(t,n):
 step=(t-TM)/n
 return step/3*sum((D(1) if i in [0,n] else D(4 if i%2 else 2))*h(TM+i*step)/(R*(TM+i*step)**2) for i in range(n+1))
old=json.loads(args.old_scene.read_text(),parse_float=D);actual=json.loads(args.actual_scene.read_text(),parse_float=D);changes=[]
def walk(a,b,path=''):
 assert type(a)==type(b),path
 if isinstance(a,dict):
  assert set(a)==set(b),path
  for k in sorted(a):walk(a[k],b[k],path+'/'+k)
 elif isinstance(a,list):
  assert len(a)==len(b),path
  for i,(u,v) in enumerate(zip(a,b)):walk(u,v,path+'/'+str(i))
 elif a!=b:
  assert re_allowed(path) and isinstance(a,D) and isinstance(b,D),(path,a,b)
  changes.append((path,a,b))
def re_allowed(path):
 import re
 return re.fullmatch(r'/slime/\d+/scene/vessels/\d+/melting_point_k',path) is not None
walk(old,actual);assert len(changes)==24
cache={};items=[]
for path,a,b in changes:
 if a not in cache:
  target=H/R*(1/TM-1/a);t=root(target);q0=quadrature(t,512);q1=quadrature(t,1024)
  assert abs(q0-q1)<D('1e-13') and abs(q1-target)<D('1e-13')
  lo=root(H/R*(1/TM-1/(a-D('0.0000005'))));hi=root(H/R*(1/TM-1/(a+D('0.0000005'))))
  cache[a]={'old_rounded_k':str(a),'inferred_old_log_activity':str(target),'independent_new_central_k':str(t),'new_interval_from_old_rounding_k':[str(lo),str(hi)],'independent_quadrature_refinement_error':str(abs(q0-q1))}
 row=cache[a];lo,hi=map(D,row['new_interval_from_old_rounding_k']);assert b+D('0.0000005')>=lo and b-D('0.0000005')<=hi
 items.append({'json_pointer':path,'old':str(a),'actual_successor':str(b),'independent_prediction_agrees_with_both_six_decimal_rounding_intervals':True})
old_pin=TM-D('0.3468');predicted=root(H/R*(1/TM-1/old_pin));pin=TM-predicted
assert abs(pin-D('0.3472'))<D('0.00005')
print(json.dumps({'source_before_fixture_correction':'51bc28ddc576d587b86fab8da2ee6ea120822bc4','registry_sha256':hashlib.sha256(args.registry.read_bytes()).hexdigest(),'old_scene_sha256':hashlib.sha256(args.old_scene.read_bytes()).hexdigest(),'captured_actual_scene_sha256':hashlib.sha256(args.actual_scene.read_bytes()).hexdigest(),'method':'55-digit independent registry antiderivatives and512/1024-step Simpson integral; old/current six-decimal rounding intervals must overlap under the new law. All non-liquidus scene fields must remain identical.','scope':'Model-dependent expectation adjudication, not empirical calibration or production execution.','changed_field_count':24,'unique_temperature_references':list(cache.values()),'scene_changes':items,'tenth_molal_model_pin':{'old_depression_k':'0.3468','independently_converted_depression_k':str(pin),'successor_rounded_depression_k':'0.3472','original_pin_tolerance_k':'0.0004','independent_physical_band_unchanged':True},'saturated_brine_contract':'Keep the valid boil and scene absence of melting point. Require checked inverse None, NaN legacy sentinel and input activity strictly below the252K activity; do not demand a finite out-of-domain root.','budget_or_physical_tolerance_widened':False},indent=2))
