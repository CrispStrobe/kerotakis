"""Quantitative checks for original frozen E101-E115 forecasts.
Physical scope qualifications are in assessment.json, not converted to passes here.
"""
def run(cases, check):
    def v(r,i=0):return next(x for x in r['bench']['vessels'] if x['id']==i)
    def q(v,s,phase=None):return sum(x['moles'] for x in v['contents'] if x['species']==s and (phase is None or x['phase']==phase))
    def close(a,b):return abs(a-b)<=max(1e-10,5e-8*max(abs(a),abs(b)))
    def obs(k):return [r for r in cases[k] if r['operator']['op']=='inspect']
    def measured(k,inst,i):return [e['value'] for r in cases[k] for e in r.get('events',[]) if e['event']=='measured' and e['instrument']==inst and e['vessel']==i]
    def equal(name,a,b):check(name,close(a,b),actual=a,expected=b)
    for k,expected in [('E101',[.009,.004,.015]),('E102',[.004,.008,.004]),('E105',[.006,.006])]:
        r=cases[k][-1]
        for i,n in enumerate(expected):
            for s in ['Na+','Cl-']:equal(f'{k} {s} vessel {i+1}',q(v(r,i),s),n)
        before=obs(k)[0]
        equal(k+' water conserved',sum(q(x,'water') for x in r['bench']['vessels']),sum(q(x,'water') for x in before['bench']['vessels'])) if k!='E105' else None
    k='E101';a=[measured(k,'conductivity_meter',i)[-1] for i in range(3)]
    check(k+' target conductivity between sources',a[0]<a[2]<a[1],readings=a)
    k='E102';a=[measured(k,'conductivity_meter',i)[-1] for i in range(3)]
    check(k+' equal conductivity',max(a)-min(a)<.01,readings=a)
    k='E103';r=cases[k][-1];a=v(r);b=v(r,1)
    equal(k+' Na retained after return',q(a,'Na+'),.01);equal(k+' Cl retained after return',q(a,'Cl-'),.01)
    check(k+' cake retained without macroscopic duplication',q(a,'CaCO3','solid')>.009 and q(b,'CaCO3','solid')<1e-8)
    equal(k+' calcium budget',sum(q(x,'Ca+2')+q(x,'CaCO3') for x in r['bench']['vessels']),1/100.087)
    k='E104';r=cases[k][-1]
    equal(k+' copper budget',sum(q(x,'Cu+2')+q(x,'Cu(OH)2')+4*q(x,'brochantite')+3*q(x,'antlerite') for x in r['bench']['vessels']),.005)
    equal(k+' sulfate budget',sum(q(x,'SO4-2')+q(x,'brochantite')+q(x,'antlerite') for x in r['bench']['vessels']),.005)
    equal(k+' sodium budget',sum(q(x,'Na+') for x in r['bench']['vessels']),.02)
    equal(k+' chloride budget',sum(q(x,'Cl-') for x in r['bench']['vessels']),.01)
    check(k+' hydroxide solid retained and neutralized source empty',q(v(r),'Cu(OH)2','solid')>.0049 and not v(r,2)['contents'])
    k='E105';r=cases[k][-1]
    check(k+' chalk remains predominantly source',q(v(r),'CaCO3','solid')>.019 and q(v(r,1),'CaCO3','solid')<1e-5)
    check(k+' each transfer remains characterised',all(v(r,0)['solution'] is not None and v(r,1)['solution'] is not None for r in cases[k] if r['operator']['op']=='decant'))
    equal(k+' calcium budget',sum(q(x,'Ca+2')+q(x,'CaCO3') for x in r['bench']['vessels']),2/100.087)
    k='E106';r=cases[k][-1]
    check(k+' drain selects lower brine',q(v(r),'water')==0 and q(v(r),'hexane')>0 and q(v(r,1),'hexane')==0 and q(v(r,1),'water')>0)
    # Frozen semantic alternative: decant is a bulk fraction, so no skimming assertion.
    for s in ['water','hexane','Na+','Cl-']:
        equal(k+' independent protocol conservation '+s,sum(q(v(r,i),s) for i in [0,1]),sum(q(v(r,i),s) for i in [2,3]))
    k='E107';r=cases[k][-1];initial=1e-5
    for i in [0,2]:equal(k+f' iodine budget pair {i}',q(v(r,i),'I2')+q(v(r,i+1),'I2'),initial)
    check(k+' multistage more effective',0<q(v(r,2),'I2')<q(v(r),'I2')<initial)
    equal(k+' equal extracting solvent',q(v(r,1),'hexane'),q(v(r,3),'hexane'))
    k='E108';r=cases[k][-1];before=obs(k)[1]
    equal(k+' extracted iodine unchanged by aqueous pour',q(v(r,1),'I2'),q(v(before,1),'I2'))
    equal(k+' residual iodine transferred',q(v(r,2),'I2'),q(v(before),'I2'))
    equal(k+' iodine budget',sum(q(x,'I2') for x in r['bench']['vessels']),initial)
    check(k+' aqueous source emptied',not v(r)['contents'])
    k='E109';r=cases[k][-1];before=obs(k)[0]
    for s in ['Na+','Cl-']:
        equal(k+' nonvolatile '+s,q(v(r),s),.012);equal(k+' no condensate '+s,q(v(r,1),s),0)
    equal(k+' equal final water split',q(v(r),'water'),q(v(r,1),'water'))
    equal(k+' water budget',q(v(r),'water')+q(v(r,1),'water'),q(v(before),'water'))
    k='E110';r=cases[k][-1];before=obs(k)[0]
    staged_success=bool(v(r,3)['contents'])
    single_ratio=q(v(r,1),'ethanol')/q(v(r,1),'water')
    check(k+' single stage volatile enrichment',single_ratio>q(v(before),'ethanol')/q(v(before),'water'))
    if staged_success:
        staged_ratio=q(v(r,3),'ethanol')/q(v(r,3),'water')
        check(k+' staged volatile enrichment',staged_ratio>single_ratio,receiver_ratios=[single_ratio,staged_ratio])
        check(k+' staged partial cut preserves both components',q(v(r,2),'ethanol')>0 and q(v(r,2),'water')>0)
    else:
        refusal=[row for row in cases[k] if row['operator']['op']=='distil' and row['operator'].get('from')==2]
        check(k+' staged numerical limitation explicitly refused',any(e['event']=='not_yet_modeled' for row in refusal for e in row.get('events',[])))
        check(k+' staged refusal preserves full source state',v(r,2)==v(before,2))
        check(k+' staged refusal leaves receiver empty',v(r,3)==v(before,3))
    for s in ['water','ethanol']:
        for i in [0,2]:equal(k+f' {s} budget pair {i}',q(v(r,i),s)+q(v(r,i+1),s),q(v(before,i),s))
    if staged_success:equal(k+' equal molar cuts',sum(q(v(r,1),s) for s in ['water','ethanol']),sum(q(v(r,3),s) for s in ['water','ethanol']))
    k='E111';r=cases[k][-1];before=obs(k)[0]
    for s in ['water','Na+','Cl-']:equal(k+' matching residues '+s,q(v(r),s),q(v(r,1),s))
    equal(k+' captured water budget',q(v(r,1),'water')+q(v(r,2),'water'),q(v(before,1),'water'))
    check(k+' uncaptured evaporation water leaves',q(v(r),'water')<q(v(before),'water'))
    for k in ['E112','E114']:
        rows=obs(k);a,b=v(rows[0]),v(rows[-1])
        equal(k+' fixed volume pressure temperature ratio',b['pressure']/a['pressure'],b['temperature']/a['temperature'])
        for s in ['N2','O2','CO2']:equal(k+' rigid gas conserved '+s,q(a,s),q(b,s))
    k='E112';rows=obs(k);check(k+' unheated reference unchanged',v(rows[0],1)==v(rows[-1],1))
    k='E113';rows=obs(k);a,b=v(rows[0]),v(rows[-1])
    equal(k+' doubled volume pressure law',b['pressure']/a['pressure'],.5*b['temperature']/a['temperature'])
    for s in ['N2','O2','CO2']:equal(k+' reseal gas conserved '+s,q(a,s),q(b,s))
    k='E114';rows=obs(k);a,b=v(rows[0],1),v(rows[-1],1)
    equal(k+' constant regulated pressure',b['pressure'],a['pressure'])
    vols=measured(k,'volume_meter',1);equal(k+' piston volume temperature ratio',vols[-1]/vols[0],b['temperature']/a['temperature'])
    for s in ['N2','O2','CO2']:equal(k+' regulated gas conserved '+s,q(a,s),q(b,s))
    k='E115';rows=obs(k);p0=v(rows[0])['pressure'];pf=v(rows[-1])['pressure']
    check(k+' reseal does not restore overpressure',p0>2*pf and abs(pf-101325)<1,initial_pa=p0,final_pa=pf)
    check(k+' added nitrogen vented rather than restored',q(v(rows[-1]),'N2')<q(v(rows[0]),'N2'))
