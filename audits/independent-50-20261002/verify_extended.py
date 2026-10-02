"""Assess the extended replay against the same frozen physical expectations.
The two formula-only sugar cases still use explicit sucrose followups. Olive
cases now use their ORIGINAL inputs and the reviewed 0.9161 g/mL density;
the previous vegetable-oil surrogate arithmetic is no longer applicable.
Historical predictions and assessment scripts remain unchanged.
"""
import json, math, statistics, os
from collections import Counter
from pathlib import Path
root=Path(__file__).resolve().parent
cases=json.loads((root/'predictions.json').read_text())
results=Path(os.environ.get('KERO_RESULTS_DIR', str(root/'extended'))).resolve()
def steps(i):
    ident=f'{i:02d}'
    corrected=results/f'{ident}.corrected.json.stdout'
    path=corrected if corrected.exists() else results/f'{ident}.json.stdout'
    return [json.loads(l) for l in path.read_text().splitlines()]
def final(i): return steps(i)[-1]['bench']['vessels']
def amount(v,species,phase=None):
    return sum(x['moles'] for x in v['contents'] if x['species']==species and (phase is None or x['phase']==phase))
def measures(i,instrument):
    return [e['value'] for s in steps(i) for e in s['events'] if e['event']=='measured' and e['instrument']==instrument]
def events(i,kind): return [e for s in steps(i) for e in s['events'] if e['event']==kind]
def close(a,b,tol): return abs(a-b)<=tol
checks=[]
def check(i,name,passed,expected=True):
    checks.append(dict(id=f'{i:02d}',check=name,passed=bool(passed),expected_to_pass=expected))
