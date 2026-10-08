#!/usr/bin/env python3
"""Construct original forecasts; never reads engine output or earlier forecasts."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
cases=[]
def m(kind,variant='a',vessel=1,**kw): return dict(kind=kind,variant=variant,vessel=vessel,**kw)
def q(key,variant='a',vessel=1): return m('species',variant,vessel,species=key)
def p(v='a'): return m('ph',v)
def t(v='a'): return m('temperature',v)
def sub(a,b): return dict(kind='difference',left=a,right=b)
def plus(*a): return dict(kind='sum',terms=list(a))
def ck(a,op,b,**kw): return dict(lhs=a,op=op,rhs=b,**kw)
def near(a,b,atol=1e-10,rtol=1e-7): return ck(a,'near',b,atol=atol,rtol=rtol)
def case(title,why,variants,checks):
 cases.append(dict(id=f'S{len(cases)+1:02}',title=title,expectation=why,variants={k:'new\nnew\nnew\n'+v.strip()+'\ninspect\n' for k,v in variants.items()},checks=checks))
def water(extra='',amount='100mL'): return f'add v1 water {amount}\n'+extra
# Mechanical ownership and equivalent operation sequences.
case('Two serial half pours leave one quarter','Successive fractions act on the remaining donor, preserving total water.',{'a':water('decant v1 v2 0.5\ndecant v1 v3 0.5', '1mol')},[near(q('water'),.25),near(q('water',vessel=2),.5),near(q('water',vessel=3),.25),near(q('water',vessel=0),1)])
case('Return a poured aliquot','Pouring the complete aliquot back restores water ownership.',{'a':water('decant v1 v2 0.3\ndecant v2 v1 1','2mol')},[near(q('water'),2),near(q('water',vessel=2),0)])
case('Empty donor does not invent liquid','A subsequent pour from an exhausted donor has no material to transfer.',{'a':water('decant v1 v2 1\ndecant v1 v3 0.5','1mol')},[near(q('water',vessel=0),1),near(q('water',vessel=3),0)])
case('Partition and recombine brine','Recombining both halves restores salt and solvent.',{'a':water('add v1 NaCl 0.01mol\ndecant v1 v2 0.5\ndecant v2 v1 1','2mol')},[near(q('water'),2),near(q('NaCl'),.01)])
case('Fine dilute tracer survives pour','A soluble micromole-scale addition must be conserved under transfer.',{'a':water('add v1 KCl 1e-7mol\ndecant v1 v2 0.4')},[near(q('KCl',vessel=0),1e-7,atol=1e-15),ck(q('KCl',vessel=2),'gt',0)])
case('Gravimetric water unit equivalence','18.01528 grams and one mole denote essentially the same water amount.',{'a':water(amount='18.01528g'),'b':water(amount='1mol')},[near(q('water'),q('water','b'),atol=1e-5,rtol=1e-4)])
case('Split salt dosing equals combined dosing','Adding the same total neutral salt in two doses preserves ownership and pH.',{'a':water('add v1 NaCl 0.005mol\nadd v1 NaCl 0.005mol'),'b':water('add v1 NaCl 0.01mol')},[near(q('NaCl'),q('NaCl','b')),near(p(),p('b'),atol=.02)])
case('Filter clear brine twice','Clear solution passes through a filter without salt creation or loss.',{'a':water('add v1 NaCl 0.001mol\nfilter v1 v2\nfilter v2 v3')},[near(q('NaCl',vessel=0),.001),ck(q('NaCl',vessel=3),'gt',.0009)])
case('Magnet leaves water and copper behind','A magnet should not carry nonmagnetic water or copper to its receiver.',{'a':water('add v1 Cu 0.001mol\nmagnet v1 v2')},[near(q('Cu',vessel=2),0),near(q('water',vessel=2),0),near(q('Cu',vessel=0),.001)])
case('Magnet separates iron from copper','A magnetic separation should selectively transfer iron.',{'a':'add v1 Fe 0.001mol\nadd v1 Cu 0.001mol\nmagnet v1 v2'},[ck(q('Fe',vessel=2),'gt',.0009),near(q('Cu',vessel=2),0),near(q('Fe',vessel=0),.001)])
case('Grinding preserves solid amount','Changing iron particle size should not change owned iron.',{'a':'add v1 Fe 0.002mol\ngrind v1 Fe 20um'},[near(q('Fe'),.002)])
case('Stirring preserves inert mixture','Stirring dilute brine must conserve its components.',{'a':water('add v1 KCl 0.001mol\nstir v1 300rpm 5s')},[near(q('KCl'),.001),ck(q('water'),'gt',5)])
case('Double filtering an insoluble mineral','Filtering should retain calcium carbonate while transferring solvent.',{'a':water('add v1 CaCO3 0.005mol\nfilter v1 v2\nfilter v2 v3')},[near(q('CaCO3',vessel=0),.005),ck(q('CaCO3'),'gt',.004),ck(q('water',vessel=3),'gt',4)])
case('Vessel independence under heating','Heating one beaker must not heat a separate control beaker.',{'a':'add v1 water 100mL\nadd v2 water 100mL\nheat v1 1kJ','b':'add v1 water 100mL\nadd v2 water 100mL'},[near(m('temperature',vessel=2),m('temperature','b',2),atol=1e-9)])
case('Empty receiver identity','Creating an unused receiver leaves its inventory empty.',{'a':water('decant v1 v2 0.25')},[near(q('water',vessel=3),0),near(q('NaCl',vessel=3),0)])
# Thermal predictions stay far from boiling/freezing.
case('Heat then equal cool restores temperature','Equal moderate energy increments should cancel for liquid water.',{'a':water('heat v1 1kJ\ncool v1 1kJ'),'b':water()},[near(t(),t('b'),atol=.1)])
case('Two heat pulses equal one','In the same phase, splitting an energy dose should produce the same temperature.',{'a':water('heat v1 500J\nheat v1 500J'),'b':water('heat v1 1kJ')},[near(t(),t('b'),atol=.1)])
case('Cooling pulse scales with dose','A larger cooling dose must lower temperature further.',{'a':water('cool v1 1kJ'),'b':water('cool v1 500J')},[ck(t(),'lt',t('b'))])
case('Double water mass halves warming','Sensible temperature rise should scale inversely with water mass.',{'a':water('heat v1 1kJ'),'b':water('heat v1 1kJ',amount='200mL'),'c':water()},[near(sub(t(),t('c')),dict(kind='sum',terms=[sub(t('b'),t('c')),sub(t('b'),t('c'))]),atol=.2,rtol=.05)])
case('Joule and kilojoule notation agree','Equivalent energy units should produce equal temperatures.',{'a':water('heat v1 1200J'),'b':water('heat v1 1.2kJ')},[near(t(),t('b'),atol=1e-8)])
case('Warm donor aliquot carries temperature','Pouring pure warm water into an empty receiver should preserve its temperature.',{'a':'add v1 water 100mL @ 40C\ndecant v1 v2 0.5'},[near(t(),m('temperature',vessel=2),atol=.2)])
case('Mix equal hot and cool water','Equal water masses at 20 and 40 Celsius should mix near 30 Celsius.',{'a':'add v1 water 100mL @ 20C\nadd v2 water 100mL @ 40C\ndecant v2 v1 1'},[ck(t(),'between',[302.15,304.15])])
case('Cooling then heating restores temperature','Reversing pulse order should also cancel away from phase changes.',{'a':water('cool v1 500J\nheat v1 500J'),'b':water()},[near(t(),t('b'),atol=.1)])
case('Thermometer is passive','Reading a thermometer must preserve material and temperature.',{'a':water('measure v1 thermometer'),'b':water()},[near(t(),t('b'),atol=1e-9),near(q('water'),q('water','b'))])
case('Energy splitting across aliquots','Heating two equal halves by half the energy and recombining should match whole heating.',{'a':water('decant v1 v2 0.5\nheat v1 500J\nheat v2 500J\ndecant v2 v1 1'),'b':water('heat v1 1kJ')},[near(t(),t('b'),atol=.2)])
# Acid/base relations with explicit canonical identifiers.
case('Ammonia is basic','An ammonia solution should be measurably basic.',{'a':water('add v1 NH3 0.001mol')},[ck(p(),'between',[9,12.5])])
case('Ammonium chloride is acidic','Ammonium hydrolysis should lower pH below neutrality.',{'a':water('add v1 NH4Cl 0.001mol')},[ck(p(),'between',[4,6.8])])
case('Ammonia dilution lowers basicity','Dilution of a weak base should move pH toward neutrality.',{'a':water('add v1 NH3 0.001mol'),'b':water('add v1 NH3 0.001mol\ndilute v1 100mL')},[ck(p(),'gt',p('b')),ck(p('b'),'gt',7)])
case('Ammonium dilution lowers acidity','Dilution of a weak acid should raise pH toward neutrality.',{'a':water('add v1 NH4Cl 0.001mol'),'b':water('add v1 NH4Cl 0.001mol\ndilute v1 100mL')},[ck(p(),'lt',p('b')),ck(p('b'),'lt',7)])
case('Acetate salt is basic','A sodium acetate solution should show conjugate-base hydrolysis.',{'a':water('add v1 NaOAc 0.001mol')},[ck(p(),'between',[7.5,10.5])])
case('Equal ammonia and ammonium buffer','An equimolar weak-base buffer should lie near the ammonium pKa.',{'a':water('add v1 NH3 0.001mol\nadd v1 NH4Cl 0.001mol')},[ck(p(),'between',[8.7,9.8])])
case('Ammonia buffer resists small acid dose','A buffer should remain basic after a small strong-acid dose.',{'a':water('add v1 NH3 0.001mol\nadd v1 NH4Cl 0.001mol\nadd v1 HCl 0.0001mol'),'b':water('add v1 NH3 0.001mol\nadd v1 NH4Cl 0.001mol')},[ck(p(),'gt',8),ck(p(),'lt',p('b')),ck(sub(p('b'),p()),'lt',.5)])
case('Ammonia buffer resists small base dose','A small base dose should raise buffer pH only modestly.',{'a':water('add v1 NH3 0.001mol\nadd v1 NH4Cl 0.001mol\nadd v1 NaOH 0.0001mol'),'b':water('add v1 NH3 0.001mol\nadd v1 NH4Cl 0.001mol')},[ck(p(),'gt',p('b')),ck(sub(p(),p('b')),'lt',.5)])
case('Weak acid buffer dilution','Diluting an equimolar acetate buffer should change pH little.',{'a':water('add v1 CH3COOH 0.001mol\nadd v1 NaOAc 0.001mol'),'b':water('add v1 CH3COOH 0.001mol\nadd v1 NaOAc 0.001mol\ndilute v1 100mL')},[near(p(),p('b'),atol=.15)])
case('Acetate buffer composition ratio','More conjugate base at fixed acid should raise buffer pH.',{'a':water('add v1 CH3COOH 0.001mol\nadd v1 NaOAc 0.002mol'),'b':water('add v1 CH3COOH 0.001mol\nadd v1 NaOAc 0.001mol')},[ck(p(),'gt',p('b')),ck(sub(p(),p('b')),'between',[.15,.5])])
case('Partial weak-acid neutralization forms buffer','Half-neutralized acetic acid should lie near its pKa.',{'a':water('add v1 CH3COOH 0.002mol\nadd v1 NaOH 0.001mol')},[ck(p(),'between',[4.3,5.2])])
case('Strong base in excess remains basic','A millimole of excess hydroxide should dominate pH.',{'a':water('add v1 HCl 0.001mol\nadd v1 NaOH 0.002mol')},[ck(p(),'between',[11,13])])
case('Strong acid in excess remains acidic','A millimole of excess strong acid should dominate pH.',{'a':water('add v1 HCl 0.002mol\nadd v1 NaOH 0.001mol')},[ck(p(),'between',[1,3])])
case('Neutralization addition order','Reversing reagent addition should not change final equilibrium pH.',{'a':water('add v1 HCl 0.001mol\nadd v1 NaOH 0.0005mol'),'b':water('add v1 NaOH 0.0005mol\nadd v1 HCl 0.001mol')},[near(p(),p('b'),atol=.05)])
case('Transferred buffer keeps pH','An aliquot of a homogeneous acetate buffer should retain its donor pH.',{'a':water('add v1 CH3COOH 0.001mol\nadd v1 NaOAc 0.001mol\ndecant v1 v2 0.4')},[near(p(),m('ph',vessel=2),atol=.1)])
# Qualitative precipitation/solubility without fitted equilibrium values.
case('Silver chloride precipitates','Mixing dilute silver nitrate and chloride should produce a solid.',{'a':water('add v1 AgNO3 0.001mol\nadd v1 NaCl 0.001mol')},[ck(m('solid'),'gt',.0005)])
case('Precipitate filtering leaves solid donor','Filtering silver chloride should retain most of the solid in its donor.',{'a':water('add v1 AgNO3 0.001mol\nadd v1 NaCl 0.001mol\nfilter v1 v2')},[ck(m('solid'),'gt',.0005),ck(m('solid',vessel=2),'lt',.0001)])
case('Calcium carbonate from soluble salts','Calcium and carbonate solutions should produce insoluble carbonate.',{'a':water('add v1 CaCl2 0.001mol\nadd v1 Na2CO3 0.001mol')},[ck(m('solid'),'gt',.0005)])
case('Barium sulfate precipitation','Mixing soluble barium and sulfate should precipitate strongly.',{'a':water('add v1 BaCl2 0.001mol\nadd v1 Na2SO4 0.001mol')},[ck(m('solid'),'gt',.0005)])
case('Dilute potassium chloride remains dissolved','A millimole of KCl in 100 mL should not precipitate.',{'a':water('add v1 KCl 0.001mol')},[ck(m('solid'),'lt',1e-8),near(q('KCl'),.001)])
# Gas boundary changes, conservation and noninvasive observation.
case('Seal preserves nitrogen ownership','Closing a nitrogen-containing vessel should not create or delete nitrogen.',{'a':'add v1 N2 0.001mol\nseal v1 100mL'},[near(q('N2'),.001)])
case('Smaller nitrogen headspace raises pressure','At equal gas amount and temperature, halving headspace raises pressure.',{'a':'add v1 N2 0.001mol\nseal v1 100mL','b':'add v1 N2 0.001mol\nseal v1 200mL'},[ck(m('pressure'),'gt',m('pressure','b')),ck(dict(kind='ratio',left=m('pressure'),right=m('pressure','b')),'between',[1.5,2.5])])
case('Heating sealed nitrogen raises pressure','A warmed sealed gas should show higher pressure than its unheated control.',{'a':'add v1 N2 0.01mol\nseal v1 1L\nheat v1 10J','b':'add v1 N2 0.01mol\nseal v1 1L'},[ck(m('pressure'),'gt',m('pressure','b')),near(q('N2'),.01)])
case('Pressure reading is passive','A pressure reading should not alter sealed gas ownership or pressure.',{'a':'add v1 N2 0.001mol\nseal v1 100mL\nmeasure v1 pressure','b':'add v1 N2 0.001mol\nseal v1 100mL'},[near(q('N2'),q('N2','b')),near(m('pressure'),m('pressure','b'),atol=1e-6)])
case('Sealed carbon dioxide inventory','A finite gas charge in a sealed aqueous vessel remains owned across inspection.',{'a':water('seal v1 200mL\nadd v1 CO2 0.0001mol')},[near(q('CO2'),.0001,atol=1e-10),ck(p(),'lt',7)])
assert len(cases)==50,len(cases)
forecast=dict(schema=1,campaign='sixth-independent-50-20261008',source_commit='7bc43e089753483710507133ccc8377591bb3490',binary_sha256='fcf424a65384c5ef5949960f228fda4e1c3e893ec93c65d58fe5b12064352967',source_scope='Original fifth hosted executable, reused without compilation; predates subsequent main repairs.',independence='New expectations authored from general reasoning before consulting any further outputs. Prior batch findings are already known: not blinded. Consulted only CLI grammar/help, inspection field declarations, and registry identity keys; no existing experiment corpus or previous forecasts used for design.',policy='All scripts must exit zero, contain valid JSON, end in inspect, and satisfy every frozen check. Any model/safety notice remains a qualification even when numerical checks pass. Author, domain and engine failures must be distinguished by raw review; no silent edits.',cases=cases)
out=ROOT/'audits/sixth-independent-50-20261008'
out.mkdir(parents=True,exist_ok=True)
(out/'forecast.json').write_text(json.dumps(forecast,indent=2,ensure_ascii=False,allow_nan=False)+'\n')
lines=['# Sixth independent fifty: frozen expectations','',forecast['independence'],'',forecast['source_scope'],'',forecast['policy'],'']
for c in cases: lines += [f"## {c['id']}: {c['title']}",'',c['expectation'],'',f"Variants: {', '.join(c['variants'])}; explicit machine checks: {len(c['checks'])}.",'']
(out/'FORECASTS.md').write_text('\n'.join(lines))
print(len(cases),'cases',sum(len(c['variants']) for c in cases),'CLI processes')
