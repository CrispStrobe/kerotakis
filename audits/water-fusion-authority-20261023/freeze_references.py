"""Independent Decimal/Simpson references, frozen before production edits.
No production evaluator is imported. These are model checks, not calibration.
"""
from decimal import Decimal as D, getcontext
from pathlib import Path
import hashlib, json, subprocess
getcontext().prec = 55
ROOT = Path(__file__).resolve().parents[2]
source = ROOT / 'data/registry/registry-source-v1.json'
rows = json.loads(source.read_text(), parse_float=D)['heat_capacity_polynomials']
curves = {p['phase']:p['intervals'][0] for p in rows if p['species_id']=='water' and p['phase'] in ['solid','liquid']}
R=D('8.31446261815324'); TM=D('273.15'); H=D('6010'); MW=D('0.018015')
def cp(phase,t):
 p=curves[phase]; t=max(D(p['t_min_k']),min(D(p['t_max_k']),t)); a=list(map(D,p['coefficients']))
 return R*(a[0]/t**2+a[1]/t+a[2]+a[3]*t+a[4]*t**2+a[5]*t**3+a[6]*t**4)
def primitive(a,t):
 a=list(map(D,a));return R*(-a[0]/t+a[1]*t.ln()+a[2]*t+a[3]*t**2/2+a[4]*t**3/3+a[5]*t**4/4+a[6]*t**5/5)
def enthalpy(t):
 return H+cp('liquid',TM)*(t-TM)-(primitive(curves['solid']['coefficients'],t)-primitive(curves['solid']['coefficients'],TM))
def ln_activity(t,n):
 step=(t-TM)/n
 return step/3*sum((D(1) if j in [0,n] else D(4 if j%2 else 2))*enthalpy(TM+j*step)/(R*(TM+j*step)**2) for j in range(n+1))
controls=[]
for t in map(D,['273.15','273.149999','270','260','252']):
 a=ln_activity(t,512); b=ln_activity(t,1024)
 assert abs(a-b)<D('1e-14')
 controls.append(dict(temperature_k=str(t),fusion_enthalpy_j_mol=str(enthalpy(t)),ln_activity=str(b),activity=str(b.exp()),quadrature_refinement_error=str(abs(a-b)),dtemperature_dlnactivity=str(R*t*t/enthalpy(t))))
cap=D(controls[-1]['activity']); result=dict(schema=1,scope='Shipped Cp with liquid endpoint holding, no empirical calibration or below-252K admission',base=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),registry_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),controls=controls,ideal_cap_particle_molality=str((1/cap-1)/MW),osmotic_phi_one_cap_particle_molality=str(-cap.ln()/MW),reference_k=str(TM),fusion_reference_j_mol=str(H))
print(json.dumps(result,indent=2))
