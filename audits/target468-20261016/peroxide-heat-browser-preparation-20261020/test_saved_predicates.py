"""Explicitly SYNTHETIC schema-shaped parser controls, never chemical/browser results."""
import copy,unittest,json
from pathlib import Path
from heat_saved_predicates import validate_case

def fixture(locale='de'):
 def vessel(peroxide,water,oxygen,t):
  # Shape copied from an actual retained public Lab.snapshot vessel; quantities below are SYNTHETIC.
  v=json.loads(Path(__file__).with_name('actual5a-empty-water-vessel-schema.json').read_text(encoding='utf-8'))
  v.update(contents=[{'species':k,'moles':n,'phase':p}for k,n,p in [('water',water,'liquid'),('H2O2',peroxide,'liquid'),('KI',.001,'aqueous'),('O2',oxygen,'gas')]],elapsed_seconds=t,solution=None,resolved={'valid':False,'solution':None},lots=[])
  return v
 a,b=vessel(.1,10,0,0),vessel(.099,10.001,.0005,1)
 heat='v1: Peroxidzersetzung setzte 98,20 J frei'if locale=='de'else'v1 : Décomposition du peroxyde a libéré 98,20 J'
 events=[{'event':'reaction_heat_released','vessel':0,'reaction':'peroxide-decomposition','energy_j':98.2},{'event':'gas_produced','vessel':0,'reaction':'peroxide-decomposition','species':'O2','moles':.0005,'rate_moles_per_second':.0005},{'event':'reacted','vessel':0,'reaction':'peroxide-decomposition','moles':.0005,'seconds':1}]
 return {'synthetic_fixture':True,'meta':{'git_rev':'18f2bebd'},'before_calls':1,'after_calls':2,'before':{'vessels':[a],'spills':[],'broken_vessels':[]},'after':{'vessels':[b],'spills':[],'broken_vessels':[]},'preparation':{'bench':{'vessels':[copy.deepcopy(a)]}},'result':{'bench':{'vessels':[copy.deepcopy(b)]},'steps':[{'operator':{'op':'wait','seconds':1},'events':events,'rendered':[heat]}]}}
class Predicates(unittest.TestCase):
 def rejected(self,change):
  r=fixture();change(r)
  # Keep independently retained snapshots consistent to isolate the intended budget rejection.
  r['preparation']['bench']['vessels']=copy.deepcopy(r['before']['vessels']);r['result']['bench']['vessels']=copy.deepcopy(r['after']['vessels'])
  with self.assertRaises((AssertionError,KeyError,TypeError,ValueError)):validate_case(r,'de')
 def test_synthetic_de_fr_schema_positive(self):
  for locale in ['de','fr']:self.assertGreater(validate_case(fixture(locale),locale)['joules_per_consumed_mol'],97000)
 def test_half_heat_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'][0].update(energy_j=49.1))
 def test_numeric_string_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'][0].update(energy_j='98.2'))
 def test_nan_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'][0].update(energy_j=float('nan')))
 def test_wrong_heat_owner_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'][0].update(vessel=1))
 def test_wrong_reaction_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'][0].update(reaction='other'))
 def test_gas_stoich_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'][1].update(moles=.001))
 def test_missing_heat_rejected(self):self.rejected(lambda r:r['result']['steps'][0]['events'].pop(0))
 def test_bad_clock_rejected(self):self.rejected(lambda r:r['after']['vessels'][0].update(elapsed_seconds=0))
 def test_water_loss_rejected(self):self.rejected(lambda r:r['after']['vessels'][0]['contents'][0].update(moles=10))
 def test_catalyst_loss_rejected(self):self.rejected(lambda r:r['after']['vessels'][0]['contents'][2].update(moles=0))
 def test_unreported_export_rejected(self):self.rejected(lambda r:r['after']['vessels'][0]['contents'][3].update(moles=0))
 def test_english_leak_rejected(self):self.rejected(lambda r:r['result']['steps'][0].update(rendered=['peroxide-decomposition released 98.20 J']))
 def test_fake_rendered_number_rejected(self):self.rejected(lambda r:r['result']['steps'][0].update(rendered=['v1: Peroxidzersetzung setzte 49,10 J frei']))
 def test_spill_boundary_rejected(self):self.rejected(lambda r:r['after']['spills'].append({'invented':True}))
 def test_unknown_species_rejected(self):self.rejected(lambda r:r['after']['vessels'][0]['contents'].append({'species':'X','moles':1,'phase':'solid'}))
 def test_nonempty_nuclides_rejected(self):self.rejected(lambda r:r['after']['vessels'][0]['nuclides']['inventory'].update({'C14':1e-9}))
 def test_wrong_empty_nuclide_schema_rejected(self):self.rejected(lambda r:r['after']['vessels'][0].update(nuclides=[]))
 def test_owned_list_reservoirs_rejected(self):
  for field in ['surfaces','unresolved_materials','material_objects','surface_colours','charred_materials','electrodes','exchanges','adsorbed','solid_solutions']:
   with self.subTest(field=field):self.rejected(lambda r:r['after']['vessels'][0].update({field:[{'synthetic_unowned':True}]}))
 def test_owned_optional_reservoirs_rejected(self):
  for field in ['soap_scum','lemon_paper_mark','surface_particles','emulsion','step_start']:
   with self.subTest(field=field):self.rejected(lambda r:r['after']['vessels'][0].update({field:{'synthetic_unowned':True}}))
 def test_foam_reservoirs_rejected(self):
  for field in ['trapped_gas_liters','volume_liters','peak_volume_liters']:
   with self.subTest(field=field):self.rejected(lambda r:r['after']['vessels'][0]['foam'].update({field:.001}))
 def test_pending_co2_rejected(self):self.rejected(lambda r:r['after']['vessels'][0].update(pending_co2_transfer_mol=.001))
 def test_nonfinite_scalar_rejected(self):
  for field in ['pressure','solute_charge','pending_co2_transfer_mol','free_proton','free_hydroxide','excess_enthalpy_j','temperature']:
   with self.subTest(field=field):self.rejected(lambda r:r['after']['vessels'][0].update({field:float('nan')}))
 def test_nonfinite_cache_rejected(self):self.rejected(lambda r:r['after']['vessels'][0].update(solution={'ph':float('nan')}))
 def test_reported_oxygen_export_reconciles(self):
  r=fixture();r['after']['vessels'][0]['contents'][3]['moles']=0;r['result']['bench']['vessels']=copy.deepcopy(r['after']['vessels']);r['result']['steps'][0]['events'].append({'event':'gas_evolved','vessel':0,'species':'O2','moles':.0005});self.assertEqual(validate_case(r,'de')['exported_mol'],{'O2':.0005})
if __name__=='__main__':unittest.main()