check(1,'10 kJ raises 100 g water about 24 K',close(final(1)[0]['temperature']-298.15,24,1))
check(2,'Equal heat and cool restore initial temperature',close(final(2)[0]['temperature'],298.15,0.001))
check(3,'Weighted hot/cold mixing gives 35 C',close(final(3)[0]['temperature'],308.15,0.05))
a,b=final(4);check(4,'Temperature rises scale inversely with water mass',close((a['temperature']-298.15)/(b['temperature']-298.15),2,0.01))
v=final(5)[0];check(5,'Cooling makes mixed ice and liquid near zero',amount(v,'water','solid')>0 and amount(v,'water','liquid')>0 and close(v['temperature'],273.15,0.01))
v=final(6)[0];check(6,'Boiling plateau and approximately 17 g water loss',close(v['temperature'],373.15,0.05) and 82<amount(v,'water')*18.01528<84)
a,b=final(7);check(7,'Salt freezes lower than equal molar sugar',a['temperature']<b['temperature']<273.15 and amount(a,'water','solid')>0 and amount(b,'water','solid')>0)
check(8,'Equal 60 C portions stay at 60 C',close(final(8)[0]['temperature'],333.15,0.01))
a,b=measures(9,'ph_meter');check(9,'Tenfold dilution raises pH about one unit',close(b-a,1,0.1))
a,b=measures(10,'ph_meter');check(10,'Neutralisation followed by alkaline overshoot',6.8<a<7.2 and b>11)
a,b=measures(11,'ph_meter');check(11,'Acetic acid pH is more than one unit above HCl',b-a>1)
check(12,'Half neutralised acetate pH near pKa',4.5<measures(12,'ph_meter')[0]<5)
a,b,c,d=measures(13,'ph_meter');check(13,'Buffer pH change is less than 1 percent of water change',abs(c-a)<0.01*abs(d-b))
a,b=measures(14,'conductivity_meter');check(14,'NaCl greatly increases conductivity',b>1000*a and 6.8<measures(14,'ph_meter')[0]<7.2)
a,b=measures(15,'conductivity_meter');check(15,'Sugar conducts much less than NaCl',b>1000*a)
check(16,'AgCl yield close to 1 mmol',close(amount(final(16)[0],'AgCl','solid'),0.001,0.00001))
a,b=final(17);check(17,'Reversed reagent order gives equal solid and pH',close(amount(a,'AgCl'),amount(b,'AgCl'),1e-9) and close(a['solution']['ph'],b['solution']['ph'],1e-5))
a,b=final(18);check(18,'Filter retains AgCl and transfers water and sodium',amount(a,'AgCl','solid')>0.00099 and amount(a,'water')==0 and amount(b,'water')>5 and close(amount(b,'Na+'),0.002,1e-8))
check(19,'BaSO4 yield near 1 mmol',amount(final(19)[0],'BaSO4','solid')>0.00098)
v=final(20)[0];check(20,'Acid dissolves chalk with accounted CO2',amount(v,'CaCO3')==0 and close(amount(v,'Ca+2'),0.001,1e-8) and final(20)[0]['solution']['ph']<3 and sum(e['moles'] for e in events(20,'gas_evolved') if e['species']=='CO2')>0.0009)
check(21,'Excess salt remains undissolved',amount(final(21)[0],'NaCl','solid')>1)
check(22,'Dilution dissolves residual NaCl',amount(final(22)[0],'NaCl','solid')==0)
v=final(23)[0];check(23,'Evaporation retains salt with 10 g solvent',close(amount(v,'water')*18.015,10,0.01) and close(amount(v,'Na+'),0.001,1e-10))
a,b=final(24);check(24,'Quarter decant partitions water and salt',close(amount(b,'water')/sum(amount(v,'water') for v in (a,b)),0.25,1e-9) and close(amount(b,'Na+'),0.0025,1e-10))
a=next(s['bench']['vessels'][0] for s in steps(25) if s['operator']['op']=='inspect');b=final(25)[0];check(25,'Split and reunite restores inventory and temperature',a['contents']==b['contents'] and close(a['temperature'],b['temperature'],1e-6))
a,b=final(26);check(26,'CaCl2 warms and KNO3 cools',a['temperature']>298.15 and b['temperature']<298.14)
v=final(27)[0];sol=v['solution'];inv=sum(amount(v,k) for k in ('CO2(aq)','HCO3-','CO3-2'));snap=sum(x['molality'] for x in sol['species'] if x['name'] in ('CO2','HCO3-','CO3-2'))*sol['solvent_kg'];check(27,'Inventory carbon agrees with speciation snapshot',close(inv,snap,1e-7))
check(28,'Fizz raises sealed pressure',final(28)[0]['pressure']>101325)
a,b=final(29);check(29,'Smaller headspace gives larger pressure',a['pressure']>b['pressure']>101325)
v=final(30)[0];check(30,'Opening restores ambient pressure and retains salt',close(v['pressure'],101325,0.01) and close(amount(v,'Na+'),0.001,1e-9) and close(amount(v,'Cl-'),0.002,1e-9))
v=final(31)[0];check(31,'Pressure regulator expands headspace at fixed pressure',close(v['pressure'],100000,0.01) and v['headspace']['volume']>0.01)
a,b=measures(32,'ph_meter');check(32,'Sweep removes carbon and raises pH',b-a>2 and sum(amount(final(32)[0],k) for k in ('CO2(aq)','HCO3-','CO3-2'))<1e-8)
v=final(33)[0];check(33,'Zinc plates copper with conserved metal totals',amount(v,'Cu','solid')>0.0009 and close(amount(v,'Zn')+amount(v,'Zn+2'),0.002,1e-8) and close(amount(v,'Cu')+amount(v,'Cu+2'),0.001,1e-8))
v=final(34)[0];check(34,'Copper does not plate zinc',close(amount(v,'Cu'),0.002,1e-9) and amount(v,'Zn','solid')==0)
a,b=final(35);check(35,'Zinc dissolves in acid while copper remains',amount(a,'Zn','solid')<0.001 and close(amount(b,'Cu','solid'),0.001,1e-9),False)
a,b=events(36,'cell_voltage');check(36,'Cell reports about 1.1 V with stable electrode identities',close(a['volts'],1.1,0.05) and a['anode']==b['anode'] and a['cathode']==b['cathode'])
check(37,'Equal cells have no potential then dilution produces tens of mV',bool(events(37,'no_cell')) and 0.01<events(37,'cell_voltage')[0]['volts']<0.04)
e=events(38,'electrolysed')[0];check(38,'Faraday product amounts and 2 to 1 gas stoichiometry',close(e['cathode_moles'],5.182e-5,1e-8) and close(e['cathode_moles']/e['anode_moles'],2,1e-9))
check(39,'Missing complete Cu spectrum is explicitly bounded',len(events(39,'not_yet_modeled'))>=2)
a,b=measures(40,'spectrophotometer');check(40,'Tenfold dilution reduces absorbance tenfold',close(a/b,10,1e-6))
check(41,'Flame test retains NaCl',close(amount(final(41)[0],'NaCl')*58.443,1,1e-6))
check(42,'Burned magnesium gains expected oxygen mass',close(amount(final(42)[0],'MgO'),0.001,1e-9) and close(measures(42,'balance')[0],0.040304,0.00001))
check(43,'Ethanol produces combustion heat or an explicit model boundary',bool(events(43,'not_yet_modeled')) or any((e.get('reaction_energy_j') or 0)>0 for e in events(43,'thermal_equilibrium')))
a,b=final(44);check(44,'Distillation enriches ethanol and conserves both components',amount(b,'ethanol')/(amount(b,'ethanol')+amount(b,'water'))>0.434131/(0.434131+4.440744) and close(amount(a,'water')+amount(b,'water'),80/18.015,1e-7) and close(amount(a,'ethanol')+amount(b,'ethanol'),20/46.069,1e-7))
# `look` lowers to the eyes instrument in the public JSON operator schema.
# At dosing, 50 mL water uses 0.997 g/mL; olive oil uses its independently
# reviewed 0.9161 g/mL scalar. Both masses must survive the layer rendering.
s=next(s for s in steps(45) if s['operator'].get('instrument')=='eyes');v=s['scene']['vessels'][0];check(45,'Oil and water are drawn as separate conserved layers',len(v['layers'])==2 and close(v['mass_g'],50*0.997+50*0.9161,0.01))
a,b=final(46);check(46,'Drain transfers lower water layer',amount(a,'water')==0 and amount(b,'water')>2)
v=final(47)[0];check(47,'Salt remains aqueous alongside unresolved oil',close(amount(v,'Na+','aqueous'),0.001,1e-9) and len(v['unresolved_materials'])==1)
v=final(48)[0];check(48,'Milk acidification conserves unresolved material and produces curd event',close(v['unresolved_materials'][0]['amount'],12.20149,1e-6) and any('curd' in e['event'] for s in steps(48) for e in s['events']))
v=final(49)[0];check(49,'Peroxide oxygen stoichiometry and unchanged catalyst',close(0.01-amount(v,'H2O2'),2*amount(v,'O2'),1e-9) and close(amount(v,'MnO2'),0.001,1e-9))
a=next(s['bench']['vessels'][0] for s in steps(50) if s['operator']['op']=='inspect');b=final(50)[0];check(50,'One hour preserves dilute salt and water inventory',a['contents']==b['contents'])
(results/'checks.json').write_text(json.dumps(checks,indent=2)+'\n')
for c in checks:
 print(c['id'],'PASS' if c['passed'] else 'FAIL',c['check'])
