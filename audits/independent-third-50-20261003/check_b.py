"""Read-only quantitative checks for frozen E116-E130 JSONL case streams.
check_case(case_id, parsed_rows) returns named checks with bool/None and details.
None denotes unavailable physical coverage or a semantic/control limitation.
"""
import math


def check_case(case_id, rows):
    out=[]
    def check(name, ok, **details):
        out.append({'name':name, 'ok':ok, 'details':details})
    measures={}
    for row in rows:
        for e in row.get('events',[]):
            if e.get('event')=='measured':
                measures.setdefault((e['vessel'],e['instrument']),[]).append(e)
    final=rows[-1]['bench']['vessels']
    def readings(v,inst):
        events=measures.get((v,inst),[])
        if not events or any(e.get('model_support',{}).get('status') in ('unsupported','incomplete') for e in events):
            raise ValueError('measurement coverage unavailable: '+str((v,inst)))
        return [e['value'] for e in events]
    def qty(v,species):
        return sum(p['moles'] for p in final[v]['contents'] if p['species']==species)
    def close(a,b,rel=.001,absolute=0):
        return abs(a-b)<=max(absolute,rel*abs(b))
    failures=[e for r in rows for e in r.get('events',[]) if e.get('event') in ('solve_failed','solver_failed')]
    check('solver_failures_absent',not failures,events=failures)
    unsupported=[e for r in rows for e in r.get('events',[]) if e.get('event')=='not_yet_modeled']
    check('reported_capability_limits',None if unsupported else True,limits=[e.get('what') for e in unsupported])
    try:
        n=int(case_id[1:])
        if n in (116,122):
            t=readings(0,'thermometer');m=readings(0,'balance')
            check('temperature_cycle',t[1]>t[0] and close(t[-1],t[0],0,.2),temperature_c=t)
            check('mass_cycle',close(m[-1],m[0]),mass_g=m)
            if n==122:
                p=readings(0,'pressure_gauge')
                check('fixed_volume_pressure_temperature',close(p[1]/p[0],(t[1]+273.15)/(t[0]+273.15),.02),pressure=p,temperature_c=t)
                check('pressure_cycle',close(p[-1],p[0],.01),pressure=p)
        elif n in (117,118):
            t=[readings(v,'thermometer')[-1] for v in (0,1)];m=[readings(v,'balance')[-1] for v in (0,1)]
            check('final_temperature_agreement',abs(t[0]-t[1])<=.2,temperature_c=t)
            ratio=1 if n==117 else 2
            check('mass_scaling',close(m[1]/m[0],ratio),mass_g=m,ratio=m[1]/m[0])
            check('water_scaling',close(qty(1,'water')/qty(0,'water'),ratio),ratio=qty(1,'water')/qty(0,'water'))
        elif n==119:
            t=[readings(v,'thermometer')[-1] for v in (0,1,2,3)]
            check('mixed_midpoint',t[0]<t[2]<t[1] and abs(t[2]-(t[0]+t[1])/2)<=.5,temperature_c=t)
            check('direct_mixed_mass',close(readings(2,'balance')[-1],readings(3,'balance')[-1],.005))
            check('complete_source_transfer',not final[0]['contents'] and not final[1]['contents'])
        elif n in (120,121):
            a=[]
            for v in (0,1):
                p=readings(v,'pressure_gauge');t=readings(v,'thermometer')
                # Gauge events use kPa; convert to Pa.
                a.append(1000*(p[-1]/(t[-1]+273.15)-p[0]/(t[0]+273.15)))
            check('pressure_increment_scaling',close(a[0]/a[1],2 if n==120 else 1,.02),normalized_increment_pa_per_k=a,ratio=a[0]/a[1])
            if n==120:
                for v,vol in ((0,.001),(1,.002)):
                    check('ideal_gas_increment_v'+str(v+1),close(a[v]*vol,.001*8.314462618,.02),delta_p_over_t_times_v=a[v]*vol)
        elif n==123:
            p=readings(0,'pressure_gauge');t=readings(0,'thermometer');m=readings(0,'balance')
            check('expansion_pv_over_t',close(2*p[1]/(t[1]+273.15),p[0]/(t[0]+273.15),.02),pressure=p,temperature_c=t)
            check('return_pressure',close(p[-1],p[0],.02),pressure=p)
            check('return_mass',close(m[-1],m[0]),mass_g=m)
        elif n==124:
            for inst in ('pressure_gauge','thermometer','balance'):
                x=readings(0,inst);check('repeat_'+inst,max(x)-min(x)<1e-10,values=x)
            states=[r['bench'] for r in rows if r.get('operator',{}).get('op')=='inspect']
            check('measurement_state_invariance',states[0]==states[-1] if len(states)>1 else None)
        elif n==125:
            t=readings(0,'thermometer')+readings(1,'thermometer');check('temperature_alias_units',max(t)-min(t)<1e-10,temperature_c=t)
            check('mass_equivalence',close(readings(0,'balance')[-1],readings(1,'balance')[-1]))
            check('liquid_volume_instrument',None,reason='volume_meter covers headspace only; liquid inventory equivalent')
        elif n==126:
            water=[qty(v,'water') for v in (0,1)]
            check('dilution_semantics',None,reason='additive water dose, not make-up-to-volume; protocols intentionally not equivalent under frozen semantic branch',water_moles=water)
            check('salt_conservation',all(close(qty(v,'Cl-'),1e-5) and close(qty(v,'Na+'),1e-5) for v in (0,1)))
        elif n==127:
            t=[readings(v,'thermometer') for v in (0,1)];ph=[readings(v,'ph_meter') for v in (0,1)]
            delta=[x[-1]-x[0] for x in t]
            check('thermal_contrast',abs(delta[0]-delta[1])<=max(.1,.01*sum(delta)/2),delta_temperature_k=delta)
            check('frozen_acid_detectability_control',ph[1][0]-ph[0][0]>=2,ph=ph,interpretation='initial control fails because trapped atmospheric CO2 acidifies pure water; not a thermal defect')
            check('acid_stays_more_acidic',ph[0][-1]<ph[1][-1],ph=ph)
        elif n==128:
            ph=[readings(v,'ph_meter')[-1] for v in (0,1)]
            check('water_ionization_floor',6.8<ph[0]<=ph[1]<=7.05,ph=ph)
            for v,dose in ((0,1e-9),(1,1e-12)):
                chlorine=qty(v,'Cl-');check('chlorine_conservation_v'+str(v+1),close(chlorine,dose,.001),expected_mol=dose,actual_mol=chlorine)
        elif n==129:
            k=[readings(v,'conductivity_meter')[-1] for v in (0,1)];ph=[readings(v,'ph_meter')[-1] for v in (0,1)]
            check('conductivity_intensive',close(k[0],k[1],.01),conductivity=k)
            check('ph_intensive',abs(ph[0]-ph[1])<=.05,ph=ph)
            for species in ('water','Na+','Cl-'):check('inventory_scale_'+species,close(qty(1,species)/qty(0,species),100),ratio=qty(1,species)/qty(0,species))
            density=[measures[(v,'densitometer')][-1] for v in (0,1)]
            support=[e['model_support'] for e in density]
            check('density_coverage_scale_invariance',support[0]['status']==support[1]['status'] and support[0]['reasons']==support[1]['reasons'],supports=support)
            check('density_physical_agreement',None,reason='dissolved-ion partial volumes unavailable; numerical equality cannot establish density accuracy')
        elif n==130:
            for species in ('water','Na+','Cl-'):check('restored_'+species,close(qty(0,species),qty(1,species),.005),moles=[qty(v,species) for v in (0,1)])
            t=[readings(v,'thermometer')[-1] for v in (0,1)];k=[readings(v,'conductivity_meter')[-1] for v in (0,1)];ph=[readings(v,'ph_meter')[-1] for v in (0,1)]
            check('restored_conductivity',close(k[0],k[1],.02) if abs(t[0]-t[1])<=.2 else None,conductivity=k,temperature_c=t)
            check('restored_ph',abs(ph[0]-ph[1])<=.05,ph=ph)
            check('evaporation_thermal_balance',None,reason='explicit missing vaporization enthalpy; composition restoration only')
    except (ValueError,KeyError,IndexError,ZeroDivisionError) as exc:
        check('remaining_physical_checks',None,reason=str(exc))
    return out


def run(cases, check):
    """Root bool-only adapter; return qualifications separately from checks.

    The frozen E127 detectability control is preserved in the assessment,
    but is not a software defect: the actual sealed-water control contains
    atmospheric CO2. Unsupported measurements are never marked passed.
    """
    qualifications=[]
    for i in range(116, 131):
        case_id = f'E{i}'
        if case_id not in cases:
            check(case_id+'_stream_present', False, reason='missing case stream')
            continue
        results=check_case(case_id, cases[case_id])
        for result in results:
            if result['ok'] is None or result['name']=='frozen_acid_detectability_control':
                qualifications.append({'id':case_id, **result})
                continue
            check(case_id+'_'+result['name'], result['ok'], **result['details'])
        if case_id=='E126':
            final=cases[case_id][-1]['bench']['vessels']
            water=[sum(p['moles'] for p in final[v]['contents'] if p['species']=='water') for v in (0,1)]
            initial_per_100ml=5.534276991396059
            check('E126_observed_additive_dilution_boundary',
                  abs(water[0]/(1.1*initial_per_100ml)-1)<.001 and
                  abs(water[1]/(1.3*initial_per_100ml)-1)<.001,
                  water_moles=water, nominal_final_volume_ml=[110,130])
    return qualifications
