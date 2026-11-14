"""Saved stock versus balance check; independent of solver success/qualification."""
from pathlib import Path
from decimal import Decimal as D
import sys,json,hashlib
out=Path(sys.argv[1]);repo=Path(sys.argv[2]);registry=repo/'crates/kerotakis-core/tests/golden/registry.json'
masses={r['key']:D(str(r['molar_mass'])) for r in json.loads(registry.read_text())}
masses['base_equivalents']=masses['OH-']
rows=[json.loads(s) for s in (out/'batch3-diverse-fe-003.stdout').read_text().splitlines()]
readings=[]
for r in rows:
 for e in r['events']:
  if e['event']!='measured' or e['instrument']!='balance':continue
  v=next(v for v in r['bench']['vessels'] if v['id']==e['vessel'])
  known=sum((D(str(p['moles']))*masses[p['species']] for p in v['contents']),D(0))
  assert all(p['basis']=='mass_fraction' for p in v['unresolved_materials'])
  opaque=sum((D(str(p['amount'])) for p in v['unresolved_materials']),D(0))
  observed=D(str(e['value']));assert e['unit']=='g'
  assert abs(observed-known-opaque)<=D('1e-10')
  readings.append({'vessel':v['id'],'known_stock_mass_g':str(known),'unresolved_yeast_mass_g':str(opaque),'balance_g':str(observed),'residual_g':str(observed-known-opaque)})
assert len(readings)==3
print(json.dumps({'registry_sha256':hashlib.sha256(registry.read_bytes()).hexdigest(),'balance_readings':readings,'molar_mass_rounding_note':'Registry molar masses give a 0.0005 g/mol sucrose-reaction mass rounding residual. Element closure is assessed separately, without changing its 1e-10 mol/eq bound. This check compares each displayed balance with the actual retained stock; it does not turn a failed solver into a qualification.'},indent=2))
