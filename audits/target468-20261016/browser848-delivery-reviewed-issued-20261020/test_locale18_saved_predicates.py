"""Synthetic schema-shaped negative controls, never browser/chemical acceptance."""
import copy,unittest
from locale18_saved_predicates import validate_family_rows

def term(name,section):return {'term':{'en':name,'section':section}}
def number(text):return {'number':{'text':text}}
def nested(key,slots):return {'phrase':{'phrase':{'key':key,'en':'synthetic schema fixture','slots':slots}}}
def fixture():
 raw={}
 for locale in ['de','fr']:
  for name,t in [('cold15',288.15),('cold25',298.15),('warm65',338.15)]:
   if name.startswith('cold'):
    actual=f'{t:.1f}';minimum='333.1';why=nested('family-gate.temperature',[['actual',number(actual)],['window',nested('family-gate.window-min',[['min',number(minimum)]])]])
    text=('Temperatur mindestens 'if locale=='de'else'température au moins ')+actual.replace('.',',')+' K / '+minimum.replace('.',',')+' K';gate='temperature_k'
   else:
    wanted=nested('family-gate.alternative',[['first',term('H2SO4','family-species')],['rest',term('H+','family-species')]])
    why=nested('family-gate.catalyst-missing',[['wanted',wanted]]);text='Katalysator Schwefelsäure oder Wasserstoffionen'if locale=='de'else'catalyseur acide sulfurique ou ions hydrogène';gate='catalyst'
   reason={'key':'not-modeled.family-declined-at-gate','en':'synthetic fixture','slots':[['family',{'text':{'text':'fischer-esterification'}}],['version',{'text':{'text':'1'}}],['gate',term(gate,'family-gate-label')],['why',why]]}
   v={'id':0,'elapsed_seconds':1.,'temperature':t,'free_proton':0.,'contents':[{'species':'CH3COOH','phase':'solid'if name=='cold15'else'liquid','moles':.01},{'species':'ethanol','phase':'liquid','moles':.01}]}
   raw[locale+'-family-'+name]={'synthetic_fixture':True,'preparation':{'bench':{'vessels':[v]},'steps':[{'events':[{'event':'not_yet_modeled','vessel':0,'reason':reason}],'rendered':[text]}]}}
 return raw

def detail(row):return row['preparation']['steps'][0]['events'][0]['reason']['slots'][-1][1]['phrase']['phrase']
class SyntheticRawPredicates(unittest.TestCase):
 def test_schema_positive_is_not_a_browser_result(self):
  raw=fixture();self.assertTrue(all(x['synthetic_fixture']for x in raw.values()));self.assertEqual(len(validate_family_rows(raw)),6)
 def invalid(self,change):
  raw=fixture();change(raw)
  with self.assertRaises((AssertionError,ValueError,TypeError,KeyError)):validate_family_rows(raw)
 def test_english_temperature_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['steps'][0].update(rendered=['the vessel stands at288.1 K']))
 def test_machine_gate_fallback_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['steps'][0]['rendered'].append('temperature_k'))
 def test_wrong_actual_text_rejected(self):
  for value in ['287.0','288.2']:
   def change(raw):
    row=raw['fr-family-cold15'];old=detail(row)['slots'][0][1]['number']['text'];detail(row)['slots'][0][1]['number']['text']=value;row['preparation']['steps'][0]['rendered'][0]=row['preparation']['steps'][0]['rendered'][0].replace(old.replace('.',','),value.replace('.',','))
   with self.subTest(value=value):self.invalid(change)
 def test_missing_actual_number_render_rejected(self):self.invalid(lambda r:r['fr-family-cold15']['preparation']['steps'][0].update(rendered=['température au moins333,1 K']))
 def test_wrong_minimum_rejected(self):
  for value in ['300.0','333.2']:
   def change(raw):
    row=raw['de-family-cold15'];detail(row)['slots'][1][1]['phrase']['phrase']['slots'][0][1]['number']['text']=value;row['preparation']['steps'][0]['rendered'][0]=row['preparation']['steps'][0]['rendered'][0].replace('333,1',value.replace('.',','))
   with self.subTest(value=value):self.invalid(change)
 def test_missing_minimum_render_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['steps'][0].update(rendered=['Temperatur mindestens288,1 K']))
 def test_missing_nested_alternative_rejected(self):self.invalid(lambda r:detail(r['de-family-warm65'])['slots'][0].__setitem__(1,term('H2SO4','family-species')))
 def test_unknown_catalyst_alternative_rejected(self):self.invalid(lambda r:detail(r['fr-family-warm65'])['slots'][0][1]['phrase']['phrase']['slots'][1].__setitem__(1,term('HCl','family-species')))
 def test_missing_localized_alternative_rejected(self):self.invalid(lambda r:r['fr-family-warm65']['preparation']['steps'][0].update(rendered=['catalyseur acide sulfurique']))
 def test_english_catalyst_fallback_rejected(self):self.invalid(lambda r:r['de-family-warm65']['preparation']['steps'][0]['rendered'].append('the family needs its catalyst present'))
 def test_present_catalyst_rejected(self):self.invalid(lambda r:r['de-family-warm65']['preparation']['bench']['vessels'][0]['contents'].append({'species':'H2SO4','moles':.001,'phase':'liquid'}))
 def test_nonzero_free_proton_rejected(self):self.invalid(lambda r:r['fr-family-warm65']['preparation']['bench']['vessels'][0].update(free_proton=.001))
 def test_wrong_event_owner_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['steps'][0]['events'][0].update(vessel=1))
 def test_inventory_loss_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['bench']['vessels'][0]['contents'][0].update(moles=.009))
 def test_negative_phase_pool_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['bench']['vessels'][0]['contents'].append({'species':'water','moles':-.001,'phase':'liquid'}))
 def test_wrong_clock_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['bench']['vessels'][0].update(elapsed_seconds=0))
 def test_product_or_yield_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['bench']['vessels'][0]['contents'].append({'species':'ethyl_acetate','moles':.001,'phase':'liquid'}))
 def test_reacted_event_rejected(self):self.invalid(lambda r:r['de-family-cold15']['preparation']['steps'][0]['events'].append({'event':'org_reacted','name':'fischer-esterification'}))
if __name__=='__main__':unittest.main()
