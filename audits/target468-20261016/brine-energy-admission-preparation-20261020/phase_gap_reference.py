#!/usr/bin/env python3
"""Independent declared-model enthalpy references, NOT native solver outcomes."""
import hashlib,json,math,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent
D=json.loads((P/'design.json').read_text())
raw=subprocess.check_output(['git','show',D['source_reviewed']+':crates/kerotakis-core/tests/golden/registry.json'])
assert hashlib.sha256(raw).hexdigest()==D['source_blobs_sha256']['crates/kerotakis-core/tests/golden/registry.json']
water=next(r for r in json.loads(raw) if r['key']=='water')
def cp(phase,t):
 rows=next(r['intervals'] for r in water['heat_capacity_polys'] if r['phase']==phase)
 t=max(rows[0]['t_min'],min(t,rows[-1]['t_max']))
 row=next(r for r in rows if r['t_min']<=t<=r['t_max']);assert row['form']=='nasa9'
 a=row['coefficients']
 return 8.31446261815324*(a[0]/t**2+a[1]/t+a[2]+a[3]*t+a[4]*t*t+a[5]*t**3+a[6]*t**4)
def sensible(phase,t):
 n=4096;step=(t-273.15)/n
 return step/3*sum((1 if i in (0,n) else 4 if i%2 else 2)*cp(phase,273.15+i*step) for i in range(n+1))
rows=[]
for dose in (0,.001,.01,.03):
 activity=1/(1+2*dose)
 t=1/(1/273.15-(8.31446261815324/6010)*math.log(activity))
 # Enthalpy of liquid relative to ice under the phase-specific Cp oracle.
 gap=6010+sensible('liquid',t)-sensible('solid',t)
 rows.append({'NaCl_pair_mol_per_one_mol_liquid_water':dose,'independent_ideal_liquidus_K':t,'declared_phase_enthalpy_gap_J_per_mol':gap,'source_constant_fusion_J_per_mol':6010,'difference_from_constant_J_per_mol':gap-6010})
print(json.dumps({'kind':'independent_analytical_model_reference_only','source':D['source_reviewed'],'rows':rows,'native_engine_executions':0,'native_controls_executed':False,'actual_TC008_cause_or_residual_assigned':False,'reference':'H_liquid(T)-H_ice(T) = 6010 + integral_273.15^T(Cp_liquid-Cp_ice)dT, using the frozen registry curves and endpoint clamps.'},indent=2))