unexpected=[c for c in checks if c['passed']!=c['expected_to_pass']]
print('Original physical expectations; zinc remains an explicit rate boundary:')
print('Checks:',len(checks),'Passed:',sum(c['passed'] for c in checks),'Known mismatches:',sum(not c['passed'] for c in checks),'Unexpected:',len(unexpected))
if unexpected: raise SystemExit(1)

# These checks deliberately supplement rather than replace the original zinc
# expectation. Missing kinetics is an honest boundary, not a dissolution pass.
assert any(e.get('reason',{}).get('key')=='not-modeled.hydrogen-overpotential-rate'
    for e in events(35,'not_yet_modeled'))
assert not any(e.get('computed') and e.get('species')=='Zn' for e in events(35,'inert'))
assert 'hydrogen is not co-evolved' not in (results/'38.text.stdout').read_text()
v=final(43)[0]
assert amount(v,'ethanol')<1e-8, 'production stack should route supported ethanol to combustion'
# Numeric composition retains every positive Newton trace. Distinguish an
# unacceptable pre-ignition phase-route loss from a bounded residual in the
# post-combustion exhaust; do not force the numeric ledger to drop that trace.
feed_step = next(s for s in steps(43) if s['operator']['op'] == 'add')
ethanol_feed = amount(feed_step['bench']['vessels'][0], 'ethanol')
assert ethanol_feed > 0
ignited = False
ethanol_exhaust = 0.0
for step in steps(43):
    for e in step['events']:
        if e['event'] == 'ignited':
            ignited = True
        if e['event'] == 'gas_evolved' and e.get('species') == 'ethanol':
            assert ignited, 'phase routes must not vent unburned fuel before combustion'
            ethanol_exhaust += e['moles']
assert ignited
assert 0 <= ethanol_exhaust <= 1e-8 * ethanol_feed, 'combustion must consume the represented fuel'
assert any((e.get('energy_j') or 0)>1000 for e in events(43,'ignited')), 'supported ethanol burns with reported chemical energy'
assert any(e.get('note_reason',{}).get('key','').startswith('measurement.aqueous-layer')
    for e in events(47,'measured'))
print('Additional rate, gas explanation, supported combustion, and layer-scope checks: PASS')

for i in range(1, 51):
    assert not events(i, 'solver_failed'), f'case {i}: solver failure hidden behind later stages'
print('All 50 assessed physical cases contain no SolverFailed events: PASS')
