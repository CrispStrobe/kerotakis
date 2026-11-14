"""Original pigment-versus-blank review; no acid/base spectral calibration claim."""
from pathlib import Path
from decimal import Decimal as D
import json,sys,hashlib
root=Path(sys.argv[1]);rows=[json.loads(l) for l in (root/'batch3-diverse-li-003.stdout').read_text().splitlines()];data=json.loads((root/'source-properties.json').read_text());forms={r['species_id']:{**{e['element']:D(str(e['count']['value'])) for e in r['elements']},'charge':D(str(r['net_charge']['value']))} for r in data['compositions']};forms['base_equivalents']=forms['OH-'];keys=['H','O','C','Na','charge'];bound=D('1e-10');expected={};checks=[];spectra=[];ph={}
def amount(v,s):return sum((D(str(p['moles'])) for p in v['contents'] if p['species']==s),D(0))
opt=data['curcumin_optical'][0];assert opt['species_id']=='curcumin';points=opt['spectrum'];peak=max(points,key=lambda p:p['molar_absorptivity']['value']);assert peak['wavelength']['value']==425;epsilon=D(str(peak['molar_absorptivity']['value']))
# Retain the original declaration's unestablished precision and absent pH/medium parameters.
assert all(p['molar_absorptivity']['uncertainty']['kind']=='unestablished' and p['molar_absorptivity']['conditions']['ph'] is None and p['molar_absorptivity']['conditions']['medium'] is None for p in points)
code=(root/'source-instrument.rs').read_text();binding=json.loads((root/'binding.json').read_text());assert hashlib.sha256(code.encode()).hexdigest()==binding['source_instrument_sha256'];assert 'spectrum.iter().copied().fold(0.0f64, f64::max)' in code
all_events=[e for r in rows for e in r.get('events',[])];assert not any(e['event'] in ['solver_failed','not_yet_modeled','gas_evolved','gas_absorbed','gas_vented'] for e in all_events)
for i,r in enumerate(rows):
 op=r['operator']
 if op['op']=='add':
  want=expected.setdefault(op['vessel'],{k:D(0) for k in keys})
  for k,c in forms[op['species']].items():want[k]+=D(str(op['moles']))*c
 if 'bench' not in r:continue
 for v in r['bench']['vessels']:
  actual={k:D(0) for k in keys};assert not v.get('unresolved_materials')
  for p in v['contents']:
   n=D(str(p['moles']));assert n.is_finite() and n>=0
   for k,c in forms[p['species']].items():actual[k]+=n*c
  want=expected.get(v['id'],{k:D(0) for k in keys});res={k:actual[k]-want[k] for k in keys};assert all(abs(n)<bound for n in res.values()),(i,v['id'],res)
  checks.append({'step':i,'vessel':v['id'],'residual_mol_eq':{k:str(n) for k,n in res.items()}})
 for e in r.get('events',[]):
  if e['event']!='measured':continue
  id=e['vessel'];v=next(v for v in r['bench']['vessels'] if v['id']==id)
  if e['instrument']=='ph_meter':ph[id]=D(str(e['value']));assert abs(ph[id]-D(str(v['solution']['ph'])))<bound
  if e['instrument']=='spectrophotometer':
   if id==0:
    assert e['unit']=='absorbance at 425 nm' and amount(v,'curcumin')==D('.000001')
    scene=next(v for v in r['scene']['vessels'] if v['id']==id);volume=D(str(scene['liquid']['volume_l']));pred=epsilon*amount(v,'curcumin')/volume
    assert abs(D(str(e['value']))-pred)<bound
   else:
    assert amount(v,'curcumin')==0 and e['value']==0 and e['unit']=='absorbance at 405 nm';pred=D(0)
   spectra.append({'step':i,'vessel':id,'reported_peak':e['unit'],'absorbance':str(e['value']),'reconstructed_neutral_pigment_absorbance':str(pred)})
assert len(spectra)==4 and [e['vessel'] for e in spectra]==[0,1,0,1]
assert set(ph)=={0,1} and ph[0]==ph[1] and ph[0]>10
final=next(r['bench']['vessels'] for r in reversed(rows) if 'bench' in r);assert len(final)==2
for v in final:
 assert amount(v,'curcumin')==(D('.000001') if v['id']==0 else 0)
 assert amount(v,'ethanol')==D('0.08563242093381666') and amount(v,'Na+')==D('.00002')
 assert v['solution'].get('scope','complete')=='complete'
# Equal nonpigment state supplies the solvent/base control without equating wavelengths.
a={p['species']:D(str(p['moles'])) for p in final[0]['contents'] if p['species']!='curcumin'};b={p['species']:D(str(p['moles'])) for p in final[1]['contents']};assert a==b
print(json.dumps({'met':4,'total':4,'criteria':{k:True for k in ['question','inventory','coverage','controls']},'solver_failures':0,'known_element_charge_checks':checks,'paired_postcarbonate_ph':{str(k):str(v) for k,v in ph.items()},'saved_spectral_peaks':spectra,'blank_interpretation':'The pinned instrument reports the maximum of the visible spectrum. A zero maximum at its default405nm is not a direct425nm measurement, but establishes zero over its sampled bands, including425nm.','optical_convention':'Unchanged neutral-curcumin declared spectrum,1cm path. Unestablished legacy absorptivity with no pH or mixed-solvent parameterisation. Tiny A change reconstructs from the reported liquid-volume change.','scope':'Original pigment-versus-solvent/base blank and explicit coverage review only; no supported acid/base spectral shift, mixed-solvent solubility validation, pKa, long-term alkaline stability or optical calibration is qualified.'},indent=2))
